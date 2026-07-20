//! GPU kernel execution helpers.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use wgpu::{Buffer, Device, Queue};

use super::buffer::{create_download_buffer, download_buffer};
use super::error::{GpuError, GpuResult};
use super::kernel::KernelCache;
use crate::par::codegen::wgsl::{
    KernelKey, OperatioReductionis, generate_map_shader, generate_reduce_shader,
};

/// Default GPU workgroup size (threads per workgroup).
pub(crate) const WORKGROUP_SIZE: usize = 64;

/// Create the 4-byte params uniform holding the logical element count.
/// Not pooled: pooled buffers carry STORAGE usage, uniforms need UNIFORM.
#[inline]
fn create_params_buffer(device: &Device) -> Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("parflumen_params"),
        size: 4,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// Checked usize -> u32 for dispatch/element counts.
#[inline]
fn to_u32(n: usize, what: &str) -> GpuResult<u32> {
    u32::try_from(n).map_err(|_| {
        GpuError::BufferCreationFailed(format!(
            "{what} {n} exceeds u32::MAX — GPU path does not support this size"
        ))
    })
}

/// Execute a map operation on the GPU using raw bytes.
///
/// # Arguments
///
/// * `input_bytes` - Input data as bytes
/// * `operation` - WGSL expression
/// * `type_name` - WGSL type name (e.g., "f32")
/// * `element_size` - Size of one element in bytes
#[inline]
pub(crate) fn execute_map_gpu_bytes(
    device: &Device,
    queue: &Queue,
    cache: &mut KernelCache,
    input_bytes: &[u8],
    operation: &str,
    type_name: &str,
    element_size: usize,
) -> GpuResult<Vec<u8>> {
    if input_bytes.is_empty() {
        return Ok(Vec::new());
    }

    let (output_buffer, output_size_bytes) = execute_map_to_buffer(
        device,
        queue,
        cache,
        input_bytes,
        operation,
        type_name,
        element_size,
    )?;

    let download_buf = create_download_buffer(device, output_size_bytes);

    // Copy output to download buffer
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &download_buf,
        0,
        output_size_bytes as u64,
    );

    // Submit
    let command_buffer = encoder.finish();
    queue.submit([command_buffer]);

    // Download results
    let result = download_buffer(
        device,
        queue,
        &output_buffer,
        &download_buf,
        output_size_bytes,
    );

    // Release output buffer to pool
    cache.release_buffer(output_buffer);

    result
}

/// Execute a map operation and return the GPU buffer (no download).
#[inline]
pub(crate) fn execute_map_to_buffer(
    device: &Device,
    queue: &Queue,
    cache: &mut KernelCache,
    input_bytes: &[u8],
    operation: &str,
    type_name: &str,
    element_size: usize,
) -> GpuResult<(Buffer, usize)> {
    if input_bytes.is_empty() {
        // Return dummy empty buffer?
        let buffer = cache.acquire_buffer(4); // Min size
        return Ok((buffer, 0));
    }

    let element_count = input_bytes.len() / element_size;
    let element_count_u32 = to_u32(element_count, "element count")?;
    let params_buffer = create_params_buffer(device);
    queue.write_buffer(&params_buffer, 0, &element_count_u32.to_le_bytes());

    // Generate shader
    let workgroup_size_u32 = to_u32(WORKGROUP_SIZE, "workgroup size")?;
    let shader_source = generate_map_shader("map", operation, workgroup_size_u32, type_name);
    let kernel_key = KernelKey::new(shader_source, "map".to_string());

    // Create buffers
    // Use buffer pool + write_buffer
    let input_buffer = cache.acquire_buffer(input_bytes.len());
    queue.write_buffer(&input_buffer, 0, input_bytes);

    // Acquire output buffer from pool
    let output_buffer = cache.acquire_buffer(input_bytes.len());

    // Get or compile pipeline
    // release both pooled buffers on compile failure.
    let cached = match cache.get_or_compile(&kernel_key) {
        Ok(c) => c,
        Err(e) => {
            cache.release_buffer(input_buffer);
            cache.release_buffer(output_buffer);
            return Err(e);
        }
    };

    // Create bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &cached.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buffer.as_entire_binding(),
            },
        ],
    });

    // Create command encoder
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // Begin compute pass
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&cached.pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        // Dispatch workgroups
        // release both pooled buffers if the count overflows u32.
        let workgroup_count =
            match to_u32(element_count.div_ceil(WORKGROUP_SIZE), "workgroup count") {
                Ok(v) => v,
                Err(e) => {
                    cache.release_buffer(input_buffer);
                    cache.release_buffer(output_buffer);
                    return Err(e);
                }
            };
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    // Submit
    let command_buffer = encoder.finish();
    queue.submit([command_buffer]);

    // Release input buffer
    cache.release_buffer(input_buffer);

    Ok((output_buffer, input_bytes.len()))
}

/// Execute reduction starting from a GPU buffer.
///
/// This assumes `input_buffer` contains `element_count` elements of type T.
/// It performs multi-pass reduction until a single element remains.
///
/// # Safety
///
/// This function performs an unsafe read of bytes into `T`.
/// The caller must ensure that `T` is a type that can be safely created from
/// the bytes returned by the GPU (e.g., `T` is `Pod` and the bytes represent a valid instance).
// Linear GPU dispatch sequence (encode, dispatch, readback); splitting it
// would separate the unsafe readback from the setup it depends on.
#[allow(clippy::too_many_lines)]
pub(crate) unsafe fn execute_reduce_from_buffer<T>(
    device: &Device,
    queue: &Queue,
    cache: &mut KernelCache,
    input_buffer: Buffer,
    element_count: usize,
    operation: &str,
    type_name: &str,
) -> GpuResult<Option<T>>
where
    T: Clone + Send + Sync + 'static,
{
    if element_count == 0 {
        // Ensure we release input buffer even in early return
        cache.release_buffer(input_buffer);
        return Ok(None);
    }

    let element_size = core::mem::size_of::<T>();

    if element_count == 1 {
        // Single element: the reduction is the element itself; the pass loop
        // below would never execute and scratch would be pool garbage.
        // release input_buffer on the fallible download below —
        // it is not needed past this point regardless of outcome.
        let download_buf = create_download_buffer(device, element_size);
        let result_bytes =
            match download_buffer(device, queue, &input_buffer, &download_buf, element_size) {
                Ok(b) => b,
                Err(e) => {
                    cache.release_buffer(input_buffer);
                    return Err(e);
                }
            };
        cache.release_buffer(input_buffer);
        if result_bytes.len() != core::mem::size_of::<T>() {
            return Ok(None);
        }
        // SAFETY: identical contract to the final read below — caller
        // guarantees the buffer holds a valid bit-pattern for `T`, and the
        // length guard above confirms exactly size_of::<T>() bytes.
        let t = unsafe { core::ptr::read_unaligned(result_bytes.as_ptr().cast::<T>()) };
        return Ok(Some(t));
    }

    let workgroup_size = WORKGROUP_SIZE;

    // Pre-compile shader (same for all passes). Done before acquiring scratch
    // buffers so a parse/compile failure only has to release `input_buffer`
    // (the scratch buffers aren't held yet).
    let op = match OperatioReductionis::parse(operation).ok_or_else(|| {
        GpuError::ShaderCompilationFailed(format!(
            "unsupported reduce operator `{operation}` (supported: + * min max)"
        ))
    }) {
        Ok(op) => op,
        Err(e) => {
            cache.release_buffer(input_buffer);
            return Err(e);
        }
    };
    let workgroup_size_u32 = match to_u32(workgroup_size, "workgroup size") {
        Ok(v) => v,
        Err(e) => {
            cache.release_buffer(input_buffer);
            return Err(e);
        }
    };
    let shader_source = generate_reduce_shader("reduce", op, workgroup_size_u32, type_name);
    let kernel_key = KernelKey::new(shader_source, "reduce".to_string());
    // Ensure compiled
    if let Err(e) = cache.get_or_compile(&kernel_key) {
        cache.release_buffer(input_buffer);
        return Err(e);
    }

    // Ping-pong buffering with pooled buffers
    // Max size needed is input_len / WORKGROUP_SIZE
    let max_output_count = element_count.div_ceil(workgroup_size);
    let scratch_size_bytes = max_output_count * element_size;

    // Acquire scratch buffers from pool
    let mut scratch_a = cache.acquire_buffer(scratch_size_bytes);
    let mut scratch_b = cache.acquire_buffer(scratch_size_bytes);

    // Initial setup: Input -> Scratch A
    // We treat the first pass specially to read from input_buffer

    let params_buffer = create_params_buffer(device);

    let mut current_count = element_count;
    let mut first_pass = true;

    // References to buffers for swapping logic
    // current_input will point to input_buffer (initially) or one of the scratches
    // next_output will point to scratch_a (initially) or scratch_b

    // We manage this loop manually
    while current_count > 1 {
        // release all three pooled buffers held across the
        // loop body (input_buffer, scratch_a, scratch_b) on any fallible early
        // return within this iteration.
        let current_count_u32 = match to_u32(current_count, "element count") {
            Ok(v) => v,
            Err(e) => {
                cache.release_buffer(input_buffer);
                cache.release_buffer(scratch_a);
                cache.release_buffer(scratch_b);
                return Err(e);
            }
        };
        queue.write_buffer(&params_buffer, 0, &current_count_u32.to_le_bytes());
        let workgroup_count = current_count.div_ceil(workgroup_size);

        // Determine input and output buffers for this pass
        let (input_res, output_res) = if first_pass {
            (
                input_buffer.as_entire_binding(),
                scratch_a.as_entire_binding(),
            )
        } else {
            // Ping-pong between scratch A and B
            // If we just wrote to A (first_pass was true previously? No, logic below)
            // Let's rely on swapping logic after the pass.
            // We use scratch_a as input, scratch_b as output.
            (scratch_a.as_entire_binding(), scratch_b.as_entire_binding())
        };

        // Get pipeline (cached)
        let cached = match cache.get_or_compile(&kernel_key) {
            Ok(c) => c,
            Err(e) => {
                cache.release_buffer(input_buffer);
                cache.release_buffer(scratch_a);
                cache.release_buffer(scratch_b);
                return Err(e);
            }
        };

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &cached.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_res,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_res,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // Encode dispatch
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&cached.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_count_u32 = match to_u32(workgroup_count, "workgroup count") {
                Ok(v) => v,
                Err(e) => {
                    cache.release_buffer(input_buffer);
                    cache.release_buffer(scratch_a);
                    cache.release_buffer(scratch_b);
                    return Err(e);
                }
            };
            compute_pass.dispatch_workgroups(workgroup_count_u32, 1, 1);
        }

        queue.submit([encoder.finish()]);

        // Update loop state
        current_count = workgroup_count;

        if first_pass {
            first_pass = false;
            // Result is in scratch_a.
            // Next pass: Input=scratch_a, Output=scratch_b.
            // Loop condition `else` branch handles (scratch_a, scratch_b).
            // So we don't swap yet.
        } else {
            // Result is in scratch_b.
            // Next pass: Input=scratch_b, Output=scratch_a.
            // We need to swap the buffers so that scratch_a becomes the one with valid data (input for next pass).
            core::mem::swap(&mut scratch_a, &mut scratch_b);
        }
    }

    // Release unused intermediate buffers
    // scratch_b is currently unused (contains previous pass data)
    cache.release_buffer(scratch_b);
    // Release input buffer
    cache.release_buffer(input_buffer);

    // Final result is in scratch_a (because if we just wrote to scratch_b, we swapped it into scratch_a)
    let current_buffer = scratch_a;

    // Now current_buffer has 1 element
    // release current_buffer on the fallible download below —
    // input_buffer and scratch_b were already released above.
    let download_buf = create_download_buffer(device, element_size);
    let result_bytes =
        match download_buffer(device, queue, &current_buffer, &download_buf, element_size) {
            Ok(b) => b,
            Err(e) => {
                cache.release_buffer(current_buffer);
                return Err(e);
            }
        };

    // Release result buffer
    cache.release_buffer(current_buffer);

    if result_bytes.len() != element_size {
        return Ok(None);
    }

    if result_bytes.len() != core::mem::size_of::<T>() {
        return Ok(None);
    }

    // SAFETY: Both length guards above (one against `element_size`, one against
    // `size_of::<T>()`) confirm that `result_bytes` holds exactly `size_of::<T>()`
    // bytes. `read_unaligned` is used because the GPU readback buffer may not satisfy
    // `T`'s alignment requirements. The caller's contract (see the `# Safety` section
    // of `execute_reduce_from_buffer`) requires that the bytes represent a valid,
    // initialized bit-pattern for `T`, which is upheld when the WGSL shader wrote a
    // value of the matching type.
    let t = unsafe { core::ptr::read_unaligned(result_bytes.as_ptr().cast::<T>()) };
    Ok(Some(t))
}
