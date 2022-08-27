//! Traits define for all consensus.

use core::{
    fmt::Debug,
    future::Future,
    iter::Sum,
    ops::{Add, AddAssign, Mul},
};

use alloc::vec::Vec;
use num_traits::{One, Zero};

use crate::{packet::Packet, Role, Voter};

/// Network for node.
pub trait Network<N, P, I, H> {
    /// Error for underline network.
    type Error: Debug + 'static;

    /// Signature generate by node key.
    type Signature: Signature;

    /// Get current node key.
    fn node_id(&self) -> N;

    /// Sign packet and send to other node.
    fn send_unsigned(&self, target: Option<N>, pkt: Packet<I, H, Self::Signature>);

    /// Future for recv method.
    type RecvFuture: Future<Output = Result<(Packet<I, H, Self::Signature>, N), Self::Error>>;
    /// Receive packet from network.
    fn recv(&self) -> Self::RecvFuture;
}

/// Application.
pub trait App<N, P, W, I, H> {
    /// Application Error
    type Error: Debug + 'static;

    /// Future for propose_epoch method.
    type ProposeEpochFuture: Future<Output = Result<(I, H), Self::Error>>;

    /// Propose a epoch.
    ///
    /// When node propose a epoch, call this function.
    /// Only called by proposer.
    fn propose_epoch(&mut self) -> Self::ProposeEpochFuture;

    /// Future for enter_step
    type EnterStepFuture: Future<Output = Result<(I, H), Self::Error>>;
    /// Hook for step
    ///
    /// When a node enter a step, call this method.
    fn enter_step(&mut self, step: u8, epoch_id: I, epoch_hash: H) -> Self::EnterStepFuture;

    /// Future for commit
    type CommitFuture: Future<Output = Result<Vec<Voter<N, P, W>>, Self::Error>>;
    /// Commit hook
    ///
    /// Means all voter confirm this epoch.
    fn commit(&mut self, epoch_id: &I, epoch_hash: &H) -> Self::CommitFuture;
}

/// EpochId type.
pub trait EpochId: Clone + Eq + Ord + Debug + Add<Output = Self> + One + Zero {}
impl<T> EpochId for T where T: Clone + Eq + Ord + Debug + Add<Output = Self> + One + Zero {}

/// EpochHash type.
pub trait EpochHash: Clone + Eq + Debug {}
impl<T> EpochHash for T where T: Clone + Eq + Debug {}

/// NodeId type.
pub trait NodeId: Clone + Eq + Debug {}
impl<T> NodeId for T where T: Clone + Eq + Debug {}

/// Weight type.
pub trait Weight: Clone + Zero + One + AddAssign + Eq + Sum + Mul<Output = Self> + Ord {}
impl<T> Weight for T where T: Clone + Zero + One + AddAssign + Eq + Sum + Mul<Output = Self> + Ord {}

/// Signature
pub trait Signature: Clone + Debug {}
impl<T> Signature for T where T: Clone + Debug {}

/// Consensus
pub trait Consensus {
    /// NodeId
    type NodeId: NodeId;

    /// PublicKey
    type PublicKey: Clone;

    /// EpochId
    ///
    /// Like a number, will increase.
    /// Each epoch id will map a epoch hash when epoch commited.
    type EpochId: EpochId;

    /// EpochHash
    ///
    /// Hash of epoch data.
    type EpochHash: EpochHash;

    /// Weight
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
