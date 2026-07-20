use criterion::{BenchmarkId, Criterion, criterion_group};
use ordofp_bayes::distributions::Normal;
use ordofp_bayes::{
    Distribution, ImportanceSampling, MetropolisHastings, Particle, ResamplingStrategy,
    SequentialMonteCarlo, Trace, TraceableModel, WeightedModel,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::{Rng, RngExt};
use std::borrow::Cow;

struct BenchModel;

impl WeightedModel<f64> for BenchModel {
    fn execute_weighted<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Particle<f64> {
        // A moderately complex model to justify parallelism
        let mut x = 0.0;
        let dist = Normal::new(0.0, 1.0);
        for _ in 0..100 {
            x += dist.sample(rng);
        }

        let log_weight = -0.5 * x * x;
        Particle::with_weight(x, log_weight)
    }
}

impl TraceableModel<f64> for BenchModel {
    fn execute_with_trace<R: rand::Rng + ?Sized>(
        &self,
        variables: &[f64],
        rng: &mut R,
    ) -> Trace<f64> {
        let mut x = 0.0;
        let mut vars = Vec::with_capacity(100);
        let mut iter = variables.iter();

        for _ in 0..100 {
            let u = if let Some(&v) = iter.next() {
                v
            } else {
                rng.random::<f64>()
            };
            vars.push(u);

            // Approximate standard normal from uniform
            // (u - 0.5) * sqrt(12) approx Normal(0, 1)
            x += (u - 0.5) * 3.4641;
        }

        let log_weight = -0.5 * x * x;
        Trace::new(vars, x, log_weight)
    }

    fn execute_with_trace_cow<'a, R: Rng + ?Sized>(
        &self,
        variables: &'a [f64],
        rng: &mut R,
    ) -> (f64, f64, Cow<'a, [f64]>) {
        let mut x = 0.0;
        let mut iter = variables.iter();
        let mut new_vars = None; // Initialize if we need to store new vars
        let mut vars_len = 0;

        for _ in 0..100 {
            if let Some(&v) = iter.next() {
                x += (v - 0.5) * 3.4641;
                vars_len += 1;
            } else {
                // Run out of variables, must sample new ones
                let u = rng.random::<f64>();

                // On first deviation, we must create a vector
                if new_vars.is_none() {
                    let mut v = Vec::with_capacity(100);
                    v.extend_from_slice(&variables[0..vars_len]);
                    new_vars = Some(v);
                }

                if let Some(ref mut v) = new_vars {
                    v.push(u);
                }
                x += (u - 0.5) * 3.4641;
            }
        }

        let log_weight = -0.5 * x * x;

        if let Some(v) = new_vars {
            (x, log_weight, Cow::Owned(v))
        } else {
            // No reallocation happened, so the input covered all 100 draws.
            // execute_with_trace's contract: the returned trace holds exactly
            // the variables used — borrow the input (or its consumed prefix).
            if variables.len() == 100 {
                (x, log_weight, Cow::Borrowed(variables))
            } else if variables.len() > 100 {
                (x, log_weight, Cow::Borrowed(&variables[0..100]))
            } else {
                // Should not happen if new_vars logic is correct
                unreachable!()
            }
        }
    }
}

struct HeavyModel;

impl WeightedModel<Vec<f64>> for HeavyModel {
    fn execute_weighted<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Particle<Vec<f64>> {
        // Create a heavy particle (e.g. 2KB state)
        // This simulates a large state that is expensive to clone
        let state = vec![0.0; 256];
        let log_weight = rng.random::<f64>();
        Particle::with_weight(state, log_weight)
    }
}

fn bench_smc(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMC");
    let particles = 1000;

    group.bench_function(BenchmarkId::new("Scalar", particles), |b| {
        b.iter(|| {
            let smc = SequentialMonteCarlo::new(particles);
            let mut rng = StdRng::seed_from_u64(42);
            let model = BenchModel;
            smc.infer_weighted(&model, &mut rng)
        });
    });

    group.bench_function(BenchmarkId::new("Parallel", particles), |b| {
        b.iter(|| {
            let smc = SequentialMonteCarlo::new(particles);
            let mut rng = StdRng::seed_from_u64(42);
            let model = BenchModel;
            smc.infer_weighted_parallel(&model, &mut rng)
        });
    });

    group.bench_function(BenchmarkId::new("Systematic", particles), |b| {
        b.iter(|| {
            let smc = SequentialMonteCarlo::new(particles)
                .with_resampling(ResamplingStrategy::Systematic);
            let mut rng = StdRng::seed_from_u64(42);
            let model = BenchModel;
            smc.infer_weighted(&model, &mut rng)
        });
    });

    // Benchmark for HeavyModel to test clone optimization
    group.bench_function(BenchmarkId::new("Heavy/Multinomial", particles), |b| {
        b.iter(|| {
            let smc = SequentialMonteCarlo::new(particles);
            let mut rng = StdRng::seed_from_u64(42);
            let model = HeavyModel;
            smc.infer_weighted(&model, &mut rng)
        });
    });

    group.bench_function(BenchmarkId::new("Heavy/Systematic", particles), |b| {
        b.iter(|| {
            let smc = SequentialMonteCarlo::new(particles)
                .with_resampling(ResamplingStrategy::Systematic);
            let mut rng = StdRng::seed_from_u64(42);
            let model = HeavyModel;
            smc.infer_weighted(&model, &mut rng)
        });
    });

    group.bench_function(BenchmarkId::new("Heavy/Parallel", particles), |b| {
        b.iter(|| {
            let smc = SequentialMonteCarlo::new(particles);
            let mut rng = StdRng::seed_from_u64(42);
            let model = HeavyModel;
            smc.infer_weighted_parallel(&model, &mut rng)
        });
    });

    group.finish();
}

fn bench_mh(c: &mut Criterion) {
    let mut group = c.benchmark_group("MH");
    let chains = 4;
    let iterations = 250; // Total 1000 samples across 4 chains

    group.bench_function(BenchmarkId::new("Scalar", chains * iterations), |b| {
        b.iter(|| {
            let mh = MetropolisHastings::new(iterations, 0);
            let mut rng = StdRng::seed_from_u64(42);
            let model = BenchModel;
            // Run chains sequentially
            for _ in 0..chains {
                mh.infer_traceable(&model, &mut rng);
            }
        });
    });

    group.bench_function(BenchmarkId::new("Parallel", chains * iterations), |b| {
        b.iter(|| {
            let mh = MetropolisHastings::new(iterations, 0);
            let mut rng = StdRng::seed_from_u64(42);
            let model = BenchModel;
            mh.infer_parallel_chains(&model, chains, &mut rng)
        });
    });

    group.finish();
}

fn bench_is(c: &mut Criterion) {
    let mut group = c.benchmark_group("ImportanceSampling");
    let samples = 1000;

    group.bench_function(BenchmarkId::new("Scalar", samples), |b| {
        b.iter(|| {
            let is = ImportanceSampling::new(samples);
            let mut rng = StdRng::seed_from_u64(42);
            let model_rng = std::sync::Mutex::new(StdRng::seed_from_u64(123));
            let dist = Normal::new(0.0, 1.0);

            is.infer(
                || {
                    let mut rng = model_rng.lock().unwrap();
                    let mut x = 0.0;
                    for _ in 0..100 {
                        x += dist.sample(&mut *rng);
                    }
                    x
                },
                &mut rng,
            )
        });
    });

    group.bench_function(BenchmarkId::new("Heavy", samples), |b| {
        b.iter(|| {
            let is = ImportanceSampling::new(samples);
            let mut rng = StdRng::seed_from_u64(42);
            // Heavy model just returns a vector, logic is cheap
            is.infer(|| vec![0.0; 256], &mut rng)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_smc, bench_mh, bench_is);
