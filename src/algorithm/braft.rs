use alloc::{boxed::Box, vec::Vec};

use crate::{
    messages::{BroadcastCommit, BroadcastPropose, Packet},
    App, Consensus, Network,
    Error, Result, Role, VoteSign, Voter,
};

/// Raft for blockchain.
///
/// Variant of raft for blockchain.
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

    weight: C::Weight,
    vote_signs: Vec<VoteSign<N::Signature>>,
    voter_set: Vec<Voter<C::NodeId, C::PublicKey, C::Weight>>,
}

impl<N, A, C> BRaft<N, A, C>
where
    N: Network<C::NodeId, C::PublicKey, C::EpochId, C::EpochHash>,
    C: Consensus,
    A: App<C::NodeId, C::PublicKey, C::Weight, C::EpochId, C::EpochHash>,
{
    /// Build braft node
    ///
    /// Pass lowlevel network, consensus and application.
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
            weight: num_traits::zero(),
            vote_signs: Vec::new(),
            round: 0,
            step: 0,
        })
    }

    /// Trigger consensus.
    ///
    /// Run this method on loop.
    pub async fn trig(&mut self) -> Result<()> {
        use futures_lite::future::FutureExt;

        if self.role.is_follower() && self.step == 0 {
            // Wait BroadcastPropose.

            let timer = async {
                self.consensus.step_timer(&Role::Follower, 0);
                Err(Error::Timeout)
            };

            let recver = async {
                let pkt = self.network.recv().await.map_err(Error::network_error)?;
                Ok(pkt)
            };

            let pkt = recver.or(timer).await;

            match pkt {
                Ok((p, sender)) => self.wait_propose(p, sender).await?,
                Err(Error::Timeout) => {
                    self.round += 1;
                    self.step = 0;
                }
                Err(e) => return Err(e),
            }
        } else if self.role.is_follower() && self.step == 1 {
            // Wait BroadcastCommit.

            let timer = async {
                self.consensus.step_timer(&Role::Follower, 1);
                Err(Error::Timeout)
            };

            let recver = async {
                let pkt = self.network.recv().await.map_err(Error::network_error)?;
                Ok(pkt)
            };

            let pkt = recver.or(timer).await;

            match pkt {
                Ok((p, _sender)) => self.wait_commit(p).await?,
                Err(Error::Timeout) => {
                    self.round += 1;
                    self.step = 0;
                }
                Err(e) => return Err(e),
            }
            /* } */
        } else if self.role.is_proposer() && self.step == 0 {
            // Propose epoch.

            self.propose_epoch().await?;
        } else if self.role.is_proposer() && self.step == 1 {
            // Collect all `ResponsePropose`.

            self.collect_propose().await?;
        }

        // Update role and weight
        if self.step == 0 && self.round == 0 {
            self.role = self.consensus.compute_role(&self.epoch_hash);
            self.weight = num_traits::zero();
        }

        Ok(())
    }

    async fn wait_propose(
        &mut self,
        pkt: Packet<C::EpochId, C::EpochHash, N::Signature>,
        sender: C::NodeId,
    ) -> Result<()> {
        match pkt {
            Packet::BroadcastPropose(bc) => self.process_propose(sender, bc).await?,
            Packet::ResponsePropose(_) => self.error_packet(&pkt),
            Packet::BroadcastCommit(bc) => self.verify_and_accept_epoch(bc).await?,
        }
        Ok(())
    }

    async fn wait_commit(
        &mut self,
        pkt: Packet<C::EpochId, C::EpochHash, N::Signature>,
    ) -> Result<()> {
        if let Packet::BroadcastCommit(bc) = pkt {
            self.verify_and_accept_epoch(bc).await?
        } else {
            self.error_packet(&pkt);
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
        let (epoch_id, epoch_hash) = self.app.propose_epoch().await.map_err(Error::app_error)?;

        self.epoch_id = epoch_id.clone();
        self.epoch_hash = epoch_hash.clone();

        self.network.sign_and_send(
            None,
            Packet::broadcast_propose_from_id_hash(epoch_id, epoch_hash),
        );

        Ok(())
    }

    async fn collect_propose(&mut self) -> Result<()> {
        use futures_lite::FutureExt;

        let mut flag = true;

        let timer = self.consensus.step_timer(&Role::Proposer, 1);

        // Make this future to unpin.
        let mut timer = Box::pin(async move {
            timer.await;
            Err(Error::Timeout)
        });

        while flag {
            let recver = self.network.recv();

            let recver = async move {
                let pkt = recver.await.map_err(Error::network_error)?;
                Ok(pkt)
            };

            let pkt = recver.or(&mut timer).await;

            match pkt {
                Ok((pkt, _sender)) => self.add_propose_wight(pkt)?,
                Err(Error::Timeout) => flag = false,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn add_propose_wight(
        &mut self,
        pkt: Packet<C::EpochId, C::EpochHash, N::Signature>,
    ) -> Result<()> {
        if let Packet::ResponsePropose(rp) = pkt {
            let epoch_id = rp.epoch_id;
            let epoch_hash = rp.epoch_hash;

            if epoch_id == self.epoch_id && epoch_hash == self.epoch_hash {
                // Only process right vote. beacuse raft is not BFT.
                let sign = rp.vote_sign.ok_or(Error::NoSignature)?;

                if let Some(voter) = self.voter_set.get(sign.vs_idx as usize) {
                    self.weight += voter.weight.clone();
                } else {
                    log::error!("index of packet out of bound");
                }

                self.vote_signs.push(sign);
            } else {
                log::error!(
                    "Error epoch_id: {:?}, expect: {:?}; epoch_hash: {:?}, expect: {:?}",
                    epoch_id,
                    self.epoch_id,
                    epoch_hash,
                    self.epoch_hash
                );
            }
        } else {
            self.error_packet(&pkt);
        }

        Ok(())
    }

    async fn process_propose(
        &mut self,
        sender: C::NodeId,
        pkt: BroadcastPropose<C::EpochId, C::EpochHash, N::Signature>,
    ) -> Result<()> {
        let epoch_id = pkt.epoch_id;
        let epoch_hash = pkt.epoch_hash;

        if self.epoch_id < epoch_id {
            self.app
                .enter_step(0, epoch_id.clone(), epoch_hash.clone())
                .await
                .map_err(Error::app_error)?;

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

            self.round = 0;
            self.step = 0;

            let vs = self.app
                .commit(epoch_id, epoch_hash)
                .await
                .map_err(Error::app_error)?;
            self.voter_set = vs;
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
