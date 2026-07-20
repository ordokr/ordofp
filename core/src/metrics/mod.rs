//! Effect Metrics - Performance Monitoring
//!
//! > *"Mensura est mater scientiae"*
//! > — Measurement is the mother of knowledge. (Latin)
//!
//! This module provides metrics collection for effect execution.
//!
//! # Overview
//!
//! Effect metrics capture performance data including operation counts,
//! durations, and error rates for effects and fibers.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Metric | Mensura | *mensura* = measure |
//! | Counter | Numerator | *numerator* = counter |
//! | Gauge | Indicium | *indicium* = indicator |
//! | Histogram | Distributio | *distributio* = distribution |
//! | Registry | Registrum | *registrum* = register |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::metrics::{RegistrumMensurarum, MensuraEffectus};
//!
//! let registry = RegistrumMensurarum::new();
//! let effect_metrics: std::sync::Arc<MensuraEffectus> = registry.effect_metrics(1, "StateEffect");
//!
//! effect_metrics.operation_start(); // pair with record_operation below
//! effect_metrics.record_operation(1000, true); // duration_ns, success
//! ```

mod effect;
mod fiber;
mod registry;

pub use effect::*;
pub use fiber::*;
pub use registry::*;

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

// =============================================================================
// Metric Types
// =============================================================================

/// A counter metric (monotonically increasing).
///
/// # Latin Etymology
/// *Numerator* = counter.
#[derive(Debug)]
pub struct Numerator {
    value: AtomicU64,
}

impl Numerator {
    /// Create a new counter.
    #[inline]
    pub fn new() -> Self {
        Numerator {
            value: AtomicU64::new(0),
        }
    }

    /// Increment the counter by 1.
    #[inline]
    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by a value.
    #[inline]
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get the current value.
    #[inline]
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for Numerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Numerator {
    fn clone(&self) -> Self {
        Numerator {
            value: AtomicU64::new(self.value()),
        }
    }
}

// =============================================================================
// Gauge
// =============================================================================

/// A gauge metric (can go up or down).
///
/// # Latin Etymology
/// *Indicium* = indicator.
#[derive(Debug)]
pub struct Indicium {
    value: AtomicU64,
}

impl Indicium {
    /// Create a new gauge.
    #[inline]
    pub fn new() -> Self {
        Indicium {
            value: AtomicU64::new(0),
        }
    }

    /// Set the gauge value.
    #[inline]
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment the gauge.
    #[inline]
    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the gauge.
    ///
    /// Saturating: an unpaired decrement (no matching increment) must not
    /// wrap the gauge to `u64::MAX` (M14).
    #[inline]
    pub fn decrement(&self) {
        let _ = self
            .value
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(1));
    }

    /// Get the current value.
    #[inline]
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for Indicium {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Indicium {
    fn clone(&self) -> Self {
        Indicium {
            value: AtomicU64::new(self.value()),
        }
    }
}

// =============================================================================
// Histogram
// =============================================================================

/// A histogram for recording distributions.
///
/// Uses fixed buckets for lock-free operation.
///
/// # Latin Etymology
/// *Distributio* = distribution.
#[derive(Debug)]
pub struct Distributio {
    /// Bucket boundaries (upper limits in nanoseconds).
    boundaries: &'static [u64],
    /// Bucket counts.
    buckets: Vec<AtomicU64>,
    /// Sum of all values.
    sum: AtomicU64,
    /// Count of values.
    count: AtomicU64,
}

impl Distributio {
    /// Default bucket boundaries for latency (in nanoseconds).
    pub const LATENCY_BUCKETS: &'static [u64] = &[
        1_000,         // 1 µs
        5_000,         // 5 µs
        10_000,        // 10 µs
        25_000,        // 25 µs
        50_000,        // 50 µs
        100_000,       // 100 µs
        250_000,       // 250 µs
        500_000,       // 500 µs
        1_000_000,     // 1 ms
        2_500_000,     // 2.5 ms
        5_000_000,     // 5 ms
        10_000_000,    // 10 ms
        25_000_000,    // 25 ms
        50_000_000,    // 50 ms
        100_000_000,   // 100 ms
        250_000_000,   // 250 ms
        500_000_000,   // 500 ms
        1_000_000_000, // 1 s
    ];

    /// Create a new histogram with default latency buckets.
    pub fn new() -> Self {
        Self::with_boundaries(Self::LATENCY_BUCKETS)
    }

    /// Create a histogram with custom boundaries.
    pub fn with_boundaries(boundaries: &'static [u64]) -> Self {
        let mut buckets = Vec::with_capacity(boundaries.len() + 1);
        for _ in 0..=boundaries.len() {
            buckets.push(AtomicU64::new(0));
        }

        Distributio {
            boundaries,
            buckets,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    /// Record a value.
    #[inline]
    pub fn record(&self, value: u64) {
        // Find the bucket
        let bucket_idx = self
            .boundaries
            .iter()
            .position(|&b| value <= b)
            .unwrap_or(self.boundaries.len());

        self.buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the count.
    #[inline]
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the sum.
    #[inline]
    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }

    /// Get the mean.
    ///
    /// `u64 as f64` loses precision only past 2^52 observations, far beyond
    /// any realistic histogram total — inherent to exporting a counter ratio
    /// as a float.
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    pub fn mean(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            0.0
        } else {
            self.sum() as f64 / count as f64
        }
    }

    /// Get bucket counts.
    #[inline]
    pub fn bucket_counts(&self) -> Vec<u64> {
        self.buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect()
    }

    /// Get the boundaries.
    #[inline]
    pub fn boundaries(&self) -> &[u64] {
        self.boundaries
    }

    /// Estimate a percentile (approximate).
    pub fn percentile(&self, p: f64) -> Option<u64> {
        let count = self.count();
        if count == 0 {
            return None;
        }

        // `p` is caller-supplied and not range-checked, so clamp the target
        // rank into u64's range before truncating: an out-of-range or
        // negative `p` saturates instead of silently losing its sign. The
        // u64<->f64 round-trip itself is inherent to estimating an integer
        // bucket rank from a floating-point percentile.
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let target = (count as f64 * p / 100.0).max(0.0).min(u64::MAX as f64) as u64;
        let mut cumulative = 0u64;

        for (i, bucket) in self.buckets.iter().enumerate() {
            cumulative += bucket.load(Ordering::Relaxed);
            if cumulative >= target {
                if i < self.boundaries.len() {
                    return Some(self.boundaries[i]);
                }
                // Return last boundary for overflow bucket
                return self.boundaries.last().copied();
            }
        }

        self.boundaries.last().copied()
    }
}

impl Default for Distributio {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Metric Snapshot
// =============================================================================

/// A snapshot of a metric for export.
#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    /// Metric name.
    pub name: String,
    /// Metric type.
    pub metric_type: MetricType,
    /// Metric value.
    pub value: MetricValue,
    /// Labels.
    pub labels: Vec<(String, String)>,
}

/// Type of metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Monotonically non-decreasing count (a [`Numerator`]).
    Counter,
    /// Instantaneous value that can rise and fall (an [`Indicium`]).
    Gauge,
    /// Distribution of recorded values over fixed buckets (a
    /// [`Distributio`]).
    Histogram,
}

/// Value of a metric.
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Counter total at snapshot time.
    Counter(u64),
    /// Gauge reading at snapshot time.
    Gauge(u64),
    /// Histogram state at snapshot time.
    Histogram {
        /// Total number of recorded observations.
        count: u64,
        /// Sum of all recorded values, in the histogram's unit
        /// (nanoseconds for the built-in latency histograms).
        sum: u64,
        /// `(upper boundary, count)` pairs, one per finite bucket. Counts
        /// are per-bucket, not cumulative, and observations above the last
        /// boundary (the overflow bucket) are not included here.
        buckets: Vec<(u64, u64)>,
    },
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numerator() {
        let counter = Numerator::new();
        assert_eq!(counter.value(), 0);

        counter.increment();
        assert_eq!(counter.value(), 1);

        counter.add(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_indicium() {
        let gauge = Indicium::new();
        assert_eq!(gauge.value(), 0);

        gauge.set(10);
        assert_eq!(gauge.value(), 10);

        gauge.increment();
        assert_eq!(gauge.value(), 11);

        gauge.decrement();
        assert_eq!(gauge.value(), 10);
    }

    /// M14 regression: an unpaired decrement must saturate at 0, not wrap to
    /// u64::MAX.
    #[test]
    fn test_indicium_decrement_saturates_at_zero() {
        let gauge = Indicium::new();
        assert_eq!(gauge.value(), 0);

        gauge.decrement();
        assert_eq!(gauge.value(), 0); // pre-fix: u64::MAX
    }

    #[test]
    fn test_distributio() {
        let hist = Distributio::new();

        hist.record(500); // < 1µs
        hist.record(5_000); // 1-5 µs
        hist.record(50_000); // 25-50 µs

        assert_eq!(hist.count(), 3);
        assert_eq!(hist.sum(), 55_500);
    }

    #[test]
    fn test_distributio_percentile() {
        let hist = Distributio::new();

        // Add many values in the 1-5µs bucket
        for _ in 0..100 {
            hist.record(3_000);
        }

        let p50 = hist.percentile(50.0);
        assert!(p50.is_some());
    }
}
