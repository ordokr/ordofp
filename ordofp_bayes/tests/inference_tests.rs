//! Tests for inference algorithms with seeded RNG for reproducibility.
// Sample counts ≪ 2^52 — the usize→f64 mean casts are exact.
#![allow(clippy::cast_precision_loss)]
// Exact float assertions are intentional: the operations under test must
// preserve log-densities bit-for-bit.
#![allow(clippy::float_cmp)]
#![cfg(feature = "std")]

use ordofp_bayes::distributions::{Normal, Uniform};
use ordofp_bayes::{
    Distribution, ImportanceSampling, MetropolisHastings, Particle, ResamplingStrategy,
    SequentialMonteCarlo, WeightedModel, WeightedSample,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1000);

fn next_seed() -> u64 {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[test]
fn test_smc_basic() {
    let smc = SequentialMonteCarlo::new(100);
    let mut rng = StdRng::seed_from_u64(42);

    let samples: Vec<f64> = smc.infer(
        || {
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let prior = Normal::new(0.0, 1.0);
            prior.sample(&mut local_rng)
        },
        &mut rng,
    );

    assert_eq!(samples.len(), 100);
    // Mean should be approximately 0 (prior mean)
    let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
    assert!(mean.abs() < 1.0, "Mean should be close to 0, got {mean}");
}

#[test]
fn test_mh_basic() {
    let mh = MetropolisHastings::new(100, 10);
    let mut rng = StdRng::seed_from_u64(42);

    let samples: Vec<f64> = mh.infer(
        || {
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let prior = Normal::new(0.0, 1.0);
            prior.sample(&mut local_rng)
        },
        &mut rng,
    );

    // Should have 100 samples (iterations)
    assert_eq!(samples.len(), 100);
}

#[test]
fn test_importance_sampling_basic() {
    let is = ImportanceSampling::new(100);
    let mut rng = StdRng::seed_from_u64(42);

    let samples: Vec<f64> = is.infer(
        || {
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let proposal = Uniform::new(-2.0, 2.0);
            proposal.sample(&mut local_rng)
        },
        &mut rng,
    );

    assert_eq!(samples.len(), 100);
    // Samples should be in the proposal range
    assert!(samples.iter().all(|&x| (-2.0..=2.0).contains(&x)));
}

#[test]
fn test_importance_sampling_weighted() {
    use ordofp_bayes::{Particle, effective_sample_size, normalized_weights};

    // Likelihood weighting: prior N(0,1) as proposal, observation at 0.5.
    struct ObservedModel;

    impl WeightedModel<f64> for ObservedModel {
        fn execute_weighted<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Particle<f64> {
            let prior = Normal::new(0.0, 1.0);
            let value = prior.sample(rng);
            let mut p = Particle::new(value);
            // log-likelihood of observing 0.5 under N(value, 1.0), up to a constant
            let diff = value - 0.5;
            p.factor(-0.5 * diff * diff);
            p
        }
    }

    let is = ImportanceSampling::new(500);
    let mut rng = StdRng::seed_from_u64(7);
    let weighted: Vec<WeightedSample<f64>> = is.infer_weighted(&ObservedModel, &mut rng);

    assert_eq!(weighted.len(), 500);
    assert!(weighted.iter().all(|w| w.log_weight.is_finite()));
    // Weights must actually vary — the old stub returned uniform 0.0.
    let (min, max) = weighted.iter().fold((f64::MAX, f64::MIN), |(lo, hi), w| {
        (lo.min(w.log_weight), hi.max(w.log_weight))
    });
    assert!(max > min, "importance weights must not be uniform");

    // Self-normalized weights sum to 1 and give a posterior-shifted mean:
    // prior mean 0, likelihood mean 0.5, equal variances → posterior mean 0.25.
    let norm = normalized_weights(&weighted);
    let total: f64 = norm.iter().sum();
    assert!((total - 1.0).abs() < 1e-12, "weights must sum to 1");
    let mean: f64 = weighted.iter().zip(&norm).map(|(w, p)| w.value * p).sum();
    assert!(
        (mean - 0.25).abs() < 0.15,
        "SNIS mean should approach the posterior mean 0.25, got {mean}"
    );

    // ESS is in (0, n] and, with a well-matched proposal, not degenerate.
    let ess = effective_sample_size(&norm);
    assert!(ess > 50.0 && ess <= 500.0, "ESS out of range: {ess}");
}

#[test]
fn test_smc_resampling() {
    // Test that SMC produces the expected number of samples
    let smc = SequentialMonteCarlo::new(10);
    let mut rng = StdRng::seed_from_u64(42);

    let samples: Vec<f64> = smc.infer(
        || {
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let prior = Normal::new(0.0, 1.0);
            prior.sample(&mut local_rng)
        },
        &mut rng,
    );

    assert_eq!(samples.len(), 10);
}

#[test]
fn test_distributions_normal() {
    let normal = Normal::new(5.0, 2.0);
    let mut rng = StdRng::seed_from_u64(42);

    // Sample several values
    let samples: Vec<f64> = (0..100).map(|_| normal.sample(&mut rng)).collect();

    // Mean should be approximately 5.0
    let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
    assert!(
        (mean - 5.0).abs() < 1.0,
        "Mean should be close to 5.0, got {mean}"
    );
}

#[cfg(feature = "rayon")]
#[test]
fn test_smc_parallel() {
    let smc = SequentialMonteCarlo::new(1000);
    let mut rng = StdRng::seed_from_u64(42);

    // Test that parallel execution produces valid samples
    let samples: Vec<f64> = smc.infer_parallel(
        || {
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let prior = Normal::new(0.0, 1.0);
            prior.sample(&mut local_rng)
        },
        &mut rng,
    );

    assert_eq!(samples.len(), 1000);

    // Mean should be approximately 0
    let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
    assert!(mean.abs() < 0.5, "Mean should be close to 0, got {mean}");
}

#[cfg(feature = "rayon")]
#[test]
fn test_smc_weighted_parallel() {
    use ordofp_bayes::{Particle, WeightedModel};

    struct SimpleModel;

    impl WeightedModel<f64> for SimpleModel {
        fn execute_weighted<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Particle<f64> {
            let prior = Normal::new(0.0, 1.0);
            let value = prior.sample(rng);
            // Assign weight based on value (e.g. observation at 0.5)
            // likelihood = N(0.5, 1.0) evaluated at value
            let diff = value - 0.5;
            let log_weight = -0.5 * diff * diff;

            Particle::with_weight(value, log_weight)
        }
    }

    let smc = SequentialMonteCarlo::new(1000);
    let mut rng = StdRng::seed_from_u64(42);
    let model = SimpleModel;

    let samples: Vec<f64> = smc.infer_weighted_parallel(&model, &mut rng);

    assert_eq!(samples.len(), 1000);

    // Mean should shift towards 0.5 due to weighting
    // Prior mean 0.0, Likelihood mean 0.5, Posterior mean should be approx 0.25 (since variance is same)
    let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
    assert!(
        (mean - 0.25).abs() < 0.2,
        "Mean should be close to 0.25, got {mean}"
    );
}

#[cfg(feature = "rayon")]
#[test]
fn test_mh_parallel_chains() {
    use ordofp_bayes::{Trace, TraceableModel};

    struct MhModel;

    impl TraceableModel<f64> for MhModel {
        fn execute_with_trace<R: rand::Rng + ?Sized>(
            &self,
            variables: &[f64],
            rng: &mut R,
        ) -> Trace<f64> {
            // Simple model: Normal(0, 1)
            // But we need to use 'variables' if provided, else sample.
            // TraceableModel protocol: if variables provided, use them.

            // For this test, we ignore complex trace logic and just sample.
            // A real TraceableModel would map variables to samples.
            let prior = Normal::new(0.0, 1.0);
            let val = prior.sample(rng);

            // Mock trace
            Trace::new(variables.to_vec(), val, -0.5 * val * val)
        }
    }

    let mh = MetropolisHastings::new(100, 10);
    let mut rng = StdRng::seed_from_u64(42);
    let model = MhModel;

    let chains = 4;
    let results: Vec<Vec<f64>> = mh.infer_parallel_chains(&model, chains, &mut rng);

    assert_eq!(results.len(), chains);
    for chain in results {
        assert_eq!(chain.len(), 100);
    }
}

#[test]
fn test_trace_pure_has_no_variables_and_zero_log_prob() {
    use ordofp_bayes::Trace;

    let trace = Trace::pure(42u32);
    assert_eq!(trace.output, 42);
    assert!(
        trace.variables.is_empty(),
        "Trace::pure must record no random variables"
    );
    assert_eq!(
        trace.log_prob_density, 0.0,
        "Trace::pure log-prob must be 0 (i.e. log(1))"
    );
}

#[test]
fn test_trace_map_transforms_output_preserving_variables_and_log_prob() {
    use ordofp_bayes::Trace;

    let original = Trace::new(vec![0.1, 0.9], "hello", -2.5_f64);
    let mapped = original.map(|s: &str| s.len());

    assert_eq!(mapped.output, 5);
    assert_eq!(mapped.variables, vec![0.1, 0.9]);
    assert_eq!(
        mapped.log_prob_density, -2.5,
        "Trace::map must not alter log_prob_density"
    );
}

#[test]
fn test_particle_factor_accumulates_log_weight() {
    use ordofp_bayes::Particle;

    let mut particle = Particle::new(1.0_f64);
    assert_eq!(
        particle.log_weight, 0.0,
        "unit particle must start at log-weight 0"
    );

    particle.factor(-1.5);
    assert!(
        (particle.log_weight - (-1.5)).abs() < f64::EPSILON,
        "after one factor(-1.5), log_weight should be -1.5, got {}",
        particle.log_weight
    );

    particle.factor(0.5);
    assert!(
        (particle.log_weight - (-1.0)).abs() < f64::EPSILON,
        "after factor(0.5), log_weight should be -1.0 (accumulated), got {}",
        particle.log_weight
    );
}

/// A deterministic model (empty trace) must trigger the `repeat_n` fast path
/// inside `infer_traceable`, returning exactly `iterations` identical outputs
/// without executing any MH proposals.
#[test]
fn test_mh_infer_traceable_deterministic_model_repeats_output() {
    use ordofp_bayes::{Trace, TraceableModel};

    struct ConstantModel;

    impl TraceableModel<u32> for ConstantModel {
        fn execute_with_trace<R: rand::Rng + ?Sized>(
            &self,
            _variables: &[f64],
            _rng: &mut R,
        ) -> Trace<u32> {
            // No random variables sampled → empty trace.
            // `infer_traceable` should detect this and skip all MH proposals.
            Trace::new(vec![], 42u32, 0.0)
        }
    }

    let mh = MetropolisHastings::new(5, 3);
    let mut rng = StdRng::seed_from_u64(99);

    let samples = mh.infer_traceable(&ConstantModel, &mut rng);

    assert_eq!(
        samples.len(),
        5,
        "infer_traceable must return exactly `iterations` samples"
    );
    assert!(
        samples.iter().all(|&x| x == 42),
        "deterministic model must always produce the same output value"
    );
}

#[test]
fn test_distributions_uniform() {
    let uniform = Uniform::new(0.0, 10.0);
    let mut rng = StdRng::seed_from_u64(42);

    // Sample several values
    let samples: Vec<f64> = (0..100).map(|_| uniform.sample(&mut rng)).collect();

    // All samples should be in range
    assert!(samples.iter().all(|&x| (0.0..=10.0).contains(&x)));

    // Mean should be approximately 5.0
    let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
    assert!(
        (mean - 5.0).abs() < 1.0,
        "Mean should be close to 5.0, got {mean}"
    );
}

/// Edge case: SMC with Systematic and Stratified resampling strategies must
/// return exactly `N` particles when given uniform log-weights.
///
/// The default strategy (Multinomial) is used everywhere else; this test
/// ensures `with_resampling` is actually exercised and that both alternative
/// strategies preserve the particle count on the serial `infer_weighted` path.
#[test]
fn test_smc_non_default_resampling_strategies_preserve_particle_count() {
    struct UniformWeightModel;

    impl WeightedModel<f64> for UniformWeightModel {
        fn execute_weighted<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Particle<f64> {
            let prior = Normal::new(0.0, 1.0);
            // log-weight 0 ⟺ weight 1: all particles are equally likely.
            Particle::with_weight(prior.sample(rng), 0.0)
        }
    }

    let n = 50;
    let model = UniformWeightModel;
    let mut rng = StdRng::seed_from_u64(7);

    for strategy in [
        ResamplingStrategy::Systematic,
        ResamplingStrategy::Stratified,
    ] {
        let smc = SequentialMonteCarlo::new(n).with_resampling(strategy);
        let samples: Vec<f64> = smc.infer_weighted(&model, &mut rng);
        assert_eq!(
            samples.len(),
            n,
            "{strategy:?} resampling must return exactly {n} particles"
        );
    }
}

/// `MetropolisHastings::infer_traceable` with zero iterations must return an
/// empty `Vec` immediately, without executing the model even once.
///
/// This exercises the early-return guard (`iterations == 0`) that sits before
/// the first `model.execute_with_trace` call.  A regression here (e.g. a
/// refactor that removes the guard) would cause a spurious model execution
/// and return one sample instead of zero.
#[test]
fn test_mh_infer_traceable_zero_iterations_returns_empty() {
    use ordofp_bayes::{Trace, TraceableModel};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingModel {
        call_count: AtomicUsize,
    }

    impl TraceableModel<i32> for CountingModel {
        fn execute_with_trace<R: rand::Rng + ?Sized>(
            &self,
            _variables: &[f64],
            _rng: &mut R,
        ) -> Trace<i32> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Trace::new(vec![0.5], 99, -1.0)
        }
    }

    let model = CountingModel {
        call_count: AtomicUsize::new(0),
    };
    let mh = MetropolisHastings::new(0, 0);
    let mut rng = StdRng::seed_from_u64(0);

    let samples: Vec<i32> = mh.infer_traceable(&model, &mut rng);

    assert!(
        samples.is_empty(),
        "infer_traceable with 0 iterations must return an empty Vec"
    );
    assert_eq!(
        model.call_count.load(Ordering::Relaxed),
        0,
        "infer_traceable with 0 iterations must not execute the model"
    );
}
