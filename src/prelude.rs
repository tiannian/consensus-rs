use core::future::Future;

use crate::{messages::Packet, Role};

// #[async_trait]
pub trait Network<I, H> {
    type NodeId;

    type RecvFuture: Future<Output = Option<(Packet<I, H>, Self::NodeId)>>;

    fn sign_and_send(&self, target: Option<Self::NodeId>, pkt: Packet<I, H>);

    fn recv(&self) -> Self::RecvFuture;
}

pub trait App {}

pub trait Consensus {
    type EpochId;

    type EpochHash;

    type Timer: Future<Output = ()> + Sized;

    fn step_timer(&self, step: u8) -> Self::Timer;

    fn role(&self, epoch_hash: &Self::EpochHash) -> Role;
}
