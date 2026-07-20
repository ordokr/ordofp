//! Core traits for probabilistic programming.

#![cfg(feature = "alloc")]

/// Trait for sampling from distributions.
///
/// > *"Samplandus"*
/// > — That which is to be sampled. (Neo-Latin)
///
/// This trait represents the ability to sample random values from distributions.
pub trait Samplandus<D> {
    /// Sample a value from distribution D.
    fn sample(&mut self, dist: D) -> D::Output
    where
        D: Distribution;
}

/// Trait for scoring (likelihood) computations.
///
/// > *"Inferendus"*
/// > — That which is to be inferred. (Neo-Latin)
///
/// This trait represents the ability to score computations with log-probability.
pub trait Inferendus {
    /// Score a computation with log-probability.
    fn score(&mut self, log_prob: f64);
}

/// Distribution trait for sampling.
pub trait Distribution {
    /// Output type of the distribution.
    type Output: Send + Sync + 'static;

    /// Sample a value from this distribution.
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::Output;
}
