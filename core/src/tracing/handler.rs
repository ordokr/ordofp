//! Tracing Handler Wrapper
//!
//! > *"Tractator vestigians omnia videt"*
//! > — The tracing handler sees all. (Neo-Latin)
//!
//! This module provides a handler wrapper that traces effect operations.

use alloc::sync::Arc;
use core::marker::PhantomData;

use super::{CollectorVestigium, EventusKind, EventusVestigium, Gradus, SpatiumId, VestigiumId};

// =============================================================================
// Tracing Configuration
// =============================================================================

/// Configuration for the tracing handler.
// Flat config struct: each bool is an independent feature toggle, not a state
// machine in disguise, so the bools-to-enum refactor the lint suggests would
// only add indirection.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct ConfigVestigium {
    /// Whether to trace operation starts.
    pub trace_starts: bool,
    /// Whether to trace operation ends.
    pub trace_ends: bool,
    /// Whether to trace errors.
    pub trace_errors: bool,
    /// Whether to include operation arguments in traces.
    ///
    /// Note: currently **decorative** — no `record_*` method consults this
    /// flag yet; arguments are never captured regardless of the setting.
    pub include_args: bool,
    /// Whether to include operation results in traces.
    ///
    /// Note: currently **decorative** — no `record_*` method consults this
    /// flag yet; results are never captured regardless of the setting.
    pub include_results: bool,
    /// Default trace level.
    pub default_level: Gradus,
}

impl ConfigVestigium {
    /// Create a minimal configuration (only errors).
    pub fn minimal() -> Self {
        ConfigVestigium {
            trace_starts: false,
            trace_ends: false,
            trace_errors: true,
            include_args: false,
            include_results: false,
            default_level: Gradus::Error,
        }
    }

    /// Create a standard configuration.
    pub fn standard() -> Self {
        ConfigVestigium {
            trace_starts: true,
            trace_ends: true,
            trace_errors: true,
            include_args: false,
            include_results: false,
            default_level: Gradus::Info,
        }
    }

    /// Create a verbose configuration.
    pub fn verbose() -> Self {
        ConfigVestigium {
            trace_starts: true,
            trace_ends: true,
            trace_errors: true,
            include_args: true,
            include_results: true,
            default_level: Gradus::Debug,
        }
    }

    /// Create a full trace configuration.
    pub fn full() -> Self {
        ConfigVestigium {
            trace_starts: true,
            trace_ends: true,
            trace_errors: true,
            include_args: true,
            include_results: true,
            default_level: Gradus::Vestigium,
        }
    }
}

impl Default for ConfigVestigium {
    fn default() -> Self {
        Self::standard()
    }
}

// =============================================================================
// Tracing Handler
// =============================================================================

/// A handler wrapper that traces effect operations.
///
/// Wraps an inner handler and records trace events for each operation.
///
/// # Latin Etymology
/// *Tractator vestigians* = tracing handler.
///
/// # Type Parameters
///
/// - `E`: The effect type
/// - `H`: The inner handler type
pub struct TractatorVestigians<E, H> {
    /// Inner handler.
    inner: H,
    /// Trace collector.
    collector: Arc<dyn CollectorVestigium>,
    /// Current trace ID.
    trace_id: VestigiumId,
    /// Current span ID.
    span_id: SpatiumId,
    /// Parent span ID.
    parent_span_id: Option<SpatiumId>,
    /// Configuration.
    config: ConfigVestigium,
    /// Effect ID.
    effect_id: u64,
    /// Effect name.
    effect_name: &'static str,
    /// Marker for effect type.
    _effect: PhantomData<E>,
}

impl<E, H> TractatorVestigians<E, H> {
    /// Create a new tracing handler.
    pub fn new(
        inner: H,
        collector: Arc<dyn CollectorVestigium>,
        effect_id: u64,
        effect_name: &'static str,
    ) -> Self {
        TractatorVestigians {
            inner,
            collector,
            trace_id: VestigiumId::generate(),
            span_id: SpatiumId::generate(),
            parent_span_id: None,
            config: ConfigVestigium::default(),
            effect_id,
            effect_name,
            _effect: PhantomData,
        }
    }

    /// Set the configuration.
    #[inline]
    pub fn with_config(mut self, config: ConfigVestigium) -> Self {
        self.config = config;
        self
    }

    /// Set the trace ID.
    #[inline]
    pub fn with_trace_id(mut self, trace_id: VestigiumId) -> Self {
        self.trace_id = trace_id;
        self
    }

    /// Set the parent span ID.
    #[inline]
    pub fn with_parent_span(mut self, parent: SpatiumId) -> Self {
        self.parent_span_id = Some(parent);
        self
    }

    /// Get the inner handler.
    #[inline]
    pub fn inner(&self) -> &H {
        &self.inner
    }

    /// Get the inner handler mutably.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut H {
        &mut self.inner
    }

    /// Get the trace ID.
    #[inline]
    pub fn trace_id(&self) -> VestigiumId {
        self.trace_id
    }

    /// Get the current span ID.
    #[inline]
    pub fn span_id(&self) -> SpatiumId {
        self.span_id
    }

    /// Record a start event.
    #[inline]
    pub fn record_start(&self, operation: &str) {
        if !self.config.trace_starts {
            return;
        }

        let mut event = EventusVestigium::new(
            self.trace_id,
            self.span_id,
            self.effect_id,
            self.effect_name,
            operation,
        )
        .with_kind(EventusKind::OperationStart)
        .with_level(self.config.default_level);

        if let Some(parent) = self.parent_span_id {
            event = event.with_parent(parent);
        }

        self.collector.record(event);
    }

    /// Record an end event.
    #[inline]
    pub fn record_end(&self, operation: &str, duration_ns: u64) {
        if !self.config.trace_ends {
            return;
        }

        let mut event = EventusVestigium::new(
            self.trace_id,
            self.span_id,
            self.effect_id,
            self.effect_name,
            operation,
        )
        .with_kind(EventusKind::OperationEnd)
        .with_level(self.config.default_level)
        .with_duration(duration_ns);

        if let Some(parent) = self.parent_span_id {
            event = event.with_parent(parent);
        }

        self.collector.record(event);
    }

    /// Record an error event.
    #[inline]
    pub fn record_error(&self, operation: &str, error: &str) {
        if !self.config.trace_errors {
            return;
        }

        let mut event = EventusVestigium::new(
            self.trace_id,
            self.span_id,
            self.effect_id,
            self.effect_name,
            operation,
        )
        .with_kind(EventusKind::OperationError)
        .with_level(Gradus::Error)
        .with_attribute("error", error);

        if let Some(parent) = self.parent_span_id {
            event = event.with_parent(parent);
        }

        self.collector.record(event);
    }

    /// Create a child span.
    #[inline]
    pub fn child_span(&self) -> SpatiumId {
        SpatiumId::generate()
    }
}

impl<E, H: Clone> Clone for TractatorVestigians<E, H> {
    fn clone(&self) -> Self {
        TractatorVestigians {
            inner: self.inner.clone(),
            collector: self.collector.clone(),
            trace_id: self.trace_id,
            span_id: SpatiumId::generate(),     // New span for clone
            parent_span_id: Some(self.span_id), // Parent is original
            config: self.config.clone(),
            effect_id: self.effect_id,
            effect_name: self.effect_name,
            _effect: PhantomData,
        }
    }
}

// =============================================================================
// Span Guard
// =============================================================================

/// Parameters for creating a span guard.
pub struct SpatiumParams<'a> {
    /// Trace collector.
    pub collector: &'a dyn CollectorVestigium,
    /// Trace ID.
    pub trace_id: VestigiumId,
    /// Span ID.
    pub span_id: SpatiumId,
    /// Parent span ID.
    pub parent_span_id: Option<SpatiumId>,
    /// Effect ID.
    pub effect_id: u64,
    /// Effect name.
    pub effect_name: &'static str,
    /// Operation name.
    pub operation: &'a str,
    /// Start time in nanoseconds.
    pub start_time_ns: u64,
}

/// RAII guard for tracing a span.
///
/// Records start on creation and end on drop.
pub struct SpatiumGuard<'a> {
    collector: &'a dyn CollectorVestigium,
    trace_id: VestigiumId,
    span_id: SpatiumId,
    parent_span_id: Option<SpatiumId>,
    effect_id: u64,
    effect_name: &'static str,
    operation: &'a str,
    start_time_ns: u64,
    record_end: bool,
}

impl<'a> SpatiumGuard<'a> {
    /// Create a new span guard.
    // The params bundle is deliberately taken by value: it is built inline at
    // call sites and fully unpacked here.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(params: SpatiumParams<'a>) -> Self {
        let SpatiumParams {
            collector,
            trace_id,
            span_id,
            parent_span_id,
            effect_id,
            effect_name,
            operation,
            start_time_ns,
        } = params;

        // Record start event
        let mut event = EventusVestigium::new(trace_id, span_id, effect_id, effect_name, operation)
            .with_kind(EventusKind::OperationStart)
            .with_timestamp(start_time_ns);

        if let Some(parent) = parent_span_id {
            event = event.with_parent(parent);
        }

        collector.record(event);

        SpatiumGuard {
            collector,
            trace_id,
            span_id,
            parent_span_id,
            effect_id,
            effect_name,
            operation,
            start_time_ns,
            record_end: true,
        }
    }

    /// Mark the span as successful (disables automatic end recording).
    pub fn success(mut self, end_time_ns: u64) {
        self.record_end = false;

        let duration = end_time_ns.saturating_sub(self.start_time_ns);
        let mut event = EventusVestigium::new(
            self.trace_id,
            self.span_id,
            self.effect_id,
            self.effect_name,
            self.operation,
        )
        .with_kind(EventusKind::OperationEnd)
        .with_timestamp(end_time_ns)
        .with_duration(duration);

        if let Some(parent) = self.parent_span_id {
            event = event.with_parent(parent);
        }

        self.collector.record(event);
    }

    /// Mark the span as failed.
    pub fn error(mut self, error: &str, end_time_ns: u64) {
        self.record_end = false;

        let duration = end_time_ns.saturating_sub(self.start_time_ns);
        let mut event = EventusVestigium::new(
            self.trace_id,
            self.span_id,
            self.effect_id,
            self.effect_name,
            self.operation,
        )
        .with_kind(EventusKind::OperationError)
        .with_level(Gradus::Error)
        .with_timestamp(end_time_ns)
        .with_duration(duration)
        .with_attribute("error", error);

        if let Some(parent) = self.parent_span_id {
            event = event.with_parent(parent);
        }

        self.collector.record(event);
    }
}

impl Drop for SpatiumGuard<'_> {
    fn drop(&mut self) {
        if self.record_end {
            // Record end event with unknown duration (not explicitly closed)
            let mut event = EventusVestigium::new(
                self.trace_id,
                self.span_id,
                self.effect_id,
                self.effect_name,
                self.operation,
            )
            .with_kind(EventusKind::OperationEnd);

            if let Some(parent) = self.parent_span_id {
                event = event.with_parent(parent);
            }

            self.collector.record(event);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::CollectorMemoriae;

    struct DummyHandler;

    #[test]
    fn test_tracing_handler_creation() {
        let collector = Arc::new(CollectorMemoriae::new(100));
        let handler: TractatorVestigians<(), DummyHandler> =
            TractatorVestigians::new(DummyHandler, collector, 1, "TestEffect");

        assert_eq!(handler.effect_name, "TestEffect");
    }

    #[test]
    fn test_tracing_handler_record_start() {
        let collector = Arc::new(CollectorMemoriae::new(100));
        let handler: TractatorVestigians<(), DummyHandler> =
            TractatorVestigians::new(DummyHandler, collector.clone(), 1, "TestEffect");

        handler.record_start("operation");

        assert_eq!(collector.event_count(), 1);
    }

    #[test]
    fn test_tracing_handler_record_error() {
        let collector = Arc::new(CollectorMemoriae::new(100));
        let handler: TractatorVestigians<(), DummyHandler> =
            TractatorVestigians::new(DummyHandler, collector.clone(), 1, "TestEffect");

        handler.record_error("operation", "something went wrong");

        let events = collector.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind(), EventusKind::OperationError);
    }

    #[test]
    fn test_config_levels() {
        let minimal = ConfigVestigium::minimal();
        assert!(!minimal.trace_starts);
        assert!(minimal.trace_errors);

        let verbose = ConfigVestigium::verbose();
        assert!(verbose.include_args);
        assert!(verbose.include_results);
    }
}
