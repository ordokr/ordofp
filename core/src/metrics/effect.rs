//! Effect Metrics
//!
//! > *"Effectus mensurati sunt effectus cogniti"*
//! > — Measured effects are known effects. (Latin)
//!
//! This module provides metrics specific to effect execution.

use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{Distributio, Numerator};

// =============================================================================
// Effect Metrics
// =============================================================================

/// Metrics for a single effect type.
///
/// # Latin Etymology
/// *Mensura effectus* = effect measure.
#[derive(Debug)]
pub struct MensuraEffectus {
    /// Effect ID.
    effect_id: u64,
    /// Effect name.
    effect_name: String,
    /// Total operations performed.
    operations: Numerator,
    /// Successful operations.
    successes: Numerator,
    /// Failed operations.
    failures: Numerator,
    /// Operation latency distribution (nanoseconds).
    latency: Distributio,
    /// Currently in-flight operations.
    in_flight: AtomicU64,
}

impl MensuraEffectus {
    /// Create new effect metrics.
    pub fn new(effect_id: u64, effect_name: impl Into<String>) -> Self {
        MensuraEffectus {
            effect_id,
            effect_name: effect_name.into(),
            operations: Numerator::new(),
            successes: Numerator::new(),
            failures: Numerator::new(),
            latency: Distributio::new(),
            in_flight: AtomicU64::new(0),
        }
    }

    /// Get the effect ID.
    #[inline]
    pub fn effect_id(&self) -> u64 {
        self.effect_id
    }

    /// Get the effect name.
    #[inline]
    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }

    /// Record the start of an operation.
    #[inline]
    pub fn operation_start(&self) {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful operation.
    #[inline]
    pub fn record_success(&self, duration_ns: u64) {
        // Saturating decrement: a success recorded without a paired
        // operation_start (the module's own docs did this) must not wrap.
        let _ = self
            .in_flight
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(1));
        self.operations.increment();
        self.successes.increment();
        self.latency.record(duration_ns);
    }

    /// Record a failed operation.
    #[inline]
    pub fn record_failure(&self, duration_ns: u64) {
        // Saturating decrement: see record_success.
        let _ = self
            .in_flight
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(1));
        self.operations.increment();
        self.failures.increment();
        self.latency.record(duration_ns);
    }

    /// Record an operation with success/failure flag.
    #[inline]
    pub fn record_operation(&self, duration_ns: u64, success: bool) {
        if success {
            self.record_success(duration_ns);
        } else {
            self.record_failure(duration_ns);
        }
    }

    /// Get total operation count.
    #[inline]
    pub fn total_operations(&self) -> u64 {
        self.operations.value()
    }

    /// Get success count.
    #[inline]
    pub fn success_count(&self) -> u64 {
        self.successes.value()
    }

    /// Get failure count.
    #[inline]
    pub fn failure_count(&self) -> u64 {
        self.failures.value()
    }

    /// Get in-flight count.
    #[inline]
    pub fn in_flight(&self) -> u64 {
        self.in_flight.load(Ordering::Relaxed)
    }

    /// Get the success rate (0.0 to 1.0).
    ///
    /// `u64 as f64` loses precision only past 2^52 operations, far beyond any
    /// realistic counter value — inherent to exporting a counter ratio as a float.
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    pub fn success_rate(&self) -> f64 {
        let total = self.total_operations();
        if total == 0 {
            1.0
        } else {
            self.success_count() as f64 / total as f64
        }
    }

    /// Get the error rate (0.0 to 1.0).
    #[inline]
    pub fn error_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }

    /// Get mean latency in nanoseconds.
    #[inline]
    pub fn mean_latency_ns(&self) -> f64 {
        self.latency.mean()
    }

    /// Get mean latency in microseconds.
    #[inline]
    pub fn mean_latency_us(&self) -> f64 {
        self.latency.mean() / 1000.0
    }

    /// Get mean latency in milliseconds.
    #[inline]
    pub fn mean_latency_ms(&self) -> f64 {
        self.latency.mean() / 1_000_000.0
    }

    /// Get a latency percentile in nanoseconds.
    #[inline]
    pub fn latency_percentile_ns(&self, p: f64) -> Option<u64> {
        self.latency.percentile(p)
    }

    /// Get the latency histogram.
    #[inline]
    pub fn latency_histogram(&self) -> &Distributio {
        &self.latency
    }

    /// Get a summary of the metrics.
    pub fn summary(&self) -> EffectMetricsSummary {
        EffectMetricsSummary {
            effect_id: self.effect_id,
            effect_name: self.effect_name.clone(),
            total_operations: self.total_operations(),
            successes: self.success_count(),
            failures: self.failure_count(),
            in_flight: self.in_flight(),
            mean_latency_ns: self.mean_latency_ns(),
            p50_latency_ns: self.latency_percentile_ns(50.0),
            p99_latency_ns: self.latency_percentile_ns(99.0),
        }
    }
}

// =============================================================================
// Effect Metrics Summary
// =============================================================================

/// Summary of effect metrics for reporting.
#[derive(Debug, Clone)]
pub struct EffectMetricsSummary {
    /// Effect ID.
    pub effect_id: u64,
    /// Effect name.
    pub effect_name: String,
    /// Total operations.
    pub total_operations: u64,
    /// Successful operations.
    pub successes: u64,
    /// Failed operations.
    pub failures: u64,
    /// Currently in-flight.
    pub in_flight: u64,
    /// Mean latency in nanoseconds.
    pub mean_latency_ns: f64,
    /// P50 latency in nanoseconds.
    pub p50_latency_ns: Option<u64>,
    /// P99 latency in nanoseconds.
    pub p99_latency_ns: Option<u64>,
}

impl EffectMetricsSummary {
    /// Get success rate.
    ///
    /// `u64 as f64` loses precision only past 2^52 operations, far beyond any
    /// realistic counter value — inherent to exporting a counter ratio as a float.
    #[allow(clippy::cast_precision_loss)]
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            1.0
        } else {
            self.successes as f64 / self.total_operations as f64
        }
    }
}

// =============================================================================
// Operation Scope
// =============================================================================

/// RAII guard for measuring an operation.
///
/// Records start on creation and duration on drop.
pub struct OperationScope<'a> {
    metrics: &'a MensuraEffectus,
    start_ns: u64,
    success: bool,
}

impl<'a> OperationScope<'a> {
    /// Create a new operation scope.
    pub fn new(metrics: &'a MensuraEffectus, start_ns: u64) -> Self {
        metrics.operation_start();
        OperationScope {
            metrics,
            start_ns,
            success: true,
        }
    }

    /// Mark the operation as failed.
    pub fn fail(&mut self) {
        self.success = false;
    }

    /// Complete the operation with a specific end time.
    pub fn complete(self, end_ns: u64) {
        let duration = end_ns.saturating_sub(self.start_ns);
        self.metrics.record_operation(duration, self.success);
        // Prevent drop from recording again
        core::mem::forget(self);
    }
}

impl Drop for OperationScope<'_> {
    fn drop(&mut self) {
        // If complete() wasn't called, record with unknown duration
        self.metrics.record_operation(0, self.success);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mensura_effectus_new() {
        let metrics = MensuraEffectus::new(1, "StateEffect");
        assert_eq!(metrics.effect_id(), 1);
        assert_eq!(metrics.effect_name(), "StateEffect");
        assert_eq!(metrics.total_operations(), 0);
    }

    #[test]
    fn test_mensura_effectus_record() {
        let metrics = MensuraEffectus::new(1, "StateEffect");

        metrics.operation_start();
        metrics.record_success(1000);

        assert_eq!(metrics.total_operations(), 1);
        assert_eq!(metrics.success_count(), 1);
        assert_eq!(metrics.failure_count(), 0);
    }

    #[test]
    fn test_mensura_effectus_failure() {
        let metrics = MensuraEffectus::new(1, "IO");

        metrics.operation_start();
        metrics.record_failure(5000);

        assert_eq!(metrics.total_operations(), 1);
        assert_eq!(metrics.success_count(), 0);
        assert_eq!(metrics.failure_count(), 1);
        assert_eq!(metrics.error_rate(), 1.0);
    }

    #[test]
    fn test_mensura_effectus_latency() {
        let metrics = MensuraEffectus::new(1, "DB");

        for _ in 0..100 {
            metrics.operation_start();
            metrics.record_success(10_000); // 10 µs
        }

        assert!(metrics.mean_latency_ns() > 0.0);
        assert!(metrics.latency_percentile_ns(50.0).is_some());
    }

    /// M14 regression: record_success without a paired operation_start wrapped
    /// the in-flight gauge to u64::MAX.
    #[test]
    fn gauge_saturates_at_zero() {
        let metrics = MensuraEffectus::new(1, "Unpaired");

        // No operation_start() call: in_flight starts at 0.
        metrics.record_success(10);

        assert_eq!(metrics.in_flight(), 0); // pre-fix: u64::MAX
    }

    #[test]
    fn test_effect_metrics_summary() {
        let metrics = MensuraEffectus::new(1, "Cache");

        metrics.operation_start();
        metrics.record_success(1000);
        metrics.operation_start();
        metrics.record_failure(2000);

        let summary = metrics.summary();
        assert_eq!(summary.total_operations, 2);
        assert_eq!(summary.successes, 1);
        assert_eq!(summary.failures, 1);
        assert_eq!(summary.success_rate(), 0.5);
    }
}
