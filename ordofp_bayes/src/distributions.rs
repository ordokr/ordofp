//! Probability distributions for sampling.

#![cfg(feature = "alloc")]
#![cfg(feature = "std")] // f64::ln/sqrt need std (libm)

use super::traits::Distribution;
use rand::Rng;
use rand::RngExt;

/// Draw one standard-normal (mean 0, variance 1) sample.
///
/// Marsaglia polar method: draw `(u, v)` uniform on the unit square, reject
/// pairs outside the open unit disc, transform the survivor. Rejection
/// probability per round is `1 - π/4 ≈ 0.215`, so the loop terminates almost
/// surely and averages ~1.27 rounds per sample.
pub(crate) fn sample_standard_normal<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    loop {
        let u = 2.0 * rng.random::<f64>() - 1.0;
        let v = 2.0 * rng.random::<f64>() - 1.0;
        let s = u * u + v * v;
        if s > 0.0 && s < 1.0 {
            return u * (-2.0 * s.ln() / s).sqrt();
        }
    }
}

/// Draw one `Exp(1)` sample by inverse CDF.
///
/// `rng.random::<f64>()` is in `[0, 1)`, so `1 - u` is in `(0, 1]` and the
/// log is always finite (0 at the endpoint).
pub(crate) fn sample_exp1<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    -(1.0 - rng.random::<f64>()).ln()
}

/// Normal (Gaussian) distribution.
#[derive(Clone, Copy, Debug)]
pub struct Normal {
    mean: f64,
    std_dev: f64,
}

impl Normal {
    /// Create a new normal distribution.
    ///
    /// # Panics
    /// Panics if `std_dev` is not finite or is negative.
    pub fn new(mean: f64, std_dev: f64) -> Self {
        assert!(
            std_dev.is_finite() && std_dev >= 0.0,
            "Normal std_dev must be finite and non-negative"
        );
        Self { mean, std_dev }
    }
}

impl Distribution for Normal {
    type Output = f64;

    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        self.mean + self.std_dev * sample_standard_normal(rng)
    }
}

/// Uniform distribution.
#[derive(Clone, Copy, Debug)]
pub struct Uniform {
    dist: rand::distr::Uniform<f64>,
}

impl Uniform {
    /// Create a new uniform distribution.
    ///
    /// # Panics
    /// Panics if `min >= max`.
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            dist: rand::distr::Uniform::new(min, max).unwrap(),
        }
    }
}

impl Distribution for Uniform {
    type Output = f64;

    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        rng.sample(self.dist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Samples from a `Normal` distribution must always be finite real numbers,
    /// never NaN or ±infinity, regardless of the seed.
    #[test]
    fn normal_samples_are_finite() {
        let dist = Normal::new(0.0, 1.0);
        let mut rng = StdRng::seed_from_u64(0xdead_beef);
        for _ in 0..256 {
            assert!(
                dist.sample(&mut rng).is_finite(),
                "Normal(0, 1) sample must be a finite f64"
            );
        }
    }

    /// `Uniform::new` with inverted bounds (`min > max`) must panic,
    /// as documented in the constructor's `# Panics` section.
    #[test]
    #[should_panic(expected = "EmptyRange")]
    fn uniform_panics_on_inverted_bounds() {
        let _ = Uniform::new(1.0, 0.0);
    }

    /// `Normal` with zero standard deviation is a degenerate point-mass: every
    /// sample must equal the mean exactly.  This edge case is permitted by the
    /// constructor (`std_dev = 0.0` is finite and non-negative) but distinct from
    /// the ordinary spread-distribution case tested by `normal_samples_are_finite`.
    #[test]
    #[allow(clippy::float_cmp)] // point-mass sample must equal the mean bit-for-bit
    fn normal_zero_std_dev_always_returns_mean() {
        let mean = std::f64::consts::PI;
        let dist = Normal::new(mean, 0.0);
        let mut rng = StdRng::seed_from_u64(0x1234_5678_9abc_def0);
        for _ in 0..64 {
            let sample = dist.sample(&mut rng);
            assert_eq!(
                sample, mean,
                "Normal(mean={mean}, std_dev=0) must always sample exactly the mean"
            );
        }
    }

    /// The Marsaglia-polar sampler must reproduce the first two moments of
    /// `Normal(2, 3)` within estimator noise (seeded, so deterministic).
    #[test]
    fn normal_moments_match() {
        let dist = Normal::new(2.0, 3.0);
        let mut rng = StdRng::seed_from_u64(42);
        let n = 100_000_i32;
        let n_f = f64::from(n);
        let samples: Vec<f64> = (0..n).map(|_| dist.sample(&mut rng)).collect();
        let mean = samples.iter().sum::<f64>() / n_f;
        let var = samples.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n_f;
        assert!((mean - 2.0).abs() < 0.05, "mean {mean} too far from 2.0");
        assert!((var - 9.0).abs() < 0.3, "variance {var} too far from 9.0");
    }

    /// The inverse-CDF `Exp(1)` sampler must reproduce mean 1 and variance 1
    /// within estimator noise, and never produce a negative or non-finite value.
    #[test]
    fn exp1_moments_match() {
        let mut rng = StdRng::seed_from_u64(43);
        let n = 100_000_i32;
        let n_f = f64::from(n);
        let samples: Vec<f64> = (0..n).map(|_| sample_exp1(&mut rng)).collect();
        for &s in &samples {
            assert!(
                s.is_finite() && s >= 0.0,
                "Exp(1) sample {s} out of support"
            );
        }
        let mean = samples.iter().sum::<f64>() / n_f;
        let var = samples.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n_f;
        assert!((mean - 1.0).abs() < 0.05, "mean {mean} too far from 1.0");
        assert!((var - 1.0).abs() < 0.1, "variance {var} too far from 1.0");
    }

    /// Every sample from `Uniform(min, max)` must lie in the half-open interval
    /// `[min, max)`.  This edge case is not covered by the panic test above.
    #[test]
    fn uniform_samples_are_in_range() {
        let min = 2.5_f64;
        let max = 7.5_f64;
        let dist = Uniform::new(min, max);
        let mut rng = StdRng::seed_from_u64(0xcafe_babe);
        for _ in 0..256 {
            let sample = dist.sample(&mut rng);
            assert!(
                sample >= min && sample < max,
                "Uniform({min}, {max}) sample {sample} is out of range"
            );
        }
    }
}
