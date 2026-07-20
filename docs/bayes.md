# ordofp_bayes - Probabilistic Programming Guide

This guide explains how to use `ordofp_bayes` for Bayesian inference in OrdoFP.

## Overview

`ordofp_bayes` provides three main inference algorithms:
- **Sequential Monte Carlo (SMC)**: Particle-based inference with resampling
- **Metropolis-Hastings (MCMC)**: Markov Chain Monte Carlo sampling
- **Importance Sampling**: Weighted sampling from proposal distributions

## Basic Usage

### Sequential Monte Carlo

SMC generates particles from your model and resamples them based on weights:

```rust
use ordofp_bayes::{Distribution, Normal, SequentialMonteCarlo};

let smc = SequentialMonteCarlo::new(1000); // 1000 particles
let mut rng = rand::rng(); // rand 0.10 thread-local RNG (the old `thread_rng()` was removed)

let samples: Vec<f64> = smc.infer(|| {
    // Your probabilistic model — the closure takes no arguments,
    // so draw from its own RNG handle rather than capturing `rng`
    let mut model_rng = rand::rng();
    let prior = Normal::new(0.0, 1.0);
    prior.sample(&mut model_rng)
}, &mut rng);
```

### Metropolis-Hastings

MCMC generates a Markov chain of samples:

```rust
use ordofp_bayes::{Distribution, MetropolisHastings, Normal};

let mh = MetropolisHastings::new(1000, 100) // 1000 iterations, 100 burn-in
    .with_step_size(0.1); // Optional: set proposal step size
let mut rng = rand::rng();

let samples: Vec<f64> = mh.infer(|| {
    let mut model_rng = rand::rng();
    let prior = Normal::new(0.0, 1.0);
    prior.sample(&mut model_rng)
}, &mut rng);
```

### Importance Sampling

`infer_weighted` implements likelihood weighting: the model samples latents
from the prior (the proposal) and accumulates observation log-likelihoods via
`Particle::factor`. Normalize the returned log-weights with
`normalized_weights` (log-sum-exp, numerically stable) before computing
expectations, and check `effective_sample_size` for weight degeneracy:

```rust
use ordofp_bayes::{
    Distribution, ImportanceSampling, Normal, Particle, WeightedModel,
    WeightedSample, effective_sample_size, normalized_weights,
};

struct ObservedModel;

impl WeightedModel<f64> for ObservedModel {
    fn execute_weighted<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Particle<f64> {
        let prior = Normal::new(0.0, 1.0);
        let value = prior.sample(rng);
        let mut p = Particle::new(value);
        let diff = value - 0.5; // observation at 0.5
        p.factor(-0.5 * diff * diff);
        p
    }
}

let is = ImportanceSampling::new(1000);
let mut rng = rand::rng();
let weighted: Vec<WeightedSample<f64>> = is.infer_weighted(&ObservedModel, &mut rng);

let weights = normalized_weights(&weighted);
let mean: f64 = weighted.iter().zip(&weights).map(|(w, p)| w.value * p).sum();
let ess = effective_sample_size(&weights);
assert!(ess > 0.0);
```

The closure-based `infer(|| …)` draws unweighted i.i.d. samples from the
model — with no observations the prior *is* the target, so no reweighting
applies.

## Distributions

### Normal Distribution

```rust
use ordofp_bayes::Normal;

let normal = Normal::new(mean, std_dev);
let sample = normal.sample(&mut rng);
```

### Uniform Distribution

```rust
use ordofp_bayes::Uniform;

let uniform = Uniform::new(min, max);
let sample = uniform.sample(&mut rng);
```

## Advanced Usage

### Seeded RNG for Reproducibility

For reproducible results, use a seeded RNG:

```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

let mut rng = StdRng::seed_from_u64(42);
let samples = smc.infer(|| {
    // model
}, &mut rng);
```

### Custom Models

You can build complex probabilistic models:

```rust
let samples = smc.infer(|| {
    let mut model_rng = rand::rng();

    // Sample parameters
    let mu = Normal::new(0.0, 1.0).sample(&mut model_rng);
    let sigma = Uniform::new(0.1, 2.0).sample(&mut model_rng);

    // Sample data
    let data: Vec<f64> = (0..10)
        .map(|_| Normal::new(mu, sigma).sample(&mut model_rng))
        .collect();

    // Return what you want to infer
    (mu, sigma)
}, &mut rng);
```

## Algorithm Details

### SMC Resampling

`SequentialMonteCarlo` resamples particles by their accumulated log-weights.
Three strategies are available via `ResamplingStrategy`: `Multinomial`
(default), `Systematic` (lower variance), and `Stratified` (unbiased, low
variance).

Note the scope: this is a *single* weight-and-resample step over independent
particles, not a full sequential SMC that propagates particles through a
series of intermediate targets.

### Metropolis-Hastings

`infer_traceable` implements single-site trace MCMC: a reflected random-walk
proposal on the unit interval (honoring `step_size`), a log-space acceptance
ratio with a trace-dimension correction, and burn-in. The closure-based
`infer(|| …)` draws i.i.d. samples from the model instead — with no
observations that is exact, but it performs no posterior exploration; use
`infer_traceable` for models with observations.

### Importance Sampling

`infer_weighted` computes real importance weights (accumulated observation
log-likelihoods with the prior as proposal). Use `normalized_weights` +
`effective_sample_size` as shown above; a low ESS means the proposal is a
poor match and the estimate should not be trusted.

## Integration with OrdoFP

Future versions will integrate with:
- OrdoFP effects system for proper weight tracking
- ParFlumen for parallel particle generation
- CPS transformers for composable inference

## Performance Tips

1. **Particle Count**: More particles = better estimates but slower
2. **Burn-in**: Longer burn-in = better MCMC convergence
3. **Step Size**: Tune MH step size for optimal acceptance rate (~0.2-0.5)
4. **Parallelization**: With the `rayon` feature, `SequentialMonteCarlo::infer_parallel` / `infer_weighted_parallel` generate particles in parallel (MCMC has `infer_parallel_chains`)

## Examples

See `ordofp_bayes/examples/simple_inference.rs` for a complete working example.

## Testing

Run tests with:
```bash
cargo test -p ordofp_bayes --features std
```

Tests use seeded RNGs for reproducibility.
