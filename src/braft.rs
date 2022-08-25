use crate::{prelude::{Consensus, Network}, Result};

pub enum Role {
    Proposer,
    Follower,
    Observer,
}

pub struct BRaft<H, N, A, C> {
    pub network: N,
    pub app: A,
    pub consensus: C,
    pub role: Role,
    pub epoch_id: u64,
    pub latest_epoch_hash: H,
    pub round: u64,
    pub step: u64,
}

impl<I, H, S, N, A, C> BRaft<H, N, A, C>
where
    N: Network<I, H, S>,
    C: Consensus<H>,
{
    pub fn trig(&mut self) -> Result<()> {

        let role = self.consensus.role(&self.latest_epoch_hash);

        Ok(())
    }
}
