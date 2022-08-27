//! Packet send and receive from network.

use alloc::vec::Vec;

use crate::VoteSign;

/// Broadcast propopse to other node
pub struct BroadcastPropose<I, H, S> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub vote_sign: Option<VoteSign<S>>,
}

/// Response propopse to proposer
pub struct ResponsePropose<I, H, S> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub vote_sign: Option<VoteSign<S>>,
}

/// Broadcast commit to other node
pub struct BroadcastCommit<I, H, S> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub vote_signs: Vec<VoteSign<S>>,
}

/// Packet for network
pub enum Packet<I, H, S> {
    BroadcastPropose(BroadcastPropose<I, H, S>),
    ResponsePropose(ResponsePropose<I, H, S>),
    BroadcastCommit(BroadcastCommit<I, H, S>),
}

impl<I, H, S> Packet<I, H, S> {
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
