//! Child Specifications for Supervision Trees
//!
//! > *"Infans est futura arbor"*
//! > — The child is the future tree. (Latin)
//!
//! This module defines the specifications for supervised children.

use alloc::boxed::Box;
use alloc::string::String;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::time::Duration;

use super::{ModusRestitutio, StatusInfantis};

/// Default start priority for worker children (lower = start earlier).
const DEFAULT_WORKER_PRIORITY: u32 = 100;

/// Default start priority for supervisor children (lower = start earlier).
const DEFAULT_SUPERVISOR_PRIORITY: u32 = 50;

// =============================================================================
// Child Specification
// =============================================================================

/// Specification for a supervised child.
///
/// Describes how to start, stop, and restart a child process.
///
/// # Latin Etymology
/// *Specificatio infantis* = child specification.
#[derive(Clone)]
pub struct InfansSpec {
    /// Unique identifier for the child.
    id: String,

    /// Type of child (worker or supervisor).
    genus: GenusInfantis,

    /// Restart mode.
    modus: ModusRestitutio,

    /// Shutdown timeout.
    shutdown_timeout: Duration,

    /// Start priority (lower = start earlier).
    priority: u32,
}

impl InfansSpec {
    /// Create a worker child specification.
    ///
    /// Workers are leaf nodes that do actual work.
    pub fn worker(id: impl Into<String>) -> Self {
        InfansSpec {
            id: id.into(),
            genus: GenusInfantis::Operarius,
            modus: ModusRestitutio::default(),
            shutdown_timeout: Duration::from_secs(5),
            priority: DEFAULT_WORKER_PRIORITY,
        }
    }

    /// Create a supervisor child specification.
    ///
    /// Supervisors manage other children.
    pub fn supervisor(id: impl Into<String>) -> Self {
        InfansSpec {
            id: id.into(),
            genus: GenusInfantis::Supervisor,
            modus: ModusRestitutio::Permanens,
            shutdown_timeout: Duration::from_secs(30),
            priority: DEFAULT_SUPERVISOR_PRIORITY,
        }
    }

    /// Set the restart mode.
    #[inline]
    pub fn with_modus(mut self, modus: ModusRestitutio) -> Self {
        self.modus = modus;
        self
    }

    /// Set the shutdown timeout.
    #[inline]
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Set the start priority.
    #[inline]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Get the child ID.
    #[inline]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the child type.
    #[inline]
    pub fn genus(&self) -> GenusInfantis {
        self.genus
    }

    /// Get the restart mode.
    #[inline]
    pub fn modus(&self) -> ModusRestitutio {
        self.modus
    }

    /// Get the shutdown timeout.
    #[inline]
    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }

    /// Get the start priority.
    #[inline]
    pub fn priority(&self) -> u32 {
        self.priority
    }
}

impl core::fmt::Debug for InfansSpec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InfansSpec")
            .field("id", &self.id)
            .field("genus", &self.genus)
            .field("modus", &self.modus)
            .field("priority", &self.priority)
            .finish_non_exhaustive()
    }
}

// =============================================================================
// Child Type
// =============================================================================

/// Type of supervised child.
///
/// # Latin Etymology
/// *Genus infantis* = type of child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenusInfantis {
    /// A worker that performs actual work.
    ///
    /// *Operarius* = worker.
    Operarius,

    /// A supervisor that manages other children.
    ///
    /// *Supervisor* = overseer.
    Supervisor,
}

// =============================================================================
// Running Child
// =============================================================================

/// State of a running supervised child.
///
/// # Latin Etymology
/// *Infans currens* = running child.
pub struct InfansCurrens {
    /// The child specification.
    spec: InfansSpec,

    /// Current status.
    status: StatusInfantis,

    /// Number of restarts.
    restart_count: u32,

    /// Time of last restart (Unix epoch seconds).
    last_restart_time: Option<u64>,
}

impl InfansCurrens {
    /// Create a new running child from a specification.
    pub fn new(spec: InfansSpec) -> Self {
        InfansCurrens {
            spec,
            status: StatusInfantis::Incipiens,
            restart_count: 0,
            last_restart_time: None,
        }
    }

    /// Get the child specification.
    #[inline]
    pub fn spec(&self) -> &InfansSpec {
        &self.spec
    }

    /// Get the current status.
    #[inline]
    pub fn status(&self) -> StatusInfantis {
        self.status
    }

    /// Set the current status.
    #[inline]
    pub fn set_status(&mut self, status: StatusInfantis) {
        self.status = status;
    }

    /// Get the restart count.
    #[inline]
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    /// Record a restart.
    #[inline]
    pub fn record_restart(&mut self, time: u64) {
        self.restart_count += 1;
        self.last_restart_time = Some(time);
        self.status = StatusInfantis::Restituens;
    }

    /// Mark as running.
    #[inline]
    pub fn mark_running(&mut self) {
        self.status = StatusInfantis::Currens;
    }

    /// Mark as failed.
    #[inline]
    pub fn mark_failed(&mut self) {
        self.status = StatusInfantis::Defectus;
    }

    /// Mark as terminated.
    #[inline]
    pub fn mark_terminated(&mut self) {
        self.status = StatusInfantis::Terminatus;
    }
}

impl core::fmt::Debug for InfansCurrens {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InfansCurrens")
            .field("id", &self.spec.id)
            .field("status", &self.status)
            .field("restart_count", &self.restart_count)
            .finish_non_exhaustive()
    }
}

// =============================================================================
// Child Factory
// =============================================================================

/// Factory for creating child processes.
///
/// This trait abstracts over how children are actually started.
///
/// # Latin Etymology
/// *Fabrica infantis* = child factory.
pub trait FabricaInfantis: Send + Sync {
    /// The output type of the child.
    type Output: Send + 'static;

    /// Create and start a child.
    fn create(&self) -> Pin<Box<dyn Future<Output = Self::Output> + Send>>;
}

/// A simple factory that wraps a closure.
pub struct FabricaSimplex<F, Fut, T>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    factory: F,
    _marker: PhantomData<(Fut, T)>,
}

impl<F, Fut, T> FabricaSimplex<F, Fut, T>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    /// Create a new simple factory.
    pub fn new(factory: F) -> Self {
        FabricaSimplex {
            factory,
            _marker: PhantomData,
        }
    }
}

impl<F, Fut, T> FabricaInfantis for FabricaSimplex<F, Fut, T>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = T> + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    type Output = T;

    fn create(&self) -> Pin<Box<dyn Future<Output = T> + Send>> {
        Box::pin((self.factory)())
    }
}

// =============================================================================
// Child Handle
// =============================================================================

/// Handle to a running child process.
///
/// # Latin Etymology
/// *Manubrium infantis* = child handle.
pub struct ManubriumInfantis<T> {
    /// The child specification.
    spec: InfansSpec,

    /// Phantom type for the output.
    _marker: PhantomData<T>,
}

impl<T> ManubriumInfantis<T> {
    /// Create a new child handle.
    pub fn new(spec: InfansSpec) -> Self {
        ManubriumInfantis {
            spec,
            _marker: PhantomData,
        }
    }

    /// Get the child specification.
    pub fn spec(&self) -> &InfansSpec {
        &self.spec
    }
}

impl<T> core::fmt::Debug for ManubriumInfantis<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ManubriumInfantis")
            .field("spec", &self.spec)
            .finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infans_spec_worker() {
        let spec = InfansSpec::worker("worker1");
        assert_eq!(spec.id(), "worker1");
        assert_eq!(spec.genus(), GenusInfantis::Operarius);
    }

    #[test]
    fn test_infans_spec_supervisor() {
        let spec = InfansSpec::supervisor("supervisor1");
        assert_eq!(spec.id(), "supervisor1");
        assert_eq!(spec.genus(), GenusInfantis::Supervisor);
    }

    #[test]
    fn test_infans_spec_builder() {
        let spec = InfansSpec::worker("worker1")
            .with_modus(ModusRestitutio::Transiens)
            .with_priority(10);

        assert_eq!(spec.modus(), ModusRestitutio::Transiens);
        assert_eq!(spec.priority(), 10);
    }

    #[test]
    fn test_infans_currens() {
        let spec = InfansSpec::worker("worker1");
        let mut child = InfansCurrens::new(spec);

        assert_eq!(child.status(), StatusInfantis::Incipiens);
        assert_eq!(child.restart_count(), 0);

        child.mark_running();
        assert_eq!(child.status(), StatusInfantis::Currens);

        child.record_restart(1000);
        assert_eq!(child.status(), StatusInfantis::Restituens);
        assert_eq!(child.restart_count(), 1);
    }
}
