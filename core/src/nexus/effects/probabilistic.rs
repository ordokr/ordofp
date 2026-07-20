//! Probabilistic Effects
//!
//! This module provides **probabilistic programming primitives** as effects.
//! Users can build probabilistic models by sampling from distributions and
//! conditioning on observations.
//!
//! # Feature Requirements
//!
//! This module requires the `std` feature because floating-point math
//! functions (`ln`, `sqrt`, `exp`, `cos`) are not available in `no_std`.
//!
//! # Key Concepts
//!
//! - **Distribution**: A probability distribution that can be sampled
//! - **Sample**: Draw a random value from a distribution
//! - **Observe**: Condition the model on observed data (soft constraint)
//! - **Score**: Adjust the log-probability of the current execution trace
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::effects::probabilistic::*;
//!
//! // A simple Bayesian model: infer the bias of a coin from observed flips.
//! fn coin_model(flips: [bool; 3]) -> ProbComputation<f64> {
//!     ProbComputation::new(move |ctx| {
//!         // Prior: uniform probability of heads.
//!         let p = ctx.sample(&Uniform::new(0.0, 1.0));
//!
//!         // Likelihood: observe the flips.
//!         for flip in flips {
//!             ctx.observe(&Bernoulli::new(p), &flip);
//!         }
//!
//!         p
//!     })
//! }
//!
//! // Run inference (model, number of samples, RNG seed).
//! let posterior = importance_sample(|| coin_model([true, true, false]), 1000, 42);
//! let mean = posterior.mean();
//! assert!(mean > 0.0 && mean < 1.0);
//! ```
//!
//! # Verification Tier
//!
//! **Tier 1**: Tested via unit tests verifying distribution properties
//! and inference correctness on simple models.
//!
//! # Limitations
//!
//! - **Requires std**: Floating-point math not available in `no_std`
//! - **No automatic differentiation**: Variational inference not supported
//! - **Simple inference only**: importance sampling provided (see `ordofp_bayes`
//!   for rejection-based inference)
//! - **No MCMC**: Metropolis-Hastings requires more infrastructure
//! - **Discrete distributions only for observe**: Continuous observe needs densities
//! - **Not a full PPL**: This is a building block, not a complete system
//!
//! # Design Philosophy
//!
//! This provides the **effect-based foundation** for probabilistic programming.
//! For production use, consider integrating with specialized PPL libraries.
//! We focus on composability with other effects, not inference efficiency.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::nexus::effect::EffectMarker;
use crate::nexus::row::Row;

// =============================================================================
// Effect Marker
// =============================================================================

/// Bit flag for Probabilistic effect.
pub const PROBABILISTIC_BIT: u128 = 1 << 34;

/// The Probabilistic effect marker type.
#[derive(Copy, Clone, Debug)]
pub struct ProbabilisticEffect;

impl EffectMarker for ProbabilisticEffect {
    const BIT: u128 = PROBABILISTIC_BIT;
    const NAME: &'static str = "Probabilistic";
}

/// Type alias for a row containing only Probabilistic.
pub type ProbabilisticRow = Row<PROBABILISTIC_BIT>;

// =============================================================================
// Random Number Generation
// =============================================================================

/// A simple linear congruential generator for reproducible randomness.
///
/// This is NOT cryptographically secure. For real applications,
/// use a proper RNG like `rand` crate.
#[derive(Clone, Debug)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    /// Create a new RNG with a seed.
    pub fn new(seed: u64) -> Self {
        SimpleRng {
            state: seed.wrapping_add(1),
        }
    }

    /// Advance the `SplitMix64` state and return the next value.
    ///
    /// `SplitMix64` (Steele/Lea/Flood 2014, public domain): passes `BigCrush`;
    /// replaces the previous glibc-LCG-constants-mod-2^64 generator whose
    /// low seeds produced degenerate first draws. Not cryptographic.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1): 53 mantissa bits, so 1.0 is unreachable.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Generate a random f64 in [low, high).
    pub fn next_f64_range(&mut self, low: f64, high: f64) -> f64 {
        low + self.next_f64() * (high - low)
    }

    /// Generate a random bool with given probability of true.
    pub fn next_bool(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }
}

impl Default for SimpleRng {
    fn default() -> Self {
        // Default seed based on a fixed value for reproducibility
        SimpleRng::new(42)
    }
}

// =============================================================================
// Distributions
// =============================================================================

/// Trait for probability distributions.
pub trait Distribution: Clone {
    /// The type of values this distribution produces.
    type Value;

    /// Sample a value from this distribution.
    fn sample(&self, rng: &mut SimpleRng) -> Self::Value;

    /// Log probability density/mass at a value.
    ///
    /// Returns `f64::NEG_INFINITY` for impossible values.
    fn log_prob(&self, value: &Self::Value) -> f64;
}

/// Bernoulli distribution (coin flip).
#[derive(Clone, Debug)]
pub struct Bernoulli {
    /// Probability of true.
    pub p: f64,
}

impl Bernoulli {
    /// Create a Bernoulli distribution.
    pub fn new(p: f64) -> Self {
        debug_assert!((0.0..=1.0).contains(&p), "p must be in [0, 1]");
        Bernoulli {
            p: p.clamp(0.0, 1.0),
        }
    }

    /// Fair coin (p = 0.5).
    pub fn fair() -> Self {
        Bernoulli { p: 0.5 }
    }
}

impl Distribution for Bernoulli {
    type Value = bool;

    fn sample(&self, rng: &mut SimpleRng) -> bool {
        rng.next_bool(self.p)
    }

    fn log_prob(&self, value: &bool) -> f64 {
        if *value {
            self.p.ln()
        } else {
            (1.0 - self.p).ln()
        }
    }
}

/// Uniform distribution over [low, high).
#[derive(Clone, Debug)]
pub struct Uniform {
    /// Lower bound (inclusive).
    pub low: f64,
    /// Upper bound (exclusive).
    pub high: f64,
}

impl Uniform {
    /// Create a uniform distribution.
    pub fn new(low: f64, high: f64) -> Self {
        debug_assert!(low < high, "low must be less than high");
        Uniform { low, high }
    }

    /// Standard uniform [0, 1).
    pub fn standard() -> Self {
        Uniform {
            low: 0.0,
            high: 1.0,
        }
    }
}

impl Distribution for Uniform {
    type Value = f64;

    fn sample(&self, rng: &mut SimpleRng) -> f64 {
        rng.next_f64_range(self.low, self.high)
    }

    fn log_prob(&self, value: &f64) -> f64 {
        if *value >= self.low && *value < self.high {
            -((self.high - self.low).ln())
        } else {
            f64::NEG_INFINITY
        }
    }
}

/// Normal (Gaussian) distribution.
#[derive(Clone, Debug)]
pub struct Normal {
    /// Mean.
    pub mu: f64,
    /// Standard deviation.
    pub sigma: f64,
}

impl Normal {
    /// Create a normal distribution.
    pub fn new(mu: f64, sigma: f64) -> Self {
        debug_assert!(sigma > 0.0, "sigma must be positive");
        Normal {
            mu,
            sigma: sigma.abs(),
        }
    }

    /// Standard normal (mu=0, sigma=1).
    pub fn standard() -> Self {
        Normal {
            mu: 0.0,
            sigma: 1.0,
        }
    }
}

impl Distribution for Normal {
    type Value = f64;

    fn sample(&self, rng: &mut SimpleRng) -> f64 {
        // Box-Muller transform
        let u1 = rng.next_f64().max(1e-10); // Avoid log(0)
        let u2 = rng.next_f64();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * core::f64::consts::PI * u2).cos();
        self.mu + self.sigma * z
    }

    fn log_prob(&self, value: &f64) -> f64 {
        let z = (value - self.mu) / self.sigma;
        -0.5 * z * z - self.sigma.ln() - 0.5 * (2.0 * core::f64::consts::PI).ln()
    }
}

/// Categorical distribution over discrete values.
#[derive(Clone, Debug)]
pub struct Categorical {
    /// Probabilities for each outcome (must sum to 1).
    pub probs: Vec<f64>,
}

impl Categorical {
    /// Create a categorical distribution from probabilities.
    ///
    /// Probabilities are normalized to sum to 1.
    ///
    /// # Panics
    ///
    /// Panics if `probs` is empty, contains a negative or non-finite value,
    /// or sums to zero.
    pub fn new(probs: &[f64]) -> Self {
        assert!(
            !probs.is_empty(),
            "Categorical::new: probs must be non-empty"
        );
        let sum: f64 = probs.iter().sum();
        assert!(
            probs.iter().all(|p| p.is_finite() && *p >= 0.0) && sum > 0.0,
            "Categorical::new: probs must be finite, non-negative, with positive sum (got {probs:?})"
        );
        let normalized: Vec<f64> = probs.iter().map(|p| p / sum).collect();
        Categorical { probs: normalized }
    }

    /// Uniform categorical over n outcomes.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0.
    pub fn uniform(n: usize) -> Self {
        assert!(n > 0, "Categorical::uniform: n must be > 0");
        let p = 1.0 / n as f64;
        Categorical {
            probs: (0..n).map(|_| p).collect(),
        }
    }
}

impl Distribution for Categorical {
    type Value = usize;

    fn sample(&self, rng: &mut SimpleRng) -> usize {
        let u = rng.next_f64();
        let mut cumsum = 0.0;
        let mut idx = self.probs.len() - 1;
        for (i, &p) in self.probs.iter().enumerate() {
            cumsum += p;
            if u < cumsum {
                idx = i;
                break;
            }
        }
        // Float-edge guard: cumsum rounding could in principle overshoot;
        // len - 1 cannot underflow since `new`/`uniform` validate non-empty.
        idx.min(self.probs.len() - 1)
    }

    fn log_prob(&self, value: &usize) -> f64 {
        if *value < self.probs.len() {
            self.probs[*value].ln()
        } else {
            f64::NEG_INFINITY
        }
    }
}

// Deliberately no Beta sampler or rejection sampling here — use ordofp_bayes
// for real inference.

/// Exponential distribution.
#[derive(Clone, Debug)]
pub struct Exponential {
    /// Rate parameter (lambda).
    pub rate: f64,
}

impl Exponential {
    /// Create an exponential distribution.
    pub fn new(rate: f64) -> Self {
        debug_assert!(rate > 0.0, "rate must be positive");
        Exponential { rate }
    }
}

impl Distribution for Exponential {
    type Value = f64;

    fn sample(&self, rng: &mut SimpleRng) -> f64 {
        // Inverse CDF: -ln(U) / lambda
        let u = rng.next_f64().max(1e-10);
        -u.ln() / self.rate
    }

    fn log_prob(&self, value: &f64) -> f64 {
        if *value < 0.0 {
            f64::NEG_INFINITY
        } else {
            self.rate.ln() - self.rate * value
        }
    }
}

// =============================================================================
// Probabilistic Computation
// =============================================================================

/// A probabilistic computation that can sample and condition.
pub struct ProbComputation<A> {
    /// The computation function.
    run_fn: Box<dyn FnOnce(&mut ProbContext) -> A>,
}

impl<A: 'static> ProbComputation<A> {
    /// Create a new probabilistic computation.
    pub fn new<F: FnOnce(&mut ProbContext) -> A + 'static>(f: F) -> Self {
        ProbComputation {
            run_fn: Box::new(f),
        }
    }

    /// Run the computation with a context.
    pub fn run(self, ctx: &mut ProbContext) -> A {
        (self.run_fn)(ctx)
    }

    /// Pure value (no sampling or conditioning).
    pub fn pure(value: A) -> Self
    where
        A: Clone,
    {
        ProbComputation::new(move |_| value)
    }

    /// Map over the result.
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> ProbComputation<B> {
        ProbComputation::new(move |ctx| {
            let a = (self.run_fn)(ctx);
            f(a)
        })
    }

    /// Chain probabilistic computations.
    pub fn and_then<B: 'static, F: FnOnce(A) -> ProbComputation<B> + 'static>(
        self,
        f: F,
    ) -> ProbComputation<B> {
        ProbComputation::new(move |ctx| {
            let a = (self.run_fn)(ctx);
            f(a).run(ctx)
        })
    }
}

// =============================================================================
// Probabilistic Context
// =============================================================================

/// Context for probabilistic computation.
///
/// Tracks the log probability of the current execution trace.
pub struct ProbContext {
    /// Random number generator.
    pub rng: SimpleRng,
    /// Log probability of current trace.
    pub log_weight: f64,
    /// Whether this trace should be rejected.
    pub rejected: bool,
}

impl ProbContext {
    /// Create a new context with a seed.
    pub fn new(seed: u64) -> Self {
        ProbContext {
            rng: SimpleRng::new(seed),
            log_weight: 0.0,
            rejected: false,
        }
    }

    /// Sample from a distribution.
    pub fn sample<D: Distribution>(&mut self, dist: &D) -> D::Value {
        dist.sample(&mut self.rng)
    }

    /// Observe (condition on) a value from a distribution.
    ///
    /// Adds the log probability to the trace weight.
    pub fn observe<D: Distribution>(&mut self, dist: &D, value: &D::Value) {
        let lp = dist.log_prob(value);
        if lp.is_finite() {
            self.log_weight += lp;
        } else {
            self.rejected = true;
        }
    }

    /// Add a score (log probability adjustment) to the trace.
    pub fn score(&mut self, log_prob: f64) {
        if log_prob.is_finite() {
            self.log_weight += log_prob;
        } else {
            self.rejected = true;
        }
    }

    /// Check if current trace is valid (not rejected).
    pub fn is_valid(&self) -> bool {
        !self.rejected && self.log_weight.is_finite()
    }

    /// Get the log-weight of the current trace.
    ///
    /// Returns `f64::NEG_INFINITY` if the trace was rejected (equivalent to
    /// a zero weight, but without the over/underflow cliff of `exp()`).
    pub fn log_weight(&self) -> f64 {
        if self.is_valid() {
            self.log_weight
        } else {
            f64::NEG_INFINITY
        }
    }
}

impl Default for ProbContext {
    fn default() -> Self {
        ProbContext::new(42)
    }
}

// =============================================================================
// Inference Methods
// =============================================================================

/// Result of inference: samples with their log-weights.
#[derive(Clone, Debug)]
pub struct InferenceResult<A> {
    /// Accepted samples, parallel to `log_weights`.
    pub samples: Vec<A>,
    /// Unnormalized log-weight (`ctx.log_weight()`) for each sample in
    /// `samples`, at the same index. Never `exp()`'d at collection time —
    /// see [`Self::mean`] for the log-sum-exp normalization this enables.
    pub log_weights: Vec<f64>,
    /// Number of accepted samples.
    pub accepted: usize,
    /// Total number of attempts.
    pub total: usize,
}

impl<A: Clone> InferenceResult<A> {
    /// Get the sample with highest log-weight.
    ///
    /// Log-weights are compared with [`f64::total_cmp`]: for the normal case
    /// of non-NaN weights this is ordinary numeric ordering; a NaN weight
    /// (which would previously have panicked) is ordered per IEEE 754
    /// totalOrder, where positive NaN sorts above every other value.
    pub fn mode(&self) -> Option<A> {
        self.samples
            .iter()
            .zip(self.log_weights.iter())
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(a, _)| a.clone())
    }

    /// Compute the expected value (for numeric types).
    ///
    /// Normalizes in log space via the log-sum-exp trick, so posteriors do
    /// not silently collapse to `NaN` (weight overflow, log-weight > ~709.8)
    /// or `0.0` (weight underflow, log-weight < ~-745) the way materializing
    /// `exp(log_weight)` directly would.
    pub fn mean(&self) -> f64
    where
        A: Into<f64> + Clone,
    {
        let max_lw = self
            .log_weights
            .iter()
            .copied()
            .filter(|w| !w.is_nan())
            .fold(f64::NEG_INFINITY, f64::max);
        if !max_lw.is_finite() {
            // All weights are -inf (impossible model) or +inf (certainty):
            // fall back to an unweighted mean rather than NaN.
            let n = self.samples.len();
            if n == 0 {
                return 0.0;
            }
            return self.samples.iter().cloned().map(Into::into).sum::<f64>() / n as f64;
        }
        let mut num = 0.0;
        let mut den = 0.0;
        for (x, lw) in self.samples.iter().cloned().zip(self.log_weights.iter()) {
            // Defensive: `importance_sample` never produces a NaN log-weight
            // (`ProbContext::is_valid` filters non-finite weights before
            // collection), but a directly-constructed `InferenceResult` can
            // carry one; skip it so it can't poison `num`/`den` with NaN
            // (mode() already tolerates NaN via total_cmp).
            if lw.is_nan() {
                continue;
            }
            let w = (lw - max_lw).exp(); // in (0, 1], no overflow/underflow cliff
            num += w * x.into();
            den += w;
        }
        num / den
    }

    /// Get acceptance rate.
    pub fn acceptance_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.accepted as f64 / self.total as f64
        }
    }
}

// Deliberately no Beta sampler or rejection sampling here — use ordofp_bayes
// for real inference.

/// Importance sampling inference.
///
/// Runs the model multiple times, weighting samples by their likelihood.
pub fn importance_sample<A: Clone + 'static, F>(
    model: F,
    num_samples: usize,
    seed: u64,
) -> InferenceResult<A>
where
    F: Fn() -> ProbComputation<A>,
{
    // Pre-allocate with worst-case capacity (all samples accepted).
    let mut samples = Vec::with_capacity(num_samples);
    let mut log_weights = Vec::with_capacity(num_samples);
    let mut rng = SimpleRng::new(seed);
    let mut accepted = 0;

    for _ in 0..num_samples {
        let sample_seed = rng.next_u64();
        let mut ctx = ProbContext::new(sample_seed);
        let result = model().run(&mut ctx);

        if ctx.is_valid() {
            samples.push(result);
            log_weights.push(ctx.log_weight());
            accepted += 1;
        }
    }

    // Normalization is deferred to InferenceResult::mean()/mode(), which
    // operate in log space (see H7: exp()-ing here over/underflows).
    InferenceResult {
        samples,
        log_weights,
        accepted,
        total: num_samples,
    }
}

/// Likelihood weighting inference.
///
/// Like importance sampling but specifically for generative models
/// where we sample from the prior and weight by the likelihood.
pub fn likelihood_weighting<A: Clone + 'static, F>(
    model: F,
    num_samples: usize,
    seed: u64,
) -> InferenceResult<A>
where
    F: Fn() -> ProbComputation<A>,
{
    // Likelihood weighting is the same as importance sampling
    // when the proposal is the prior (which is what we're doing)
    importance_sample(model, num_samples, seed)
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Sample from a distribution (creates a computation).
pub fn sample<D: Distribution + 'static>(dist: D) -> ProbComputation<D::Value>
where
    D::Value: 'static,
{
    ProbComputation::new(move |ctx| ctx.sample(&dist))
}

/// Observe a value from a distribution (creates a computation).
pub fn observe<D: Distribution + 'static>(dist: D, value: D::Value) -> ProbComputation<()>
where
    D::Value: 'static,
{
    ProbComputation::new(move |ctx| ctx.observe(&dist, &value))
}

/// Add a score to the trace (creates a computation).
pub fn score(log_prob: f64) -> ProbComputation<()> {
    ProbComputation::new(move |ctx| ctx.score(log_prob))
}

/// Pure value in probabilistic context.
pub fn pure<A: Clone + 'static>(value: A) -> ProbComputation<A> {
    ProbComputation::pure(value)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// M4 regression: glibc LCG constants used mod 2^64 made the first draw
    /// from seed 42 equal 2.57e-9 every time, and next_f64 could return 1.0.
    #[test]
    fn rng_first_draw_not_degenerate() {
        let mut rng = SimpleRng::new(42);
        let first = rng.next_f64();
        assert!(
            first > 1e-6 && first < 1.0 - 1e-6,
            "degenerate first draw: {first}"
        );
    }

    #[test]
    fn rng_f64_strictly_below_one() {
        // SplitMix64 with 53-bit mantissa construction cannot reach 1.0;
        // spot-check a large sample.
        let mut rng = SimpleRng::new(0xDEAD_BEEF);
        for _ in 0..1_000_000 {
            let x = rng.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    /// M2 regression: empty/invalid probs were silent wrongness in release.
    #[test]
    #[should_panic(expected = "Categorical")]
    fn categorical_empty_probs_panics() {
        let _ = Categorical::new(&[]);
    }

    #[test]
    #[should_panic(expected = "Categorical")]
    fn categorical_zero_sum_panics() {
        let _ = Categorical::new(&[0.0, 0.0]);
    }

    #[test]
    #[should_panic(expected = "Categorical")]
    fn categorical_negative_prob_panics() {
        let _ = Categorical::new(&[0.5, -0.5, 1.0]);
    }

    #[test]
    fn test_simple_rng() {
        let mut rng = SimpleRng::new(42);
        let x1 = rng.next_f64();
        let x2 = rng.next_f64();
        assert!((0.0..1.0).contains(&x1));
        assert!((0.0..1.0).contains(&x2));
        assert_ne!(x1, x2);
    }

    #[test]
    fn test_rng_reproducibility() {
        let mut rng1 = SimpleRng::new(123);
        let mut rng2 = SimpleRng::new(123);

        for _ in 0..10 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_bernoulli_sample() {
        let mut rng = SimpleRng::new(42);
        let dist = Bernoulli::new(0.7);

        let mut count = 0;
        let n = 1000;
        for _ in 0..n {
            if dist.sample(&mut rng) {
                count += 1;
            }
        }

        // Should be approximately 70%
        let ratio = f64::from(count) / f64::from(n);
        assert!(ratio > 0.6 && ratio < 0.8, "ratio was {ratio}");
    }

    #[test]
    fn test_bernoulli_log_prob() {
        let dist = Bernoulli::new(0.3);

        let lp_true = dist.log_prob(&true);
        let lp_false = dist.log_prob(&false);

        assert!((lp_true.exp() - 0.3).abs() < 0.001);
        assert!((lp_false.exp() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_uniform_sample() {
        let mut rng = SimpleRng::new(42);
        let dist = Uniform::new(10.0, 20.0);

        for _ in 0..100 {
            let x = dist.sample(&mut rng);
            assert!((10.0..20.0).contains(&x));
        }
    }

    #[test]
    fn test_uniform_log_prob() {
        let dist = Uniform::new(0.0, 10.0);

        let lp_in = dist.log_prob(&5.0);
        let lp_out = dist.log_prob(&15.0);

        assert!((lp_in.exp() - 0.1).abs() < 0.001);
        assert!(lp_out.is_infinite() && lp_out.is_sign_negative());
    }

    #[test]
    fn test_normal_sample() {
        let mut rng = SimpleRng::new(42);
        let dist = Normal::new(100.0, 10.0);

        let mut sum = 0.0;
        let n = 1000;
        for _ in 0..n {
            sum += dist.sample(&mut rng);
        }
        let mean = sum / f64::from(n);

        // Mean should be approximately 100
        assert!((mean - 100.0).abs() < 5.0, "mean was {mean}");
    }

    #[test]
    fn test_categorical_sample() {
        let mut rng = SimpleRng::new(42);
        let dist = Categorical::new(&[0.2, 0.3, 0.5]);

        let mut counts = [0, 0, 0];
        let n = 1000;
        for _ in 0..n {
            counts[dist.sample(&mut rng)] += 1;
        }

        // Check approximate proportions
        let p0 = f64::from(counts[0]) / f64::from(n);
        let p1 = f64::from(counts[1]) / f64::from(n);
        let p2 = f64::from(counts[2]) / f64::from(n);

        assert!(p0 > 0.1 && p0 < 0.3, "p0 was {p0}");
        assert!(p1 > 0.2 && p1 < 0.4, "p1 was {p1}");
        assert!(p2 > 0.4 && p2 < 0.6, "p2 was {p2}");
    }

    #[test]
    fn test_exponential_sample() {
        let mut rng = SimpleRng::new(42);
        let dist = Exponential::new(2.0); // mean = 1/2 = 0.5

        let mut sum = 0.0;
        let n = 1000;
        for _ in 0..n {
            sum += dist.sample(&mut rng);
        }
        let mean = sum / f64::from(n);

        // Mean should be approximately 0.5
        assert!((mean - 0.5).abs() < 0.1, "mean was {mean}");
    }

    #[test]
    fn test_prob_computation_pure() {
        let comp = ProbComputation::pure(42);
        let mut ctx = ProbContext::new(1);
        let result = comp.run(&mut ctx);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_prob_computation_sample() {
        let comp = sample(Bernoulli::fair());
        let mut ctx = ProbContext::new(42);
        let _result: bool = comp.run(&mut ctx);
        // Just verify it runs
    }

    #[test]
    fn test_prob_computation_map() {
        let comp = sample(Uniform::new(0.0, 10.0)).map(|x| x * 2.0);
        let mut ctx = ProbContext::new(42);
        let result = comp.run(&mut ctx);
        assert!((0.0..20.0).contains(&result));
    }

    #[test]
    fn test_prob_computation_and_then() {
        let comp = sample(Bernoulli::fair()).and_then(|b| {
            if b {
                sample(Uniform::new(0.0, 1.0))
            } else {
                sample(Uniform::new(1.0, 2.0))
            }
        });
        let mut ctx = ProbContext::new(42);
        let _result: f64 = comp.run(&mut ctx);
    }

    #[test]
    fn test_observe_affects_weight() {
        let mut ctx = ProbContext::new(42);

        // Observe likely value
        ctx.observe(&Bernoulli::new(0.9), &true);
        let high_weight = ctx.log_weight;

        let mut ctx2 = ProbContext::new(42);
        // Observe unlikely value
        ctx2.observe(&Bernoulli::new(0.1), &true);
        let low_weight = ctx2.log_weight;

        assert!(high_weight > low_weight);
    }

    #[test]
    fn test_importance_sampling() {
        // Simple model: sample from prior, observe data
        let model = || {
            ProbComputation::new(|ctx| {
                let p = ctx.sample(&Uniform::new(0.0, 1.0));
                // Observe 3 heads out of 4 flips
                ctx.observe(&Bernoulli::new(p), &true);
                ctx.observe(&Bernoulli::new(p), &true);
                ctx.observe(&Bernoulli::new(p), &true);
                ctx.observe(&Bernoulli::new(p), &false);
                p
            })
        };

        let result = importance_sample(model, 1000, 42);

        // Posterior mean should be around 0.75 (Beta(4,2) has mean 4/6)
        // But our simple importance sampling won't be perfect
        let mean = result.mean();
        assert!(mean > 0.5 && mean < 0.9, "mean was {mean}");
    }

    #[test]
    fn test_inference_result_mode() {
        // log-weights are ln() of the old linear weights (0.1, 0.5, 0.4);
        // ln is monotonic, so the highest-weight sample is unchanged.
        let result = InferenceResult {
            samples: vec![1, 2, 3],
            log_weights: vec![0.1_f64.ln(), 0.5_f64.ln(), 0.4_f64.ln()],
            accepted: 3,
            total: 3,
        };
        assert_eq!(result.mode(), Some(2));
    }

    #[test]
    fn test_inference_result_mean() {
        let result: InferenceResult<f64> = InferenceResult {
            samples: vec![1.0, 3.0],
            log_weights: vec![0.0, 0.0], // equal log-weights -> unweighted mean
            accepted: 2,
            total: 2,
        };
        let mean = result.mean();
        assert!((mean - 2.0).abs() < 0.001);
    }

    /// H7 regression: mode() must select deterministically (and without
    /// panicking) when a non-finite log-weight is present. `+inf` can only
    /// reach `InferenceResult` via direct construction (importance_sample's
    /// `ProbContext::is_valid` rejects non-finite log-weights before they are
    /// collected), but `mode()` must still handle it correctly rather than
    /// relying on that invariant.
    #[test]
    fn mode_ignores_infinite_log_weights() {
        let result = InferenceResult {
            samples: vec![1, 2, 3],
            log_weights: vec![-1.0, f64::INFINITY, -0.5],
            accepted: 3,
            total: 3,
        };
        // +inf must deterministically outrank every finite log-weight.
        assert_eq!(result.mode(), Some(2));
    }

    /// Companion to `mode_ignores_infinite_log_weights`: a directly-injected
    /// NaN log-weight must not panic `max_by`. `f64::total_cmp` orders by
    /// IEEE 754 totalOrder, under which a canonical (sign-bit-clear) NaN
    /// sorts *above* +infinity -- so this documents "no panic, deterministic
    /// pick" rather than "NaN is excluded"; real code paths can never
    /// produce a NaN log-weight here since `ProbContext::is_valid` filters
    /// non-finite weights (NaN and +-inf alike) before they reach an
    /// `InferenceResult`.
    #[test]
    fn mode_with_nan_log_weight_does_not_panic() {
        let result = InferenceResult {
            samples: vec![1, 2, 3],
            log_weights: vec![-1.0, f64::NAN, -0.5],
            accepted: 3,
            total: 3,
        };
        assert_eq!(result.mode(), Some(2));
    }

    #[test]
    fn test_score() {
        let comp = score(-1.0).and_then(|()| pure(42));
        let mut ctx = ProbContext::new(1);
        let result = comp.run(&mut ctx);

        assert_eq!(result, 42);
        assert!((ctx.log_weight - (-1.0)).abs() < 0.001);
    }

    /// H7 regression: many observations drive log-weights below exp()'s
    /// underflow point (f64::MIN_POSITIVE's subnormal floor is ~exp(-744.44));
    /// the posterior mean must survive.
    #[test]
    fn importance_sampling_survives_many_observations() {
        let model = || {
            ProbComputation::new(|ctx| {
                let theta = ctx.sample(&Uniform::new(0.0, 1.0));
                let coin = Bernoulli::new(0.7);
                for _ in 0..2200 {
                    ctx.observe(&coin, &true); // log_weight ~ 2200*ln(0.7) ~ -785: exp() underflows to 0.0
                }
                theta
            })
        };
        let result = importance_sample(model, 1000, 42);
        let mean = result.mean();
        assert!(mean.is_finite(), "mean must not be NaN (overflow path)");
        assert!(mean > 0.0, "mean 0.0 means every weight underflowed");
    }

    /// Test a simple Bayesian model: estimating coin bias.
    #[test]
    fn test_bayesian_coin_bias() {
        // True bias is 0.7
        let observed_flips = [true, true, true, false, true, true, false, true, true, true];

        let model = || {
            let flips = observed_flips;
            ProbComputation::new(move |ctx| {
                // Prior: uniform
                let p = ctx.sample(&Uniform::new(0.0, 1.0));

                // Likelihood: observe flips
                for flip in &flips {
                    ctx.observe(&Bernoulli::new(p), flip);
                }

                p
            })
        };

        let result = importance_sample(model, 2000, 123);
        let estimated_bias = result.mean();

        // 8 heads out of 10 -> posterior should be around 0.7-0.8
        assert!(
            estimated_bias > 0.6 && estimated_bias < 0.9,
            "estimated bias was {estimated_bias}"
        );
    }
}
