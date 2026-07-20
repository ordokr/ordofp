//! Probatum — accumulating validation (a `Result` that collects *all* errors
//! instead of short-circuiting on the first).
//!
//! The implementation lives in [`ordofp_core::validated`]; this module
//! preserves the long-standing facade paths (`ordofp::validated::Probatum`).

pub use ordofp_core::validated::*;
