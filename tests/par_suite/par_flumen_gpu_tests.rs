//! GPU backend tests for `ParFlumen`.
//!
//! These tests validate GPU execution correctness and performance.
//!
//! # Status
//!
//! GPU shader compilation is implemented using wgpu's `ShaderSource` API.
//! Tests are ignored by default as they require GPU hardware.
//!
//! See `core/src/par/backend/wgpu/` for the GPU backend implementation.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

#[cfg(feature = "rayon")]
use ordofp_core::par::backend::CpuRayon;
#[cfg(feature = "gpu-wgpu")]
use ordofp_core::par::backend::wgpu::GpuWgpu;
use ordofp_core::par::{ParFlumen, backend::CpuScalar};

/// Generate test data: 10M f32 values.
fn generate_10m_f32() -> Vec<f32> {
    (0..10_000_000).map(|i| i as f32).collect()
}

/// Test the new GPU execution API with `map_gpu` and `collect_gpu`.
#[test]
#[ignore = "Requires GPU hardware"]
fn test_gpu_map_collect_f32_api() {
    let data: Vec<f32> = (0..10_000).map(|i| i as f32).collect();

    // Expected result from CPU
    let expected: Vec<f32> = data.iter().map(|x| x * 2.0).collect();

    let backend = match GpuWgpu::new() {
        Ok(gpu) => gpu,
        Err(e) => {
            eprintln!("GPU initialization failed: {e:?}");
            return;
        }
    };

    // Use the new GPU API
    let stream = ParFlumen::from_vec_gpu(data);
    assert!(stream.is_gpu_capable(), "Stream should be GPU-capable");

    let gpu_result = stream.map_gpu("x * 2.0", |x| x * 2.0).collect_gpu(&backend);

    // Validate results match
    assert_eq!(expected.len(), gpu_result.len());
    for (i, (exp, got)) in expected.iter().zip(gpu_result.iter()).enumerate() {
        let diff = (exp - got).abs();
        assert!(
            diff < 1e-5,
            "Mismatch at index {i}: expected={exp}, got={got}"
        );
    }
}

/// Test chained GPU operations.
#[test]
#[ignore = "Requires GPU hardware"]
fn test_gpu_chained_operations() {
    let data: Vec<f32> = (1..1001).map(|i| i as f32).collect();

    // Expected: (x * 2.0) + 1.0
    let expected: Vec<f32> = data.iter().map(|x| (x * 2.0) + 1.0).collect();

    let backend = match GpuWgpu::new() {
        Ok(gpu) => gpu,
        Err(e) => {
            eprintln!("GPU initialization failed: {e:?}");
            return;
        }
    };

    // Chain multiple GPU operations
    let gpu_result = ParFlumen::from_vec_gpu(data)
        .map_gpu("x * 2.0", |x| x * 2.0)
        .map_gpu("x + 1.0", |x| x + 1.0)
        .collect_gpu(&backend);

    // Validate results match
    assert_eq!(expected.len(), gpu_result.len());
    for (i, (exp, got)) in expected.iter().zip(gpu_result.iter()).enumerate() {
        let diff = (exp - got).abs();
        assert!(
            diff < 1e-5,
            "Mismatch at index {i}: expected={exp}, got={got}"
        );
    }
}

/// Test GPU map with i32 types.
#[test]
#[ignore = "Requires GPU hardware"]
fn test_gpu_map_collect_i32_api() {
    let data: Vec<i32> = (0..10_000).collect();

    // Expected result from CPU
    let expected: Vec<i32> = data.iter().map(|x| x * 2).collect();

    let backend = match GpuWgpu::new() {
        Ok(gpu) => gpu,
        Err(e) => {
            eprintln!("GPU initialization failed: {e:?}");
            return;
        }
    };

    // Use the new GPU API
    let stream = ParFlumen::from_vec_gpu(data);
    assert!(stream.is_gpu_capable(), "Stream should be GPU-capable");

    let gpu_result = stream.map_gpu("x * 2", |x| x * 2).collect_gpu(&backend);

    // Validate results match
    assert_eq!(expected, gpu_result);
}

/// Test GPU chain composition.
#[test]
fn test_gpu_chain_composition() {
    let data: Vec<f32> = vec![1.0, 2.0, 3.0];

    let stream = ParFlumen::from_vec_gpu(data)
        .map_gpu("x * 2.0", |x| x * 2.0)
        .map_gpu("x + 1.0", |x| x + 1.0);

    // Check that the chain is GPU-capable
    assert!(stream.is_gpu_capable());

    // Check that the composed WGSL expression is correct
    if let Some(chain) = stream.gpu_chain() {
        let wgsl = chain.compose_wgsl();
        // The composed expression should nest the operations
        // First op: "x * 2.0" -> replaces x with input
        // Second op: "x + 1.0" -> replaces x with "(x * 2.0)"
        assert!(
            wgsl.contains("2.0") && wgsl.contains("1.0"),
            "WGSL expression should contain both operations: {wgsl}"
        );
    }
}

/// Test map+reduce on 10M f32 values.
///
/// This is the primary acceptance criterion for Phase 3.
/// Validates that GPU execution produces the same results as CPU.
#[test]
#[ignore = "Requires GPU hardware"]
fn test_gpu_map_reduce_10m_f32() {
    let data = generate_10m_f32();

    // CPU baseline
    let cpu_result = ParFlumen::from_vec(data.clone())
        .map(|x| x * 2.0)
        .reduce(&CpuScalar, |a, b| a + b)
        .unwrap_or(0.0);

    // GPU execution
    #[cfg(feature = "gpu-wgpu")]
    {
        let backend = match GpuWgpu::new() {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("GPU initialization failed: {e:?}");
                return;
            }
        };
        let gpu_result = ParFlumen::from_vec_gpu(data)
            .map_gpu("x * 2.0", |x| x * 2.0)
            .reduce_gpu(&backend, "+", |a, b| a + b)
            .unwrap_or(0.0);

        // Validate GPU matches CPU
        let diff = (cpu_result - gpu_result).abs();
        assert!(
            diff < 1e-5,
            "GPU result {gpu_result} differs from CPU {cpu_result} by {diff}"
        );
    }
}

/// Test map operation on large f32 array.
#[test]
#[ignore = "Requires GPU hardware"]
fn test_gpu_map_10m_f32() {
    let data = generate_10m_f32();

    // CPU baseline
    let cpu_result: Vec<f32> = ParFlumen::from_vec(data.clone())
        .map(|x| x * 2.0)
        .collect_vec(&CpuScalar);

    // GPU execution
    #[cfg(feature = "gpu-wgpu")]
    {
        let backend = match GpuWgpu::new() {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("GPU initialization failed: {e:?}");
                return;
            }
        };
        let gpu_result: Vec<f32> = ParFlumen::from_vec(data)
            .map(|x| x * 2.0)
            .collect_vec(&backend);

        // Validate GPU matches CPU
        assert_eq!(cpu_result.len(), gpu_result.len());
        for (i, (cpu_val, gpu_val)) in cpu_result.iter().zip(gpu_result.iter()).enumerate() {
            let diff = (cpu_val - gpu_val).abs();
            assert!(
                diff < 1e-5,
                "Mismatch at index {i}: CPU={cpu_val}, GPU={gpu_val}, diff={diff}"
            );
        }
    }
}

/// Test small-N heuristic (fallback to CPU for small arrays).
#[test]
fn test_gpu_small_n_heuristic() {
    let small_data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];

    // Small arrays should fallback to CPU
    let cpu_result: Vec<f32> = ParFlumen::from_vec(small_data.clone())
        .map(|x| x * 2.0)
        .collect_vec(&CpuScalar);

    #[cfg(feature = "gpu-wgpu")]
    {
        let backend = match GpuWgpu::new() {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("GPU initialization failed: {e:?}");
                // Test passes - we just validate CPU path
                assert_eq!(cpu_result, vec![2.0, 4.0, 6.0, 8.0, 10.0]);
                return;
            }
        };
        // GPU backend should fallback to CPU for small arrays
        let gpu_result: Vec<f32> = ParFlumen::from_vec(small_data)
            .map(|x| x * 2.0)
            .collect_vec(&backend);

        assert_eq!(cpu_result, gpu_result);
    }
}

/// Test GPU vs CPU vs Rayon equivalence on medium-sized arrays.
#[test]
#[ignore = "Requires GPU hardware"]
fn test_gpu_cpu_rayon_equivalence() {
    let data: Vec<f32> = (0..100_000).map(|i| i as f32).collect();

    let cpu_result: Vec<f32> = ParFlumen::from_vec(data.clone())
        .map(|x| x * 2.0)
        .collect_vec(&CpuScalar);

    #[cfg(feature = "rayon")]
    let rayon_result: Vec<f32> = ParFlumen::from_vec(data.clone())
        .map(|x| x * 2.0)
        .collect_vec(&CpuRayon::default());

    #[cfg(feature = "gpu-wgpu")]
    {
        let backend = match GpuWgpu::new() {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("GPU initialization failed: {e:?}");
                return;
            }
        };
        let gpu_result: Vec<f32> = ParFlumen::from_vec(data)
            .map(|x| x * 2.0)
            .collect_vec(&backend);

        // All backends should produce same results
        assert_eq!(cpu_result.len(), gpu_result.len());
        for (cpu_val, gpu_val) in cpu_result.iter().zip(gpu_result.iter()) {
            let diff = (cpu_val - gpu_val).abs();
            assert!(diff < 1e-5, "GPU mismatch: CPU={cpu_val}, GPU={gpu_val}");
        }

        #[cfg(feature = "rayon")]
        {
            assert_eq!(cpu_result.len(), rayon_result.len());
            for (cpu_val, rayon_val) in cpu_result.iter().zip(rayon_result.iter()) {
                let diff = (cpu_val - rayon_val).abs();
                assert!(
                    diff < 1e-5,
                    "Rayon mismatch: CPU={cpu_val}, Rayon={rayon_val}"
                );
            }
        }
    }
}

/// Microbenchmark: GPU map performance.
///
/// This test can be run with `cargo test --release -- --nocapture --ignored`
/// to measure GPU performance once shader compilation is working.
#[test]
#[ignore = "Microbenchmark - run manually with --ignored"]
fn bench_gpu_map_performance() {
    let data = generate_10m_f32();

    // CPU baseline timing
    let cpu_start = std::time::Instant::now();
    let _cpu_result: Vec<f32> = ParFlumen::from_vec(data.clone())
        .map(|x| x * 2.0)
        .collect_vec(&CpuScalar);
    let cpu_duration = cpu_start.elapsed();

    #[cfg(feature = "gpu-wgpu")]
    {
        let backend = match GpuWgpu::new() {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("GPU initialization failed: {e:?}");
                println!("CPU: {cpu_duration:?} (GPU backend not available)");
                return;
            }
        };

        // GPU timing
        let gpu_start = std::time::Instant::now();
        let _gpu_result: Vec<f32> = ParFlumen::from_vec(data)
            .map(|x| x * 2.0)
            .collect_vec(&backend);
        let gpu_duration = gpu_start.elapsed();

        println!(
            "CPU: {:?}, GPU: {:?}, Speedup: {:.2}x",
            cpu_duration,
            gpu_duration,
            cpu_duration.as_secs_f64() / gpu_duration.as_secs_f64()
        );
    }
}
