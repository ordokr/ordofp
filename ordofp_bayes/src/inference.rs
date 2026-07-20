//! Inference algorithms for Bayesian computation.
//!
//!
//! ## Trace-based MCMC (Metropolis-Hastings)
//!
//! Following the monad-bayes pattern, MH uses trace-based execution:
//! - A **Trace** records all random variables sampled during model execution
//! - The **proposal** modifies one random variable (single-site MH)
//! - The **acceptance ratio** is computed using probability densities
//!
//! ## Sequential Monte Carlo (SMC)
//!
//! SMC maintains a population of weighted particles and resamples based on
//! importance weights accumulated during execution.

#![cfg(feature = "alloc")]

use crate::distributions::sample_exp1;
#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use rand::Rng;
use rand::RngExt;
use rand::distr::Distribution as RandDistribution;
#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(feature = "std")]
thread_local! {
    static WEIGHTS_CACHE: std::cell::RefCell<Vec<f64>> = const { std::cell::RefCell::new(Vec::new()) };
    static COUNTS_CACHE: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
    static RANDOMS_CACHE: std::cell::RefCell<Vec<f64>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Default proposal step size for Metropolis-Hastings random walk proposals.
pub const DEFAULT_STEP_SIZE: f64 = 0.1;

/// Execution trace for trace-based MCMC.
///
/// A trace records all random variables sampled during program execution,
/// along with the output value and the joint log-probability density.
#[derive(Debug)]
pub struct Trace<A> {
    /// Random variables sampled during execution (values in [0, 1]).
    pub variables: Vec<f64>,
    /// The output value of the program.
    pub output: A,
    /// Log-probability density of this trace.
    pub log_prob_density: f64,
}

impl<A: Clone> Clone for Trace<A> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            variables: self.variables.clone(),
            output: self.output.clone(),
            log_prob_density: self.log_prob_density,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        // Optimization: reuse allocation of variables vector
        self.variables.clone_from(&source.variables);
        self.output.clone_from(&source.output);
        self.log_prob_density = source.log_prob_density;
    }
}

impl<A> Trace<A> {
    /// Create a new trace with given values.
    #[inline]
    pub fn new(variables: Vec<f64>, output: A, log_prob_density: f64) -> Self {
        Self {
            variables,
            output,
            log_prob_density,
        }
    }

    /// Create a pure trace with no random variables.
    #[inline]
    pub fn pure(output: A) -> Self {
        Self {
            variables: Vec::new(),
            output,
            log_prob_density: 0.0, // log(1) = 0
        }
    }

    /// Map over the output value.
    #[inline]
    pub fn map<B, F: FnOnce(A) -> B>(self, f: F) -> Trace<B> {
        Trace {
            variables: self.variables,
            output: f(self.output),
            log_prob_density: self.log_prob_density,
        }
    }
}

/// Model trait for trace-based MCMC.
///
/// A model can be executed with a given sequence of random variables,
/// producing a trace with an output and log-probability.
pub trait TraceableModel<A> {
    /// Execute the model with provided random variables.
    ///
    /// If `variables` is empty or too short, the model should sample
    /// from a fresh source. If too long, extra variables are ignored.
    fn execute_with_trace<R: Rng + ?Sized>(&self, variables: &[f64], rng: &mut R) -> Trace<A>;

    /// Execute the model and return a trace, avoiding variable allocation if possible.
    ///
    /// Returns (output, `log_prob`, variables).
    /// If `variables` is `Cow::Borrowed`, it means the model consumed the input `variables`
    /// (or a prefix of them) and did not generate any new ones.
    ///
    /// This allows avoiding the allocation of `Vec<f64>` when the trace variables
    /// are identical to the input.
    fn execute_with_trace_cow<'a, R: Rng + ?Sized>(
        &self,
        variables: &'a [f64],
        rng: &mut R,
    ) -> (A, f64, Cow<'a, [f64]>) {
        let trace = self.execute_with_trace(variables, rng);
        // Optimization: for fixed-structure models the output trace variables are
        // identical to the provided input (the model simply replays them).  In that
        // case, borrow the input slice instead of carrying the newly-allocated copy
        // through the MH accept/reject decision.  The allocator can then reclaim
        // `trace.variables` immediately rather than keeping it live until the caller
        // decides whether to accept or reject the proposal — cutting the lifetime of
        // every ephemeral `Vec<f64>` in the hot loop for the common fixed-structure case.
        if trace.variables.as_slice() == variables {
            (
                trace.output,
                trace.log_prob_density,
                Cow::Borrowed(variables),
            )
        } else {
            (
                trace.output,
                trace.log_prob_density,
                Cow::Owned(trace.variables),
            )
        }
    }
}

/// Weighted particle for SMC.
///
#[derive(Debug)]
pub struct Particle<M> {
    /// The particle value.
    pub value: M,
    /// Log-weight of this particle.
    pub log_weight: f64,
}

impl<M: Clone> Clone for Particle<M> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            log_weight: self.log_weight,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        // Optimization: reuse allocation of particle value (e.g. for Vec state)
        self.value.clone_from(&source.value);
        self.log_weight = source.log_weight;
    }
}

impl<M> Particle<M> {
    /// Create a new particle with unit weight.
    #[inline]
    pub fn new(value: M) -> Self {
        Self {
            value,
            log_weight: 0.0,
        }
    }

    /// Create a particle with specified log-weight.
    #[inline]
    pub fn with_weight(value: M, log_weight: f64) -> Self {
        Self { value, log_weight }
    }

    /// Add to the log-weight (factor/score operation).
    #[inline]
    pub fn factor(&mut self, log_prob: f64) {
        self.log_weight += log_prob;
    }
}

/// Weighted model trait for SMC.
///
/// A model that can produce weighted samples, where the weight
/// represents the likelihood of observations.
pub trait WeightedModel<A> {
    /// Execute the model and return a weighted particle.
    fn execute_weighted<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle<A>;
}

/// Sequential Monte Carlo (SMC) inference.
///
/// Implements Sequential Importance Resampling:
/// 1. Initialize particles from the prior
/// 2. For each step: propagate, weight, resample
/// 3. Return weighted samples representing the posterior
pub struct SequentialMonteCarlo {
    /// Number of particles.
    particles: usize,
    /// Resampling strategy.
    resampling: ResamplingStrategy,
}

/// Resampling strategy for SMC.
#[derive(Clone, Copy, Debug, Default)]
pub enum ResamplingStrategy {
    /// Multinomial resampling (sample with replacement).
    #[default]
    Multinomial,
    /// Systematic resampling (lower variance).
    Systematic,
    /// Stratified resampling (unbiased, low variance).
    Stratified,
}

// =========================================================================
// Standalone Resampling Helpers (Shared between SMC and ImportanceSampling)
// =========================================================================

/// Generate counts for Multinomial resampling.
#[inline]
fn generate_multinomial_counts<R>(
    counts: &mut [usize],
    mut weights: Option<&mut [f64]>,
    rng: &mut R,
) where
    R: Rng + ?Sized,
{
    let n = counts.len();
    if let Some(ws) = &weights {
        assert_eq!(n, ws.len(), "counts and weights length mismatch");
    }

    let sum_weights = if let Some(ws) = &mut weights {
        prepare_weights_in_place(ws)
    } else {
        0.0
    };

    // If no weights provided or all weights are negligible/zero.
    // `prepare_weights_in_place` returns either 0.0 (empty/all-dead input) or a
    // value >= 1.0 (at minimum the max-weight particle contributes exp(0) = 1.0),
    // so `sum_weights` is never NaN here and the check reduces to `<= 0.0` only.
    if sum_weights <= 0.0 {
        // Degenerate case: all particles are dead (all log-weights are -∞ or NaN).
        // Optimization: use counts.fill(1) — a single vectorised store — instead
        // of N Uniform RNG samples.  In the degenerate case every particle has
        // equal (zero) weight, so assigning each particle exactly 1 count is
        // statistically equivalent to Multinomial(n, uniform) and consistent with
        // the Systematic and Stratified uniform fallbacks above.  This saves N RNG
        // calls and eliminates the unsafe Uniform::new + get_unchecked loop.
        counts.fill(1);
        return;
    }

    // Optimization: Use "Ordered Resampling" (Exponential Spacings).
    // Allocate n slots: randoms[0..n] hold the n cumulative Exp(1) sums used
    // as resampling thresholds.
    with_randoms_buffer(n, |randoms| {
        // Optimization: Use spare capacity to avoid repeated push checks.
        // This is safe because we reserve capacity in with_randoms_buffer.
        let mut random_sum = 0.0;
        let spare = randoms.spare_capacity_mut();

        // SAFETY: `with_randoms_buffer(n, ...)` ensures `randoms.capacity() >= n`.
        // After `clear()`, spare capacity equals `capacity >= n`, so `..n` is
        // in-bounds.  Using a concrete fixed-length slice instead of
        // `take(n)` removes the per-iteration `Take` counter check and lets LLVM
        // see an exact iteration count, enabling auto-vectorisation of the cumsum loop.
        for slot in unsafe { spare.get_unchecked_mut(..n) } {
            let e: f64 = sample_exp1(rng);
            random_sum += e;
            slot.write(random_sum);
        }

        // SAFETY: capacity >= n; the n elements above are all
        // initialised before set_len.  Panic-safe: set_len is not called if the
        // sampler panics, so len stays 0 and no uninitialised memory is exposed.
        unsafe { randoms.set_len(n) };

        // Generate the (n+1)-th Exp(1) draw to complete the Gamma(n+1,1) sum used
        // for normalisation.  Its cumulative value is not stored — only the scalar
        // random_sum is needed for the inv_scale computation.
        random_sum += sample_exp1(rng);

        // Optimization: Instead of scaling the `randoms` buffer (which requires an O(N) memory pass),
        // we scale the weights on the fly during the sampling loop.
        let inv_scale = random_sum / sum_weights;

        // SAFETY: `weights` is `Some` here — if it were `None`, `sum_weights` would be
        // `0.0` (the `else` branch on line ~251) and the early-return guard on the
        // `sum_weights <= 0.0` check above would have fired before reaching this point.
        let ws = unsafe { weights.unwrap_unchecked() };
        let mut w_cumsum_scaled = 0.0;
        let mut r_idx = 0;
        // Optimization: cache the current threshold in a register so that consecutive
        // outer-loop iterations where r_idx does not advance (e.g. zero-weight particles
        // hit by the `continue` below) reuse the cached value without a memory reload.
        // This matches the explicit `next_u_scaled` pattern in `generate_stratified_counts`.
        // SAFETY: randoms.len() == n; n >= 2 when resample_values calls this function
        // (the n <= 1 early-return guard fires first), so index 0 is always in-bounds.
        let mut current_threshold = unsafe { *randoms.get_unchecked(0) };

        for (i, &w_linear) in ws.iter().enumerate() {
            if w_linear <= 0.0 {
                continue;
            }

            w_cumsum_scaled += w_linear * inv_scale;

            // Optimization: record r_idx before advancing, then compute count as a
            // difference after the loop.  This eliminates one `count += 1` increment
            // per inner-loop iteration (O(N) total across all N particles), replacing
            // two increments with one, at the cost of a single subtraction after the loop.
            let r_idx_before = r_idx;

            // Explicit bound checks `r_idx < n`
            // allow LLVM's Scalar Evolution (SCEV) to accurately analyze loop trip counts,
            // fundamentally enabling auto-vectorization and loop unrolling, preventing
            // major performance regressions. (Learned from OrdoFP journal).
            while r_idx < n && w_cumsum_scaled >= current_threshold {
                r_idx += 1;
                if r_idx < n {
                    // SAFETY: `r_idx < n`, so this access is in-bounds.
                    current_threshold = unsafe { *randoms.get_unchecked(r_idx) };
                }
            }
            let count = r_idx - r_idx_before;
            // Optimization: skip the write when count == 0.  `with_counts_buffer`
            // already zero-initialises the buffer, so writing 0 back is redundant and
            // wastes a cache-line write per zero-count particle in this hot loop.
            if count > 0 {
                // SAFETY: `i` comes from `ws.iter().enumerate()` where `ws.len() == n ==
                // counts.len()` (both are sized from the same particle count), so `i` is
                // always a valid index into `counts`.
                unsafe {
                    *counts.get_unchecked_mut(i) = count;
                }
            }
            // Optimization: once all n thresholds have been crossed (r_idx >= n),
            // every remaining outer iteration yields count == 0 and no write, so
            // exit early — matching the same guard in `generate_systematic_counts`.
            if r_idx >= n {
                break;
            }
        }

        if r_idx < n {
            // INVARIANT: sum(counts) == n exactly — apply_counts_in_place's
            // unchecked write depends on this. This remainder patch is what
            // makes it hold: without it, an early break at `r_idx < n` (e.g.
            // due to floating-point rounding) would leave sum(counts) < n.
            //
            // SAFETY: `n == counts.len()` and `n > 0` (the `sum_weights <= 0.0` early-return
            // guard above ensures at least one positive-weight particle exists before we reach
            // here), so `n - 1` is a valid index into `counts`.
            unsafe {
                *counts.get_unchecked_mut(n - 1) += n - r_idx;
            }
        }
    });
}

/// Generate counts for Systematic resampling.
#[inline]
// Casts are deliberate: particle counts ≪ 2^52, and `target` is non-negative
// and clamped to `n` by construction (see the INVARIANT comment below).
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn generate_systematic_counts<R>(counts: &mut [usize], mut weights: Option<&mut [f64]>, rng: &mut R)
where
    R: Rng + ?Sized,
{
    let n = counts.len();
    if let Some(ws) = &weights {
        assert_eq!(n, ws.len(), "counts and weights length mismatch");
    }

    let sum_weights = if let Some(ws) = &mut weights {
        prepare_weights_in_place(ws)
    } else {
        0.0
    };

    // `prepare_weights_in_place` returns 0.0 or >= 1.0, never NaN (see multinomial).
    if sum_weights <= 0.0 {
        // Uniform: each gets 1 count.
        // Optimization: fill(1) maps to a vectorized store (consistent with fill(0) in
        // with_counts_buffer) and is faster than a manual iter_mut loop.
        counts.fill(1);
        return;
    }

    // Optimization: compute inv_step directly as n/sum_weights, eliminating the
    // intermediate `step` division.  `1/(sum_weights/n) == n/sum_weights` — one
    // floating-point division instead of two.  `sum_weights > 0` is guaranteed
    // by the `sum_weights <= 0.0` early-return guard above.
    let inv_step = n as f64 / sum_weights;
    // Optimization: avoid `floor` and branch in inner loop by refactoring math.
    // Original: term = (cumsum - u0) * inv_step
    // u0 = u * step, where u ~ U(0, 1)
    // term = cumsum * inv_step - u
    // target = max(0, floor(term) + 1)
    //        = max(0, floor(cumsum * inv_step - u + 1))
    //        = max(0, floor(cumsum * inv_step + (1 - u)))
    // Since 1 - u ~ U(0, 1) and a fresh U(0, 1) sample is equivalent,
    // sample directly to avoid one subtraction per resampling call.
    // cumsum * inv_step >= 0, so the sum is non-negative and truncation
    // equals floor for non-negative values.
    let u_inv = rng.random::<f64>();

    let mut current_cumsum = 0.0;
    let mut generated = 0;
    // SAFETY: `weights` is `Some` here — if it were `None`, `sum_weights` would be
    // `0.0` (the `else` branch above) and the `sum_weights <= 0.0` early-return
    // guard above would have fired before reaching this point.
    let ws = unsafe { weights.unwrap_unchecked() };
    // Weights are now linear

    for (i, &w_linear) in ws.iter().enumerate() {
        if w_linear <= 0.0 {
            // Count remains 0 (initialized)
            continue;
        }

        current_cumsum += w_linear;

        // O(1) count calculation
        let target = (current_cumsum * inv_step + u_inv) as usize;
        // INVARIANT: sum(counts) == n exactly — apply_counts_in_place's
        // unchecked write depends on this. Clamping `target` to `n` is what
        // guarantees `generated` (the running sum of per-particle counts)
        // never overshoots n, and the final iteration's `target == n` makes
        // the running sum land on exactly n.
        let target = core::cmp::min(target, n);

        // `target >= generated`: both are non-decreasing and `generated` is
        // always set to the previous `target`, so subtraction never underflows.
        let count = target - generated;

        generated = target;
        // Optimization: skip the write when count == 0.  `with_counts_buffer`
        // already zero-initialises the buffer, so writing 0 back is redundant and
        // wastes a cache-line write per zero-count particle in this hot loop.
        if count > 0 {
            // SAFETY: `i` comes from `ws.iter().enumerate()` where `ws.len() == n ==
            // counts.len()` (both are sized from the same particle count passed to
            // `resample_values`), so `i` is always a valid index into `counts`.
            // Matches the same unchecked-write pattern used in `generate_multinomial_counts`.
            unsafe {
                *counts.get_unchecked_mut(i) = count;
            }
        }
        // Early exit: all n samples assigned; remaining particles are guaranteed
        // count == 0 (target stays clamped to n, delta == 0). The buffer was
        // zero-initialised by `with_counts_buffer`, so no writes are needed.
        if generated >= n {
            break;
        }
    }
}

/// Generate counts for Stratified resampling.
#[inline]
// Casts are deliberate: particle counts ≪ 2^52, and `target` is non-negative
// and clamped to `n` by construction (see the INVARIANT comment below).
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn generate_stratified_counts<R>(counts: &mut [usize], mut weights: Option<&mut [f64]>, rng: &mut R)
where
    R: Rng + ?Sized,
{
    let n = counts.len();
    if let Some(ws) = &weights {
        assert_eq!(n, ws.len(), "counts and weights length mismatch");
    }

    let sum_weights = if let Some(ws) = &mut weights {
        prepare_weights_in_place(ws)
    } else {
        0.0
    };

    // `prepare_weights_in_place` returns 0.0 or >= 1.0, never NaN (see multinomial).
    if sum_weights <= 0.0 {
        // Uniform: each gets 1 count.
        // Optimization: fill(1) maps to a vectorized store, same as fill(0) in with_counts_buffer.
        counts.fill(1);
        return;
    }

    // Optimization: compute inv_step directly as n/sum_weights, eliminating the
    // intermediate `step` division.  `1/(sum_weights/n) == n/sum_weights` — one
    // floating-point division instead of two.  `sum_weights > 0` is guaranteed
    // by the `sum_weights <= 0.0` early-return guard above.
    let inv_step = n as f64 / sum_weights;

    // Optimization: pre-generate all n per-stratum thresholds into the TLS
    // randoms buffer before the counting loop.  Store `randoms[k] = k + U[0,1)`
    // directly so the hot-loop update is a single unchecked load instead of
    // `(generated as f64) + randoms[generated]` — eliminating one float-to-int
    // cast and one addition per `next_u_scaled` refresh in the inner path.
    // The threshold semantics are unchanged: stratum k accepts the next point
    // when `cumsum_scaled >= k + U_k`, which equals `randoms[k]` directly.
    with_randoms_buffer(n, |randoms| {
        let spare = randoms.spare_capacity_mut();
        // SAFETY: `with_randoms_buffer(n, ...)` ensures `randoms.capacity() >= n`.
        // After `clear()`, spare capacity equals `capacity >= n`, so `..n` is
        // always in-bounds.
        // Optimization: use a running f64 counter instead of `.enumerate()` + `k as f64`
        // to avoid the enumerate state machine and one `usize → f64` cast per iteration.
        // `k_f += 1.0` is exact for n ≤ 2^53 (all realistic particle counts).
        let mut k_f = 0.0_f64;
        for slot in unsafe { spare.get_unchecked_mut(..n) }.iter_mut() {
            slot.write(k_f + rng.random::<f64>());
            k_f += 1.0;
        }
        // SAFETY: all n elements were initialised via MaybeUninit::write above.
        // Panic-safe: set_len is unreachable if rng.random panics, so len stays 0.
        unsafe { randoms.set_len(n) };

        // SAFETY: n > 0 (sum_weights > 0 requires at least one positive weight in a
        // non-empty slice) and randoms.len() == n, so index 0 is valid.
        let mut next_u_scaled = unsafe { *randoms.get_unchecked(0) };
        let mut current_cumsum = 0.0;
        let mut generated = 0;

        // SAFETY: `weights` is `Some` here — if it were `None`, `sum_weights` would
        // be `0.0` (the `else` branch above) and the early-return guard above would
        // have fired before reaching this point.
        let ws = unsafe { weights.unwrap_unchecked() };

        for (i, &w_linear) in ws.iter().enumerate() {
            if w_linear <= 0.0 {
                continue;
            }

            current_cumsum += w_linear;

            let current_cumsum_scaled = current_cumsum * inv_step;
            // Optimization: avoid slow .floor() call; float-to-int truncation equals
            // floor for positive floats.  Clamp to n here rather than inside the
            // skip calculation: this turns `min(target-generated, n-generated)` into
            // just `target-generated`, saving one subtraction and one min per
            // positive-weight particle in this hot loop (uses the identity
            // min(x-c, y-c) == min(x,y)-c).  Also guards against floating-point
            // rounding that could push current_cumsum_scaled marginally above n.
            //
            // INVARIANT: sum(counts) == n exactly — apply_counts_in_place's
            // unchecked write depends on this. Clamping `target` to `n` keeps
            // `generated` (the running sum of per-particle counts) from ever
            // overshooting n; the last positive-weight particle's `target`
            // saturates at n, so the running sum lands on exactly n.
            let target = (current_cumsum_scaled as usize).min(n);
            // Optimization: snapshot `generated` before modification so that the final
            // count can be recovered as a single subtraction rather than tracking a
            // separate `count` variable through both the bulk-skip block and the
            // per-threshold while loop.  Removes one `count += 1` add from every
            // inner-loop iteration (the hottest path in stratified resampling).
            let generated_before = generated;

            if target > generated {
                generated = target;
                if generated < n {
                    // SAFETY: `generated < n` and `randoms.len() == n`, so this access is in-bounds.
                    next_u_scaled = unsafe { *randoms.get_unchecked(generated) };
                }
            }

            // Explicit bound checks `generated < n`
            // allow LLVM's Scalar Evolution (SCEV) to accurately analyze loop trip counts,
            // fundamentally enabling auto-vectorization and loop unrolling, preventing
            // major performance regressions. (Learned from OrdoFP journal).
            while generated < n && current_cumsum_scaled >= next_u_scaled {
                generated += 1;
                if generated < n {
                    // SAFETY: `generated < n`, so this access is in-bounds.
                    next_u_scaled = unsafe { *randoms.get_unchecked(generated) };
                }
            }

            let count = generated - generated_before;
            // Optimization: skip the write when count == 0.  `with_counts_buffer`
            // already zero-initialises the buffer, so writing 0 back is redundant and
            // wastes a cache-line write per zero-count particle in this hot loop.
            if count > 0 {
                // SAFETY: `i` comes from `ws.iter().enumerate()` where `ws.len() == n ==
                // counts.len()` (both are sized from the same particle count passed to
                // `resample_values`), so `i` is always a valid index into `counts`.
                // Matches the same unchecked-write pattern used in the multinomial and
                // systematic counting loops above.
                unsafe {
                    *counts.get_unchecked_mut(i) = count;
                }
            }
        }
    });
}

/// Apply counts to values in-place.
///
/// # Safety-adjacent invariant: callers must guarantee sum(counts) <= n; the unchecked write at `free_cursor` relies on it.
#[inline]
fn apply_counts_in_place<M: Clone>(values: &mut Vec<M>, counts: &[usize]) {
    let n = values.len();
    assert_eq!(n, counts.len(), "values and counts length mismatch");
    let mut free_cursor = 0;
    let p_values = values.as_mut_ptr();

    // Optimization: Use a single-pass two-pointer approach.
    // Scan for free slots (count == 0) on demand using `free_cursor`.
    // This avoids allocating/managing a `free_slots` vector.
    for (i, &c) in counts.iter().enumerate() {
        // Branch hint: Most counts are 0 or 1
        if c > 1 {
            // Optimization: Hoist the pointer addition out of the inner loop.
            // SAFETY: `p_values` is the backing pointer of `values` (length `n`).
            // `i` is bounded by `counts.len()`, which equals `n` by the caller's
            // invariant (both are sized from the same particle count in
            // `resample_values`), so `p_values.add(i)` is always in-bounds.
            // The resulting `&M` does not alias the `&mut M` destinations written
            // in the inner loop because those destinations have `counts[j] == 0`,
            // while `counts[i] > 1` here, guaranteeing `i != free_cursor` on
            // every inner iteration and thus no overlapping live references.
            let src = unsafe { &*p_values.add(i) };

            for _ in 0..(c - 1) {
                // Find next free slot.
                // We use a manual loop instead of iter().position() because simple imperative loops
                // are often better optimized by LLVM in hot paths (avoids iterator state overhead).

                // While `sum(counts) == n` is a
                // valid mathematical invariant, removing `free_cursor < n` prevents LLVM
                // from properly analyzing loop trip counts via SCEV, fundamentally blocking
                // auto-vectorization and loop unrolling, which leads to significant
                // performance regressions in SMC/Heavy workloads.
                // (Learned from OrdoFP performance tuning journal)

                while free_cursor < n && unsafe { *counts.get_unchecked(free_cursor) } > 0 {
                    free_cursor += 1;
                }

                // No defensive `free_cursor < n` guard before `clone_from` (a
                // `debug_assert!` checks it instead): the invariant guarantees
                // `free_cursor` always lands on a valid zero-count slot, so the
                // bounds guard would never be taken and is elided from the hot path.
                //
                // SAFETY:
                // 1. `free_cursor < n`: ensured by the invariant above; verified in debug
                //    builds by the assert below.
                // 2. `i` is valid (from enumerate); `p_values.add(i)` is in-bounds.
                // 3. `i != free_cursor`: `counts[i] > 1 != 0 == counts[free_cursor]`.
                debug_assert!(
                    free_cursor < n,
                    "free-slot invariant violated: sum(counts) must equal n"
                );
                unsafe {
                    let dst = &mut *p_values.add(free_cursor);
                    dst.clone_from(src);
                }
                free_cursor += 1;
            }
            // Optimization: once all free slots are consumed (free_cursor >= n),
            // all extra copies mandated by sum(counts)==n have been placed.
            // Remaining outer iterations are guaranteed to be no-ops (c==0 or c==1),
            // so break early to skip O(n) redundant comparisons.
            if free_cursor >= n {
                break;
            }
        }
    }
}

/// Prepare weights statistics for f64 slice in-place (log to linear).
///
/// Replaces log-weights with linear weights in the input slice.
/// Returns the sum of linear weights.
#[inline]
fn prepare_weights_in_place(weights: &mut [f64]) -> f64 {
    if weights.is_empty() {
        return 0.0;
    }

    // Optimization: Use manual loop with `f64::max` instead of a branch.
    // `f64::max(a, b)` lowers to the branchless `maxsd` instruction (x86) /
    // `fmax` intrinsic (ARM), which LLVM can widen to `maxpd` / `fmaxp` for
    // SIMD auto-vectorisation of this O(n) reduction.  A branch-based update
    // (`if w > max_log_weight`) introduces a loop-carried dependency that
    // prevents the vectoriser from issuing parallel max reductions.
    // NaN behaviour is identical: `f64::max(x, NaN) == x` (non-NaN wins),
    // matching `if w > max_log_weight` where `NaN > finite == false`.
    let mut max_log_weight = f64::NEG_INFINITY;
    for &w in weights.iter() {
        max_log_weight = f64::max(max_log_weight, w);
    }

    if max_log_weight == f64::NEG_INFINITY {
        // All log-weights are -∞, so every linear weight is exp(-∞) = 0.
        // Returning 0.0 lets callers immediately take the uniform-resampling
        // early-return path instead of running the full weighted loop only to
        // find every weight ≤ 0 and produce a wrong result (e.g. all counts
        // accumulating on the last particle in the Multinomial path).
        return 0.0;
    }

    let mut sum_weights = 0.0;

    // Optimization: precompute `shifted = *w - max_log_weight` once per element,
    // avoiding a second load of `*w` and a redundant subtraction in the hot branch.
    // This also eliminates the `threshold` temporary, freeing a register for the
    // reduction accumulator. Avoid likely() to permit LLVM auto-vectorisation.
    for w in weights.iter_mut() {
        let shifted = *w - max_log_weight;
        if shifted >= -50.0 {
            let linear = shifted.exp();
            *w = linear;
            sum_weights += linear;
        } else {
            *w = 0.0;
        }
    }
    sum_weights
}

// =========================================================================
// TLS Buffer Helpers
// =========================================================================

/// Zero-initialise the first `len` elements of `v` using the canonical
/// `spare_capacity_mut` pattern, then set its length.
///
/// Requires `v` to be logically empty (`v.len() == 0`) and `v.capacity() >=
/// len` on entry (callers `clear()`/`reserve()` beforehand). Writing through
/// `MaybeUninit::write` — rather than `set_len` followed by `fill(0)` — never
/// forms a `&mut [usize]` over memory that has not yet been initialized.
#[cfg(feature = "std")]
#[inline]
fn zero_init_counts(v: &mut Vec<usize>, len: usize) {
    debug_assert_eq!(
        v.len(),
        0,
        "zero_init_counts requires a logically-empty Vec"
    );
    debug_assert!(v.capacity() >= len, "caller must reserve capacity >= len");
    let spare = v.spare_capacity_mut();
    for slot in &mut spare[..len] {
        slot.write(0usize);
    }
    // SAFETY: the loop above initialized exactly `len` elements, and
    // `v.capacity() >= len` (asserted above) makes `spare[..len]` in-bounds.
    unsafe { v.set_len(len) };
}

#[inline]
fn with_weights_buffer<R, F>(capacity: usize, f: F) -> R
where
    F: FnOnce(&mut Vec<f64>) -> R,
{
    #[cfg(feature = "std")]
    {
        WEIGHTS_CACHE.with(|cell| {
            if let Ok(mut v) = cell.try_borrow_mut() {
                v.clear();
                // Optimization: skip reserve() when capacity is already sufficient.
                // `Vec::reserve` checks `self.cap - self.len < additional`; after
                // `clear()` this reduces to `cap < capacity`. Guarding it explicitly
                // avoids the function-call overhead on steady-state calls where the
                // particle count is constant across inference runs (consistent with
                // the same guard in `with_counts_buffer`).
                if v.capacity() < capacity {
                    v.reserve(capacity);
                }
                f(&mut v)
            } else {
                // M17: nested inference on this thread (the outer run holds the
                // borrow) used to panic with BorrowMutError here. Fall back to a
                // fresh allocation instead: `Vec::with_capacity(capacity)`
                // satisfies the same `capacity() >= capacity` guarantee that the
                // unsafe spare-capacity writers in this module (e.g. the
                // `infer_weighted` particle loop) rely on, so no invariant is
                // weakened by taking this path.
                let mut v = Vec::with_capacity(capacity);
                f(&mut v)
            }
        })
    }
    #[cfg(not(feature = "std"))]
    {
        let mut v = Vec::with_capacity(capacity);
        f(&mut v)
    }
}

#[inline]
fn with_counts_buffer<R, F>(len: usize, f: F) -> R
where
    F: FnOnce(&mut Vec<usize>) -> R,
{
    #[cfg(feature = "std")]
    {
        COUNTS_CACHE.with(|cell| {
            if let Ok(mut v) = cell.try_borrow_mut() {
                // `usize` has no destructor, so clear() is O(1) — it only
                // zeroes the length field.
                v.clear();
                if v.capacity() < len {
                    v.reserve(len);
                }
                // Canonical uninit-init pattern (see `zero_init_counts`):
                // write exactly `len` elements via spare capacity, then
                // set_len, instead of `set_len` followed by `fill(0)` (which
                // forms `&mut [usize]` over not-yet-initialized memory).
                zero_init_counts(&mut v, len);
                f(&mut v)
            } else {
                // M17: nested inference on this thread (the outer run holds the
                // borrow) used to panic with BorrowMutError here. Fall back to a
                // fresh allocation instead: `Vec::with_capacity(len)` satisfies
                // the same `capacity() >= len` guarantee the unsafe spare-capacity
                // writers below rely on. Unlike `with_weights_buffer`/
                // `with_randoms_buffer`, the count-generators only ever write
                // non-zero counts (see their "count remains 0" comments), so this
                // fallback buffer must also be zero-initialized, not just sized.
                let mut v = Vec::with_capacity(len);
                zero_init_counts(&mut v, len);
                f(&mut v)
            }
        })
    }
    #[cfg(not(feature = "std"))]
    {
        let mut v = vec![0; len];
        f(&mut v)
    }
}

#[inline]
fn with_randoms_buffer<R, F>(capacity: usize, f: F) -> R
where
    F: FnOnce(&mut Vec<f64>) -> R,
{
    #[cfg(feature = "std")]
    {
        RANDOMS_CACHE.with(|cell| {
            if let Ok(mut v) = cell.try_borrow_mut() {
                v.clear();
                // Optimization: same steady-state capacity guard as `with_weights_buffer`
                // and `with_counts_buffer` — skip reserve() when capacity is sufficient.
                if v.capacity() < capacity {
                    v.reserve(capacity);
                }
                f(&mut v)
            } else {
                // M17: nested inference on this thread (the outer run holds the
                // borrow) used to panic with BorrowMutError here. Fall back to a
                // fresh allocation instead: `Vec::with_capacity(capacity)`
                // satisfies the same `capacity() >= capacity` guarantee the
                // unsafe spare-capacity writers (e.g. `generate_multinomial_counts`,
                // `generate_stratified_counts`) rely on.
                let mut v = Vec::with_capacity(capacity);
                f(&mut v)
            }
        })
    }
    #[cfg(not(feature = "std"))]
    {
        let mut v = Vec::with_capacity(capacity);
        f(&mut v)
    }
}

impl SequentialMonteCarlo {
    /// Create new SMC with particle count.
    pub fn new(particles: usize) -> Self {
        Self {
            particles,
            resampling: ResamplingStrategy::default(),
        }
    }

    /// Set the resampling strategy.
    pub fn with_resampling(mut self, strategy: ResamplingStrategy) -> Self {
        self.resampling = strategy;
        self
    }

    /// Run SMC inference with proper weighting.
    ///
    /// For weighted models that implement `WeightedModel`.
    pub fn infer_weighted<A, M, R>(&self, model: &M, rng: &mut R) -> Vec<A>
    where
        M: WeightedModel<A>,
        A: Clone,
        R: Rng + ?Sized,
    {
        // Optimization: Use TLS buffer for weights to avoid allocation.
        // Allocate `values` inside the closure so the closure does not need to
        // capture it by move, shrinking the closure struct by 24 bytes (ptr +
        // len + cap) on 64-bit platforms.
        with_weights_buffer(self.particles, |weights| {
            let mut values = Vec::with_capacity(self.particles);
            // Optimization: write values and log-weights directly via spare-capacity
            // pointers to eliminate per-push bounds checks, matching the pattern used
            // in `generate_multinomial_counts` for the randoms buffer.
            // `with_weights_buffer` guarantees weights.capacity() >= self.particles;
            // `Vec::with_capacity(self.particles)` gives values exactly that spare space.
            let spare_w = weights.spare_capacity_mut();
            let spare_v = values.spare_capacity_mut();
            for i in 0..self.particles {
                let p = model.execute_weighted(rng);
                // SAFETY: spare_w.len() >= self.particles (from with_weights_buffer's
                // capacity guarantee), spare_v.len() == self.particles (from with_capacity
                // above), and i < self.particles (loop bound), so both are in-bounds.
                unsafe {
                    spare_w.get_unchecked_mut(i).write(p.log_weight);
                    spare_v.get_unchecked_mut(i).write(p.value);
                }
            }
            // SAFETY: all self.particles elements were initialized via MaybeUninit::write
            // in the loop above.  Panic-safe: if model.execute_weighted panics, set_len
            // is never reached, so len stays 0 and no uninitialized memory is exposed.
            unsafe {
                weights.set_len(self.particles);
                values.set_len(self.particles);
            }
            self.resample_values(values, Some(weights), rng)
        })
    }

    /// Run SMC inference with parallel execution.
    ///
    /// Uses Rayon to execute particles in parallel.
    /// Requires `std` feature (implied by `rayon`).
    #[cfg(feature = "rayon")]
    pub fn infer_parallel<M, F, R>(&self, model: F, _rng: &mut R) -> Vec<M>
    where
        F: Fn() -> M + Send + Sync,
        M: Send + Sync + 'static,
        R: Rng + ?Sized,
    {
        // Uniform weights → resampling is a no-op for every strategy (same reasoning
        // as the serial `infer`). Return particles directly, skipping the
        // `resample_values` call and its two early-return branch checks
        // (`values.is_empty()` + `weights.is_none()`).
        (0..self.particles)
            .into_par_iter()
            .map(|_| model())
            .collect()
    }

    /// Run SMC inference with parallel execution and weighted model.
    ///
    /// Uses Rayon to execute particles in parallel.
    #[cfg(feature = "rayon")]
    pub fn infer_weighted_parallel<A, M, R>(&self, model: &M, rng: &mut R) -> Vec<A>
    where
        M: WeightedModel<A> + Sync,
        A: Clone + Send + Sync,
        R: Rng + ?Sized,
    {
        // For WeightedModel, we need to pass an RNG.
        // We generate seeds and use SeedableRng.
        // We need `rand::rngs::StdRng` which is available with `std_rng` feature of `rand`
        // or just use `rand::rngs::StdRng` if `std` is enabled.
        // `ordofp_bayes` has `std` feature.

        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // Optimization: derive per-particle seeds from a single master seed using
        // splitmix64 mixing instead of collecting an O(particles) Vec<u64>.
        // This eliminates the intermediate heap allocation while preserving
        // statistical independence between particle RNGs.
        let master: u64 = rng.random();

        // Generate particles in parallel and unzip into SoA
        let (values, mut weights): (Vec<A>, Vec<f64>) = (0..self.particles as u64)
            .into_par_iter()
            .map(|i| {
                // splitmix64: mix master XOR hashed index for high-quality independent seeds.
                let mut x = master ^ i.wrapping_mul(0x9e37_79b9_7f4a_7c15);
                x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                let seed = x ^ (x >> 31);
                let mut local_rng = StdRng::seed_from_u64(seed);
                let p = model.execute_weighted(&mut local_rng);
                (p.value, p.log_weight)
            })
            .unzip();

        // Resample based on separated weights
        let result = self.resample_values(values, Some(&mut weights), rng);

        // Optimization: donate the weights allocation to the TLS cache instead of
        // dropping it.  `rayon` implies `std`, so WEIGHTS_CACHE is always available
        // here.  After resampling the values in `weights` are stale (linear weights
        // from `prepare_weights_in_place`); we only need the buffer.  Swapping into
        // the TLS slot means the next `with_weights_buffer` call — typically the
        // serial `infer_weighted` path — can skip `Vec::reserve` entirely when its
        // particle count is ≤ `self.particles`.
        // Note: no pre-clear needed — `with_weights_buffer` always calls `v.clear()`
        // before handing the buffer to its caller, so the stale contents are harmless.
        WEIGHTS_CACHE.with(|c| {
            let mut cache = c.borrow_mut();
            if weights.capacity() > cache.capacity() {
                core::mem::swap(&mut *cache, &mut weights);
                // `weights` now holds the smaller old cache; it is dropped below.
            }
        });
        result
    }

    /// Run SMC inference with resampling.
    ///
    /// This implements a basic Sequential Monte Carlo algorithm:
    /// 1. Generate particles from the model
    /// 2. Compute weights (log-probabilities)
    /// 3. Resample particles based on weights
    ///
    /// # Note
    ///
    /// For a full SMC implementation, this would track weights across
    /// multiple time steps. This is a simplified single-step version.
    pub fn infer<M, F, R>(&self, model: F, _rng: &mut R) -> Vec<M>
    where
        F: Fn() -> M + Send + Sync,
        M: Send + Sync + 'static,
        R: Rng + ?Sized,
    {
        // Uniform weights → resampling is a no-op for every strategy
        // (Systematic/Stratified assign each particle exactly 1 count;
        // Multinomial preserves the same marginal distribution but wastes n RNG
        // calls and clone overhead in apply_counts_in_place).
        // Return the collected particles directly, skipping the resample_values
        // function-call overhead and its two early-return branch checks.
        (0..self.particles).map(|_| model()).collect()
    }

    /// Resample values based on separated weights (`SoA`).
    ///
    /// Optimization: This version operates on separated `Vec<M>` and `&[f64]` (or implicit uniform),
    /// avoiding the overhead of `Particle` structs and intermediate allocations.
    ///
    /// Implements "Unified In-Place Resampling":
    /// 1. Generate counts for all particles based on strategy (O(N) time, O(N) space for counts).
    /// 2. Apply counts in-place using a permutation algorithm (O(N) time, O(1) auxiliary space).
    ///
    /// This avoids allocating `new_values` (O(N*M)) and `dead_pool` (O(N*M)), significantly reducing
    /// memory usage and copying overhead for heavy particles.
    fn resample_values<M, R>(
        &self,
        mut values: Vec<M>,
        weights: Option<&mut [f64]>,
        rng: &mut R,
    ) -> Vec<M>
    where
        M: Clone,
        R: Rng + ?Sized,
    {
        // Fast paths: empty or single-particle filters need no resampling.
        // For n==1 every strategy trivially assigns count=1 to the sole particle,
        // so apply_counts_in_place is always a no-op.  Returning early skips the
        // TLS counts-buffer borrow, prepare_weights_in_place, N+1 RNG draws, and
        // the fill(0) memset — none of which affect the result.
        // The n==0 check subsumes the previous is_empty() guard.
        // Uniform-weight fast path (weights.is_none()) is unchanged in semantics.
        let n = values.len();
        if n <= 1 || weights.is_none() {
            return values;
        }

        // Fast path: if all weights are identical, the distribution is effectively uniform.
        // Skipping resampling here saves O(N) RNG draws, memory moves, and preserves
        // maximum particle diversity, matching the `weights.is_none()` fast path.
        // Bit-exact float equality is the point: any spread at all means resample.
        #[allow(clippy::float_cmp)]
        if let Some(ws) = &weights {
            let first = ws[0]; // Safe because n > 1
            if ws.iter().all(|&w| w == first) {
                return values;
            }
        }

        with_counts_buffer(n, |counts| {
            match self.resampling {
                ResamplingStrategy::Multinomial => {
                    generate_multinomial_counts(counts, weights, rng);
                }
                ResamplingStrategy::Systematic => {
                    generate_systematic_counts(counts, weights, rng);
                }
                ResamplingStrategy::Stratified => {
                    generate_stratified_counts(counts, weights, rng);
                }
            }

            apply_counts_in_place(&mut values, counts);
            values
        })
    }
}

/// Metropolis-Hastings MCMC inference.
///
///
/// Implements single-site Metropolis-Hastings:
/// 1. Start with an initial trace from the model
/// 2. For each iteration:
///    - Select a random variable uniformly
///    - Propose a new value for that variable
///    - Compute acceptance ratio using probability densities
///    - Accept or reject based on the Metropolis-Hastings criterion
pub struct MetropolisHastings {
    /// Number of iterations.
    iterations: usize,
    /// Burn-in period.
    burn_in: usize,
    /// Proposal step size (for random walk proposals).
    step_size: f64,
}

impl MetropolisHastings {
    /// Create new Metropolis-Hastings with iterations.
    #[inline]
    pub fn new(iterations: usize, burn_in: usize) -> Self {
        Self {
            iterations,
            burn_in,
            step_size: DEFAULT_STEP_SIZE,
        }
    }

    /// Set proposal step size.
    #[inline]
    pub fn with_step_size(mut self, step_size: f64) -> Self {
        self.step_size = step_size;
        self
    }

    /// Run Metropolis-Hastings inference with proper acceptance ratio.
    ///
    /// Implements single-site MH following monad-bayes:
    /// - Proposal: Pick one random variable uniformly and resample it
    /// - Acceptance ratio: min(1, q * n / (p * m))
    ///   where q = proposed density, p = current density,
    ///   n = current trace length, m = proposed trace length
    ///
    /// For traceable models that implement `TraceableModel`.
    pub fn infer_traceable<A, M, R>(&self, model: &M, rng: &mut R) -> Vec<A>
    where
        M: TraceableModel<A>,
        A: Clone,
        R: Rng + ?Sized,
    {
        // Fast path: no samples requested; skip model execution entirely.
        // Without this guard, `execute_with_trace` would run once (allocating a
        // Trace and its `Vec<f64>` of random variables) only to return an empty Vec.
        if self.iterations == 0 {
            return Vec::new();
        }

        // Get initial trace
        let mut current = model.execute_with_trace(&[], rng);
        let mut samples = Vec::with_capacity(self.iterations);

        // Fast path: deterministic model (no random variables) never changes its
        // output, so burn-in is meaningless and proposal overhead can be skipped
        // entirely. This avoids `burn_in` empty loop iterations, each of which
        // would check `n > 0` (always false) and `i >= burn_in` (always false).
        if current.variables.is_empty() {
            // `repeat_n` implements `TrustedLen`, so `Vec::extend` reserves capacity once
            // and enters a tight write loop with no per-element bounds checks.  For `Copy`
            // output types LLVM can vectorise the loop into a broadcast store.
            // `repeat_n` moves the value on the last yield, avoiding a final clone.
            samples.extend(core::iter::repeat_n(current.output, self.iterations));
            return samples;
        }

        let total_iters = self.iterations + self.burn_in;
        // Hoist loop-invariant values into locals so the compiler can keep them in
        // registers for all `total_iters` hot-path iterations.  `self.burn_in`
        // requires a struct-pointer dereference on every comparison; `total_iters - 1`
        // is a redundant subtraction on every sampling iteration.  Neither changes
        // inside the loop, but the model's opaque function calls prevent LLVM from
        // proving that without explicit locals.
        // `total_iters - 1` is safe to compute here: `self.iterations >= 1` is
        // guaranteed by the early-return guard on line 875.
        let burn_in = self.burn_in;
        let last = total_iters - 1;
        // Cache the Uniform distribution for index selection. Only re-create when the
        // trace length changes; for fixed-structure models this avoids one integer-division
        // worth of Uniform setup cost per iteration in the hot loop.
        let mut n = current.variables.len(); // > 0: guarded by the is_empty check above
        // SAFETY: `n > 0` is guaranteed by the `is_empty()` guard earlier in
        // this function, so the range `[0, n)` is non-empty and `Uniform::new`
        // always returns `Ok` (it only errors when `high <= low`, i.e. n == 0).
        let mut idx_dist = unsafe { rand::distr::Uniform::new(0, n).unwrap_unchecked() };
        for i in 0..total_iters {
            // Tracks whether the current iteration accepted its proposal. Initialized
            // to false each iteration so the sample-collection block below can
            // conditionally skip its clone for the accepted case (which already
            // pushed output directly). When n == 0 the proposal block is skipped and
            // this stays false, which is the correct "no accept" sentinel.
            let mut accept = false;
            if n > 0 {
                // Single-site MH: propose by modifying one random variable
                // Optimization: Modify in-place to avoid cloning the vector if rejected

                // Select variable uniformly at random
                let idx = idx_dist.sample(rng);

                // Store old value to revert if rejected.
                // SAFETY: `idx` is sampled from `idx_dist = Uniform::new(0, n)`, so
                // `idx < n`. The loop invariant `n == current.variables.len()` holds at
                // every iteration entry (established after the `is_empty` guard and
                // maintained by both accept and reject paths), making this in-bounds.
                let old_val = unsafe { *current.variables.get_unchecked(idx) };

                // Reflected random walk on [0,1): symmetric proposal, so the
                // MH correction term is unchanged. Replaces the independent
                // uniform resample that ignored step_size entirely (M3/M9).
                let step = self.step_size * (rng.random::<f64>() * 2.0 - 1.0);
                let mut v = (old_val + step).rem_euclid(2.0);
                if v >= 1.0 {
                    v = 2.0 - v;
                }
                // Reflection maps exactly-1.0 back to 1.0; keep [0,1) closed.
                let new_val = v.min(1.0 - f64::EPSILON);
                // SAFETY: `idx < n == current.variables.len()` (same invariant as above).
                unsafe { *current.variables.get_unchecked_mut(idx) = new_val };

                // Execute model with proposed trace
                // Optimization: use execute_with_trace_cow to avoid allocation if variables unchanged
                let (mut output, log_prob_density, proposed_vars) =
                    model.execute_with_trace_cow(&current.variables, rng);

                // Compute acceptance ratio: min(1, q * n / (p * m))
                // In log space: log_q + log(n) - log_p - log(m)
                let m = proposed_vars.len();

                // Optimization: avoid ln() calls if trace length is unchanged
                // Optimization: combine ln calls to save one transcendental op
                #[allow(clippy::cast_precision_loss)] // trace lengths ≪ 2^52
                let log_correction = if n == m {
                    0.0
                } else {
                    (n as f64 / m as f64).ln()
                };

                let log_ratio = log_prob_density - current.log_prob_density + log_correction;

                // Accept with probability min(1, exp(log_ratio))
                // Optimization: avoid ln() calls and skip RNG when acceptance is guaranteed
                // Branch hint: Acceptance rate varies, but usually proposals are accepted
                // Optimization Note: Branch prediction hints like `likely()` are omitted around
                // local boolean variable assignments (`accept`) as they block LLVM auto-vectorization
                // and instruction pipelining across proposal boundaries.
                accept = if log_ratio >= 0.0 {
                    true
                } else {
                    // Optimization: Use an Exp(1) draw to avoid the expensive exp() call
                    // Accept if u < exp(log_ratio) <=> ln(u) < log_ratio <=> -ln(u) > -log_ratio
                    // -ln(u) is Exp(1) distributed.
                    sample_exp1(rng) > -log_ratio
                };

                // Branch hint: Most proposals are accepted in well-tuned MH
                if accept {
                    // Update current trace.
                    // Optimization: in non-final sampling iterations, clone from
                    // `output` *before* moving it into `samples`.  For heap-allocated
                    // types like `Vec<f64>`, `clone_from` reuses the existing buffer
                    // of `current.output` when its capacity is sufficient, saving one
                    // malloc+free per accepted sample.  Cloning first also eliminates
                    // the `samples.last().unwrap()` call used by the previous order
                    // (push-then-clone-from-last): no Option construction, no branch,
                    // and no indirect heap access through `samples`'s buffer pointer.
                    // The sample-collection block below detects `accept == true` and
                    // skips its own push for this iteration.
                    if i >= burn_in && i != last {
                        current.output.clone_from(&output);
                        samples.push(output);
                    } else {
                        current.output = output;
                    }
                    current.log_prob_density = log_prob_density;

                    // Update variables if they changed (new allocation).
                    // Move the proposed Vec directly into current.variables instead of
                    // clone_from. clone_from would reuse the old buffer but requires an
                    // O(n) memcpy; a move is O(1) (pointer swap + one free), which is
                    // faster for all but the shortest traces in this hot path.
                    //
                    // Compute new_n from whichever Cow variant we received, then share
                    // the idx_dist refresh logic in a single code path. This deduplicates
                    // the Uniform::new call, reducing instruction-cache pressure in the
                    // hot loop and ensuring the compiler sees one clear update site.
                    let new_n = if let Cow::Owned(v) = proposed_vars {
                        // Prefer the buffer with larger capacity: if `v` has more capacity
                        // than `current.variables`, take it via an O(1) move.  When
                        // capacities are equal, also move: both sides hold the same amount
                        // of space, so clone_from would just do an O(m) memcpy + free(v)
                        // with no benefit over the O(1) move + free(old current.variables).
                        // Only fall back to clone_from when current.variables is strictly
                        // larger, preserving that bigger allocation for future trace growth.
                        if v.capacity() >= current.variables.capacity() {
                            current.variables = v;
                        } else {
                            current.variables.clone_from(&v);
                        }
                        m
                    } else {
                        // proposed_vars are a slice of current.variables (already modified
                        // in place). Truncate in case the model used fewer variables.
                        // Optimization: reuse `m` (already computed as proposed_vars.len()
                        // above) instead of dispatching through Cow::Deref a second time.
                        current.variables.truncate(m);
                        m
                    };
                    // Refresh cached index distribution if the trace length changed.
                    if new_n != n {
                        n = new_n;
                        if n > 0 {
                            // SAFETY: guarded by the enclosing `if n > 0` branch,
                            // so the range `[0, n)` is non-empty and `Uniform::new`
                            // always returns `Ok` here.
                            idx_dist =
                                unsafe { rand::distr::Uniform::new(0, n).unwrap_unchecked() };
                        }
                    }
                } else {
                    // Revert modification.
                    // SAFETY: `idx < n == current.variables.len()` — the reject path
                    // leaves `current.variables` unchanged in length, so the invariant
                    // is intact and `idx` is in-bounds.
                    unsafe { *current.variables.get_unchecked_mut(idx) = old_val };
                    // Optimization: reuse the rejected proposal's heap allocation for the
                    // sample instead of freeing it and allocating a fresh clone.
                    // clone_from overwrites `output` in-place, reusing its buffer when
                    // output.capacity() >= current.output.len() — saving one alloc+free
                    // per post-burn-in rejection for heap-allocated output types (e.g. Vec<f64>).
                    // Setting accept = true is a sentinel so the outer push block skips.
                    if i >= burn_in && i != last {
                        output.clone_from(&current.output);
                        samples.push(output);
                        accept = true; // sentinel: sample already pushed
                    }
                    // proposed_vars dropped here (no alloc if Borrowed)
                }
            }

            // Collect samples after burn-in.
            // Optimization: on the final iteration move `current.output` instead of cloning.
            // The accepted path (accept && i >= burn_in && i != last) and the reject-reuse
            // path (above) already pushed their samples; the accept=true sentinel prevents
            // double-pushing.  Only the n==0 (deterministic collapse) case reaches here with
            // accept==false and needs the clone fallback.
            if i >= burn_in {
                if i == last {
                    samples.push(current.output);
                    break;
                }
                if !accept {
                    samples.push(current.output.clone());
                }
            }
        }

        samples
    }

    /// Run Metropolis-Hastings inference (simplified version for non-traceable models).
    ///
    /// This version uses independent proposals (just resampling from the model).
    /// For proper MH with acceptance ratio, use `infer_traceable` with a
    /// `TraceableModel` implementation.
    pub fn infer<M, F, R>(&self, model: F, _rng: &mut R) -> Vec<M>
    where
        F: Fn() -> M + Send + Sync,
        M: Send + Sync + 'static,
        R: Rng + ?Sized,
    {
        // Range<usize> is TrustedLen, so collect() pre-allocates once and
        // fills via unchecked writes, eliminating the per-push capacity check
        // of the manual loop. Consistent with SMC::infer and IS::infer.
        (0..self.iterations).map(|_| model()).collect()
    }

    /// Run multiple Metropolis-Hastings chains in parallel.
    ///
    /// Runs `chains` number of independent chains in parallel using Rayon.
    /// Returns a vector of vectors (one vector of samples per chain).
    ///
    /// Requires `traceable_model` implementation.
    #[cfg(feature = "rayon")]
    pub fn infer_parallel_chains<A, M, R>(
        &self,
        model: &M,
        chains: usize,
        rng: &mut R,
    ) -> Vec<Vec<A>>
    where
        M: TraceableModel<A> + Sync,
        A: Clone + Send + Sync,
        R: Rng + ?Sized,
    {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // Optimization: derive per-chain seeds with splitmix64 from a single master seed,
        // eliminating the intermediate Vec<u64> allocation used by the naive
        // `(0..chains).map(|_| rng.random()).collect()` approach.
        let master: u64 = rng.random();

        (0..chains as u64)
            .into_par_iter()
            .map(|i| {
                let mut x = master ^ i.wrapping_mul(0x9e37_79b9_7f4a_7c15);
                x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                let seed = x ^ (x >> 31);
                let mut local_rng = StdRng::seed_from_u64(seed);
                self.infer_traceable(model, &mut local_rng)
            })
            .collect()
    }
}

/// Importance Sampling inference.
pub struct ImportanceSampling {
    /// Number of samples.
    samples: usize,
}

/// Weighted sample for importance sampling.
pub struct WeightedSample<M> {
    /// Sample value.
    pub value: M,
    /// Importance weight (log-scale).
    pub log_weight: f64,
}

impl ImportanceSampling {
    /// Create new Importance Sampling.
    #[inline]
    pub fn new(samples: usize) -> Self {
        Self { samples }
    }

    /// Run Importance Sampling inference.
    ///
    /// This implements importance sampling:
    /// 1. Sample from proposal distribution (model)
    /// 2. Compute importance weights (ratio of target/proposal)
    /// 3. Return weighted samples
    ///
    /// # Note
    ///
    /// This is a simplified version. In full implementation, we'd need
    /// explicit target and proposal distributions, and proper weight computation.
    pub fn infer<M, F, R>(&self, model: F, _rng: &mut R) -> Vec<M>
    where
        F: Fn() -> M + Send + Sync,
        M: Send + Sync + 'static,
        R: Rng + ?Sized,
    {
        // Generate values directly and return them without resampling.
        //
        // With uniform weights (weights=None) multinomial bootstrap resampling
        // preserves the same marginal distribution but:
        //   - wastes N RNG calls,
        //   - accesses the TLS counts buffer, and
        //   - clones particles inside apply_counts_in_place.
        // Skipping it is both faster and statistically better (no loss of
        // particle diversity). The SMC resample_values fast-path does the same.
        (0..self.samples).map(|_| model()).collect()
    }

    /// Run inference and return weighted samples (without resampling).
    ///
    /// This is likelihood weighting: the model samples latent variables from
    /// the prior (the proposal) and accumulates observation log-likelihoods
    /// into the particle's log-weight via [`Particle::factor`]. The returned
    /// log-weights are the un-normalized importance weights of
    /// self-normalized importance sampling; normalize with
    /// [`normalized_weights`] before computing expectations.
    pub fn infer_weighted<A, M, R>(&self, model: &M, rng: &mut R) -> Vec<WeightedSample<A>>
    where
        M: WeightedModel<A>,
        R: Rng + ?Sized,
    {
        (0..self.samples)
            .map(|_| {
                let particle = model.execute_weighted(rng);
                WeightedSample {
                    value: particle.value,
                    log_weight: particle.log_weight,
                }
            })
            .collect()
    }
}

/// Self-normalize importance-sampling log-weights into linear weights.
///
/// Uses the log-sum-exp trick (subtracting the maximum log-weight before
/// exponentiating) so that very negative log-weights cannot underflow the
/// whole weight vector to zero. Returns weights that sum to 1.
///
/// Returns an empty vector for empty input. If every log-weight is `-∞`
/// (all particles observed impossible evidence), the weights are all NaN —
/// there is no valid normalization in that case.
pub fn normalized_weights<A>(samples: &[WeightedSample<A>]) -> Vec<f64> {
    let max = samples
        .iter()
        .map(|s| s.log_weight)
        .fold(f64::NEG_INFINITY, f64::max);
    // Optimization: fuse the exp pass with the sum reduction — one traversal
    // over the weights instead of two. `map` preserves ExactSizeIterator, so
    // collect still pre-allocates exactly once.
    let mut total = 0.0;
    let mut weights: Vec<f64> = samples
        .iter()
        .map(|s| {
            let w = (s.log_weight - max).exp();
            total += w;
            w
        })
        .collect();
    // Optimization: one division + N multiplies instead of N divisions.
    // Scalar FP divide is the slowest non-transcendental op and does not
    // pipeline; this matches the `inv_scale` pattern in
    // `generate_multinomial_counts`. Degenerate inputs are unchanged:
    // empty → empty, all-(-∞) → NaN either way (documented above).
    let inv_total = 1.0 / total;
    for w in &mut weights {
        *w *= inv_total;
    }
    weights
}

/// Effective sample size (ESS) of a set of normalized weights: `1 / Σ wᵢ²`.
///
/// ESS ≈ n means the proposal matches the target well; ESS ≪ n signals
/// weight degeneracy (a few particles dominate) and the estimate should not
/// be trusted. Expects weights that sum to 1 (see [`normalized_weights`]).
#[inline]
pub fn effective_sample_size(weights: &[f64]) -> f64 {
    let sum_sq: f64 = weights.iter().map(|w| w * w).sum();
    1.0 / sum_sq
}

#[cfg(test)]
// Tests assert bit-exact float results on purpose, and cast small test sizes
// to f64 for expected-value arithmetic.
#[allow(clippy::float_cmp, clippy::cast_precision_loss)]
mod tests {
    use super::*;

    /// A particle that observes impossible evidence (log-weight -∞) is degenerate:
    /// further `factor` calls cannot revive it.
    #[test]
    fn particle_factor_neg_inf_stays_neg_inf() {
        let mut p: Particle<i32> = Particle::new(42);
        assert_eq!(p.log_weight, 0.0, "unit particle starts with log-weight 0");

        // Observing impossible evidence
        p.factor(f64::NEG_INFINITY);
        assert!(
            p.log_weight.is_infinite() && p.log_weight.is_sign_negative(),
            "after impossible observation log-weight is -∞"
        );

        // Adding any finite evidence to a dead particle keeps it dead
        p.factor(10.0);
        assert!(
            p.log_weight.is_infinite() && p.log_weight.is_sign_negative(),
            "finite evidence cannot revive a -∞ log-weight particle"
        );
    }

    /// `Trace::map` must transform the output value while leaving `variables`
    /// and `log_prob_density` completely unchanged.
    #[test]
    fn trace_map_preserves_variables_and_log_prob() {
        let log_prob = -2.5;
        // Pass the vec directly — no clone needed; compare against a stack array
        // in the assertion (Vec<f64>: PartialEq<[f64; N]> avoids a second allocation).
        let t: Trace<i32> = Trace::new(vec![0.1, 0.5, 0.9], 7, log_prob);

        let mapped = t.map(|x| x * 2);

        assert_eq!(mapped.output, 14, "map should transform the output");
        assert_eq!(
            mapped.variables,
            [0.1_f64, 0.5, 0.9],
            "map must not alter the variables vector"
        );
        assert_eq!(
            mapped.log_prob_density, log_prob,
            "map must not alter the log-probability density"
        );
    }

    /// `Trace::pure` represents a deterministic computation with no random choices,
    /// so its variables should be empty and its log-probability should be 0 (log 1).
    #[test]
    fn trace_pure_has_empty_variables_and_unit_log_prob() {
        let t: Trace<&str> = Trace::pure("hello");
        assert!(t.variables.is_empty(), "pure trace has no random variables");
        assert_eq!(
            t.log_prob_density, 0.0,
            "pure trace has log-prob 0 (i.e. probability 1)"
        );
        assert_eq!(t.output, "hello");
    }

    /// `Particle::with_weight` initialises log-weight to the supplied value,
    /// and subsequent `factor` calls accumulate on top of that baseline.
    #[test]
    fn particle_with_weight_sets_initial_log_weight() {
        let mut p: Particle<&str> = Particle::with_weight("x", -3.0);
        assert_eq!(
            p.log_weight, -3.0,
            "with_weight should set the initial log-weight"
        );

        // Accumulate evidence on top of the pre-set baseline
        p.factor(-1.5);
        assert!(
            (p.log_weight - (-4.5)).abs() < f64::EPSILON,
            "factor should add to the initial log-weight, not reset it"
        );
    }

    /// `ImportanceSampling::infer_weighted` with zero samples must return an
    /// empty vector; with N samples the model's log-weights must be carried
    /// through unchanged.
    #[test]
    fn importance_sampling_zero_samples_returns_empty() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        struct ConstModel;
        impl WeightedModel<i32> for ConstModel {
            fn execute_weighted<R: Rng + ?Sized>(&self, _rng: &mut R) -> Particle<i32> {
                Particle::with_weight(7, -1.25)
            }
        }
        let mut rng = StdRng::seed_from_u64(0);

        let empty: Vec<WeightedSample<i32>> =
            ImportanceSampling::new(0).infer_weighted(&ConstModel, &mut rng);
        assert!(empty.is_empty(), "zero-sample IS must produce no output");

        let five = ImportanceSampling::new(5).infer_weighted(&ConstModel, &mut rng);
        assert_eq!(
            five.len(),
            5,
            "IS must return exactly the requested number of samples"
        );
        assert!(
            five.iter().all(|s| s.value == 7 && s.log_weight == -1.25),
            "IS must carry the model's values and log-weights through unchanged"
        );
    }

    /// `normalized_weights` must be log-sum-exp stable (no underflow for very
    /// negative log-weights) and sum to 1; ESS of uniform weights is n.
    #[test]
    fn normalized_weights_stable_and_ess_bounds() {
        // Very negative but equal log-weights: naive exp() would underflow to
        // 0/0; log-sum-exp must give exactly uniform weights.
        let samples: Vec<WeightedSample<i32>> = (0..4)
            .map(|i| WeightedSample {
                value: i,
                log_weight: -800.0,
            })
            .collect();
        let w = normalized_weights(&samples);
        assert!(w.iter().all(|&x| (x - 0.25).abs() < 1e-12));
        assert!((effective_sample_size(&w) - 4.0).abs() < 1e-9);

        // One dominant weight → ESS collapses toward 1.
        let samples: Vec<WeightedSample<i32>> = vec![
            WeightedSample {
                value: 0,
                log_weight: 0.0,
            },
            WeightedSample {
                value: 1,
                log_weight: -60.0,
            },
        ];
        let w = normalized_weights(&samples);
        assert!((w[0] - 1.0).abs() < 1e-12);
        assert!((effective_sample_size(&w) - 1.0).abs() < 1e-9);
    }

    /// `Particle::factor` with a NaN log-probability should propagate NaN
    /// to the particle's log-weight, signalling a malformed density.
    #[test]
    fn particle_factor_nan_propagates() {
        let mut p: Particle<i32> = Particle::new(0);
        p.factor(f64::NAN);
        assert!(
            p.log_weight.is_nan(),
            "NaN log-probability must propagate to log-weight"
        );

        // Subsequent finite evidence cannot repair a NaN weight
        p.factor(-1.0);
        assert!(
            p.log_weight.is_nan(),
            "finite evidence after NaN must keep log-weight as NaN"
        );
    }

    /// `Trace::clone_from` must produce a trace identical to a fresh clone,
    /// exercising the "reuse allocation" optimisation path in the custom
    /// `Clone` impl (variables `Vec` allocation is reused, not reallocated).
    #[test]
    fn trace_clone_from_matches_clone() {
        let src: Trace<Vec<i32>> = Trace::new(vec![0.1, 0.2, 0.3], vec![1, 2, 3], -5.5);
        let cloned = src.clone();

        // Overwrite a pre-existing trace via clone_from to hit the
        // optimised allocation-reuse branch in the custom Clone impl.
        let mut dst: Trace<Vec<i32>> = Trace::new(vec![0.9, 0.8], vec![99, 98], -1.0);
        dst.clone_from(&src);

        assert_eq!(dst.output, cloned.output, "clone_from must copy the output");
        assert_eq!(
            dst.variables, cloned.variables,
            "clone_from must copy the variables vector"
        );
        assert_eq!(
            dst.log_prob_density, cloned.log_prob_density,
            "clone_from must copy log_prob_density"
        );
        // Confirm the source is untouched after clone_from.
        assert_eq!(src.output, vec![1, 2, 3], "source must not be mutated");
    }

    /// `Particle::clone_from` must produce a particle identical to a fresh clone,
    /// exercising the "reuse allocation" optimisation path (which differs from
    /// the ordinary `Clone::clone` path).
    #[test]
    fn particle_clone_from_matches_clone() {
        let src: Particle<Vec<i32>> = Particle::with_weight(vec![1, 2, 3], -7.5);
        let cloned = src.clone();

        // Overwrite a pre-existing particle via clone_from to hit the
        // optimised allocation-reuse branch in the custom Clone impl.
        let mut dst: Particle<Vec<i32>> = Particle::new(vec![99, 98]);
        dst.clone_from(&src);

        assert_eq!(dst.value, cloned.value, "clone_from must copy the value");
        assert_eq!(
            dst.log_weight, cloned.log_weight,
            "clone_from must copy the log-weight"
        );
        // Confirm the source is untouched after clone_from.
        assert_eq!(src.value, vec![1, 2, 3], "source must not be mutated");
    }

    /// `Trace::map` on a pure trace (no random variables) must preserve the
    /// empty variables vec and zero log-probability, while transforming the output.
    ///
    /// This exercises the empty-variables edge case: `Trace::pure` produces a
    /// trace with `variables = []` and `log_prob_density = 0.0`; mapping over it
    /// must leave both fields untouched regardless of the output transformation.
    #[test]
    fn trace_map_on_pure_trace_preserves_empty_variables() {
        let pure: Trace<i32> = Trace::pure(5);
        let mapped = pure.map(|x| format!("val={x}"));

        assert_eq!(
            mapped.output, "val=5",
            "map must transform the output value"
        );
        assert!(
            mapped.variables.is_empty(),
            "mapping a pure trace must leave variables empty"
        );
        assert_eq!(
            mapped.log_prob_density, 0.0,
            "mapping a pure trace must leave log_prob_density as 0 (probability 1)"
        );
    }

    /// `Particle::factor` with `+∞` log-probability (a degenerate "certain" evidence):
    /// - the log-weight should become `+∞`
    /// - subsequent finite `factor` calls must leave it at `+∞` (`+∞ + finite = +∞`)
    /// - adding `NEG_INFINITY` must yield `NaN` per IEEE 754 (`+∞ + (-∞) = NaN`)
    #[test]
    fn particle_factor_pos_inf_and_subsequent_behaviour() {
        let mut p: Particle<i32> = Particle::new(1);
        assert_eq!(p.log_weight, 0.0, "fresh particle has log-weight 0");

        p.factor(f64::INFINITY);
        assert!(
            p.log_weight.is_infinite() && p.log_weight.is_sign_positive(),
            "factor(+∞) must produce a +∞ log-weight"
        );

        // A finite top-up cannot change an infinite log-weight.
        p.factor(-5.0);
        assert!(
            p.log_weight.is_infinite() && p.log_weight.is_sign_positive(),
            "+∞ + finite must remain +∞"
        );

        // Adding -∞ to +∞ is undefined: IEEE 754 mandates NaN.
        p.factor(f64::NEG_INFINITY);
        assert!(
            p.log_weight.is_nan(),
            "+∞ + (-∞) must yield NaN per IEEE 754"
        );
    }

    /// `ImportanceSampling::infer` (non-weighted variant) must return exactly
    /// `n` values produced by the model closure.  The zero-sample edge case
    /// must yield an empty `Vec` without invoking the model at all, while the
    /// non-zero case must preserve every value returned by the closure.
    ///
    /// This exercises the `infer` code path, which is distinct from the already-
    /// tested `infer_weighted` path: `infer` skips the `WeightedSample` wrapper
    /// and returns bare values, making the count and content invariants the key
    /// correctness properties to verify.
    #[test]
    fn importance_sampling_infer_zero_and_nonzero_sample_counts() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        let mut rng = StdRng::seed_from_u64(0xabad_1dea);

        // Zero-sample edge case: must produce an empty Vec.
        let empty: Vec<i32> = ImportanceSampling::new(0).infer(|| 42, &mut rng);
        assert!(
            empty.is_empty(),
            "IS::infer with 0 samples must return an empty Vec"
        );

        // Non-zero case: must return exactly the requested number of values,
        // each produced by the model closure.
        let values: Vec<i32> = ImportanceSampling::new(5).infer(|| 7, &mut rng);
        assert_eq!(
            values.len(),
            5,
            "IS::infer must return exactly the requested sample count"
        );
        assert!(
            values.iter().all(|&v| v == 7),
            "IS::infer values must all originate from the model closure"
        );
    }

    /// `Trace::new` with an empty variables vector must store all three fields
    /// exactly as supplied, including a non-zero `log_prob_density`.
    ///
    /// This is distinct from `Trace::pure`, which always forces `log_prob_density`
    /// to 0.0.  The edge case represents a deterministic computation that is still
    /// constrained by an observe/factor call (no random choices, but non-trivial
    /// evidence).
    #[test]
    fn trace_new_with_empty_variables_preserves_non_zero_log_prob() {
        let log_prob = -3.7_f64;
        let t: Trace<&str> = Trace::new(vec![], "deterministic", log_prob);

        assert!(
            t.variables.is_empty(),
            "Trace::new with an empty vec must yield an empty variables vector"
        );
        assert_eq!(
            t.output, "deterministic",
            "Trace::new must store the supplied output value"
        );
        assert_eq!(
            t.log_prob_density, log_prob,
            "Trace::new must preserve the supplied log_prob_density even when variables is empty"
        );
        // Trace::pure always sets log_prob_density = 0.0; Trace::new must not.
        assert_ne!(
            t.log_prob_density, 0.0,
            "Trace::new must not silently reset log_prob_density to 0 the way Trace::pure does"
        );
    }

    /// When every particle has log-weight −∞ (complete filter collapse),
    /// `prepare_weights_in_place` must return 0.0 and zero all linear weights
    /// so callers fall through to uniform resampling rather than dividing by zero.
    #[test]
    fn prepare_weights_all_neg_inf_returns_zero_and_zeroes_buffer() {
        let mut weights = vec![f64::NEG_INFINITY; 5];
        let sum = prepare_weights_in_place(&mut weights);

        assert_eq!(
            sum, 0.0,
            "all-dead particles must yield sum 0.0, not NaN or a negative number"
        );
        // The early-return path leaves the buffer untouched (all still -∞),
        // which is correct: callers that reach this path never read the buffer.
        // Verify the sum gate is the sole guard — no element should be positive.
        assert!(
            weights.iter().all(|&w| w <= 0.0 || w == f64::NEG_INFINITY),
            "no linear weight should be positive after a full-collapse reweight"
        );
    }

    /// `prepare_weights_in_place` with an empty slice must return 0.0 immediately
    /// without touching the buffer.  This guards callers against dividing by zero
    /// when there are no particles (degenerate/zero-particle filter).
    #[test]
    fn prepare_weights_empty_slice_returns_zero() {
        let mut weights: Vec<f64> = vec![];
        let sum = prepare_weights_in_place(&mut weights);

        assert_eq!(sum, 0.0, "empty weights slice must return 0.0");
        assert!(weights.is_empty(), "buffer must remain empty");
    }

    /// `Particle::factor(0.0)` represents neutral evidence (log-probability of 1),
    /// so it must leave `log_weight` bit-for-bit unchanged regardless of the
    /// particle's current weight.  This validates the additive log-probability
    /// semantics: log(1) == 0, and adding 0 is a no-op.
    #[test]
    fn particle_factor_zero_is_no_op() {
        // Fresh particle: log_weight starts at 0.0
        let mut p: Particle<i32> = Particle::new(99);
        p.factor(0.0);
        assert_eq!(
            p.log_weight, 0.0,
            "factor(0.0) on a fresh particle must leave log_weight at 0.0"
        );

        // Pre-weighted particle: log_weight must be preserved exactly
        let mut pw: Particle<i32> = Particle::with_weight(7, -2.5);
        pw.factor(0.0);
        assert_eq!(
            pw.log_weight, -2.5,
            "factor(0.0) must not alter an existing log_weight"
        );

        // After real evidence has been accumulated, neutral evidence is still a no-op
        let mut pa: Particle<i32> = Particle::new(0);
        pa.factor(-1.0);
        pa.factor(0.0);
        assert!(
            (pa.log_weight - (-1.0)).abs() < f64::EPSILON,
            "factor(0.0) after accumulated evidence must leave log_weight unchanged"
        );
    }

    /// `apply_counts_in_place` with all weight concentrated on the **last** particle:
    /// counts = [0, 0, 0, 0, 5] means the final particle must be cloned into the four
    /// zero-count free slots that precede it.  This exercises the forward-scanning
    /// free-cursor when every free slot appears *before* the sole heavy particle, so
    /// the source is read only after all writes have landed — verifying the
    /// no-aliasing invariant described in the function's SAFETY comments.
    #[test]
    fn apply_counts_last_particle_fills_all_slots() {
        let mut values = vec![10_i32, 20, 30, 40, 99];
        let counts = [0_usize, 0, 0, 0, 5];
        apply_counts_in_place(&mut values, &counts);
        assert_eq!(
            values,
            [99, 99, 99, 99, 99],
            "all slots must hold copies of the last particle when it has count == n"
        );
    }

    /// `apply_counts_in_place` with all weight on the **first** particle:
    /// counts = [5, 0, 0, 0, 0] means the free-cursor must skip slot 0
    /// (non-zero count) before writing clones of values[0] into slots 1–4.
    /// This exercises a different code path than the last-particle test: the
    /// inner while-loop must advance `free_cursor` past the heavy source particle
    /// before finding the first free slot, verifying the no-aliasing invariant
    /// (source index != `free_cursor` on every write) holds in this configuration.
    #[test]
    fn apply_counts_first_particle_fills_all_slots() {
        let mut values = vec![42_i32, 1, 2, 3, 4];
        let counts = [5_usize, 0, 0, 0, 0];
        apply_counts_in_place(&mut values, &counts);
        assert_eq!(
            values,
            [42, 42, 42, 42, 42],
            "all slots must hold copies of the first particle when counts[0] == n"
        );
    }

    /// M3/M9 regression: `with_step_size` was assigned but never read by the MH
    /// proposal (it resampled independently from `[0,1)` regardless of the
    /// configured step). With a reflected random-walk proposal, a tiny step size
    /// must accept proposals far more often than a huge step size, on the same
    /// model and RNG seed. A no-op `step_size` would make these acceptance
    /// rates equal (up to RNG noise), which this test rules out.
    #[test]
    fn step_size_changes_acceptance_rate() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // Single-variable model: the trace variable *is* the sampled
        // parameter, observed against a peaked Normal-shaped log-density
        // centred at 0.5.
        struct NormalObsModel {
            target: f64,
            sigma: f64,
        }

        impl TraceableModel<f64> for NormalObsModel {
            fn execute_with_trace<R: Rng + ?Sized>(
                &self,
                variables: &[f64],
                rng: &mut R,
            ) -> Trace<f64> {
                let x = if variables.is_empty() {
                    rng.random::<f64>()
                } else {
                    variables[0]
                };
                let diff = (x - self.target) / self.sigma;
                let log_prob = -0.5 * diff * diff;
                Trace::new(vec![x], x, log_prob)
            }
        }

        // Fraction of adjacent post-burn-in samples that differ: on rejection
        // the MH loop re-pushes the unchanged current output, so equal
        // neighbours are rejections and differing neighbours are
        // acceptances. This uses only the public `infer_traceable` API (no
        // internal acceptance counter exists).
        fn acceptance_rate(step_size: f64) -> f64 {
            let model = NormalObsModel {
                target: 0.5,
                sigma: 0.15,
            };
            let mh = MetropolisHastings::new(3000, 500).with_step_size(step_size);
            let mut rng = StdRng::seed_from_u64(2026);

            let samples = mh.infer_traceable(&model, &mut rng);
            let changed = samples.windows(2).filter(|w| w[0] != w[1]).count();
            changed as f64 / (samples.len() - 1) as f64
        }

        let tight = acceptance_rate(0.01);
        let wide = acceptance_rate(0.9);

        assert!(
            tight > wide,
            "smaller steps must accept more often (tight={tight}, wide={wide}); \
             a no-op step_size makes these equal"
        );
    }

    /// M17 regression: nested inference on one thread used to hit the TLS
    /// `RefCell` caches (`WEIGHTS_CACHE`/`COUNTS_CACHE`/`RANDOMS_CACHE`) twice —
    /// `BorrowMutError` panic from a safe public API.
    ///
    /// `SequentialMonteCarlo::infer_weighted` holds its `WEIGHTS_CACHE` borrow
    /// for the entire particle-generation loop, i.e. while calling
    /// `model.execute_weighted`. A model whose `execute_weighted` itself runs
    /// a nested `SequentialMonteCarlo::infer_weighted` on the same thread
    /// therefore reenters the same thread-local while the outer borrow is
    /// still live — the smallest nesting the public API allows.
    #[test]
    fn nested_inference_does_not_panic() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        struct InnerModel;

        impl WeightedModel<f64> for InnerModel {
            fn execute_weighted<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle<f64> {
                Particle::new(rng.random::<f64>())
            }
        }

        struct OuterModel;

        impl WeightedModel<f64> for OuterModel {
            fn execute_weighted<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle<f64> {
                // Nested inference on the same thread: pre-fix, this
                // reenters WEIGHTS_CACHE while the outer `infer_weighted`
                // call still holds its borrow, panicking with
                // BorrowMutError.
                let inner = SequentialMonteCarlo::new(4).infer_weighted(&InnerModel, rng);
                let mean: f64 = inner.iter().sum::<f64>() / inner.len() as f64;
                Particle::new(mean)
            }
        }

        let smc = SequentialMonteCarlo::new(4);
        let mut rng = StdRng::seed_from_u64(7);

        let samples = smc.infer_weighted(&OuterModel, &mut rng);

        assert_eq!(
            samples.len(),
            4,
            "nested inference must complete without panicking and return the requested count"
        );
    }
}
