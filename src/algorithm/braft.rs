use core::mem;

use futures_lite::future::FutureExt;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    packet::{BroadcastCommit, BroadcastPropose, Packet},
    App, Consensus, Error, Network, Result, Role, VoteSign, Voter,
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

    latest_epoch_id: C::EpochId,
    epoch_id: C::EpochId,
    epoch_hash: C::EpochHash,

    round: u64,
    step: u8,

    weight: C::Weight,
    total_weight: C::Weight,
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
    pub fn new(network: N, consensus: C, app: A) -> Self {
        let node_id = network.node_id();

        let (epoch_id, epoch_hash) = consensus.latest_epoch();

        let voter_set = consensus.latest_voter_set();

        let total_weight = voter_set.iter().map(|e| e.weight.clone()).sum();

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

        log::info!("Start node at epoch_id: {:?}", epoch_id);

        Self {
            network,
            consensus,
            latest_epoch_id: epoch_id.clone(),
            epoch_id,
            epoch_hash,
            role,
            app,
            voter_set,
            weight: num_traits::zero(),
            total_weight,
            vote_signs: Vec::new(),
            round: 0,
            step: 0,
        }
    }

    /// Trigger consensus.
    ///
    /// Run this method on loop.
    pub async fn do_tick(&mut self) -> Result<()> {
        log::info!(
            "On epoch_id/round/step: {:?}/{}/{}",
            self.latest_epoch_id,
            self.round,
            self.step
        );
        log::info!("Self node role is {:?}", self.role);

        if self.role.is_follower() && self.step == 0 {
            // Wait BroadcastPropose.
            self.wait_broadcast_propose().await?;
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

    async fn wait_broadcast_propose(&mut self) -> Result<()> {
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

    // ---------------------------- propose_epoch

    async fn propose_epoch(&mut self) -> Result<()> {
        log::debug!("Enter propose epoch");

        let (epoch_id, epoch_hash) = self.app.propose_epoch().await.map_err(Error::app_error)?;

        log::debug!("propose epoch: {:?} => {:?}", epoch_id, epoch_hash);

        self.epoch_id = epoch_id.clone();
        self.epoch_hash = epoch_hash.clone();

        self.network.send_unsigned(
            None,
            Packet::broadcast_propose_from_id_hash(epoch_id, epoch_hash),
        );

        self.step = 1;

        Ok(())
    }

    // ---------------------------- collect_propose
    async fn collect_propose(&mut self) -> Result<()> {
        log::debug!("Enter collect propose");

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
            log::debug!("receive packt: {:?}", pkt);

            match pkt {
                Ok((pkt, _sender)) => self.collect_propose_packet(pkt)?,
                Err(Error::Timeout) => flag = false,
                Err(e) => return Err(e),
            }
        }

        let vote_signs = mem::take(&mut self.vote_signs);

        let one: C::Weight = num_traits::one();
        let two: C::Weight = one.clone() + one;
        if self.weight.clone() * two <= self.total_weight {
            self.round += 1;
            self.step = 0;
        } else {
            self.network.send_unsigned(
                None,
                Packet::broadcast_commit_from_id_hash(
                    self.epoch_id.clone(),
                    self.epoch_hash.clone(),
                    vote_signs,
                ),
            );
        }

        self.step = 0;
        self.round = 0;
        self.latest_epoch_id = self.epoch_id.clone();

        self.app.commit(&self.epoch_id, &self.epoch_hash).await.map_err(Error::app_error)?;

        Ok(())
    }

    fn add_weight(
        &mut self,
        epoch_id: C::EpochId,
        epoch_hash: C::EpochHash,
        vote_sign: Option<VoteSign<N::Signature>>,
    ) -> Result<()> {
        if epoch_id == self.epoch_id && epoch_hash == self.epoch_hash {
            // Only process right vote. beacuse raft is not BFT.
            let sign = vote_sign.ok_or(Error::NoSignature)?;

            if let Some(voter) = self.voter_set.get(sign.idx as usize) {
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
        Ok(())
    }

    fn collect_propose_packet(
        &mut self,
        pkt: Packet<C::EpochId, C::EpochHash, N::Signature>,
    ) -> Result<()> {
        match pkt {
            Packet::ResponsePropose(rp) => {
                let epoch_id = rp.epoch_id;
                let epoch_hash = rp.epoch_hash;
                let vote_sign = rp.vote_sign;

                self.add_weight(epoch_id, epoch_hash, vote_sign)?;
            }
            Packet::BroadcastPropose(rp) => {
                let epoch_id = rp.epoch_id;
                let epoch_hash = rp.epoch_hash;
                let vote_sign = rp.vote_sign;

                self.add_weight(epoch_id, epoch_hash, vote_sign)?;
            }
            _ => {
                self.error_packet(&pkt);
            }
        }

        Ok(())
    }

    // -----------------------------------------

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

            self.network.send_unsigned(
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

            let vs = self
                .app
                .commit(&epoch_id, &epoch_hash)
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

#[cfg(test)]
mod tests {
    use core::{pin::Pin, time::Duration};

    use alloc::{boxed::Box, string::String, vec::Vec};
    use futures_lite::Future;
    use smol::{
        channel::{unbounded, Receiver, Sender},
        Timer,
    };

    use crate::{packet::Packet, App, Consensus, Network, Role, Voter, VoteSign};

    use super::BRaft;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    struct SingleApp {
        pub epoch_id: u64,
        pub epoch_hash: u64,
        pub voter: Voter<Vec<u8>, Vec<u8>, u64>,
    }

    impl SingleApp {
        pub fn new() -> Self {
            let voter = Voter {
                voter_id: alloc::vec![1],
                public_key: alloc::vec![1],
                weight: 1,
            };

            Self {
                epoch_id: 0,
                epoch_hash: 0,
                voter,
            }
        }
    }

    impl App<Vec<u8>, Vec<u8>, u64, u64, u64> for SingleApp {
        type Error = String;

        type ProposeEpochFuture = Pin<Box<dyn Future<Output = Result<(u64, u64), String>>>>;

        type EnterStepFuture = Pin<Box<dyn Future<Output = Result<(u64, u64), Self::Error>>>>;

        type CommitFuture =
            Pin<Box<dyn Future<Output = Result<Vec<Voter<Vec<u8>, Vec<u8>, u64>>, Self::Error>>>>;

        fn propose_epoch(&mut self) -> Self::ProposeEpochFuture {
            let epoch_id = self.epoch_id + 1;
            let epoch_hash = self.epoch_hash + 1;

            Box::pin(async move { Ok((epoch_id, epoch_hash)) })
        }

        fn enter_step(
            &mut self,
            _step: u8,
            epoch_id: u64,
            epoch_hash: u64,
        ) -> Self::EnterStepFuture {
            Box::pin(async move { Ok((epoch_id, epoch_hash)) })
        }

        fn commit(&mut self, epoch_id: &u64, epoch_hash: &u64) -> Self::CommitFuture {
            self.epoch_id = *epoch_id;
            self.epoch_hash = *epoch_hash;

            let voter = alloc::vec![self.voter.clone()];

            Box::pin(async move { Ok(voter) })
        }
    }

    struct SingleConsensus {
        pub voter: Voter<Vec<u8>, Vec<u8>, u64>,
    }

    impl SingleConsensus {
        pub fn new() -> Self {
            let voter = Voter {
                voter_id: alloc::vec![1],
                public_key: alloc::vec![1],
                weight: 1,
            };

            Self { voter }
        }
    }

    impl Consensus for SingleConsensus {
        type Timer = Pin<Box<dyn Future<Output = ()>>>;

        type NodeId = Vec<u8>;

        type Weight = u64;

        type EpochId = u64;

        type PublicKey = Vec<u8>;

        type EpochHash = u64;

        fn step_timer(&self, _role: &Role, _step: u8) -> Self::Timer {
            Box::pin(async move {
                Timer::after(Duration::from_secs(1)).await;
            })
        }

        fn latest_epoch(&self) -> (Self::EpochId, Self::EpochHash) {
            (0, 0)
        }

        fn latest_voter_set(&self) -> Vec<Voter<Self::NodeId, Self::PublicKey, Self::Weight>> {
            alloc::vec![self.voter.clone()]
        }

        fn compute_role(&self, _epoch_hash: &Self::EpochHash) -> Role {
            Role::Proposer
        }
    }

    struct SingleNetwork {
        sender: Sender<Packet<u64, u64, Vec<u8>>>,
        recver: Receiver<Packet<u64, u64, Vec<u8>>>,
    }

    impl SingleNetwork {
        pub fn new() -> Self {
            let (sender, recver) = unbounded();

            Self { sender, recver }
        }
    }

    impl Network<Vec<u8>, Vec<u8>, u64, u64> for SingleNetwork {
        type Error = String;

        type Signature = Vec<u8>;

        type RecvFuture = Pin<
            Box<
                dyn Future<
                    Output = Result<(Packet<u64, u64, Self::Signature>, Vec<u8>), Self::Error>,
                >,
            >,
        >;

        fn node_id(&self) -> Vec<u8> {
            alloc::vec![1]
        }

        fn send_unsigned(&self, _target: Option<Vec<u8>>, pkt: Packet<u64, u64, Self::Signature>) {
            self.sender.try_send(pkt).unwrap();
        }

        fn recv(&self) -> Self::RecvFuture {
            let recver = self.recver.clone();
            let node_id = self.node_id();

            Box::pin(async move {
                let mut pkt = recver.recv().await.unwrap();

                let sign = VoteSign {
                    idx: 0,
                    sign: alloc::vec![1],
                };

                match &mut pkt {
                    Packet::BroadcastPropose(rp) => rp.vote_sign = Some(sign),
                    Packet::ResponsePropose(rp) => rp.vote_sign = Some(sign),
                    _ => {}
                }

                Ok((pkt, node_id))
            })
        }
    }

    #[test]
    fn signle_node() {
        init();

        let network = SingleNetwork::new();
        let app = SingleApp::new();
        let consensus = SingleConsensus::new();

        let mut braft = BRaft::new(network, consensus, app);

        smol::block_on(async move {
            for _ in 0..30 {
                braft.do_tick().await.unwrap();
            }
        })
    }
}
