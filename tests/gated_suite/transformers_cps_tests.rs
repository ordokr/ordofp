//! Tests for CPS transformers.

#![cfg(feature = "transformers-cps")]

// Access transformers through internal module path
// They're feature-gated, so we access them directly
use ordofp_core::transformers::ecclesia::lector_ecclesia::LectorEcclesiaT;

#[test]
fn test_lector_ecclesia_basic() {
    let reader = LectorEcclesiaT::new(|env: i32| env * 2);

    let result = reader.run(5);
    assert_eq!(result, 10);
}

#[test]
fn test_lector_ecclesia_map() {
    let reader = LectorEcclesiaT::new(|env: i32| env);
    let mapped = reader.map(|x| x * 2);

    let result = mapped.run(5);
    assert_eq!(result, 10);
}

#[test]
fn test_lector_ecclesia_flat_map() {
    let reader1 = LectorEcclesiaT::new(|env: i32| env);
    let reader2 = reader1.flat_map(|x| LectorEcclesiaT::new(move |env: i32| env + x));

    let result = reader2.run(5);
    assert_eq!(result, 10); // 5 + 5
}

#[test]
fn test_lector_ecclesia_ask() {
    let reader = LectorEcclesiaT::<i32, i32>::ask();

    let result = reader.run(42);
    assert_eq!(result, 42);
}

#[test]
fn test_lector_ecclesia_local() {
    let reader = LectorEcclesiaT::new(|env: i32| env);
    let local_reader = reader.local(|env| env * 2);

    let result = local_reader.run(5);
    assert_eq!(result, 10);
}

#[test]
fn test_cps_left_associative_chains() {
    // Test that CPS transformers handle left-associated chains efficiently
    let mut chain = LectorEcclesiaT::new(|env: i32| env);

    // Create a deep chain
    for _ in 0..100 {
        chain = chain.flat_map(|x| LectorEcclesiaT::new(move |env: i32| env + x));
    }

    // Should complete without stack overflow
    let result = chain.run(1);
    assert!(result > 0);
}

#[test]
fn test_cps_composition() {
    // Test composition of different transformers
    let reader = LectorEcclesiaT::new(|env: i32| env);
    let mapped = reader.map(|x| x * 2);
    let local = mapped.local(|env| env + 1);

    let result = local.run(5);
    assert_eq!(result, 12); // (5 + 1) * 2
}
