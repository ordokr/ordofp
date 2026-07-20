//! Benchmark Infrastructure for Effect System
//!
//! This module provides utilities for *driving* effect-system workloads and
//! verifying their semantics. It is deliberately **measurement-free**: the
//! `measure_*` helpers run the workload under `black_box` but record no
//! timing — `Measurement` carries only an iteration count and a
//! correctness flag. Wall-clock timing is the job of an outer harness
//! (e.g. criterion in `benches/nexus_benchmarks.rs`), which uses these
//! helpers to verify the acceptance criteria from the technical analysis.
//!
//! # Acceptance Criteria
//!
//! | Pattern | Target Overhead |
//! |---------|-----------------|
//! | Pure | ~0% |
//! | State-only | <5% vs hand-written |
//! | Reader-only | <5% vs hand-written |
//! | Error-only | ~0% (isomorphic to Result) |
//! | 2 effects | 5-20% |
//! | 3+ effects | 20-50% |
//!
//! # Usage
//!
//! ```rust
//! use ordofp_core::nexus::bench::*;
//! use ordofp_core::nexus::pure;
//!
//! // Drive the baseline and the effectful computation identically; an outer
//! // harness times the two calls and computes the overhead fraction, which
//! // can then be recorded via `OverheadReport::new`.
//! let baseline = measure_baseline(|| 42_i32, 10_000);
//! let effectful = measure_pure_eff(|| pure(42_i32), 10_000);
//! assert_eq!(baseline.iterations, effectful.iterations);
//! ```

use core::hint::black_box;

use super::effect::Eff;
use super::effects::error::ErrorComputation;
use super::effects::reader::ReaderComputation;
use super::effects::state::StatefulComputation;
use super::row::Pure;

// =============================================================================
// Measurement Types
// =============================================================================

/// Result of a benchmark measurement.
#[derive(Clone, Copy, Debug)]
pub struct Measurement {
    /// Number of iterations performed.
    pub iterations: u64,
    /// Whether the result matches expected.
    pub correct: bool,
}

impl Measurement {
    /// Create a new measurement.
    #[inline]
    pub fn new(iterations: u64, correct: bool) -> Self {
        Measurement {
            iterations,
            correct,
        }
    }
}

// =============================================================================
// Baseline Measurements
// =============================================================================

/// Measure a baseline computation with no effect wrapper.
///
/// Runs `f` exactly `iterations` times, using [`core::hint::black_box`] to
/// prevent the optimizer from eliding the work.  The returned [`Measurement`]
/// always has `correct = true`; use [`verify_pure_semantics`] (or the
/// analogous `verify_*` helpers) if you need to confirm output correctness.
///
/// This is intended as the **denominator** for overhead comparisons.  Pair it
/// with [`measure_pure_eff`], [`measure_state`], etc. and compute the
/// fractional difference to check whether effect-system overhead stays within
/// the acceptance thresholds documented in the module header.
///
/// # Parameters
///
/// * `f`          — A zero-argument factory that produces the value being
///   benchmarked.  Called once per iteration.
/// * `iterations` — Number of timed repetitions.  Use a value large enough
///   that measurement noise is negligible (e.g. `10_000` for µs-range work).
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::bench::{measure_baseline, measure_pure_eff};
/// use ordofp_core::nexus::pure;
///
/// let baseline = measure_baseline(|| 42_i32, 10_000);
/// let effectful = measure_pure_eff(|| pure(42_i32), 10_000);
/// assert_eq!(baseline.iterations, effectful.iterations);
/// ```
#[inline(never)]
pub fn measure_baseline<A, F: Fn() -> A>(f: F, iterations: u64) -> Measurement {
    for _ in 0..iterations {
        black_box(f());
    }
    Measurement::new(iterations, true)
}

/// Measure a pure effect computation.
#[inline(never)]
pub fn measure_pure_eff<A, F: Fn() -> Eff<Pure, A>>(f: F, iterations: u64) -> Measurement {
    for _ in 0..iterations {
        black_box(f().run_pure());
    }
    Measurement::new(iterations, true)
}

/// Measure a stateful computation.
#[inline(never)]
pub fn measure_state<S: Clone + 'static, A: 'static, F: Fn() -> StatefulComputation<S, A>>(
    f: F,
    initial: S,
    iterations: u64,
) -> Measurement {
    for _ in 0..iterations {
        let comp = f();
        black_box(comp.run(initial.clone()));
    }
    Measurement::new(iterations, true)
}

/// Measure a reader computation.
#[inline(never)]
pub fn measure_reader<E: Clone + 'static, A: 'static, F: Fn() -> ReaderComputation<E, A>>(
    f: F,
    env: &E,
    iterations: u64,
) -> Measurement {
    for _ in 0..iterations {
        let comp = f();
        black_box(comp.run(env));
    }
    Measurement::new(iterations, true)
}

/// Measure an error computation.
#[inline(never)]
pub fn measure_error<Err, A, F: Fn() -> ErrorComputation<Err, A>>(
    f: F,
    iterations: u64,
) -> Measurement {
    for _ in 0..iterations {
        let comp = f();
        let _ = black_box(comp.run());
    }
    Measurement::new(iterations, true)
}

// =============================================================================
// Hand-Written Baselines
// =============================================================================

/// Hand-written state passing for comparison.
#[inline(never)]
pub fn measure_handwritten_state<S: Clone, A, F: Fn(S) -> (A, S)>(
    f: F,
    initial: S,
    iterations: u64,
) -> Measurement {
    for _ in 0..iterations {
        black_box(f(initial.clone()));
    }
    Measurement::new(iterations, true)
}

/// Hand-written reader passing for comparison.
#[inline(never)]
pub fn measure_handwritten_reader<E, A, F: Fn(&E) -> A>(
    f: F,
    env: &E,
    iterations: u64,
) -> Measurement {
    for _ in 0..iterations {
        black_box(f(env));
    }
    Measurement::new(iterations, true)
}

/// Hand-written result for comparison.
#[inline(never)]
pub fn measure_handwritten_result<E, A, F: Fn() -> Result<A, E>>(
    f: F,
    iterations: u64,
) -> Measurement {
    for _ in 0..iterations {
        let _ = black_box(f());
    }
    Measurement::new(iterations, true)
}

// =============================================================================
// Verification Tests
// =============================================================================

/// Verify that pure effect matches baseline behavior.
pub fn verify_pure_semantics<A: PartialEq>(
    baseline: impl Fn() -> A,
    effectful: impl Fn() -> Eff<Pure, A>,
) -> bool {
    baseline() == effectful().run_pure()
}

/// Verify that state effect matches hand-written behavior.
pub fn verify_state_semantics<S: Clone + PartialEq + 'static, A: PartialEq + 'static>(
    handwritten: impl Fn(S) -> (A, S),
    effectful: impl Fn() -> StatefulComputation<S, A>,
    initial: S,
) -> bool {
    let (a1, s1) = handwritten(initial.clone());
    let (a2, s2) = effectful().run(initial);
    a1 == a2 && s1 == s2
}

/// Verify that reader effect matches hand-written behavior.
pub fn verify_reader_semantics<E: 'static, A: PartialEq + 'static>(
    handwritten: impl Fn(&E) -> A,
    effectful: impl Fn() -> ReaderComputation<E, A>,
    env: &E,
) -> bool {
    handwritten(env) == effectful().run(env)
}

/// Verify that error effect matches Result behavior.
pub fn verify_error_semantics<Err: PartialEq, A: PartialEq>(
    handwritten: impl Fn() -> Result<A, Err>,
    effectful: impl Fn() -> ErrorComputation<Err, A>,
) -> bool {
    handwritten() == effectful().run()
}

// =============================================================================
// Overhead Calculation
// =============================================================================

/// Performance comparison result.
#[derive(Clone, Copy, Debug)]
pub struct OverheadReport {
    /// Name of the pattern being tested.
    pub pattern: &'static str,
    /// Target maximum overhead.
    pub target: f64,
    /// Whether the target was met.
    pub meets_target: bool,
}

impl OverheadReport {
    /// Create an `OverheadReport` from a measured overhead fraction.
    ///
    /// Compares `actual` against `target` to populate
    /// [`OverheadReport::meets_target`].  Note that `actual` itself is **not**
    /// stored in the report — only the boolean outcome is retained.
    ///
    /// # Parameters
    ///
    /// * `pattern` — Human-readable label for the effect pattern under test
    ///   (e.g. `"pure"`, `"state-only"`, `"2-effects"`).
    /// * `target`  — Maximum acceptable overhead as a fraction where `0.05`
    ///   means 5 %.
    /// * `actual`  — Measured overhead as a fraction.  Values ≤ `target`
    ///   cause `meets_target` to be `true`.
    #[inline]
    pub fn new(pattern: &'static str, target: f64, actual: f64) -> Self {
        OverheadReport {
            pattern,
            target,
            meets_target: actual <= target,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nexus::ops::pure;

    #[test]
    fn test_verify_pure_semantics() {
        assert!(verify_pure_semantics(|| 42, || pure(42),));
    }

    #[test]
    fn test_verify_state_semantics() {
        assert!(verify_state_semantics(
            |s: i32| (s + 1, s + 10),
            || StatefulComputation::new(|s: i32| (s + 1, s + 10)),
            5,
        ));
    }

    #[test]
    fn test_verify_reader_semantics() {
        assert!(verify_reader_semantics(
            |e: &i32| e + 1,
            || ReaderComputation::new(|e: &i32| e + 1),
            &10,
        ));
    }

    #[test]
    fn test_verify_error_semantics() {
        assert!(verify_error_semantics(
            || Ok::<i32, &str>(42),
            || ErrorComputation::ok(42),
        ));
    }

    #[test]
    fn test_measure_baseline() {
        let m = measure_baseline(|| 42, 100);
        assert_eq!(m.iterations, 100);
        assert!(m.correct);
    }

    #[test]
    fn test_measure_pure_eff() {
        let m = measure_pure_eff(|| pure(42), 100);
        assert_eq!(m.iterations, 100);
        assert!(m.correct);
    }

    #[test]
    fn test_measure_state() {
        let m = measure_state(|| StatefulComputation::new(|s: i32| (s + 1, s)), 0, 100);
        assert_eq!(m.iterations, 100);
        assert!(m.correct);
    }

    #[test]
    fn test_measure_reader() {
        let m = measure_reader(|| ReaderComputation::new(|e: &i32| *e + 1), &10, 100);
        assert_eq!(m.iterations, 100);
        assert!(m.correct);
    }

    #[test]
    fn test_measure_error() {
        let m = measure_error(|| ErrorComputation::<&str, i32>::ok(42), 100);
        assert_eq!(m.iterations, 100);
        assert!(m.correct);
    }
}
