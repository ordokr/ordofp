//! Fiber Metrics
//!
//! > *"Fibrae numeratae sunt fibrae gubernatae"*
//! > — Counted fibers are governed fibers. (Latin)
//!
//! This module provides metrics for fiber execution.

use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{Distributio, Indicium, Numerator};

// =============================================================================
// Fiber Metrics
// =============================================================================

/// Metrics for fiber execution.
///
/// # Latin Etymology
/// *Mensura fibrae* = fiber measure.
#[derive(Debug)]
pub struct MensuraFibrae {
    /// Total fibers spawned.
    spawned: Numerator,
    /// Successfully completed fibers.
    completed: Numerator,
    /// Failed fibers.
    failed: Numerator,
    /// Cancelled fibers.
    cancelled: Numerator,
    /// Currently active fibers.
    active: Indicium,
    /// Fiber lifetime distribution (nanoseconds).
    lifetime: Distributio,
    /// Peak concurrent fibers.
    peak_active: AtomicU64,
}

impl MensuraFibrae {
    /// Create new fiber metrics.
    pub fn new() -> Self {
        MensuraFibrae {
            spawned: Numerator::new(),
            completed: Numerator::new(),
            failed: Numerator::new(),
            cancelled: Numerator::new(),
            active: Indicium::new(),
            lifetime: Distributio::new(),
            peak_active: AtomicU64::new(0),
        }
    }

    /// Record a fiber spawn.
    #[inline]
    pub fn record_spawn(&self) {
        self.spawned.increment();
        self.active.increment();

        // Update peak
        let current = self.active.value();
        let mut peak = self.peak_active.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_active.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    /// Record a fiber completion.
    #[inline]
    pub fn record_completed(&self, lifetime_ns: u64) {
        self.completed.increment();
        self.active.decrement();
        self.lifetime.record(lifetime_ns);
    }

    /// Record a fiber failure.
    #[inline]
    pub fn record_failed(&self, lifetime_ns: u64) {
        self.failed.increment();
        self.active.decrement();
        self.lifetime.record(lifetime_ns);
    }

    /// Record a fiber cancellation.
    #[inline]
    pub fn record_cancelled(&self, lifetime_ns: u64) {
        self.cancelled.increment();
        self.active.decrement();
        self.lifetime.record(lifetime_ns);
    }

    /// Get total spawned count.
    #[inline]
    pub fn total_spawned(&self) -> u64 {
        self.spawned.value()
    }

    /// Get completed count.
    #[inline]
    pub fn completed_count(&self) -> u64 {
        self.completed.value()
    }

    /// Get failed count.
    #[inline]
    pub fn failed_count(&self) -> u64 {
        self.failed.value()
    }

    /// Get cancelled count.
    #[inline]
    pub fn cancelled_count(&self) -> u64 {
        self.cancelled.value()
    }

    /// Get currently active count.
    #[inline]
    pub fn active_count(&self) -> u64 {
        self.active.value()
    }

    /// Get peak active count.
    #[inline]
    pub fn peak_active(&self) -> u64 {
        self.peak_active.load(Ordering::Relaxed)
    }

    /// Get mean lifetime in nanoseconds.
    #[inline]
    pub fn mean_lifetime_ns(&self) -> f64 {
        self.lifetime.mean()
    }

    /// Get mean lifetime in milliseconds.
    #[inline]
    pub fn mean_lifetime_ms(&self) -> f64 {
        self.lifetime.mean() / 1_000_000.0
    }

    /// Get lifetime percentile in nanoseconds.
    #[inline]
    pub fn lifetime_percentile_ns(&self, p: f64) -> Option<u64> {
        self.lifetime.percentile(p)
    }

    /// Get the success rate.
    ///
    /// `u64 as f64` loses precision only past 2^52 fibers, far beyond any
    /// realistic counter value — inherent to exporting a counter ratio as a float.
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    pub fn success_rate(&self) -> f64 {
        let total_finished = self.completed_count() + self.failed_count() + self.cancelled_count();
        if total_finished == 0 {
            1.0
        } else {
            self.completed_count() as f64 / total_finished as f64
        }
    }

    /// Get a summary.
    pub fn summary(&self) -> FiberMetricsSummary {
        FiberMetricsSummary {
            total_spawned: self.total_spawned(),
            completed: self.completed_count(),
            failed: self.failed_count(),
            cancelled: self.cancelled_count(),
            active: self.active_count(),
            peak_active: self.peak_active(),
            mean_lifetime_ns: self.mean_lifetime_ns(),
        }
    }
}

impl Default for MensuraFibrae {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Fiber Metrics Summary
// =============================================================================

/// Summary of fiber metrics.
#[derive(Debug, Clone)]
pub struct FiberMetricsSummary {
    /// Total fibers spawned.
    pub total_spawned: u64,
    /// Completed fibers.
    pub completed: u64,
    /// Failed fibers.
    pub failed: u64,
    /// Cancelled fibers.
    pub cancelled: u64,
    /// Currently active.
    pub active: u64,
    /// Peak active.
    pub peak_active: u64,
    /// Mean lifetime in nanoseconds.
    pub mean_lifetime_ns: f64,
}

// =============================================================================
// Supervisor Metrics
// =============================================================================

/// Metrics for a supervision tree.
///
/// # Latin Etymology
/// *Mensura arboris* = tree measure.
#[derive(Debug)]
pub struct MensuraArboris {
    /// Supervisor name.
    name: String,
    /// Total restarts.
    restarts: Numerator,
    /// Restart intensity violations.
    intensity_violations: Numerator,
    /// Currently healthy children.
    healthy_children: Indicium,
    /// Total children.
    total_children: Indicium,
    /// Time since last failure (nanoseconds).
    time_since_failure: AtomicU64,
}

impl MensuraArboris {
    /// Create new supervisor metrics.
    pub fn new(name: impl Into<String>) -> Self {
        MensuraArboris {
            name: name.into(),
            restarts: Numerator::new(),
            intensity_violations: Numerator::new(),
            healthy_children: Indicium::new(),
            total_children: Indicium::new(),
            time_since_failure: AtomicU64::new(0),
        }
    }

    /// Get the supervisor name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Record a restart.
    #[inline]
    pub fn record_restart(&self) {
        self.restarts.increment();
        self.time_since_failure.store(0, Ordering::Relaxed);
    }

    /// Record an intensity violation.
    #[inline]
    pub fn record_intensity_violation(&self) {
        self.intensity_violations.increment();
    }

    /// Update child counts.
    #[inline]
    pub fn set_child_counts(&self, healthy: u64, total: u64) {
        self.healthy_children.set(healthy);
        self.total_children.set(total);
    }

    /// Get restart count.
    #[inline]
    pub fn restart_count(&self) -> u64 {
        self.restarts.value()
    }

    /// Get intensity violation count.
    #[inline]
    pub fn intensity_violation_count(&self) -> u64 {
        self.intensity_violations.value()
    }

    /// Get healthy children count.
    #[inline]
    pub fn healthy_children(&self) -> u64 {
        self.healthy_children.value()
    }

    /// Get total children count.
    #[inline]
    pub fn total_children(&self) -> u64 {
        self.total_children.value()
    }

    /// Get health percentage.
    ///
    /// `u64 as f64` loses precision only past 2^52 children, far beyond any
    /// realistic counter value — inherent to exporting a counter ratio as a float.
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    pub fn health_percentage(&self) -> f64 {
        let total = self.total_children();
        if total == 0 {
            100.0
        } else {
            (self.healthy_children() as f64 / total as f64) * 100.0
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mensura_fibrae_spawn() {
        let metrics = MensuraFibrae::new();

        metrics.record_spawn();
        metrics.record_spawn();

        assert_eq!(metrics.total_spawned(), 2);
        assert_eq!(metrics.active_count(), 2);
        assert_eq!(metrics.peak_active(), 2);
    }

    #[test]
    fn test_mensura_fibrae_completion() {
        let metrics = MensuraFibrae::new();

        metrics.record_spawn();
        metrics.record_spawn();
        metrics.record_completed(1_000_000); // 1ms

        assert_eq!(metrics.active_count(), 1);
        assert_eq!(metrics.completed_count(), 1);
        assert_eq!(metrics.peak_active(), 2);
    }

    #[test]
    fn test_mensura_fibrae_failures() {
        let metrics = MensuraFibrae::new();

        metrics.record_spawn();
        metrics.record_failed(500_000);

        assert_eq!(metrics.failed_count(), 1);
        assert_eq!(metrics.active_count(), 0);
    }

    #[test]
    fn test_mensura_arboris() {
        let metrics = MensuraArboris::new("supervisor");

        metrics.set_child_counts(3, 5);
        metrics.record_restart();

        assert_eq!(metrics.restart_count(), 1);
        assert_eq!(metrics.health_percentage(), 60.0);
    }
}
