use core::future::Future;

use crate::{messages::Packet, Role};

// #[async_trait]
pub trait Network<I, H> {
    type NodeId;

    type PublicKey;

    type Signature;

    type RecvFuture: Future<Output = Option<(Packet<I, H, Self::Signature>, Self::NodeId)>>;

    fn sign_and_send(&self, target: Option<Self::NodeId>, pkt: Packet<I, H, Self::Signature>);

    fn recv(&self) -> Self::RecvFuture;
}

pub trait App<I, H> {
    fn enter_step(&mut self, step: u8, epoch_id: I, epoch_hash: H);

    fn commit(&mut self, epoch_id: I, epoch_hash: H);
}

pub trait Consensus {
    type EpochId;

    type EpochHash;

    type Timer: Future<Output = ()> + Sized;

    fn step_timer(&self, step: u8) -> Self::Timer;

    fn role(&self, epoch_hash: &Self::EpochHash) -> Role;
}
