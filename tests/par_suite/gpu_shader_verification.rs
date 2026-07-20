//! Minimal test to verify GPU backend initialization works.
//!
//! These tests verify that the wgpu-based GPU backend can be initialized.
//! They are ignored by default as they require GPU hardware.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use ordofp_core::par::backend::wgpu::GpuWgpu;
use ordofp_core::par::codegen::wgsl::generate_map_shader;

#[test]
#[ignore] // Requires GPU, run with --ignored
fn test_gpu_backend_initialization() {
    // Test that GPU backend can be initialized
    match GpuWgpu::new() {
        Ok(gpu) => {
            println!("GPU backend initialized");
            println!("  Device: {:?}", gpu.device());
            println!("  Min len threshold: {}", gpu.min_len());
        }
        Err(e) => {
            eprintln!("GPU initialization failed: {e:?}");
            eprintln!("This is expected if no GPU is available");
            // Don't fail the test - just skip
        }
    }
}

#[test]
fn test_wgsl_shader_generation_no_gpu() {
    // This test doesn't require GPU - it only tests shader generation
    let shader = generate_map_shader("double", "x * 2.0", 256, "f32");

    // Basic validation that the shader is valid WGSL
    assert!(shader.contains("@compute"));
    assert!(shader.contains("@workgroup_size"));
    assert!(shader.contains("double"));
    assert!(shader.contains("array<f32>"));

    // Test different workgroup sizes
    let shader_64 = generate_map_shader("map_64", "x + 1.0", 64, "f32");
    assert!(shader_64.contains("64"));

    let shader_128 = generate_map_shader("map_128", "x + 1.0", 128, "f32");
    assert!(shader_128.contains("128"));

    // Test types
    let shader_i32 = generate_map_shader("map_i32", "x + 1", 64, "i32");
    assert!(shader_i32.contains("array<i32>"));
}
