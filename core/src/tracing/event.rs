//! Trace Events
//!
//! > *"Eventus est momentum temporis"*
//! > — An event is a moment in time. (Latin)
//!
//! This module defines the event types for effect tracing.

use alloc::string::String;
use alloc::vec::Vec;

use super::{Gradus, SpatiumId, VestigiumId};

// =============================================================================
// Trace Event
// =============================================================================

/// A trace event capturing an effect operation.
///
/// # Latin Etymology
/// *Eventus vestigium* = trace event.
#[derive(Debug, Clone)]
pub struct EventusVestigium {
    /// Unique event ID.
    id: EventusId,

    /// Trace ID (groups related events).
    trace_id: VestigiumId,

    /// Parent span ID (for causal relationship).
    parent_span_id: Option<SpatiumId>,

    /// Current span ID.
    span_id: SpatiumId,

    /// Effect ID (identifies the effect type).
    effect_id: u64,

    /// Effect name.
    effect_name: String,

    /// Operation name.
    operation: String,

    /// Timestamp (nanoseconds since epoch or monotonic).
    timestamp_ns: u64,

    /// Duration of the operation (nanoseconds).
    duration_ns: Option<u64>,

    /// Severity level.
    level: Gradus,

    /// Event kind.
    kind: EventusKind,

    /// Additional attributes.
    attributes: Vec<Attributum>,
}

impl EventusVestigium {
    /// Create a new trace event.
    pub fn new(
        trace_id: VestigiumId,
        span_id: SpatiumId,
        effect_id: u64,
        effect_name: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        EventusVestigium {
            id: EventusId::generate(),
            trace_id,
            parent_span_id: None,
            span_id,
            effect_id,
            effect_name: effect_name.into(),
            operation: operation.into(),
            timestamp_ns: 0,
            duration_ns: None,
            level: Gradus::Info,
            kind: EventusKind::OperationStart,
            attributes: Vec::new(),
        }
    }

    /// Set the parent span ID.
    #[inline]
    pub fn with_parent(mut self, parent: SpatiumId) -> Self {
        self.parent_span_id = Some(parent);
        self
    }

    /// Set the timestamp.
    #[inline]
    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }

    /// Set the duration.
    #[inline]
    pub fn with_duration(mut self, duration_ns: u64) -> Self {
        self.duration_ns = Some(duration_ns);
        self
    }

    /// Set the severity level.
    #[inline]
    pub fn with_level(mut self, level: Gradus) -> Self {
        self.level = level;
        self
    }

    /// Set the event kind.
    #[inline]
    pub fn with_kind(mut self, kind: EventusKind) -> Self {
        self.kind = kind;
        self
    }

    /// Add an attribute.
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<AttributumValue>,
    ) -> Self {
        self.attributes.push(Attributum {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Get the event ID.
    #[inline]
    pub fn id(&self) -> EventusId {
        self.id
    }

    /// Get the trace ID.
    #[inline]
    pub fn trace_id(&self) -> VestigiumId {
        self.trace_id
    }

    /// Get the parent span ID.
    #[inline]
    pub fn parent_span_id(&self) -> Option<SpatiumId> {
        self.parent_span_id
    }

    /// Get the span ID.
    #[inline]
    pub fn span_id(&self) -> SpatiumId {
        self.span_id
    }

    /// Get the effect ID.
    #[inline]
    pub fn effect_id(&self) -> u64 {
        self.effect_id
    }

    /// Get the effect name.
    #[inline]
    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }

    /// Get the operation name.
    #[inline]
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// Get the timestamp.
    #[inline]
    pub fn timestamp_ns(&self) -> u64 {
        self.timestamp_ns
    }

    /// Get the duration.
    #[inline]
    pub fn duration_ns(&self) -> Option<u64> {
        self.duration_ns
    }

    /// Get the severity level.
    #[inline]
    pub fn level(&self) -> Gradus {
        self.level
    }

    /// Get the event kind.
    #[inline]
    pub fn kind(&self) -> EventusKind {
        self.kind
    }

    /// Get the attributes.
    #[inline]
    pub fn attributes(&self) -> &[Attributum] {
        &self.attributes
    }
}

// =============================================================================
// Event ID
// =============================================================================

/// Unique identifier for an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventusId(u64);

impl EventusId {
    /// Create a new event ID.
    #[inline]
    pub fn new(id: u64) -> Self {
        EventusId(id)
    }

    /// Generate a new unique event ID.
    #[inline]
    pub fn generate() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        EventusId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value.
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}

// =============================================================================
// Event Kind
// =============================================================================

/// The kind of trace event.
///
/// # Latin Etymology
/// *Genus eventus* = event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventusKind {
    /// Start of an operation.
    OperationStart,
    /// End of an operation (success).
    OperationEnd,
    /// Operation error.
    OperationError,
    /// Effect performed.
    EffectPerform,
    /// Effect handled.
    EffectHandle,
    /// Effect resumed.
    EffectResume,
    /// Fiber spawned.
    FibraSpawn,
    /// Fiber completed.
    FibraComplete,
    /// Fiber cancelled.
    FibraCancel,
    /// Custom event.
    Custom,
}

impl EventusKind {
    /// Get the kind name.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            EventusKind::OperationStart => "operation.start",
            EventusKind::OperationEnd => "operation.end",
            EventusKind::OperationError => "operation.error",
            EventusKind::EffectPerform => "effect.perform",
            EventusKind::EffectHandle => "effect.handle",
            EventusKind::EffectResume => "effect.resume",
            EventusKind::FibraSpawn => "fibra.spawn",
            EventusKind::FibraComplete => "fibra.complete",
            EventusKind::FibraCancel => "fibra.cancel",
            EventusKind::Custom => "custom",
        }
    }

    /// Check if this is a start event.
    #[inline]
    pub fn is_start(&self) -> bool {
        matches!(self, EventusKind::OperationStart | EventusKind::FibraSpawn)
    }

    /// Check if this is an end event.
    #[inline]
    pub fn is_end(&self) -> bool {
        matches!(self, EventusKind::OperationEnd | EventusKind::FibraComplete)
    }

    /// Check if this is an error event.
    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self, EventusKind::OperationError)
    }
}

// =============================================================================
// Attribute
// =============================================================================

/// An attribute on a trace event.
///
/// # Latin Etymology
/// *Attributum* = attribute.
#[derive(Debug, Clone)]
pub struct Attributum {
    /// Attribute key.
    pub key: String,
    /// Attribute value.
    pub value: AttributumValue,
}

/// Value of an attribute.
#[derive(Debug, Clone)]
pub enum AttributumValue {
    /// String value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Unsigned integer value.
    UInt(u64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
}

impl From<String> for AttributumValue {
    fn from(s: String) -> Self {
        AttributumValue::String(s)
    }
}

impl From<&str> for AttributumValue {
    fn from(s: &str) -> Self {
        AttributumValue::String(s.into())
    }
}

impl From<i64> for AttributumValue {
    fn from(v: i64) -> Self {
        AttributumValue::Int(v)
    }
}

impl From<i32> for AttributumValue {
    fn from(v: i32) -> Self {
        AttributumValue::Int(i64::from(v))
    }
}

impl From<u64> for AttributumValue {
    fn from(v: u64) -> Self {
        AttributumValue::UInt(v)
    }
}

impl From<u32> for AttributumValue {
    fn from(v: u32) -> Self {
        AttributumValue::UInt(u64::from(v))
    }
}

impl From<f64> for AttributumValue {
    fn from(v: f64) -> Self {
        AttributumValue::Float(v)
    }
}

impl From<bool> for AttributumValue {
    fn from(v: bool) -> Self {
        AttributumValue::Bool(v)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = EventusVestigium::new(
            VestigiumId::generate(),
            SpatiumId::generate(),
            1,
            "StateEffect",
            "get",
        );

        assert_eq!(event.effect_name(), "StateEffect");
        assert_eq!(event.operation(), "get");
    }

    #[test]
    fn test_event_with_attributes() {
        let event = EventusVestigium::new(
            VestigiumId::generate(),
            SpatiumId::generate(),
            1,
            "IO",
            "read",
        )
        .with_attribute("path", "/tmp/file.txt")
        .with_attribute("size", 1024u64);

        assert_eq!(event.attributes().len(), 2);
    }

    #[test]
    fn test_eventus_kind() {
        assert!(EventusKind::OperationStart.is_start());
        assert!(EventusKind::OperationEnd.is_end());
        assert!(EventusKind::OperationError.is_error());
    }
}
