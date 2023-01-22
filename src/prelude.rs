//! Traits define for all consensus.

use core::{
    fmt::Debug,
    iter::Sum,
    ops::{Add, AddAssign, Mul},
};

use num_traits::{One, Zero};

use crate::{ErrorBehavior, Packet, Role, Voter};

/// EpochId type.
///
/// Typical type is {integer}.
pub trait EpochId: Clone + Eq + Ord + Debug + Add<Output = Self> + One + Zero {}
impl<T> EpochId for T where T: Clone + Eq + Ord + Debug + Add<Output = Self> + One + Zero {}

/// EpochHash type.
///
/// Typical type is bytes: `Vec<u8>`, `[u8; N]`, or any type can compare with ==
pub trait EpochHash: Clone + Eq + PartialEq + Debug {}
impl<const N: usize, T: Clone + Eq + PartialEq + Debug> EpochHash for [T; N] {}

/// NodeId type.
///
/// Typical type is bytes: `Vec<u8>`, `[u8; N]`, or any type can compare with ==
pub trait NodeId: Clone + Eq + Debug {}
impl<const N: usize, T: Clone + Eq + PartialEq + Debug> NodeId for [T; N] {}

/// Weight type.
///
/// Typical type is {integer}.
pub trait Weight: Clone + Zero + One + AddAssign + Eq + Sum + Mul<Output = Self> + Ord {}
impl<T> Weight for T where T: Clone + Zero + One + AddAssign + Eq + Sum + Mul<Output = Self> + Ord {}

pub trait Error: Debug {
    fn behavior(&self) -> ErrorBehavior;
}

pub trait Core {
    type NodeId: NodeId;

    type EpochId: EpochId;

    type EpochHash: EpochHash;

    type Weight: Weight;

    type Error: Error;
}

pub trait Network: Core {
    fn node_id(&self) -> Self::NodeId;

    async fn send_unsigned(
        &self,
        target: Option<Self::NodeId>,
        packet: Packet<Self::EpochId, Self::EpochHash>,
    ) -> Result<(), Self::Error>;

    async fn recv_signed(
        &self,
    ) -> Result<(Self::NodeId, Packet<Self::EpochId, Self::EpochHash>), Self::Error>;
}

pub trait Setup: Core {
    /// Build a timer for step.
    async fn step_timer(&self, role: &Role, step: u8);
}

pub trait Startup: Core {
    /// Future of latest_epoch
    // type LatestEpochFuture: Future<Output = (Self::EpochId, Self::EpochHash)>;
    /// Get latest epoch.
    // fn latest_epoch(&self) -> Self::LatestEpochFuture;
    async fn latest_epoch(&self) -> (Self::EpochId, Self::EpochHash);

    /// Got latest voter set.
    async fn latest_voter_set(&self) -> &[Voter<Self::NodeId, Self::Weight>];
}

pub trait Consensus: Core {
    /// Propose a epoch.
    ///
    /// When node propose a epoch, call this function.
    /// Only called by proposer.
    async fn propose_epoch(&mut self) -> (Self::EpochId, Self::EpochHash);

    /// Hook for step
    ///
    /// When a node enter a step, call this method.
    async fn enter_step(
        &mut self,
        step: u8,
        epoch_id: &Self::EpochId,
        epoch_hash: &Self::EpochHash,
    ) -> Result<(), Self::Error>;

    /// Commit hook
    ///
    /// Means all voter confirm this epoch.
    async fn commit(
        &mut self,
        epoch_id: &Self::EpochId,
        epoch_hash: &Self::EpochHash,
    ) -> Result<(), Self::Error>;
}
