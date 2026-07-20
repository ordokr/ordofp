//! Serde serialization benchmarks for PFDS types vs std collections.
//!
//! These benchmarks use criterion for stable Rust compatibility.
//!
//! Run with: `cargo bench --features serde --bench serde`
//!
//! These benchmarks compare serialization/deserialization performance of
//! `OrdoFP`'s persistent data structures against their std equivalents.

use criterion::{Criterion, criterion_group, criterion_main};
use ordofp_core::pfds::{OrdMap, OrdSet, Queue, Seq, Stack};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hint::black_box;

// ============================================================================
// Stack vs Vec Benchmarks
// ============================================================================

fn stack_serialize_50(c: &mut Criterion) {
    // Stack uses recursive structure - limit to 50 for JSON depth safety
    let mut stack = Stack::new();
    for i in 0..50 {
        stack = stack.push(i);
    }
    c.bench_function("stack_serialize_50", |b| {
        b.iter(|| serde_json::to_string(black_box(&stack)).unwrap());
    });
}

fn vec_serialize_100(c: &mut Criterion) {
    let vec: Vec<i32> = (0..100).collect();
    c.bench_function("vec_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&vec)).unwrap());
    });
}

fn stack_deserialize_50(c: &mut Criterion) {
    // Stack uses recursive structure - limit to 50 for JSON depth safety
    let mut stack = Stack::new();
    for i in 0..50 {
        stack = stack.push(i);
    }
    let json = serde_json::to_string(&stack).unwrap();
    c.bench_function("stack_deserialize_50", |b| {
        b.iter(|| serde_json::from_str::<Stack<i32>>(black_box(&json)).unwrap());
    });
}

fn vec_deserialize_100(c: &mut Criterion) {
    let vec: Vec<i32> = (0..100).collect();
    let json = serde_json::to_string(&vec).unwrap();
    c.bench_function("vec_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<Vec<i32>>(black_box(&json)).unwrap());
    });
}

// ============================================================================
// Queue vs VecDeque Benchmarks
// ============================================================================

fn queue_serialize_100(c: &mut Criterion) {
    let mut queue = Queue::new();
    for i in 0..100 {
        queue = queue.enqueue(i);
    }
    c.bench_function("queue_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&queue)).unwrap());
    });
}

fn vecdeque_serialize_100(c: &mut Criterion) {
    let deque: VecDeque<i32> = (0..100).collect();
    c.bench_function("vecdeque_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&deque)).unwrap());
    });
}

fn queue_deserialize_100(c: &mut Criterion) {
    let mut queue = Queue::new();
    for i in 0..100 {
        queue = queue.enqueue(i);
    }
    let json = serde_json::to_string(&queue).unwrap();
    c.bench_function("queue_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<Queue<i32>>(black_box(&json)).unwrap());
    });
}

fn vecdeque_deserialize_100(c: &mut Criterion) {
    let deque: VecDeque<i32> = (0..100).collect();
    let json = serde_json::to_string(&deque).unwrap();
    c.bench_function("vecdeque_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<VecDeque<i32>>(black_box(&json)).unwrap());
    });
}

// ============================================================================
// Seq vs Vec Benchmarks (random access sequence)
// ============================================================================

fn seq_serialize_100(c: &mut Criterion) {
    let mut seq = Seq::new();
    for i in 0..100 {
        seq = seq.push_back(i);
    }
    c.bench_function("seq_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&seq)).unwrap());
    });
}

fn seq_deserialize_100(c: &mut Criterion) {
    let mut seq = Seq::new();
    for i in 0..100 {
        seq = seq.push_back(i);
    }
    let json = serde_json::to_string(&seq).unwrap();
    c.bench_function("seq_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<Seq<i32>>(black_box(&json)).unwrap());
    });
}

// ============================================================================
// OrdMap vs BTreeMap Benchmarks
// ============================================================================

fn ordmap_serialize_100(c: &mut Criterion) {
    let mut map = OrdMap::new();
    for i in 0..100 {
        map = map.insert(format!("key_{i}"), i);
    }
    c.bench_function("ordmap_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&map)).unwrap());
    });
}

fn btreemap_serialize_100(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    for i in 0..100 {
        map.insert(format!("key_{i}"), i);
    }
    c.bench_function("btreemap_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&map)).unwrap());
    });
}

fn ordmap_deserialize_100(c: &mut Criterion) {
    let mut map = OrdMap::new();
    for i in 0..100 {
        map = map.insert(format!("key_{i}"), i);
    }
    let json = serde_json::to_string(&map).unwrap();
    c.bench_function("ordmap_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<OrdMap<String, i32>>(black_box(&json)).unwrap());
    });
}

fn btreemap_deserialize_100(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    for i in 0..100 {
        map.insert(format!("key_{i}"), i);
    }
    let json = serde_json::to_string(&map).unwrap();
    c.bench_function("btreemap_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<BTreeMap<String, i32>>(black_box(&json)).unwrap());
    });
}

// ============================================================================
// OrdSet vs BTreeSet Benchmarks
// ============================================================================

fn ordset_serialize_100(c: &mut Criterion) {
    let mut set = OrdSet::new();
    for i in 0..100 {
        set = set.insert(i);
    }
    c.bench_function("ordset_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&set)).unwrap());
    });
}

fn btreeset_serialize_100(c: &mut Criterion) {
    let set: BTreeSet<i32> = (0..100).collect();
    c.bench_function("btreeset_serialize_100", |b| {
        b.iter(|| serde_json::to_string(black_box(&set)).unwrap());
    });
}

fn ordset_deserialize_100(c: &mut Criterion) {
    let mut set = OrdSet::new();
    for i in 0..100 {
        set = set.insert(i);
    }
    let json = serde_json::to_string(&set).unwrap();
    c.bench_function("ordset_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<OrdSet<i32>>(black_box(&json)).unwrap());
    });
}

fn btreeset_deserialize_100(c: &mut Criterion) {
    let set: BTreeSet<i32> = (0..100).collect();
    let json = serde_json::to_string(&set).unwrap();
    c.bench_function("btreeset_deserialize_100", |b| {
        b.iter(|| serde_json::from_str::<BTreeSet<i32>>(black_box(&json)).unwrap());
    });
}

// ============================================================================
// Large Collection Benchmarks (reduced sizes due to JSON recursion limits)
// ============================================================================

fn vec_serialize_500(c: &mut Criterion) {
    let vec: Vec<i32> = (0..500).collect();
    c.bench_function("vec_serialize_500", |b| {
        b.iter(|| serde_json::to_string(black_box(&vec)).unwrap());
    });
}

fn ordmap_serialize_200(c: &mut Criterion) {
    // OrdMap uses tree structure - more depth efficient
    let mut map = OrdMap::new();
    for i in 0..200 {
        map = map.insert(i, i * 2);
    }
    c.bench_function("ordmap_serialize_200", |b| {
        b.iter(|| serde_json::to_string(black_box(&map)).unwrap());
    });
}

fn btreemap_serialize_500(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    for i in 0..500 {
        map.insert(i, i * 2);
    }
    c.bench_function("btreemap_serialize_500", |b| {
        b.iter(|| serde_json::to_string(black_box(&map)).unwrap());
    });
}

criterion_group!(
    serde_benches,
    // Stack vs Vec
    stack_serialize_50,
    vec_serialize_100,
    stack_deserialize_50,
    vec_deserialize_100,
    // Queue vs VecDeque
    queue_serialize_100,
    vecdeque_serialize_100,
    queue_deserialize_100,
    vecdeque_deserialize_100,
    // Seq
    seq_serialize_100,
    seq_deserialize_100,
    // OrdMap vs BTreeMap
    ordmap_serialize_100,
    btreemap_serialize_100,
    ordmap_deserialize_100,
    btreemap_deserialize_100,
    // OrdSet vs BTreeSet
    ordset_serialize_100,
    btreeset_serialize_100,
    ordset_deserialize_100,
    btreeset_deserialize_100,
    // Larger collections (reduced sizes for JSON limits)
    vec_serialize_500,
    ordmap_serialize_200,
    btreemap_serialize_500,
);

criterion_main!(serde_benches);
