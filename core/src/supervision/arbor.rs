//! Supervision Tree (Arbor)
//!
//! > *"Arbor magna e semine parvo crescit"*
//! > — A great tree grows from a small seed. (Latin)
//!
//! This module implements the core supervision tree structure.

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[cfg(test)]
use super::ModusRestitutio;
use super::{
    ActioRestitutio, InfansCurrens, InfansSpec, OrdoTerminatio, RestartDecision, StatusInfantis,
    StrategiaSupervisionis, SupervisioError, SupervisioResult,
};

// =============================================================================
// Arbor ID
// =============================================================================

/// Counter for unique arbor IDs.
static ARBOR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique identifier for a supervision tree node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArborId(u64);

impl ArborId {
    /// Generate a new unique arbor ID.
    #[inline]
    pub fn new() -> Self {
        ArborId(ARBOR_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value.
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for ArborId {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for ArborId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Arbor({})", self.0)
    }
}

// =============================================================================
// Arbor State
// =============================================================================

/// State of the supervision tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusArbor {
    /// Tree is initializing.
    Initians,
    /// Tree is running.
    Currens,
    /// Tree is stopping.
    Terminans,
    /// Tree has stopped.
    Terminatus,
}

// =============================================================================
// Arbor - Supervision Tree
// =============================================================================

/// A supervision tree node.
///
/// Manages a collection of children with a specified restart strategy.
///
/// # Latin Etymology
/// *Arbor* = tree.
///
/// # Example
///
/// ```rust
/// use core::time::Duration;
/// use ordofp_core::supervision::{Arbor, InfansSpec, StrategiaSupervisionis};
///
/// let supervisor = Arbor::new("my_supervisor")
///     .with_strategy(StrategiaSupervisionis::unus_pro_uno(3, Duration::from_secs(60)))
///     .add_child(InfansSpec::worker("worker1"))
///     .add_child(InfansSpec::worker("worker2"));
/// # let _ = supervisor;
/// ```
pub struct Arbor {
    /// Unique identifier.
    id: ArborId,

    /// Human-readable name.
    name: String,

    /// Supervision strategy.
    strategy: StrategiaSupervisionis,

    /// Child specifications.
    children_specs: Vec<InfansSpec>,

    /// Running children.
    children: Vec<InfansCurrens>,

    /// Current state.
    status: StatusArbor,

    /// Shutdown order.
    shutdown_order: OrdoTerminatio,

    /// Whether the supervisor should stop.
    stopping: AtomicBool,
}

impl Arbor {
    /// Create a new supervision tree.
    pub fn new(name: impl Into<String>) -> Self {
        Arbor {
            id: ArborId::new(),
            name: name.into(),
            strategy: StrategiaSupervisionis::default(),
            children_specs: Vec::with_capacity(4),
            children: Vec::with_capacity(4),
            status: StatusArbor::Initians,
            shutdown_order: OrdoTerminatio::default(),
            stopping: AtomicBool::new(false),
        }
    }

    /// Set the supervision strategy.
    #[inline]
    pub fn with_strategy(mut self, strategy: StrategiaSupervisionis) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the shutdown order.
    #[inline]
    pub fn with_shutdown_order(mut self, order: OrdoTerminatio) -> Self {
        self.shutdown_order = order;
        self
    }

    /// Add a child specification.
    #[inline]
    pub fn add_child(mut self, spec: InfansSpec) -> Self {
        self.children_specs.push(spec);
        self
    }

    /// Add multiple child specifications.
    #[inline]
    pub fn add_children(mut self, specs: impl IntoIterator<Item = InfansSpec>) -> Self {
        self.children_specs.extend(specs);
        self
    }

    /// Get the arbor ID.
    #[inline]
    pub fn id(&self) -> ArborId {
        self.id
    }

    /// Get the arbor name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the current status.
    #[inline]
    pub fn status(&self) -> StatusArbor {
        self.status
    }

    /// Get the number of children.
    #[inline]
    pub fn child_count(&self) -> usize {
        self.children_specs.len()
    }

    /// Get the strategy.
    #[inline]
    pub fn strategy(&self) -> &StrategiaSupervisionis {
        &self.strategy
    }

    /// Get the child specifications.
    #[inline]
    pub fn children_specs(&self) -> &[InfansSpec] {
        &self.children_specs
    }

    /// Get the running children.
    #[inline]
    pub fn children(&self) -> &[InfansCurrens] {
        &self.children
    }

    /// Check if the supervisor is stopping.
    #[inline]
    pub fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }

    /// Request the supervisor to stop.
    #[inline]
    pub fn request_stop(&self) {
        self.stopping.store(true, Ordering::Release);
    }

    /// Initialize the supervision tree.
    ///
    /// Creates `InfansCurrens` instances for each child specification.
    ///
    /// # Errors
    ///
    /// Currently infallible — always returns `Ok(())`. The `Result` return
    /// type is kept so that child startup can become fallible without a
    /// breaking API change.
    pub fn initialize(&mut self) -> SupervisioResult<()> {
        // Sort children by priority
        let mut specs = self.children_specs.clone();
        specs.sort_by_key(super::infans::InfansSpec::priority);

        // Create running children
        self.children = specs.into_iter().map(InfansCurrens::new).collect();

        self.status = StatusArbor::Currens;
        Ok(())
    }

    /// Handle a child failure.
    ///
    /// Applies the supervision strategy and returns the action to take.
    ///
    /// `normal_exit`: true if the child completed without error — restart
    /// modes (`ModusRestitutio`) distinguish normal from abnormal exits.
    ///
    /// # Errors
    ///
    /// - [`SupervisioError::Alius`] if `child_index` is out of bounds (the
    ///   tree was not initialized, or the index exceeds the child count).
    /// - [`SupervisioError::IntensitasExcedit`] when the strategy escalates:
    ///   the restart budget within the intensity window is exhausted, so the
    ///   failure must propagate to the parent supervisor.
    pub fn handle_failure(
        &mut self,
        child_index: usize,
        current_time: u64,
        normal_exit: bool,
    ) -> SupervisioResult<ActioRestitutio> {
        if child_index >= self.children.len() {
            return Err(SupervisioError::Alius("invalid child index".into()));
        }

        // Consult the child's restart mode BEFORE burning strategy/intensity
        // budget: a Temporarius child never restarts; a Transiens child
        // restarts only on abnormal exit.
        if !self.children[child_index]
            .spec()
            .modus()
            .should_restart(normal_exit)
        {
            self.children[child_index].mark_terminated();
            return Ok(ActioRestitutio::Ignorare);
        }

        // Mark the child as failed
        self.children[child_index].mark_failed();

        // Get the restart decision from the strategy
        let decision = self
            .strategy
            .decide(child_index, self.children.len(), current_time);

        match decision {
            RestartDecision::Restart(indices) => {
                // Record restarts for affected children
                for &idx in &indices {
                    if idx < self.children.len() {
                        self.children[idx].record_restart(current_time);
                    }
                }
                Ok(ActioRestitutio::Restituere)
            }
            RestartDecision::Escalate => {
                let child = &self.children[child_index];
                Err(SupervisioError::IntensitasExcedit {
                    child_id: child.spec().id().into(),
                    restarts: child.restart_count(),
                    // Sourced from the strategy's own intensity window;
                    // strategies without a windowed tracker (Simplex,
                    // Proprius) fall back to the 60s default.
                    window_secs: self.strategy.window_secs().unwrap_or(60),
                })
            }
            RestartDecision::Terminate => {
                self.status = StatusArbor::Terminans;
                Ok(ActioRestitutio::Terminare)
            }
            RestartDecision::Ignore => Ok(ActioRestitutio::Ignorare),
        }
    }

    /// Get the indices of children to restart based on shutdown order.
    pub fn shutdown_indices(&self) -> Vec<usize> {
        let count = self.children.len();
        match self.shutdown_order {
            OrdoTerminatio::Primus => (0..count).collect(),
            OrdoTerminatio::Ultimus => (0..count).rev().collect(),
            OrdoTerminatio::Simul => (0..count).collect(),
        }
    }

    /// Mark a child as running.
    #[inline]
    pub fn mark_child_running(&mut self, index: usize) {
        if index < self.children.len() {
            self.children[index].mark_running();
        }
    }

    /// Mark a child as terminated.
    #[inline]
    pub fn mark_child_terminated(&mut self, index: usize) {
        if index < self.children.len() {
            self.children[index].mark_terminated();
        }
    }

    /// Check if all children are running.
    #[inline]
    pub fn all_children_running(&self) -> bool {
        self.children
            .iter()
            .all(|c| c.status() == StatusInfantis::Currens)
    }

    /// Check if all children are terminated.
    #[inline]
    pub fn all_children_terminated(&self) -> bool {
        self.children
            .iter()
            .all(|c| c.status() == StatusInfantis::Terminatus)
    }

    /// Get a summary of child statuses.
    pub fn status_summary(&self) -> ArborSummary {
        let mut running = 0;
        let mut failed = 0;
        let mut restarting = 0;
        let mut terminated = 0;

        for child in &self.children {
            match child.status() {
                StatusInfantis::Currens => running += 1,
                StatusInfantis::Defectus => failed += 1,
                StatusInfantis::Restituens => restarting += 1,
                StatusInfantis::Terminatus => terminated += 1,
                StatusInfantis::Incipiens => {}
            }
        }

        ArborSummary {
            total: self.children.len(),
            running,
            failed,
            restarting,
            terminated,
        }
    }
}

impl core::fmt::Debug for Arbor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Arbor")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("status", &self.status)
            .field("children", &self.children.len())
            .finish_non_exhaustive()
    }
}

// =============================================================================
// Arbor Summary
// =============================================================================

/// Summary of supervision tree status.
#[derive(Debug, Clone, Copy, Default)]
pub struct ArborSummary {
    /// Total number of children.
    pub total: usize,
    /// Number of running children.
    pub running: usize,
    /// Number of failed children.
    pub failed: usize,
    /// Number of restarting children.
    pub restarting: usize,
    /// Number of terminated children.
    pub terminated: usize,
}

impl ArborSummary {
    /// Check if all children are healthy (running).
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.running == self.total && self.failed == 0
    }

    /// Get the percentage of running children.
    #[inline]
    pub fn health_percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.running as f64 / self.total as f64) * 100.0
        }
    }
}

// =============================================================================
// Arbor Builder
// =============================================================================

/// Builder for creating supervision trees with a fluent API.
pub struct ArborBuilder {
    arbor: Arbor,
}

impl ArborBuilder {
    /// Create a new arbor builder.
    pub fn new(name: impl Into<String>) -> Self {
        ArborBuilder {
            arbor: Arbor::new(name),
        }
    }

    /// Set the supervision strategy.
    pub fn strategy(mut self, strategy: StrategiaSupervisionis) -> Self {
        self.arbor.strategy = strategy;
        self
    }

    /// Set the shutdown order.
    pub fn shutdown_order(mut self, order: OrdoTerminatio) -> Self {
        self.arbor.shutdown_order = order;
        self
    }

    /// Add a worker child.
    pub fn worker(mut self, id: impl Into<String>) -> Self {
        self.arbor.children_specs.push(InfansSpec::worker(id));
        self
    }

    /// Add a supervisor child.
    pub fn supervisor(mut self, id: impl Into<String>) -> Self {
        self.arbor.children_specs.push(InfansSpec::supervisor(id));
        self
    }

    /// Add a child with a custom specification.
    pub fn child(mut self, spec: InfansSpec) -> Self {
        self.arbor.children_specs.push(spec);
        self
    }

    /// Build the supervision tree.
    pub fn build(self) -> Arbor {
        self.arbor
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_arbor_id_unique() {
        let id1 = ArborId::new();
        let id2 = ArborId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_arbor_new() {
        let arbor = Arbor::new("test_supervisor");
        assert_eq!(arbor.name(), "test_supervisor");
        assert_eq!(arbor.status(), StatusArbor::Initians);
        assert_eq!(arbor.child_count(), 0);
    }

    #[test]
    fn test_arbor_add_children() {
        let arbor = Arbor::new("supervisor")
            .add_child(InfansSpec::worker("worker1"))
            .add_child(InfansSpec::worker("worker2"));

        assert_eq!(arbor.child_count(), 2);
    }

    #[test]
    fn test_arbor_builder() {
        let arbor = ArborBuilder::new("supervisor")
            .strategy(StrategiaSupervisionis::simplex())
            .worker("worker1")
            .worker("worker2")
            .build();

        assert_eq!(arbor.child_count(), 2);
        assert_eq!(arbor.strategy().name(), "Simplex");
    }

    #[test]
    fn test_arbor_initialize() {
        let mut arbor = Arbor::new("supervisor")
            .add_child(InfansSpec::worker("worker1"))
            .add_child(InfansSpec::worker("worker2"));

        arbor
            .initialize()
            .expect("arbor with two workers should initialize without error");

        assert_eq!(arbor.status(), StatusArbor::Currens);
        assert_eq!(arbor.children().len(), 2);
    }

    #[test]
    fn test_arbor_handle_failure() {
        let mut arbor = Arbor::new("supervisor")
            .with_strategy(StrategiaSupervisionis::simplex())
            .add_child(InfansSpec::worker("worker1"));

        arbor
            .initialize()
            .expect("arbor with one worker should initialize without error");

        let action = arbor
            .handle_failure(0, 1000, false)
            .expect("simplex strategy should return a restart action for a valid child index");
        assert_eq!(action, ActioRestitutio::Restituere);
    }

    /// M13 regression: a Temporarius ("never restart") child must not be
    /// restarted; pre-fix the modus was never consulted.
    #[test]
    fn temporarius_child_is_not_restarted() {
        let mut arbor = Arbor::new("supervisor")
            .with_strategy(StrategiaSupervisionis::simplex())
            .add_child(InfansSpec::worker("worker1").with_modus(ModusRestitutio::Temporarius));

        arbor
            .initialize()
            .expect("arbor with one worker should initialize without error");

        let actio = arbor.handle_failure(0, 0, false).unwrap();
        assert_eq!(actio, ActioRestitutio::Ignorare);
    }

    /// Transiens restarts on abnormal exit only.
    #[test]
    fn transiens_child_normal_vs_abnormal() {
        let mut arbor = Arbor::new("supervisor")
            .with_strategy(StrategiaSupervisionis::simplex())
            .add_child(InfansSpec::worker("worker1").with_modus(ModusRestitutio::Transiens));

        arbor
            .initialize()
            .expect("arbor with one worker should initialize without error");
        assert_eq!(
            arbor.handle_failure(0, 0, true).unwrap(),
            ActioRestitutio::Ignorare
        );

        let mut arbor = Arbor::new("supervisor")
            .with_strategy(StrategiaSupervisionis::simplex())
            .add_child(InfansSpec::worker("worker1").with_modus(ModusRestitutio::Transiens));

        arbor
            .initialize()
            .expect("arbor with one worker should initialize without error");
        assert_eq!(
            arbor.handle_failure(0, 0, false).unwrap(),
            ActioRestitutio::Restituere
        );
    }

    #[test]
    fn test_arbor_summary() {
        let mut arbor = Arbor::new("supervisor")
            .add_child(InfansSpec::worker("worker1"))
            .add_child(InfansSpec::worker("worker2"))
            .add_child(InfansSpec::worker("worker3"));

        arbor
            .initialize()
            .expect("arbor with three workers should initialize without error");

        // Mark some children as running
        arbor.mark_child_running(0);
        arbor.mark_child_running(1);

        let summary = arbor.status_summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.running, 2);
    }

    #[test]
    fn test_arbor_shutdown_order() {
        let mut arbor = Arbor::new("supervisor")
            .with_shutdown_order(OrdoTerminatio::Ultimus)
            .add_child(InfansSpec::worker("worker1"))
            .add_child(InfansSpec::worker("worker2"))
            .add_child(InfansSpec::worker("worker3"));

        arbor.initialize().expect(
            "arbor with three workers and Ultimus shutdown order should initialize without error",
        );

        let indices = arbor.shutdown_indices();
        assert_eq!(indices, vec![2, 1, 0]);
    }
}
