use core::{
    fmt::Debug,
    future::Future,
    ops::{Add, AddAssign},
};

use alloc::vec::Vec;
use num_traits::{One, Zero};

use crate::{messages::Packet, Role, Voter};

// #[async_trait]
pub trait Network<N, P, I, H> {
    type Error: Debug + 'static;

    type Signature: Clone;

    fn node_id(&self) -> N;

    fn sign_and_send(&self, target: Option<N>, pkt: Packet<I, H, Self::Signature>);

    type RecvFuture: Future<Output = Result<(Packet<I, H, Self::Signature>, N), Self::Error>>;
    fn recv(&self) -> Self::RecvFuture;
}

pub trait App<N, P, W, I, H> {
    type Error: Debug + 'static;

    type ProposeEpochFuture: Future<Output = Result<(I, H), Self::Error>>;
    fn propose_epoch(&mut self) -> Self::ProposeEpochFuture;

    type ProcessStepFuture: Future<Output = Result<(I, H), Self::Error>>;
    fn enter_step(&mut self, step: u8, epoch_id: I, epoch_hash: H) -> Self::ProcessStepFuture;

    type CommitFuture: Future<Output = Result<(), Self::Error>>;
    fn commit(&mut self, epoch_id: I, epoch_hash: H) -> Self::CommitFuture;

    type VoterSetFuture: Future<Output = Result<Vec<Voter<N, P, W>>, Self::Error>>;
    fn voter_set(&self, epoch_hash: &H) -> Self::VoterSetFuture;
}

pub trait EpochId: Clone + Eq + Ord + Debug + Add<Output = Self> + One + Zero {}

pub trait EpochHash: Clone + Eq + Debug {}

pub trait NodeId: Clone + Eq {}

pub trait Weight: Clone + Zero + AddAssign + Eq {}

pub trait Consensus {
    type NodeId: NodeId;

    type PublicKey: Clone;

    type EpochId: EpochId;

    type EpochHash: EpochHash;

    type Weight: Weight;

    /// Got latest epoch.
    ///
    /// This method only call on node startup.
    fn latest_epoch(&self) -> (Self::EpochId, Self::EpochHash);

    /// Got latest voter set.
    ///
    /// This method only call on node startup.
    fn latest_voter_set(&self) -> Vec<Voter<Self::NodeId, Self::PublicKey, Self::Weight>>;

    /// Future of `step_timer`.
    type Timer: Future<Output = ()>;
    /// Build a timer for step.
    fn step_timer(&self, role: &Role, step: u8) -> Self::Timer;

    /// Compute proposer based on epoch hash.
    fn compute_role(&self, epoch_hash: &Self::EpochHash) -> Role;
}
