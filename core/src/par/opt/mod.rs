//! Optimization passes for `ParFlumen`.
//!
//! > *"Optimus"*
//! > — Best. (Latin)
//!
//! This module provides optimization passes for `ParFlumen` pipelines:
//!
//! - Layout transforms (AoS/SoA, chunking)
//! - Buffer reuse and pooling
//!
//! Adapted third-party patterns are inventoried in `ORIGINAL_SOURCE.md` in
//! this directory and the repo-root `THIRD_PARTY_NOTICES.md`.

#![cfg(all(feature = "par", feature = "gpu-wgpu"))]

#[cfg(feature = "gpu-buffer-pool")]
pub mod buffer_pool;
