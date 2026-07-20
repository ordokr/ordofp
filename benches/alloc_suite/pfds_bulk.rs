//! Benchmarks for PFDS bulk operations.
//!
//! Compares bulk builder performance vs repeated insert operations.

use criterion::{BenchmarkId, Criterion, criterion_group};
use ordofp_core::pfds::{OrdMap, OrdSet};
use std::hint::black_box;

fn bench_ordmap_bulk_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("OrdMap/BulkBuild");

    for size in &[100, 1000, 10000] {
        let entries: Vec<(i32, i32)> = (0..*size).map(|i| (i, i * 2)).collect();

        // Bulk build with structor
        group.bench_with_input(BenchmarkId::new("Structor", size), size, |b, _| {
            b.iter(|| {
                let mut builder = OrdMap::structor();
                for (k, v) in &entries {
                    builder.insert(*k, *v);
                }
                black_box(builder.finish())
            });
        });

        // Repeated insert (baseline)
        group.bench_with_input(BenchmarkId::new("RepeatedInsert", size), size, |b, _| {
            b.iter(|| {
                let mut map = OrdMap::new();
                for (k, v) in &entries {
                    map = map.insert(*k, *v);
                }
                black_box(map)
            });
        });

        // Parallel bulk build (if rayon feature available)
        #[cfg(feature = "rayon")]
        group.bench_with_input(BenchmarkId::new("StructorPar", size), size, |b, _| {
            b.iter(|| {
                let mut builder = OrdMap::structor();
                for (k, v) in &entries {
                    builder.insert(*k, *v);
                }
                black_box(builder.finish_par())
            });
        });
    }

    group.finish();
}

fn bench_ordset_bulk_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("OrdSet/BulkBuild");

    for size in &[100, 1000, 10000] {
        let items: Vec<i32> = (0..*size).collect();

        // Bulk build with structor
        group.bench_with_input(BenchmarkId::new("Structor", size), size, |b, _| {
            b.iter(|| {
                let mut builder = OrdSet::structor();
                for item in &items {
                    builder.push(*item);
                }
                black_box(builder.finish())
            });
        });

        // Repeated insert (baseline)
        group.bench_with_input(BenchmarkId::new("RepeatedInsert", size), size, |b, _| {
            b.iter(|| {
                let mut set = OrdSet::new();
                for item in &items {
                    set = set.insert(*item);
                }
                black_box(set)
            });
        });

        // Parallel bulk build (if rayon feature available)
        #[cfg(feature = "rayon")]
        group.bench_with_input(BenchmarkId::new("StructorPar", size), size, |b, _| {
            b.iter(|| {
                let mut builder = OrdSet::structor();
                for item in &items {
                    builder.push(*item);
                }
                black_box(builder.finish_par())
            });
        });
    }

    group.finish();
}

fn bench_ordmap_parallel_algebra(c: &mut Criterion) {
    let mut group = c.benchmark_group("OrdMap/ParallelAlgebra");

    for size in &[1000, 10000] {
        let map1: OrdMap<i32, i32> = (0..*size).map(|i| (i, i)).collect();
        let map2: OrdMap<i32, i32> = (*size / 2..*size * 3 / 2).map(|i| (i, i * 2)).collect();

        // Sequential union
        group.bench_with_input(BenchmarkId::new("Union", size), size, |b, _| {
            b.iter(|| black_box(map1.union(&map2)));
        });

        // Parallel union (if rayon feature available)
        #[cfg(feature = "rayon")]
        group.bench_with_input(BenchmarkId::new("UnionPar", size), size, |b, _| {
            b.iter(|| black_box(map1.union_par(&map2)));
        });

        // Sequential intersection
        group.bench_with_input(BenchmarkId::new("Intersection", size), size, |b, _| {
            b.iter(|| black_box(map1.intersection(&map2)));
        });

        // Parallel intersection (if rayon feature available)
        #[cfg(feature = "rayon")]
        group.bench_with_input(BenchmarkId::new("IntersectionPar", size), size, |b, _| {
            b.iter(|| black_box(map1.intersection_par(&map2)));
        });
    }

    group.finish();
}

criterion_group!(
    pfds_benches,
    bench_ordmap_bulk_build,
    bench_ordset_bulk_build,
    bench_ordmap_parallel_algebra
);
