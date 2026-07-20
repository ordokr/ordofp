//! Benchmarks for stream fusion and Free monad performance.
//!
//! These benchmarks verify that inline annotations and Church encoding
//! provide the expected performance improvements.

use criterion::{BenchmarkId, Criterion, criterion_group};
use ordofp_core::free::{CodIdentity, CodOption, Liber, LiberEcclesia, OptionFWitness};
use std::hint::black_box;

// =============================================================================
// Flumen fusion (boxed chain vs FlumenFusus)
// =============================================================================

#[cfg(all(feature = "async", feature = "fusion"))]
mod flumen_fusus_benches {
    use super::{Criterion, black_box};
    use core::pin::Pin;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use ordofp_core::async_core::Flumen;

    fn noop_waker() -> Waker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(core::ptr::null(), &VTABLE)
        }
        unsafe fn wake(_: *const ()) {}
        unsafe fn wake_by_ref(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
    }

    fn block_on<F: core::future::Future>(mut fut: F) -> F::Output {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        // Safety: we never move `fut` after pinning it.
        let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(v) => return v,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    pub fn bench_flumen_fusion(c: &mut Criterion) {
        let data: Vec<i32> = (0..50_000).collect();
        let mut group = c.benchmark_group("Flumen/Fusion");

        group.bench_function("BoxedChain", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone())
                    .fmap(|x| x + 1)
                    .filter(|x| *x % 2 == 0)
                    .take(10_000)
                    .collect_vec();
                black_box(block_on(fut))
            });
        });

        group.bench_function("FusedChain", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone())
                    .fuse()
                    .map(|x| x + 1)
                    .filter(|x| *x % 2 == 0)
                    .take(10_000)
                    .collect_vec();
                black_box(block_on(fut))
            });
        });

        group.finish();
    }
}

// =============================================================================
// ParFlumen Benchmarks
// =============================================================================

#[cfg(feature = "par")]
mod par_benches {
    use super::{Criterion, black_box};
    use ordofp_core::par::ParFlumen;
    use ordofp_core::par::backend::CpuScalar;

    pub fn bench_par_take(c: &mut Criterion) {
        let mut group = c.benchmark_group("ParFlumen/Take");
        let size = 100_000;
        let data: Vec<i32> = (0..size).collect();

        // Benchmark: take small amount from large stream with expensive map
        group.bench_function("take_with_map", |b| {
            b.iter(|| {
                let s = ParFlumen::from_vec(data.clone())
                    .map(|x| {
                        // Simulate work
                        let mut sum = 0;
                        for i in 0..100 {
                            sum += i + x;
                        }
                        sum
                    })
                    .take(100);
                black_box(s.collect_vec(&CpuScalar))
            });
        });

        group.finish();
    }

    pub fn bench_par_zip(c: &mut Criterion) {
        let mut group = c.benchmark_group("ParFlumen/Zip");
        let size = 10_000;
        let data1: Vec<i32> = (0..size).collect();
        let data2: Vec<i32> = (0..size).collect();

        group.bench_function("zip_scalar_backend", |b| {
            b.iter(|| {
                let s1 = ParFlumen::from_vec(data1.clone());
                let s2 = ParFlumen::from_vec(data2.clone());
                let zipped = s1.zip(s2);
                black_box(zipped.collect_vec(&CpuScalar))
            });
        });

        group.finish();
    }

    pub fn bench_par_zip_unindexed(c: &mut Criterion) {
        let mut group = c.benchmark_group("ParFlumen/ZipUnindexed");
        let size = 10_000;
        let data1: Vec<i32> = (0..size).collect();
        let data2: Vec<i32> = (0..size).collect();

        // Baseline: both indexed
        group.bench_function("zip_indexed_indexed", |b| {
            b.iter(|| {
                let s1 = ParFlumen::from_vec(data1.clone());
                let s2 = ParFlumen::from_vec(data2.clone());
                let zipped = s1.zip(s2);
                black_box(zipped.collect_vec(&CpuScalar))
            });
        });

        // Test: first indexed, second unindexed
        group.bench_function("zip_indexed_unindexed", |b| {
            b.iter(|| {
                let s1 = ParFlumen::from_vec(data1.clone());
                let s2 = ParFlumen::from_vec(data2.clone()).filter(|_| true);
                let zipped = s1.zip(s2);
                black_box(zipped.collect_vec(&CpuScalar))
            });
        });

        // Test: first unindexed, second indexed
        group.bench_function("zip_unindexed_indexed", |b| {
            b.iter(|| {
                let s1 = ParFlumen::from_vec(data1.clone()).filter(|_| true);
                let s2 = ParFlumen::from_vec(data2.clone());
                let zipped = s1.zip(s2);
                black_box(zipped.collect_vec(&CpuScalar))
            });
        });

        // Test: both unindexed
        group.bench_function("zip_unindexed_unindexed", |b| {
            b.iter(|| {
                let s1 = ParFlumen::from_vec(data1.clone()).filter(|_| true);
                let s2 = ParFlumen::from_vec(data2.clone()).filter(|_| true);
                let zipped = s1.zip(s2);
                black_box(zipped.collect_vec(&CpuScalar))
            });
        });

        group.finish();
    }
}

// =============================================================================
// Free Monad Performance: Liber vs LiberEcclesia
// =============================================================================

/// Extract value from Liber if pure
fn extract_liber<F: ordofp_core::typeclasses::hkt::FunctorHKT, A>(free: Liber<F, A>) -> Option<A> {
    match free {
        Liber::Purus(v) => Some(v),
        Liber::Suspensus(_) => None,
    }
}

/// Benchmark left-associated bind operations on Liber (O(n²) expected)
fn bench_liber_left_assoc(c: &mut Criterion) {
    let mut group = c.benchmark_group("FreeMonad/LeftAssociated");

    for n in &[10, 50, 100, 200] {
        group.bench_with_input(BenchmarkId::new("Liber", n), n, |b, &n| {
            b.iter(|| {
                let mut free: Liber<OptionFWitness, i32> = Liber::purus(0);
                for i in 0..n {
                    free = free.flat_map(move |x| Liber::purus(x + i));
                }
                black_box(extract_liber(free))
            });
        });
    }

    group.finish();
}

/// Benchmark left-associated bind operations on `LiberEcclesia` (O(n) expected)
fn bench_liber_ecclesia_left_assoc(c: &mut Criterion) {
    let mut group = c.benchmark_group("FreeMonad/LeftAssociated");

    for n in &[10, 50, 100, 200] {
        group.bench_with_input(BenchmarkId::new("LiberEcclesia", n), n, |b, &n| {
            b.iter(|| {
                let mut free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(0);
                for i in 0..n {
                    free = free.flat_map(move |x| LiberEcclesia::purus(x + i));
                }
                black_box(free.extract_pure())
            });
        });
    }

    group.finish();
}

// =============================================================================
// Codensity Performance
// =============================================================================

/// Benchmark Codensity transform for Option (O(1) bind)
fn bench_codensity_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("Codensity/Option");

    for n in &[10, 50, 100, 500, 1000] {
        // CodOption with O(1) bind
        group.bench_with_input(BenchmarkId::new("CodOption", n), n, |b, &n| {
            b.iter(|| {
                let mut cod: CodOption<i32> = CodOption::purus(0);
                for i in 0..n {
                    cod = cod.flat_map(move |x| CodOption::purus(x + i));
                }
                black_box(cod.lower())
            });
        });

        // Raw Option for comparison
        group.bench_with_input(BenchmarkId::new("RawOption", n), n, |b, &n| {
            b.iter(|| {
                let mut opt: Option<i32> = Some(0);
                for i in 0..n {
                    opt = opt.map(move |x| x + i);
                }
                black_box(opt)
            });
        });
    }

    group.finish();
}

/// Benchmark Codensity transform for Identity (pure computation)
fn bench_codensity_identity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Codensity/Identity");

    for n in &[100, 500, 1000, 5000] {
        group.bench_with_input(BenchmarkId::new("CodIdentity", n), n, |b, &n| {
            b.iter(|| {
                let mut cod: CodIdentity<i32> = CodIdentity::purus(0);
                for i in 0..n {
                    cod = cod.flat_map(move |x| CodIdentity::purus(x + i));
                }
                black_box(cod.run())
            });
        });

        // Direct computation for comparison
        group.bench_with_input(BenchmarkId::new("Direct", n), n, |b, &n| {
            b.iter(|| {
                let mut result = 0;
                for i in 0..n {
                    result += i;
                }
                black_box(result)
            });
        });
    }

    group.finish();
}

// =============================================================================
// Iterator/Collection Fusion
// =============================================================================

/// Benchmark iterator fusion with multiple chained operations
fn bench_iterator_fusion(c: &mut Criterion) {
    let data: Vec<i32> = (0..10000).collect();

    let mut group = c.benchmark_group("Iterator/Fusion");

    // Chained iterator operations (should fuse with inline)
    group.bench_function("Chained", |b| {
        b.iter(|| {
            let result: Vec<i32> = data
                .iter()
                .map(|x| x + 1)
                .filter(|x| x % 2 == 0)
                .map(|x| x * 2)
                .take(1000)
                .collect();
            black_box(result)
        });
    });

    // Separate allocations (no fusion)
    group.bench_function("Separate", |b| {
        b.iter(|| {
            let v1: Vec<i32> = data.iter().map(|x| x + 1).collect();
            let v2: Vec<i32> = v1.into_iter().filter(|x| x % 2 == 0).collect();
            let v3: Vec<i32> = v2.into_iter().map(|x| x * 2).collect();
            let result: Vec<i32> = v3.into_iter().take(1000).collect();
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark functor map fusion
fn bench_map_fusion(c: &mut Criterion) {
    let data: Vec<i32> = (0..10000).collect();

    let mut group = c.benchmark_group("Functor/MapFusion");

    // Chained maps (should fuse)
    group.bench_function("ChainedMaps", |b| {
        b.iter(|| {
            let result: Vec<i32> = data
                .iter()
                .map(|x| x + 1)
                .map(|x| x * 2)
                .map(|x| x - 1)
                .map(|x| x / 2)
                .collect();
            black_box(result)
        });
    });

    // Single composed map
    group.bench_function("ComposedMap", |b| {
        b.iter(|| {
            let result: Vec<i32> = data.iter().map(|x| ((x + 1) * 2 - 1) / 2).collect();
            black_box(result)
        });
    });

    group.finish();
}

// =============================================================================
// Free Monad Interpretation Performance
// =============================================================================

/// Benchmark pure value extraction
fn bench_pure_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("FreeMonad/PureExtraction");

    group.bench_function("Liber", |b| {
        b.iter(|| {
            let free: Liber<OptionFWitness, i32> = Liber::purus(42);
            black_box(extract_liber(free))
        });
    });

    group.bench_function("LiberEcclesia", |b| {
        b.iter(|| {
            let free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42);
            black_box(free.extract_pure())
        });
    });

    group.finish();
}

/// Benchmark monad law associativity verification
fn bench_associativity(c: &mut Criterion) {
    let mut group = c.benchmark_group("MonadLaws/Associativity");

    // Test (m >>= f) >>= g vs m >>= (\x -> f x >>= g)
    group.bench_function("LeftAssociated", |b| {
        b.iter(|| {
            let m = Some(5);
            let f = |x: i32| Some(x * 2);
            let g = |x: i32| Some(x + 1);

            // Left associated: (m >>= f) >>= g
            black_box(m.and_then(f).and_then(g))
        });
    });

    group.bench_function("RightAssociated", |b| {
        b.iter(|| {
            let m = Some(5);
            let f = |x: i32| Some(x * 2);
            let g = |x: i32| Some(x + 1);

            // Right associated: m >>= (\x -> f x >>= g)
            black_box(m.and_then(|x| f(x).and_then(g)))
        });
    });

    group.finish();
}

// =============================================================================
// Memory Coalescing: AcervusParvus (SmallStack) Performance
// =============================================================================

/// Benchmark inline vs heap storage for small stacks
fn bench_inline_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("MemoryCoalescing/InlineStorage");

    // Small stack (inline) - typical monadic pipeline
    for n in &[2, 3, 4] {
        group.bench_with_input(BenchmarkId::new("SmallChain", n), n, |b, &n| {
            b.iter(|| {
                let mut free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(0);
                for i in 0..n {
                    free = free.flat_map(move |x| LiberEcclesia::purus(x + i));
                }
                black_box(free.extract_pure())
            });
        });
    }

    // Compare with larger chains that spill to heap
    for n in &[5, 10, 20] {
        group.bench_with_input(BenchmarkId::new("LargeChain", n), n, |b, &n| {
            b.iter(|| {
                let mut free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(0);
                for i in 0..n {
                    free = free.flat_map(move |x| LiberEcclesia::purus(x + i));
                }
                black_box(free.extract_pure())
            });
        });
    }

    group.finish();
}

/// Benchmark Codensity with varying chain lengths
fn bench_codensity_chain_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("MemoryCoalescing/CodensityChains");

    for n in &[4, 8, 16, 32, 64] {
        group.bench_with_input(BenchmarkId::new("CodOption", n), n, |b, &n| {
            b.iter(|| {
                let mut cod: CodOption<i32> = CodOption::purus(0);
                for i in 0..n {
                    cod = cod.flat_map(move |x| CodOption::purus(x + i));
                }
                black_box(cod.lower())
            });
        });

        group.bench_with_input(BenchmarkId::new("CodIdentity", n), n, |b, &n| {
            b.iter(|| {
                let mut cod: CodIdentity<i32> = CodIdentity::purus(0);
                for i in 0..n {
                    cod = cod.flat_map(move |x| CodIdentity::purus(x + i));
                }
                black_box(cod.run())
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_liber_left_assoc,
    bench_liber_ecclesia_left_assoc,
    bench_codensity_option,
    bench_codensity_identity,
    bench_iterator_fusion,
    bench_map_fusion,
    bench_pure_extraction,
    bench_associativity,
    bench_inline_storage,
    bench_codensity_chain_lengths,
);

#[cfg(all(feature = "async", feature = "fusion"))]
criterion_group!(
    flumen_fusion_benches,
    flumen_fusus_benches::bench_flumen_fusion
);

#[cfg(feature = "par")]
criterion_group!(
    par_flumen_benches,
    par_benches::bench_par_zip,
    par_benches::bench_par_zip_unindexed,
    par_benches::bench_par_take
);
