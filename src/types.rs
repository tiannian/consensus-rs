use core::ops::{Deref, DerefMut};

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

#[derive(Debug)]
pub struct Step(pub u8);

impl AsRef<u8> for Step {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl Deref for Step {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Step {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
