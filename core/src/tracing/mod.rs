//! Effect Tracing - Causal Observability
//!
//! > *"Vestigia effectuum lux veritatis"*
//! > — The traces of effects are the light of truth. (Neo-Latin)
//!
//! This module provides causal effect tracing for full observability
//! into effect execution.
//!
//! # Overview
//!
//! Effect tracing captures the execution of effects with causal relationships,
//! enabling debugging, profiling, and distributed tracing integration.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Trace | Vestigium | *vestigium* = footprint, trace |
//! | Event | Eventus | *eventus* = event, occurrence |
//! | Span | Spatium | *spatium* = space, span |
//! | Collector | Collector | *collector* = gatherer |
//! | Context | Contextus | *contextus* = connection, context |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::tracing::{TractatorVestigians, CollectorMemoriae};
//! use std::sync::Arc;
//!
//! // CollectorMemoriae is an in-memory CollectorVestigium implementation
//! let collector = Arc::new(CollectorMemoriae::new(1024));
//! let handler: TractatorVestigians<(), ()> =
//!     TractatorVestigians::new((), collector.clone(), 1, "State");
//!
//! handler.record_start("operation");
//! assert_eq!(collector.event_count(), 1);
//! ```

mod collector;
mod context;
mod event;
mod handler;

pub use collector::*;
pub use context::*;
pub use event::*;
pub use handler::*;

// =============================================================================
// Trace ID
// =============================================================================

/// Unique identifier for a trace.
///
/// # Latin Etymology
/// *Vestigium* + *Id* = trace identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VestigiumId(u64);

impl VestigiumId {
    /// Create a new trace ID from a raw value.
    #[inline]
    pub fn new(id: u64) -> Self {
        VestigiumId(id)
    }

    /// Generate a new unique trace ID.
    #[inline]
    pub fn generate() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        VestigiumId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value.
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for VestigiumId {
    fn default() -> Self {
        Self::generate()
    }
}

impl core::fmt::Display for VestigiumId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

// =============================================================================
// Span ID
// =============================================================================

/// Unique identifier for a span within a trace.
///
/// # Latin Etymology
/// *Spatium* + *Id* = span identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpatiumId(u64);

impl SpatiumId {
    /// Create a new span ID from a raw value.
    #[inline]
    pub fn new(id: u64) -> Self {
        SpatiumId(id)
    }

    /// Generate a new unique span ID.
    #[inline]
    pub fn generate() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        SpatiumId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value.
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for SpatiumId {
    fn default() -> Self {
        Self::generate()
    }
}

impl core::fmt::Display for SpatiumId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

// =============================================================================
// Trace Level
// =============================================================================

/// Severity level for trace events.
///
/// # Latin Etymology
/// *Gradus* = step, level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Gradus {
    /// Detailed trace information.
    Vestigium,
    /// Debug information.
    Debug,
    /// Informational message.
    #[default]
    Info,
    /// Warning.
    Monitum,
    /// Error.
    Error,
}

impl Gradus {
    /// Get the level name.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Gradus::Vestigium => "TRACE",
            Gradus::Debug => "DEBUG",
            Gradus::Info => "INFO",
            Gradus::Monitum => "WARN",
            Gradus::Error => "ERROR",
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vestigium_id_unique() {
        let id1 = VestigiumId::generate();
        let id2 = VestigiumId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_vestigium_id_display() {
        let id = VestigiumId::new(255);
        let display = alloc::format!("{id}");
        assert_eq!(display, "00000000000000ff");
    }

    #[test]
    fn test_spatium_id_unique() {
        let id1 = SpatiumId::generate();
        let id2 = SpatiumId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_gradus_ordering() {
        assert!(Gradus::Vestigium < Gradus::Debug);
        assert!(Gradus::Debug < Gradus::Info);
        assert!(Gradus::Info < Gradus::Monitum);
        assert!(Gradus::Monitum < Gradus::Error);
    }
}
