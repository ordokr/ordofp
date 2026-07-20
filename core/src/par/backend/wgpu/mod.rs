//! wgpu backend for `ParFlumen` - GPU execution via WGSL compute shaders.
//!
//! > *"Machina Graphica Computandi"*
//! > — Graphics computing machine. (Neo-Latin)
//!
//! This module provides GPU execution for `FlumenParallelum` by emitting WGSL
//! compute shaders and dispatching via wgpu.
//!
//! Adapted third-party patterns are inventoried in `ORIGINAL_SOURCE.md` in
//! this directory and the repo-root `THIRD_PARTY_NOTICES.md`.
//!
//! # Example
//!
//! This requires a physical GPU adapter, so it cannot run in a headless
//! doctest environment.
//!
//! ```rust,no_run
//! use ordofp_core::par::{ParFlumen, backend::wgpu::GpuWgpu};
//!
//! # fn run() -> Result<(), Box<dyn std::error::Error>> {
//! // Create GPU-capable stream
//! let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
//! let backend = GpuWgpu::new()?;
//! let result = ParFlumen::from_vec_gpu(data)
//!     .map_gpu("x * 2.0", |x| x * 2.0)
//!     .collect_gpu(&backend);
//! # let _ = result;
//! # Ok(())
//! # }
//! ```

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

mod block_on;
mod buffer;
mod device;
mod error;
mod execute;
mod kernel;

pub use device::GpuWgpu;
pub use error::{GpuError, GpuResult};

use alloc::string::ToString;
use alloc::vec::Vec;
use core::any::TypeId;

use super::Backend;
use crate::par::{GpuMapChain, Nodus};

/// Trait for types supported by the GPU backend.
///
/// This trait extends `bytemuck::Pod` to ensure data can be safely copied to/from GPU buffers.
/// It also provides the corresponding WGSL type name.
pub trait GpuScalar: bytemuck::Pod + Send + Sync + 'static {
    /// Get the WGSL type name for this type (e.g., "f32", "i32", "u32").
    fn wgsl_type() -> &'static str;
}

impl GpuScalar for f32 {
    fn wgsl_type() -> &'static str {
        "f32"
    }
}

impl GpuScalar for i32 {
    fn wgsl_type() -> &'static str {
        "i32"
    }
}

impl GpuScalar for u32 {
    fn wgsl_type() -> &'static str {
        "u32"
    }
}

impl GpuWgpu {
    /// Execute a GPU map chain on input data.
    fn execute_gpu_chain_bytes(
        &self,
        input_bytes: &[u8],
        chain: &GpuMapChain,
        type_name: &str,
        element_size: usize,
    ) -> GpuResult<Vec<u8>> {
        if input_bytes.is_empty() {
            return Ok(Vec::new());
        }

        // Compose the WGSL expression
        let wgsl_expr = chain.compose_wgsl();

        // Get or compile pipeline
        let mut cache = self.kernel_cache().lock().map_err(|_| {
            GpuError::DeviceCreationFailed("Failed to lock kernel cache".to_string())
        })?;

        // Execute map
        execute::execute_map_gpu_bytes(
            self.device(),
            self.queue(),
            &mut cache,
            input_bytes,
            &wgsl_expr,
            type_name,
            element_size,
        )
    }
}

impl Backend for GpuWgpu {
    fn collect<T>(&self, node: &dyn Nodus<Item = T>) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        // Check if we should use GPU
        if node.len() < self.min_len() {
            // Fallback to CPU for small arrays
            return node.collect_scalar();
        }

        // Try to get GPU map chain
        if let Some(chain) = node.try_gpu_map_chain() {
            // Try to get source data as bytes + type info
            if let Some((source_bytes, type_name)) = node.try_as_gpu_source() {
                // SAFETY: We must ensure T is one of the supported GPU scalar types (f32, i32, u32)
                // to prevent interpreting arbitrary bytes as non-POD types (e.g., String).
                if TypeId::of::<T>() != TypeId::of::<f32>()
                    && TypeId::of::<T>() != TypeId::of::<i32>()
                    && TypeId::of::<T>() != TypeId::of::<u32>()
                {
                    return node.collect_scalar();
                }

                // Execute on GPU
                // We know T corresponds to type_name/source_bytes because the chain was built from it.
                // Assuming T is consistent with source if chain exists.
                match self.execute_gpu_chain_bytes(
                    source_bytes,
                    &chain,
                    type_name,
                    core::mem::size_of::<T>(),
                ) {
                    Ok(output_bytes) => {
                        // Cast back to Vec<T>
                        // Safety:
                        // 1. The GPU execution returned bytes that are valid representation of Vec<T> (Pod).
                        // 2. We verify size and alignment.

                        let element_size = core::mem::size_of::<T>();
                        let element_align = core::mem::align_of::<T>();

                        if output_bytes.len() % element_size != 0 {
                            // This shouldn't happen if execution was correct
                            return node.collect_scalar();
                        }

                        let len = output_bytes.len() / element_size;
                        let ptr = output_bytes.as_ptr();

                        // Check alignment
                        if !(ptr as usize).is_multiple_of(element_align) {
                            // This is unlikely for WGPU buffers but theoretically possible?
                            // WGPU maps are usually aligned.
                            // Fallback if not aligned.
                            return node.collect_scalar();
                        }

                        // SAFETY:
                        // 1. `ptr` is aligned to `align_of::<T>()` — verified by the
                        //    alignment check above (we return early if unaligned).
                        // 2. The GPU buffer contains `len` initialized values of type T
                        //    written by the WGSL shader; the bytes are therefore a valid
                        //    bit-pattern for `[T]` (T is restricted to Pod types such as
                        //    f32/i32/u32 at all call sites that reach this path).
                        // 3. `to_vec()` copies the data into a freshly allocated `Vec<T>`
                        //    owned by the Rust allocator, so the resulting `Vec` is safe
                        //    to drop.  We intentionally avoid `Vec::from_raw_parts` here
                        //    because the backing memory was not allocated by the Rust
                        //    global allocator (it is a WGPU mapped buffer slice), and
                        //    reconstituting a `Vec` from it would cause a double-free /
                        //    mismatched-allocator UB on drop.
                        unsafe {
                            let slice = core::slice::from_raw_parts(ptr.cast::<T>(), len);
                            return slice.to_vec();
                        }
                    }
                    Err(_e) => {
                        // GPU execution failed, fallback to CPU
                    }
                }
            }
        }

        // Fallback to CPU
        use super::CpuScalar;
        CpuScalar.collect(node)
    }

    fn reduce<T, F>(&self, node: &dyn Nodus<Item = T>, f: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T, T) -> T + Send + Sync,
    {
        // GPU reduce requires the operation to be known at compile time (WGSL).
        // Since Universalis F is opaque, we must fallback to CPU.
        // Use reduce_gpu() if you want GPU reduction with a known operation.
        use super::CpuScalar;
        CpuScalar.reduce(node, f)
    }

    fn reduce_gpu<T, F>(&self, node: &dyn Nodus<Item = T>, wgsl_op: &str, fallback: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T, T) -> T + Send + Sync,
    {
        // Fused map+reduce (redomap-style): when the node is a GPU-able map
        // chain, the mapped result stays in a device buffer and feeds the
        // reduce kernel directly — no host readback between the stages
        // (cf. Futhark's redomap, Henriksen et al. 2016).

        // Check length threshold
        if node.len() < self.min_len() {
            return super::reduce_scalar(node, &fallback);
        }

        // Try to get GPU map chain
        if let Some(chain) = node.try_gpu_map_chain() {
            // Try to get source data as bytes + type info
            if let Some((source_bytes, type_name)) = node.try_as_gpu_source() {
                // We need T to match the GPU scalar type
                // Since try_as_gpu_source() comes from a typed context, if T is the Item type,
                // it should match type_name.

                let element_size = core::mem::size_of::<T>();

                // Lock kernel cache
                let Ok(mut cache) = self.kernel_cache().lock() else {
                    return super::reduce_scalar(node, &fallback);
                };

                // 1. Execute Map Chain -> Output Buffer
                let map_result = execute::execute_map_to_buffer(
                    self.device(),
                    self.queue(),
                    &mut cache,
                    source_bytes,
                    &chain.compose_wgsl(),
                    type_name,
                    element_size,
                );

                if let Ok((buffer, _size_bytes)) = map_result {
                    // 2. Execute Reduce on the buffer
                    // Note: T is not bounded by Pod here, but execute_reduce_from_buffer handles that via unsafe reading.
                    // Safety: try_as_gpu_source() guarantees T corresponds to the WGSL type (GpuScalar/Pod).

                    // Safety: We restrict T to known GpuScalar types (f32, i32, u32) to prevent
                    // undefined behavior when reading arbitrary bytes as T in execute_reduce_from_buffer.
                    // execute_reduce_from_buffer uses unsafe read_unaligned which requires T to be valid for the bytes.
                    if TypeId::of::<T>() != TypeId::of::<f32>()
                        && TypeId::of::<T>() != TypeId::of::<i32>()
                        && TypeId::of::<T>() != TypeId::of::<u32>()
                    {
                        return super::reduce_scalar(node, &fallback);
                    }

                    // SAFETY: We verified T is one of f32, i32, u32 above. These types are Pod and
                    // correspond to the expected GpuScalar types.
                    let result: GpuResult<Option<T>> = unsafe {
                        execute::execute_reduce_from_buffer::<T>(
                            self.device(),
                            self.queue(),
                            &mut cache,
                            buffer,
                            node.len(), // element count
                            wgsl_op,
                            type_name,
                        )
                    };

                    if let Ok(res) = result {
                        return res;
                    } /* Fallback */
                } else { /* Fallback */
                }
            }
        }

        super::reduce_scalar(node, &fallback)
    }

    fn fold<T, B, F>(&self, node: &dyn Nodus<Item = T>, init: B, f: F) -> B
    where
        T: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        F: Fn(B, T) -> B + Send + Sync,
    {
        // Fold is inherently sequential, use CPU
        use super::CpuScalar;
        CpuScalar.fold(node, init, f)
    }

    fn for_each<T, F>(&self, node: &dyn Nodus<Item = T>, f: F)
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) + Send + Sync,
    {
        use super::CpuScalar;
        CpuScalar.for_each(node, f);
    }

    fn any<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        use super::CpuScalar;
        CpuScalar.any(node, predicate)
    }

    fn all<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        use super::CpuScalar;
        CpuScalar.all(node, predicate)
    }

    fn find<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        use super::CpuScalar;
        CpuScalar.find(node, predicate)
    }

    fn count<T>(&self, node: &dyn Nodus<Item = T>) -> usize
    where
        T: Clone + Send + Sync + 'static,
    {
        // Count is O(1) for indexed nodes, use that
        if node.is_indexed() {
            return node.len();
        }
        use super::CpuScalar;
        CpuScalar.count(node)
    }
}
