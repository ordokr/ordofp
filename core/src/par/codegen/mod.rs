//! WGSL code generation for `ParFlumen` GPU execution.
//!
//! > *"Codex Generandi"*
//! > — Code generator. (Neo-Latin)
//!
//! This module generates WGSL compute shaders from `ParFlumen` `Nodus` IR nodes.
//!
//! Adapted third-party patterns are inventoried in
//! `core/src/par/backend/wgpu/ORIGINAL_SOURCE.md` and the repo-root
//! `THIRD_PARTY_NOTICES.md`.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::par::codegen::wgsl::generate_map_shader;
//!
//! // Generate WGSL for: map(|x| x * 2.0) over f32 buffers,
//! // with workgroup size 64
//! let shader = generate_map_shader("multiply", "x * 2.0", 64, "f32");
//! assert!(shader.contains("x * 2.0"));
//! assert!(shader.contains("array<f32>"));
//! ```

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

pub mod wgsl;
