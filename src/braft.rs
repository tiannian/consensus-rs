use alloc::vec::Vec;

use crate::{
    messages::{BroadcastCommit, BroadcastPropose, Packet},
    prelude::{App, Consensus, Network},
    utils, Error, Result, Role, VoteSign,
};

pub struct BRaft<N, A, C>
where
    C: Consensus,
    N: Network<C::EpochId, C::EpochHash>,
{
    pub network: N,
    pub app: A,
    pub consensus: C,
    pub role: Role,
    pub epoch_id: C::EpochId,
    pub epoch_hash: C::EpochHash,
    pub vote_signs: Vec<VoteSign<N::Signature>>,
    pub round: u64,
    pub step: u8,
}

type EpochId<C> = <C as Consensus>::EpochId;
type EpochHash<C> = <C as Consensus>::EpochHash;
type Signature<C, N> = <N as Network<EpochId<C>, EpochHash<C>>>::Signature;
type Pkt<C, N> = Packet<EpochId<C>, EpochHash<C>, Signature<C, N>>;

impl<N, A, C> BRaft<N, A, C>
where
    N: Network<C::EpochId, C::EpochHash>,
    C: Consensus,
    A: App<C::EpochId, C::EpochHash>,
{
    /// Inital state.
    pub fn new() -> Result<()> {
        Ok(())
    }

    pub async fn trig(&mut self) -> Result<()> {
        use futures_lite::future::FutureExt;
        use utils::MapFutureExt;

        let timer_map_helper = |_| -> Option<(Pkt<C, N>, N::NodeId)> { None };

        if self.role.is_follower() && self.step == 0 {
            let timer = self.consensus.step_timer(self.step).map(timer_map_helper);

            if let Some((pkt, sender)) = self.network.recv().or(timer).await {
                match pkt {
                    Packet::BroadcastPropose(bc) => self.process_propose(sender, bc),
                    Packet::ResponsePropose(_) => {}
                    Packet::BroadcastCommit(bc) => self.accept_epoch(bc),
                }
            } else {
                self.round = 1;
                self.step = 0;
            }
        } else if self.role.is_follower() && self.step == 1 {
            let timer = self.consensus.step_timer(self.step).map(timer_map_helper);

            if let Some((pkt, sender)) = self.network.recv().or(timer).await {

            }
        }

        Ok(())
    }

    fn process_propose(&mut self, sender: N::NodeId, pkt: BroadcastPropose<C::EpochId, C::EpochHash>) {
        self.network
            .sign_and_send(Some(sender), Packet::response_from_broadcast_propose(pkt));
        self.app.enter_step(0, pkt.epoch_id, pkt.epoch_hash);
    }

    fn accept_epoch(&mut self, pkt: BroadcastCommit<C::EpochId, C::EpochHash, N::Signature>) {
        self.vote_signs = Vec::new();
        self.epoch_id = pkt.epoch_id;
        self.epoch_hash = pkt.epoch_hash;
    }
}
