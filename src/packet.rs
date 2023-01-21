//! Packet send and receive from network.

use crate::Step;

#[derive(Debug)]
pub struct BroadcastPropose<I, H> {
    pub epoch_id: I,
    pub epoch_hash: H,
}

#[derive(Debug)]
pub struct BroadcastStep<I, H> {
    pub epoch_id: I,
    pub epoch_hash: H,
    pub step: Step,
}

#[derive(Debug)]
pub struct BroadcastCommit<I, H> {
    pub epoch_id: I,
    pub epoch_hash: H,
}

/// Packet for network
#[derive(Debug)]
pub enum Packet<I, H> {
    BroadcastPropose(BroadcastPropose<I, H>),
    BroadcastStep(BroadcastStep<I, H>),
    BroadcastCommit(BroadcastCommit<I, H>),
}
