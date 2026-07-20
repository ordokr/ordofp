//! Kernel compilation and caching for wgpu compute shaders.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use wgpu::{BindGroupLayout, Buffer, BufferUsages, ComputePipeline, Device, ShaderSource};

use super::error::GpuResult;
use crate::par::codegen::wgsl::KernelKey;

/// Cached compute pipeline.
pub(crate) struct CachedPipeline {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
}

/// Kernel cache for compiled shader pipelines.
///
/// This cache stores compiled WGSL shaders to avoid recompilation
/// on repeated use of the same operations.
pub(crate) struct KernelCache {
    /// Map from kernel key to compiled pipeline.
    pipelines: BTreeMap<alloc::string::String, CachedPipeline>,
    /// Device for creating pipelines.
    device: Arc<Device>,
    /// Pool of reusable buffers to avoid per-frame allocation.
    buffer_pool: Vec<Buffer>,
}

impl KernelCache {
    /// Create a new kernel cache.
    #[inline]
    pub(crate) fn new(device: Arc<Device>) -> Self {
        Self {
            pipelines: BTreeMap::new(),
            device,
            // Pre-size pool for a few reusable buffers; avoids first-frame realloc.
            buffer_pool: Vec::with_capacity(8),
        }
    }

    /// Acquire a buffer from the pool or create a new one.
    ///
    /// Finds a buffer in the pool with size >= requested size.
    /// If none found, creates a new one using `create_output_buffer`
    /// (which sets STORAGE | `COPY_SRC` usage).
    /// (Infallible today; the `GpuResult` return matches the fallible GPU-op
    /// interface its `?`-style callers rely on.)
    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub(crate) fn acquire_buffer(&mut self, size: usize) -> Buffer {
        // Reuse buffers to avoid allocation overhead.
        // We look for any buffer that is large enough.
        // Ideally we pick the smallest one that fits, but for now first fit is fine.
        let idx = self
            .buffer_pool
            .iter()
            .position(|b| b.size() >= size as u64);

        if let Some(i) = idx {
            // Found a suitable buffer
            self.buffer_pool.swap_remove(i)
        } else {
            // Create new buffer
            // Ensure COPY_DST is set so we can use write_buffer (reusing buffer as input)
            // Also STORAGE and COPY_SRC are needed for shader output/readback.
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: size as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        }
    }

    /// Release a buffer back to the pool for reuse.
    #[inline]
    pub(crate) fn release_buffer(&mut self, buffer: Buffer) {
        self.buffer_pool.push(buffer);
    }

    /// Get or compile a compute pipeline for the given kernel key.
    ///
    /// Optimization: Uses entry API to avoid double `BTreeMap` lookup (`contains_key` + get)
    /// and eliminates `cache_key.clone()` on insertion.
    #[inline]
    pub(crate) fn get_or_compile(&mut self, key: &KernelKey) -> GpuResult<&CachedPipeline> {
        // Use shader source as cache key — pre-size to avoid realloc.
        let mut cache_key = alloc::string::String::with_capacity(
            key.entry_point.len() + 1 + key.shader_source.len(),
        );
        cache_key.push_str(&key.entry_point);
        cache_key.push(':');
        cache_key.push_str(&key.shader_source);

        // Optimization: Use entry API for single lookup instead of contains_key + get/insert
        use alloc::collections::btree_map::Entry;
        let entry = self.pipelines.entry(cache_key);

        match entry {
            Entry::Occupied(occupied) => Ok(occupied.into_mut()),
            Entry::Vacant(vacant) => {
                // Catch WGSL validation errors instead of panicking in wgpu's
                // default uncaptured-error handler; a bad user expression must
                // route to the CPU fallback (GpuError -> fallback), not abort.
                //
                // wgpu 29: `push_error_scope` returns an `ErrorScopeGuard`; the
                // scope is popped via `guard.pop()` (there is no
                // `Device::pop_error_scope`), unlike older wgpu releases.
                let error_scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);

                // Create shader module from WGSL string
                // Pattern from wgpu's examples/standalone/01_hello_compute/src/main.rs
                let shader_module =
                    self.device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some(&alloc::format!("parflumen_{}", key.entry_point)),
                            source: ShaderSource::Wgsl(Cow::Borrowed(&key.shader_source)),
                        });

                // Create bind group layout for storage buffers (input + output)
                let bind_group_layout =
                    self.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &[
                                // Input buffer (binding 0)
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                                        min_binding_size: Some(
                                            core::num::NonZeroU64::new(4).unwrap(),
                                        ),
                                        has_dynamic_offset: false,
                                    },
                                    count: None,
                                },
                                // Output buffer (binding 1)
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                                        min_binding_size: Some(
                                            core::num::NonZeroU64::new(4).unwrap(),
                                        ),
                                        has_dynamic_offset: false,
                                    },
                                    count: None,
                                },
                                // Params uniform (binding 2): logical element count.
                                wgpu::BindGroupLayoutEntry {
                                    binding: 2,
                                    visibility: wgpu::ShaderStages::COMPUTE,
                                    ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        min_binding_size: Some(
                                            core::num::NonZeroU64::new(4).unwrap(),
                                        ),
                                        has_dynamic_offset: false,
                                    },
                                    count: None,
                                },
                            ],
                        });

                // Create pipeline layout
                let pipeline_layout =
                    self.device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: None,
                            // wgpu 29: bind group layout slots are now optional
                            // (`&[Option<&BindGroupLayout>]`) so unused slots can be None.
                            bind_group_layouts: &[Some(&bind_group_layout)],
                            immediate_size: 0,
                        });

                // Create compute pipeline
                // wgpu 27.0: cache field added
                let pipeline =
                    self.device
                        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                            label: Some(&alloc::format!("parflumen_{}", key.entry_point)),
                            layout: Some(&pipeline_layout),
                            module: &shader_module,
                            entry_point: Some(&key.entry_point),
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                            cache: None,
                        });

                if let Some(e) = super::block_on::block_on(error_scope.pop()) {
                    return Err(super::error::GpuError::ShaderCompilationFailed(
                        alloc::format!("{e}"),
                    ));
                }

                // Insert and return reference - no clone needed with entry API
                Ok(vacant.insert(CachedPipeline {
                    pipeline,
                    bind_group_layout,
                }))
            }
        }
    }
}
