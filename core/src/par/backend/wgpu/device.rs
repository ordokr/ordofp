//! wgpu device initialization and management.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Mutex;
use wgpu::{Device, Instance, Queue};

use super::error::{GpuError, GpuResult};
use super::kernel::KernelCache;

/// GPU backend using wgpu/WGSL.
///
/// This backend executes `ParFlumen` pipelines on the GPU by emitting WGSL
/// compute shaders and dispatching via wgpu.
///
/// # Configuration
///
/// - `min_len`: Minimum array length to use GPU (default: 1024)
///   Arrays smaller than this will fallback to CPU execution.
///
/// # Example
///
/// This requires a physical GPU adapter, so it cannot run in a headless
/// doctest environment.
///
/// ```rust,no_run
/// use ordofp_core::par::{ParFlumen, backend::wgpu::GpuWgpu};
///
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let backend = GpuWgpu::new()?;
/// let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
/// let result = ParFlumen::from_vec(data)
///     .map(|x| x * 2.0)
///     .collect_vec(&backend);
/// # let _ = result;
/// # Ok(())
/// # }
/// ```
pub struct GpuWgpu {
    /// wgpu device for creating resources.
    device: Arc<Device>,
    /// wgpu queue for submitting work.
    queue: Arc<Queue>,
    /// Minimum array length to use GPU.
    min_len: usize,
    /// Kernel cache for compiled shaders.
    kernel_cache: Arc<Mutex<KernelCache>>,
}

impl GpuWgpu {
    /// Create a new GPU backend with default settings.
    ///
    /// This will request an adapter and create a device. On systems without
    /// a GPU or when wgpu cannot find a suitable adapter, this will return
    /// an error.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::AdapterNotFound` if wgpu finds no suitable
    /// adapter, `GpuError::ComputeNotSupported` if the adapter lacks
    /// compute-shader support, or `GpuError::DeviceCreationFailed` if the
    /// device/queue request fails.
    #[inline]
    pub fn new() -> GpuResult<Self> {
        Self::with_min_len(1024)
    }

    /// Create a new GPU backend with a custom minimum length threshold.
    ///
    /// Arrays shorter than `min_len` are processed on the CPU instead of
    /// being dispatched to the GPU.
    ///
    /// # Errors
    ///
    /// Returns `GpuError::AdapterNotFound` if wgpu finds no suitable
    /// adapter, `GpuError::ComputeNotSupported` if the adapter lacks
    /// compute-shader support, or `GpuError::DeviceCreationFailed` if the
    /// device/queue request fails.
    pub fn with_min_len(min_len: usize) -> GpuResult<Self> {
        // Initialize wgpu instance.
        // wgpu 29 removed `InstanceDescriptor: Default` (the descriptor now carries
        // an optional boxed display handle); for compute-only use there is no
        // surface, so build the descriptor without a display handle.
        let instance = Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        // Request adapter (this is async in WebGPU, but we block on native)
        // request_adapter returns Result
        let adapter = super::block_on::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions::default()),
        )
        .map_err(|_| GpuError::AdapterNotFound)?;

        // Check compute shader support
        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        if !downlevel_capabilities
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            return Err(GpuError::ComputeNotSupported);
        }

        // Create device and queue
        let (device, queue) =
            super::block_on::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::default(),
            }))
            .map_err(|e| GpuError::DeviceCreationFailed(alloc::format!("{e:?}")))?;

        let device_arc: Arc<Device> = Arc::new(device);
        Ok(Self {
            // Pass the clone to KernelCache so the struct owns the original handle.
            kernel_cache: Arc::new(Mutex::new(KernelCache::new(Arc::clone(&device_arc)))),
            device: device_arc,
            queue: Arc::new(queue),
            min_len,
        })
    }

    /// Get the wgpu device.
    #[inline]
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get the wgpu queue.
    #[inline]
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Get the minimum length threshold.
    #[inline]
    pub fn min_len(&self) -> usize {
        self.min_len
    }

    /// Get the kernel cache (for internal use).
    #[inline]
    pub(crate) fn kernel_cache(&self) -> &Arc<Mutex<KernelCache>> {
        &self.kernel_cache
    }
}

impl Default for GpuWgpu {
    /// # Panics
    ///
    /// Panics if no suitable GPU adapter can be found (e.g. no GPU, missing
    /// drivers, or an unsupported backend). Use [`GpuWgpu::new`] to handle
    /// the failure as a `Result` instead.
    fn default() -> Self {
        Self::new().expect("Failed to create GPU backend. Ensure a GPU is available and wgpu can find a suitable adapter.")
    }
}
