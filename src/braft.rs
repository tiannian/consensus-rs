use alloc::{collections::BTreeMap, vec::Vec};

use crate::{
    messages::{BroadcastCommit, BroadcastPropose, Packet},
    prelude::{App, Consensus, Network},
    utils, Error, Result, Role, VoteSign, Voter,
};

pub struct BRaft<N, A, C>
where
    C: Consensus,
    N: Network<C::NodeId, C::PublicKey, C::EpochId, C::EpochHash>,
{
    network: N,
    app: A,
    consensus: C,

    role: Role,

    epoch_id: C::EpochId,
    epoch_hash: C::EpochHash,

    round: u64,
    step: u8,

    weights: BTreeMap<C::EpochHash, C::Weight>,
    vote_signs: Vec<VoteSign<N::Signature>>,
    voter_set: Vec<Voter<C::NodeId, C::PublicKey, C::Weight>>,
}

impl<N, A, C> BRaft<N, A, C>
where
    N: Network<C::NodeId, C::PublicKey, C::EpochId, C::EpochHash>,
    C: Consensus,
    A: App<C::EpochId, C::EpochHash>,
{
    /// Inital state.
    pub fn new(network: N, consensus: C, app: A) -> Result<Self> {
        let node_id = network.node_id();

        let (epoch_id, epoch_hash) = consensus.latest_epoch();

        let voter_set = consensus.latest_voter_set();

        let mut role = Role::Observer;

        for i in &voter_set {
            if i.voter_id == node_id {
                role = Role::Follower;
                break;
            }
        }

        if role == Role::Follower {
            role = consensus.compute_role(&epoch_hash);
        }

        Ok(Self {
            network,
            consensus,
            epoch_id,
            epoch_hash,
            role,
            app,
            voter_set,
            vote_signs: Vec::new(),
            weights: BTreeMap::new(),
            round: 0,
            step: 0,
        })
    }

    pub async fn trig(&mut self) -> Result<()> {
        use futures_lite::future::FutureExt;
        use utils::MapFutureExt;

        let timer_map_helper =
            |_| -> Option<(Packet<C::EpochId, C::EpochHash, N::Signature>, C::NodeId)> { None };

        if self.role.is_follower() && self.step == 0 {
            // Wait BroadcastPropose.

            let timer = self
                .consensus
                .step_timer(&self.role, self.step)
                .map(timer_map_helper);

            if let Some((pkt, sender)) = self.network.recv().or(timer).await {
                match pkt {
                    Packet::BroadcastPropose(bc) => self.process_propose(sender, bc).await?,
                    Packet::ResponsePropose(_) => self.error_packet(&pkt),
                    Packet::BroadcastCommit(bc) => self.verify_and_accept_epoch(bc).await?,
                }
            } else {
                self.round = 1;
                self.step = 0;
            }
        } else if self.role.is_follower() && self.step == 1 {
            // Wait BroadcastCommit.

            let timer = self
                .consensus
                .step_timer(&self.role, self.step)
                .map(timer_map_helper);

            if let Some((pkt, _sender)) = self.network.recv().or(timer).await {
                match pkt {
                    Packet::BroadcastCommit(bc) => self.verify_and_accept_epoch(bc).await?,
                    _ => self.error_packet(&pkt),
                }
            }
        } else if self.role.is_proposer() && self.step == 0 {
            // Propose epoch.

            self.propose_epoch().await?;
        } else if self.role.is_proposer() && self.step == 1 {
            // Collect all `ResponsePropose`.

            let timer = self
                .consensus
                .step_timer(&self.role, self.step)
                .map(timer_map_helper);
        }

        if self.step == 0 && self.round == 0 {
            self.role = self.consensus.compute_role(&self.epoch_hash);
        }

        Ok(())
    }

    fn error_packet(&self, _pkt: &Packet<C::EpochId, C::EpochHash, N::Signature>) {
        log::warn!(
            "Error packet, ignore it. epoch: {:?}, round: {}, step: {}",
            self.epoch_id,
            self.round,
            self.step
        )
    }

    async fn propose_epoch(&mut self) -> Result<()> {
        let (epoch_id, epoch_hash) = self
            .app
            .propose_epoch()
            .await
            .map_err(Error::from_core_debug)?;

        self.network.sign_and_send(
            None,
            Packet::broadcast_propose_from_id_hash(epoch_id, epoch_hash),
        );

        Ok(())
    }

    async fn process_propose(
        &mut self,
        sender: C::NodeId,
        pkt: BroadcastPropose<C::EpochId, C::EpochHash>,
    ) -> Result<()> {
        let epoch_id = pkt.epoch_id;
        let epoch_hash = pkt.epoch_hash;

        if self.epoch_id < epoch_id {
            self.app
                .enter_step(0, epoch_id.clone(), epoch_hash.clone())
                .await
                .map_err(Error::from_core_debug)?;

            self.network.sign_and_send(
                Some(sender),
                Packet::response_propose_from_id_hash(epoch_id, epoch_hash),
            );
        } else {
            log::warn!(
                "Receive error epoch id on `BroadcastPropose`, expect: > {:?}, got: {:?}. ignore this packet",
                self.epoch_id,
                epoch_id
            );
        }

        Ok(())
    }

    async fn verify_and_accept_epoch(
        &mut self,
        pkt: BroadcastCommit<C::EpochId, C::EpochHash, N::Signature>,
    ) -> Result<()> {
        let epoch_id = pkt.epoch_id;
        let epoch_hash = pkt.epoch_hash;

        if self.epoch_id < epoch_id {
            self.vote_signs = Vec::new();
            self.epoch_id = epoch_id.clone();
            self.epoch_hash = epoch_hash.clone();

            self.app
                .commit(epoch_id, epoch_hash)
                .await
                .map_err(Error::from_core_debug)?;
        } else {
            log::warn!(
                "Receive error epoch id on `BroadcastCommit`, expect: > {:?}, got: {:?}. ignore this packet",
                self.epoch_id,
                epoch_id
            );
        }

        Ok(())
    }
}
