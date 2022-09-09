//! Packet send and receive from network.

use alloc::vec::Vec;

use crate::{EpochHash, EpochId, Signature, VoteSign};

/// Broadcast propopse to other node
#[derive(Debug)]
pub struct BroadcastPropose<I: EpochId, H: EpochHash, S: Signature> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub vote_sign: Option<VoteSign<S>>,
}

/// Response propopse to proposer
#[derive(Debug)]
pub struct ResponsePropose<I: EpochId, H: EpochHash, S: Signature> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub vote_sign: Option<VoteSign<S>>,
}

/// Broadcast commit to other node
#[derive(Debug)]
pub struct BroadcastCommit<I: EpochId, H: EpochHash, S: Signature> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub vote_signs: Vec<VoteSign<S>>,
}

/// Packet for network
#[derive(Debug)]
pub enum Packet<I: EpochId, H: EpochHash, S: Signature> {
    BroadcastPropose(BroadcastPropose<I, H, S>),
    ResponsePropose(ResponsePropose<I, H, S>),
    BroadcastCommit(BroadcastCommit<I, H, S>),
}

impl<I: EpochId, H: EpochHash, S: Signature> Packet<I, H, S> {
    pub fn is_broadcast_propose(&self) -> bool {
        matches!(self, Packet::BroadcastPropose(_))
    }

    pub fn is_response_propose(&self) -> bool {
        matches!(self, Packet::ResponsePropose(_))
    }

    pub fn response_propose_from_id_hash(epoch_id: I, epoch_hash: H) -> Self {
        Self::ResponsePropose(ResponsePropose {
            epoch_id,
            epoch_hash,
            vote_sign: None,
        })
    }

    pub fn broadcast_propose_from_id_hash(epoch_id: I, epoch_hash: H) -> Self {
        Self::BroadcastPropose(BroadcastPropose {
            epoch_hash,
            epoch_id,
            vote_sign: None,
        })
    }

    pub fn broadcast_commit_from_id_hash(
        epoch_id: I,
        epoch_hash: H,
        vote_signs: Vec<VoteSign<S>>,
    ) -> Self {
        Self::BroadcastCommit(BroadcastCommit {
            epoch_hash,
            epoch_id,
            vote_signs,
        })
    }
}
