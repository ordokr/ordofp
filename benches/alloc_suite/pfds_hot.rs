//! Hot-path micro-benchmarks mirroring the *real* downstream usage of pfds
//! persistent collections (a production LMS application's state maps).
//!
//! Unlike `pfds_bulk` (bulk builders + parallel algebra — not used by any live
//! consumer), this bench measures the single-operation pattern consumers hit on
//! every state update / query:
//!   * `get`  — field lookup (the most frequent op)
//!   * `insert` — single field update on an existing map
//!   * `clone` — confirm structural-sharing Clone is O(1)
//!
//! Crucially it isolates the *forced allocation* an owned-key `get(&self, &K)`
//! pattern imposes: a consumer holding `OrdMap<String, _>` and looking up by a
//! `&str` key must allocate a `String` per lookup (a now-retired UI
//! consumer did exactly this via `self.fields.get(&name.to_string())`).
//! The delta between `get_hit_owned_key` (allocates) and `get_hit_ref_key`
//! (no alloc) is the ceiling for a `Borrow`-based lookup API win.

use criterion::{BenchmarkId, Criterion, criterion_group};
use ordofp_core::pfds::OrdMap;
use std::hint::black_box;

/// Build an `OrdMap`<String,String> of `n` form-style fields: "`field_000`".. .
fn build_map(n: usize) -> (OrdMap<String, String>, Vec<String>) {
    let mut m = OrdMap::new();
    let mut keys = Vec::with_capacity(n);
    for i in 0..n {
        let k = format!("field_{i:03}");
        m = m.insert(k.clone(), format!("value_{i}"));
        keys.push(k);
    }
    (m, keys)
}

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("pfds_hot/OrdMap_get");
    // Realistic form / UI-state sizes.
    for &n in &[8usize, 32, 128] {
        let (map, keys) = build_map(n);
        let probe: &str = keys[n / 2].as_str(); // a key that exists

        // Current forced pattern: caller has &str, must allocate a String key.
        group.bench_with_input(BenchmarkId::new("hit_owned_key", n), &n, |b, _| {
            b.iter(|| black_box(map.get(&black_box(probe).to_string())));
        });

        // Cost without the allocation (caller already owns the String key).
        let owned = probe.to_string();
        group.bench_with_input(BenchmarkId::new("hit_ref_key", n), &n, |b, _| {
            b.iter(|| black_box(map.get(black_box(&owned))));
        });

        // NEW alloc-free path enabled by Borrow generalization: look up by &str
        // directly. This is what the consumer gets after dropping `.to_string()`.
        group.bench_with_input(BenchmarkId::new("hit_str_key", n), &n, |b, _| {
            b.iter(|| black_box(map.get(black_box(probe))));
        });
    }
    group.finish();
}

fn bench_insert_one(c: &mut Criterion) {
    let mut group = c.benchmark_group("pfds_hot/OrdMap_insert_one");
    for &n in &[8usize, 32, 128] {
        let (map, _keys) = build_map(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(map.insert("new_field".to_string(), "v".to_string())));
        });
    }
    group.finish();
}

fn bench_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("pfds_hot/OrdMap_clone");
    for &n in &[8usize, 128, 1024] {
        let (map, _keys) = build_map(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(map.clone()));
        });
    }
    group.finish();
}

/// Macro-ish: a form lifecycle — set N fields then read them all back once.
/// Mirrors `ImmutableFormState` building up then a render pass reading values.
fn bench_form_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("pfds_hot/form_cycle");
    for &n in &[8usize, 32] {
        let keys: Vec<String> = (0..n).map(|i| format!("field_{i:03}")).collect();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut m: OrdMap<String, String> = OrdMap::new();
                for (i, k) in keys.iter().enumerate() {
                    m = m.insert(k.clone(), format!("value_{i}"));
                }
                // render pass: alloc-free &str lookup (post-Borrow consumer pattern).
                let mut hits = 0usize;
                for k in &keys {
                    if m.get(black_box(k.as_str())).is_some() {
                        hits += 1;
                    }
                }
                black_box(hits)
            });
        });
    }
    group.finish();
}

criterion_group!(
    pfds_hot,
    bench_get,
    bench_insert_one,
    bench_clone,
    bench_form_cycle
);
