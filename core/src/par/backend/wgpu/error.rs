//! Error types for wgpu backend.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

use alloc::string::String;

/// Error type for GPU operations.
#[derive(Debug)]
pub enum GpuError {
    /// wgpu device creation failed.
    DeviceCreationFailed(String),
    /// Adapter request failed.
    AdapterNotFound,
    /// Compute shaders not supported.
    ComputeNotSupported,
    /// Buffer creation failed.
    BufferCreationFailed(String),
    /// Shader compilation failed.
    ShaderCompilationFailed(String),
    /// Buffer mapping failed.
    BufferMappingFailed(String),
}

impl core::fmt::Display for GpuError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GpuError::DeviceCreationFailed(msg) => {
                write!(f, "Failed to create wgpu device: {msg}")
            }
            GpuError::AdapterNotFound => {
                write!(f, "No suitable GPU adapter found")
            }
            GpuError::ComputeNotSupported => {
                write!(f, "GPU adapter does not support compute shaders")
            }
            GpuError::BufferCreationFailed(msg) => {
                write!(f, "Failed to create buffer: {msg}")
            }
            GpuError::ShaderCompilationFailed(msg) => {
                write!(f, "Shader compilation failed: {msg}")
            }
            GpuError::BufferMappingFailed(msg) => {
                write!(f, "Buffer mapping failed: {msg}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GpuError {}

/// Result type for GPU operations.
pub type GpuResult<T> = Result<T, GpuError>;
