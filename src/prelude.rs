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

    /// Signature
    type Signature: Signature;

    /// Future of latest_epoch
    type LatestEpochFuture: Future<Output = (Self::EpochId, Self::EpochHash)>;
    /// Get latest epoch.
    ///
    /// This method only call on node startup.
    fn latest_epoch(&self) -> Self::LatestEpochFuture;

    /// Future of latest_voter_set
    type LatestVoterSetFuture: Future<
        Output = Vec<Voter<Self::NodeId, Self::PublicKey, Self::Weight>>,
    >;
    /// Got latest voter set.
    ///
    /// This method only call on node startup.
    fn latest_voter_set(&self) -> Self::LatestVoterSetFuture;

    /// Future of `step_timer`.
    type Timer: Future<Output = ()>;
    /// Build a timer for step.
    fn step_timer(&self, role: &Role, step: u8) -> Self::Timer;

    /// Future of compute_proposer
    type ComputeProposerFuture: Future<Output = Self::NodeId>;
    /// Compute proposer based on epoch hash.
    fn compute_proposer(&self, epoch_hash: &Self::EpochHash) -> Self::ComputeProposerFuture;

    // TODO: Add EpochHash unique check.
}
/// Network for node.
pub trait Network<C: Consensus> {
    /// Error for underline network.
    type Error: Debug + 'static;

    /// Get current node key.
    fn node_id(&self) -> C::NodeId;

    /// Sign packet and send to other node.
    fn send_unsigned(
        &self,
        target: Option<C::NodeId>,
        pkt: Packet<C::EpochId, C::EpochHash, C::Signature>,
    );

    /// Future for recv method.
    type RecvFuture: Future<
        Output = Result<(Packet<C::EpochId, C::EpochHash, C::Signature>, C::NodeId), Self::Error>,
    >;
    /// Receive packet from network.
    fn recv(&self) -> Self::RecvFuture;
}

/// Application.
pub trait App<C: Consensus> {
    /// Application Error
    type Error: Debug + 'static;

    /// Future for propose_epoch method.
    type ProposeEpochFuture: Future<Output = Result<(C::EpochId, C::EpochHash), Self::Error>>;

    /// Propose a epoch.
    ///
    /// When node propose a epoch, call this function.
    /// Only called by proposer.
    fn propose_epoch(&mut self) -> Self::ProposeEpochFuture;

    /// Future for enter_step
    type EnterStepFuture: Future<Output = Result<(C::EpochId, C::EpochHash), Self::Error>>;
    /// Hook for step
    ///
    /// When a node enter a step, call this method.
    fn enter_step(
        &mut self,
        step: u8,
        epoch_id: C::EpochId,
        epoch_hash: C::EpochHash,
    ) -> Self::EnterStepFuture;

    /// Future for commit
    type CommitFuture: Future<
        Output = Result<Vec<Voter<C::NodeId, C::PublicKey, C::Weight>>, Self::Error>,
    >;
    /// Commit hook
    ///
    /// Means all voter confirm this epoch.
    fn commit(&mut self, epoch_id: &C::EpochId, epoch_hash: &C::EpochHash) -> Self::CommitFuture;
}
