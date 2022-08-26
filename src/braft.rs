use crate::{
    messages::Packet,
    prelude::{Consensus, Network},
    utils, Error, Result, Role,
};

pub struct BRaft<N, A, C> {
    pub network: N,
    pub app: A,
    pub consensus: C,
    pub role: Role,
    pub epoch_id: u64,
    pub round: u64,
    pub step: u8,
}

impl<N, A, C> BRaft<N, A, C>
where
    N: Network<C::EpochId, C::EpochHash>,
    C: Consensus,
{
    /// Inital state.
    pub fn new() -> Result<()> {
        Ok(())
    }

    pub async fn trig(&mut self) -> Result<()> {
        use futures_lite::future::FutureExt;
        use utils::MapFutureExt;

        if self.role.is_follower() && self.step == 0 {
            let timer = self
                .consensus
                .step_timer(self.step)
                .map(|_| -> Option<(Packet<C::EpochId, C::EpochHash>, N::NodeId)> { None });

            if let Some((pkt, sender)) = self.network.recv().or(timer).await {
                if let Packet::BroadcastPropose(bc) = pkt {
                    // Sign and send
                    self.network.sign_and_send(Some(sender), Packet::response_from_broadcast_propose(bc));
                } else {
                    return Err(Error::NoneTimer);
                }
            } else {
                self.round = 1;
                self.step = 0;
            }
        } else if self.role.is_follower() && self.step == 1 {

        }

        Ok(())
    }
}
