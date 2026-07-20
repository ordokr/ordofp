//! Simple Bayesian inference example.
//!
//! This example demonstrates basic usage of `ordofp_bayes` for probabilistic programming.
// Sample counts ≪ 2^52 — the usize→f64 mean casts are exact.
#![allow(clippy::cast_precision_loss)]
#![cfg(feature = "std")]

use ordofp_bayes::distributions::{Normal, Uniform};
use ordofp_bayes::{
    Distribution, ImportanceSampling, MetropolisHastings, Particle, SequentialMonteCarlo,
    WeightedModel, effective_sample_size, normalized_weights,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1000);

fn next_seed() -> u64 {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn main() {
    println!("OrdoFP Bayes - Simple Inference Example\n");

    // Example: Estimate mean of a normal distribution
    // We generate samples from different inference algorithms

    let mut rng = StdRng::seed_from_u64(42);

    // Sequential Monte Carlo - samples from a prior
    println!("Sequential Monte Carlo (SMC):");
    let smc = SequentialMonteCarlo::new(100);
    let smc_samples: Vec<f64> = smc.infer(
        || {
            // Sample from prior using a seeded RNG (reproducible)
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let prior = Normal::new(1.8, 0.5); // Prior centered near observations
            prior.sample(&mut local_rng)
        },
        &mut rng,
    );

    println!("  Generated {} samples", smc_samples.len());
    if !smc_samples.is_empty() {
        let mean: f64 = smc_samples.iter().sum::<f64>() / smc_samples.len() as f64;
        println!("  Sample mean: {mean:.3}");
    }

    // Metropolis-Hastings - MCMC sampling
    println!("\nMetropolis-Hastings (MCMC):");
    let mh = MetropolisHastings::new(100, 10);
    let mh_samples: Vec<f64> = mh.infer(
        || {
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let prior = Normal::new(0.0, 1.0);
            prior.sample(&mut local_rng)
        },
        &mut rng,
    );

    println!(
        "  Generated {} samples (after {} burn-in)",
        mh_samples.len(),
        10
    );
    if !mh_samples.is_empty() {
        let mean: f64 = mh_samples.iter().sum::<f64>() / mh_samples.len() as f64;
        println!("  Sample mean: {mean:.3}");
    }

    // Importance Sampling - proposal-based sampling
    println!("\nImportance Sampling:");
    let is = ImportanceSampling::new(100);
    let is_samples: Vec<f64> = is.infer(
        || {
            let mut local_rng = StdRng::seed_from_u64(next_seed());
            let proposal = Uniform::new(-3.0, 3.0);
            proposal.sample(&mut local_rng)
        },
        &mut rng,
    );

    println!("  Generated {} samples", is_samples.len());
    if !is_samples.is_empty() {
        let mean: f64 = is_samples.iter().sum::<f64>() / is_samples.len() as f64;
        println!("  Sample mean: {mean:.3}");
    }

    // Demonstrate weighted importance sampling (likelihood weighting):
    // prior N(0,1) as the proposal, one observation at 0.5.
    println!("\nImportance Sampling (Weighted):");

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

    let is_weighted = ImportanceSampling::new(500);
    let weighted_samples = is_weighted.infer_weighted(&ObservedModel, &mut rng);

    println!("  Generated {} weighted samples", weighted_samples.len());
    if !weighted_samples.is_empty() {
        let weights = normalized_weights(&weighted_samples);
        let mean: f64 = weighted_samples
            .iter()
            .zip(&weights)
            .map(|(w, p)| w.value * p)
            .sum();
        let ess = effective_sample_size(&weights);
        // Posterior mean for equal prior/likelihood variances is 0.25.
        println!("  Self-normalized posterior mean: {mean:.3} (expected ≈ 0.250)");
        println!(
            "  Effective sample size: {ess:.1} of {}",
            weighted_samples.len()
        );
    }
}
