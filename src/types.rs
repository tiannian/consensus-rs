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

pub enum ErrorBehavior {
    Panic,
    Ignore,
    Retry,
}

pub struct Voter<Id, Weight> {
    pub id: Id,
    pub weight: Weight,
}
