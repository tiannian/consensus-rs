pub struct Voter<V, P> {
    pub voter_id: V,
    pub public_key: P,
}

pub struct VoteSign<S> {
    pub vs_idx: u64,
    pub sign: S,
}

pub enum Role {
    Proposer,
    Follower,
    Observer,
}

impl Role {
    pub fn is_proposer(&self) -> bool {
        matches!(self, Role::Proposer)
    }

    pub fn is_follower(&self) -> bool {
        matches!(self, Role::Follower)
    }

    pub fn is_observer(&self) -> bool {
        matches!(self, Role::Observer)
    }
}
