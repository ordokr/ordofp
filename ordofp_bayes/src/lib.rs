//! `ordofp_bayes` - Probabilistic Programming for `OrdoFP`
//!
//! > *"Probabilitas est ratio incertitudinis."*
//! > — Probability is the measure of uncertainty. (Modern)
//!
//! This crate provides Bayesian inference algorithms for probabilistic programming,
//! using functional abstractions, as part of the `OrdoFP` ecosystem.
//!
//! Adapted third-party algorithms are inventoried in `ORIGINAL_SOURCE.md`
//! in this crate and the repo-root `THIRD_PARTY_NOTICES.md`.
//!
//! # Features
//!
//! - Sequential Monte Carlo (SMC)
//! - Metropolis-Hastings (MCMC)
//! - Importance Sampling
//! - Parallel particle generation via Rayon (`rayon` feature)
//!
//! # Performance
//!
//! Weight calculations are written as tight slice loops that LLVM
//! auto-vectorizes; there is no explicit SIMD (`std::simd`) code in this
//! crate.
//!
//! # Example
//!
//! ```rust
//! use ordofp_bayes::distributions::Normal;
//! use ordofp_bayes::{Distribution, MetropolisHastings};
//! use rand::SeedableRng;
//! use rand::rngs::StdRng;
//! use std::sync::atomic::{AtomicU64, Ordering};
//!
//! // Give every closure call its own seed, so the 1000 draws below are
//! // genuinely independent samples rather than 1000 copies of one number.
//! static SEED: AtomicU64 = AtomicU64::new(1);
//! fn next_seed() -> u64 {
//!     SEED.fetch_add(1, Ordering::Relaxed)
//! }
//!
//! // Define a simple model: sample once from a prior distribution.
//! let mut rng = StdRng::seed_from_u64(42);
//! // Note: `MetropolisHastings::infer` draws `iterations` independent
//! // samples via resampling rather than running an MCMC chain, so
//! // `burn_in` (the `100` below) has no effect on this code path — it
//! // only matters for `infer_traceable`'s trace-based chain.
//! let mh = MetropolisHastings::new(1000, 100);
//!
//! // Run inference
//! let samples: Vec<f64> = mh.infer(
//!     || {
//!         let mut local_rng = StdRng::seed_from_u64(next_seed());
//!         Normal::new(0.0, 1.0).sample(&mut local_rng)
//!     },
//!     &mut rng,
//! );
//! assert_eq!(samples.len(), 1000);
//! // Real sampling variation: 1000 draws from a continuous Normal are not
//! // all identical (this cannot flake — the odds of a false failure are
//! // astronomically small).
//! assert!(samples.windows(2).any(|w| w[0] != w[1]));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod distributions;
pub mod inference;
pub mod traits;

pub use inference::{
    ImportanceSampling, MetropolisHastings, Particle, ResamplingStrategy, SequentialMonteCarlo,
    Trace, TraceableModel, WeightedModel, WeightedSample, effective_sample_size,
    normalized_weights,
};
pub use traits::{Distribution, Inferendus, Samplandus};
