//! GPU end-to-end correctness tests. Each test self-skips (passes trivially)
//! when no adapter is present, so the suite stays green on headless CI-less
//! boxes while actually verifying on GPU-equipped machines.
#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use ordofp_core::par::{ParFlumen, backend::wgpu::GpuWgpu};

// `min_len` defaults to 1024 in `GpuWgpu::new()`; several regressions below
// (per-op identity/workgroup-padding codegen, N==1, shader-compile-error
// fallback, unknown-operator fallback) live in the actual GPU dispatch path,
// which the small-N CPU heuristic in `Backend::collect`/`reduce_gpu` would
// otherwise bypass entirely for inputs under 1024 elements. Force `min_len`
// down to 1 so every test body genuinely reaches the GPU codegen/dispatch
// code these tests exist to guard, not just the size-heuristic fallback.
macro_rules! gpu_or_skip {
    () => {
        match GpuWgpu::with_min_len(1) {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("skipping GPU test: {e}");
                return;
            }
        }
    };
}

/// C1 regression: a large op followed by a smaller one recycles a larger
/// pooled buffer; the reduce must not fold the stale tail.
#[test]
fn reduce_after_pool_reuse_is_exact() {
    let gpu = gpu_or_skip!();
    // Prime the pool with an 8192-element buffer.
    let big: Vec<f32> = (0..8192).map(|i| i as f32).collect();
    let _ = ParFlumen::from_vec_gpu(big)
        .map_gpu("x", |x| x)
        .collect_gpu(&gpu);
    // Now reduce 1024 ones; correct sum is exactly 1024.
    let small: Vec<f32> = vec![1.0; 1024];
    let sum = ParFlumen::from_vec_gpu(small).reduce_gpu(&gpu, "+", |a, b| a + b);
    assert_eq!(sum, Some(1024.0));
}

/// H1 regression: product identity must be 1, and non-multiple-of-64 sizes
/// must not zero the result.
#[test]
fn reduce_product_non_multiple_of_workgroup() {
    let gpu = gpu_or_skip!();
    let xs: Vec<f32> = vec![1.0; 100]; // 100 % 64 != 0
    let prod = ParFlumen::from_vec_gpu(xs).reduce_gpu(&gpu, "*", |a, b| a * b);
    assert_eq!(prod, Some(1.0));
}

/// H1 regression: advertised min/max must work (previously invalid WGSL).
#[test]
fn reduce_min_max() {
    let gpu = gpu_or_skip!();
    let xs: Vec<i32> = (1..=4097).collect(); // force multi-pass
    assert_eq!(
        ParFlumen::from_vec_gpu(xs.clone()).reduce_gpu(&gpu, "min", std::cmp::Ord::min),
        Some(1)
    );
    assert_eq!(
        ParFlumen::from_vec_gpu(xs).reduce_gpu(&gpu, "max", std::cmp::Ord::max),
        Some(4097)
    );
}

/// M7 regression: single element.
#[test]
fn reduce_single_element() {
    let gpu = gpu_or_skip!();
    let xs: Vec<f32> = vec![42.0];
    assert_eq!(
        ParFlumen::from_vec_gpu(xs).reduce_gpu(&gpu, "+", |a, b| a + b),
        Some(42.0)
    );
}

/// H3 regression: garbage WGSL must take the CPU fallback, not panic.
#[test]
fn bad_wgsl_falls_back_to_cpu() {
    let gpu = gpu_or_skip!();
    let xs: Vec<f32> = vec![1.0, 2.0, 3.0];
    let out = ParFlumen::from_vec_gpu(xs)
        .map_gpu("this is not wgsl !!!", |x| x * 2.0)
        .collect_gpu(&gpu);
    assert_eq!(out, vec![2.0, 4.0, 6.0]);
}

/// Unknown reduce operator: Err path -> fallback, not invalid shader.
#[test]
fn unknown_operator_falls_back() {
    let gpu = gpu_or_skip!();
    let xs: Vec<f32> = vec![1.0, 2.0, 3.0];
    let out = ParFlumen::from_vec_gpu(xs).reduce_gpu(&gpu, "^", |acc, x| acc + x);
    assert_eq!(out, Some(6.0));
}
