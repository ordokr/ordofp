//! Buffer management for GPU operations.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use alloc::vec::Vec;
use wgpu::{Buffer, BufferUsages, Device, Queue};

use super::error::{GpuError, GpuResult};

// Storage/output buffer creation goes through the KernelCache buffer pool
// (`cache.acquire_buffer` + `queue.write_buffer`), not free constructors here.

/// Create a CPU-readable download buffer.
///
/// Infallible today, but keeps the `GpuResult` signature shared by the other
/// buffer helpers — callers already route the error branch to scalar fallback.
#[allow(clippy::unnecessary_wraps)]
#[inline]
pub(crate) fn create_download_buffer(device: &Device, size_bytes: usize) -> Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: size_bytes as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

/// Copy buffer data from GPU to CPU.
pub(crate) fn download_buffer(
    device: &Device,
    queue: &Queue,
    source: &Buffer,
    dest: &Buffer,
    size_bytes: usize,
) -> GpuResult<Vec<u8>> {
    // Create command encoder
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // Copy from source to dest
    encoder.copy_buffer_to_buffer(source, 0, dest, 0, size_bytes as u64);

    // Submit
    let command_buffer = encoder.finish();
    queue.submit([command_buffer]);

    // Map buffer for reading; capture the async result instead of discarding
    // it — on device loss/OOM, get_mapped_range() would panic.
    let (tx, rx) = std::sync::mpsc::channel();
    let buffer_slice = dest.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });

    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|e| GpuError::BufferMappingFailed(alloc::format!("device poll failed: {e:?}")))?;

    match rx.try_recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return Err(GpuError::BufferMappingFailed(alloc::format!(
                "map_async failed: {e}"
            )));
        }
        Err(_) => {
            return Err(GpuError::BufferMappingFailed(
                "map_async callback did not fire after poll".into(),
            ));
        }
    }

    // Read data — pre-allocate the output Vec to avoid a realloc inside to_vec().
    // wgpu 30: get_mapped_range is fallible instead of panicking on bad ranges.
    let data = buffer_slice.get_mapped_range().map_err(|e| {
        GpuError::BufferMappingFailed(alloc::format!("get_mapped_range failed: {e}"))
    })?;
    let mut out = Vec::with_capacity(size_bytes);
    out.extend_from_slice(&data);
    Ok(out)
}
