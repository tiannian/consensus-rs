use async_trait::async_trait;

use crate::{
    braft::Role,
    messages::{RecvPacket, SendPacket},
};

#[async_trait]
pub trait Network<I, H, S> {
    fn send(&self, pkt: SendPacket<I, H>);

    async fn recv(&self) -> Option<RecvPacket<I, H, S>>;
}

pub trait App {}

pub trait Consensus<H> {
    fn role(&self, epoch_hash: &H) -> Role;
}
