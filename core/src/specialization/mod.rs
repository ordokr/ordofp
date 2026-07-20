//! Compile-Time Specialization Hints
//!
//! > *"Specialis notitia celeritatem affert."*
//! > — Specialized knowledge brings speed. (Optimization maxim)
//!
//! This module provides compile-time hints and markers for specialization,
//! helping the compiler generate optimal code for effect handlers.
//!
//! # Overview
//!
//! Rust's monomorphization creates specialized versions of generic code,
//! but sometimes the compiler needs hints to optimize effectively. This
//! module provides:
//!
//! - **Specialization markers** - Guide monomorphization decisions
//! - **Inline hints** - Control function inlining for fusion
//! - **Size hints** - Help the compiler with layout optimizations
//! - **Branch hints** - Guide branch prediction
//!
//! # Usage
//!
//! ```rust
//! use ordofp_core::specialization::{likely, cold_path};
//!
//! fn handle_error(x: i32) -> i32 {
//!     -x
//! }
//!
//! fn process(x: i32) -> i32 {
//!     if likely(x > 0) {
//!         // Hot path - optimized for this case
//!         x * 2
//!     } else {
//!         // Cold path
//!         cold_path(|| handle_error(x))
//!     }
//! }
//!
//! assert_eq!(process(5), 10);
//! assert_eq!(process(-5), 5);
//! ```

mod hints;
mod layout;
mod markers;

pub use hints::*;
pub use layout::*;
pub use markers::*;
