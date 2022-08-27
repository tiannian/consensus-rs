/// Voter info
///
/// Map voter id and voter's public key.
/// The public key use to verify packet from other node and vote epoch.
/// We can give voter a weight.
pub struct Voter<V, P, W> {
    pub voter_id: V,
    pub public_key: P,
    pub weight: W,
}

/// Signature for voter
///
/// `idx` is the order of voter set.
pub struct VoteSign<S> {
    pub idx: u64,
    pub sign: S,
}

/// Node role
///
/// Proposer and Follower is Voter, do consensus among these.
/// Observer only sync data from other node.
#[derive(Debug, PartialEq, Eq)]
pub enum Role {
    Proposer,
    Follower,
    Observer,
}

impl Role {
    pub(crate) fn is_proposer(&self) -> bool {
        matches!(self, Role::Proposer)
    }

    pub(crate) fn is_follower(&self) -> bool {
        matches!(self, Role::Follower)
    }

    pub(crate) fn is_observer(&self) -> bool {
        matches!(self, Role::Observer)
    }
}
