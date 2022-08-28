use core::{pin::Pin, time::Duration};

use futures_lite::Future;
use smol::{
    channel::{unbounded, Receiver, Sender},
    Timer,
};
use std::{boxed::Box, string::String, vec::Vec};

use consensus_rs::{packet::Packet, App, Consensus, Network, Role, VoteSign, Voter};

pub struct SingleApp {
    pub epoch_id: u64,
    pub epoch_hash: u64,
    pub voter: Voter<Vec<u8>, Vec<u8>, u64>,
}

impl SingleApp {
    pub fn new() -> Self {
        let voter = Voter {
            voter_id: vec![1],
            public_key: vec![1],
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

    fn enter_step(&mut self, _step: u8, epoch_id: u64, epoch_hash: u64) -> Self::EnterStepFuture {
        Box::pin(async move { Ok((epoch_id, epoch_hash)) })
    }

    fn commit(&mut self, epoch_id: &u64, epoch_hash: &u64) -> Self::CommitFuture {
        self.epoch_id = *epoch_id;
        self.epoch_hash = *epoch_hash;

        let voter = vec![self.voter.clone()];

        Box::pin(async move { Ok(voter) })
    }
}

pub struct SingleConsensus {
    pub voter: Voter<Vec<u8>, Vec<u8>, u64>,
    pub proposer: Vec<u8>,
}

impl SingleConsensus {
    pub fn new(proposer: Vec<u8>) -> Self {
        let voter = Voter {
            voter_id: vec![1],
            public_key: vec![1],
            weight: 1,
        };

        Self { voter, proposer }
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
        vec![self.voter.clone()]
    }

    fn compute_proposer(&self, _epoch_hash: &Self::EpochHash) -> Self::NodeId {
        self.proposer.clone()
    }
}

pub struct SingleNetwork {
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
        Box<dyn Future<Output = Result<(Packet<u64, u64, Self::Signature>, Vec<u8>), Self::Error>>>,
    >;

    fn node_id(&self) -> Vec<u8> {
        vec![1]
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
                sign: vec![1],
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
