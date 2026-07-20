//! Trace Context
//!
//! > *"Contextus est nexus omnium"*
//! > — Context is the connection of all things. (Latin)
//!
//! This module provides trace context propagation.

use alloc::vec::Vec;

use super::{SpatiumId, VestigiumId};

// =============================================================================
// Trace Context
// =============================================================================

/// Context for a trace, carrying trace ID and span information.
///
/// # Latin Etymology
/// *Contextus vestigium* = trace context.
#[derive(Debug, Clone, Copy)]
pub struct ContextusVestigium {
    /// Trace ID.
    trace_id: VestigiumId,
    /// Current span ID.
    span_id: SpatiumId,
    /// Parent span ID.
    parent_span_id: Option<SpatiumId>,
    /// Sampling decision.
    sampled: bool,
}

impl ContextusVestigium {
    /// Create a new trace context.
    #[inline]
    pub fn new(trace_id: VestigiumId) -> Self {
        ContextusVestigium {
            trace_id,
            span_id: SpatiumId::generate(),
            parent_span_id: None,
            sampled: true,
        }
    }

    /// Create a root context (new trace).
    #[inline]
    pub fn root() -> Self {
        Self::new(VestigiumId::generate())
    }

    /// Create a child context.
    #[inline]
    pub fn child(&self) -> Self {
        ContextusVestigium {
            trace_id: self.trace_id,
            span_id: SpatiumId::generate(),
            parent_span_id: Some(self.span_id),
            sampled: self.sampled,
        }
    }

    /// Set the sampling decision.
    #[inline]
    pub fn with_sampled(mut self, sampled: bool) -> Self {
        self.sampled = sampled;
        self
    }

    /// Get the trace ID.
    #[inline]
    pub fn trace_id(&self) -> VestigiumId {
        self.trace_id
    }

    /// Get the span ID.
    #[inline]
    pub fn span_id(&self) -> SpatiumId {
        self.span_id
    }

    /// Get the parent span ID.
    #[inline]
    pub fn parent_span_id(&self) -> Option<SpatiumId> {
        self.parent_span_id
    }

    /// Check if this trace is sampled.
    #[inline]
    pub fn is_sampled(&self) -> bool {
        self.sampled
    }

    /// Convert to W3C trace context header format.
    ///
    /// Format: `{version}-{trace_id}-{span_id}-{flags}`
    pub fn to_traceparent(&self) -> [u8; 55] {
        let mut buf = [0u8; 55];
        // Version (00)
        buf[0] = b'0';
        buf[1] = b'0';
        buf[2] = b'-';
        // Trace ID (32 hex chars)
        Self::write_hex_u64(self.trace_id.value(), &mut buf[3..19]);
        Self::write_hex_u64(0, &mut buf[19..35]); // Lower 64 bits (we only have 64-bit IDs)
        buf[35] = b'-';
        // Span ID (16 hex chars)
        Self::write_hex_u64(self.span_id.value(), &mut buf[36..52]);
        buf[52] = b'-';
        // Flags
        buf[53] = b'0';
        buf[54] = if self.sampled { b'1' } else { b'0' };
        buf
    }

    fn write_hex_u64(value: u64, buf: &mut [u8]) {
        const HEX: &[u8] = b"0123456789abcdef";
        for i in 0..16.min(buf.len()) {
            let shift = (15 - i) * 4;
            let nibble = ((value >> shift) & 0xf) as usize;
            buf[i] = HEX[nibble];
        }
    }
}

impl Default for ContextusVestigium {
    fn default() -> Self {
        Self::root()
    }
}

// =============================================================================
// Context Stack (Thread-Local Alternative)
// =============================================================================

/// A stack of trace contexts for propagation.
///
/// Since we're in `no_std`, we can't use thread-local storage directly.
/// This provides a manual stack that can be passed around.
#[derive(Debug, Clone)]
pub struct ContextusStack {
    /// Stack of contexts.
    stack: Vec<ContextusVestigium>,
}

impl ContextusStack {
    /// Create a new empty context stack.
    pub fn new() -> Self {
        ContextusStack {
            stack: Vec::with_capacity(8),
        }
    }

    /// Push a context onto the stack.
    #[inline]
    pub fn push(&mut self, ctx: ContextusVestigium) {
        self.stack.push(ctx);
    }

    /// Pop a context from the stack.
    #[inline]
    pub fn pop(&mut self) -> Option<ContextusVestigium> {
        self.stack.pop()
    }

    /// Get the current context.
    #[inline]
    pub fn current(&self) -> Option<&ContextusVestigium> {
        self.stack.last()
    }

    /// Create a child context and push it.
    #[inline]
    pub fn enter(&mut self) -> ContextusVestigium {
        let ctx = if let Some(parent) = self.current() {
            parent.child()
        } else {
            ContextusVestigium::root()
        };
        self.stack.push(ctx);
        ctx
    }

    /// Pop the current context.
    #[inline]
    pub fn exit(&mut self) {
        self.stack.pop();
    }

    /// Check if the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get the depth of the stack.
    #[inline]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

impl Default for ContextusStack {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Sampler
// =============================================================================

/// Trait for sampling decisions.
///
/// # Latin Etymology
/// *Samplator* = sampler.
pub trait Samplator: Send + Sync {
    /// Decide whether to sample a trace.
    fn should_sample(&self, trace_id: VestigiumId) -> bool;
}

/// Always sample.
pub struct SamplatorSemper;

impl Samplator for SamplatorSemper {
    #[inline]
    fn should_sample(&self, _trace_id: VestigiumId) -> bool {
        true
    }
}

/// Never sample.
pub struct SamplatorNunquam;

impl Samplator for SamplatorNunquam {
    #[inline]
    fn should_sample(&self, _trace_id: VestigiumId) -> bool {
        false
    }
}

/// Probabilistic sampler.
pub struct SamplatorProbabilis {
    /// Sampling rate (0.0 to 1.0).
    rate: f64,
}

impl SamplatorProbabilis {
    /// Create a new probabilistic sampler.
    pub fn new(rate: f64) -> Self {
        SamplatorProbabilis {
            rate: rate.clamp(0.0, 1.0),
        }
    }
}

/// `SplitMix64` finalizer: bit-mixes a value so that sequential inputs
/// (e.g. the counter-generated `VestigiumId`s) map to uniformly distributed
/// `u64`s. Without this, every early trace id falls below any non-zero
/// threshold and is always sampled.
#[inline]
fn splitmix64_mix(id: u64) -> u64 {
    let mut z = id.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Samplator for SamplatorProbabilis {
    // `self.rate` is clamped to [0.0, 1.0] in `SamplatorProbabilis::new` (the
    // only constructor), so `self.rate * u64::MAX as f64` is always in
    // `[0.0, u64::MAX as f64]` — the round-trip cast below cannot lose its
    // sign or truncate outside u64's range; the remaining precision loss is
    // inherent to mapping a sampling rate onto a u64 threshold.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn should_sample(&self, trace_id: VestigiumId) -> bool {
        // Deterministic sampling: mix the (sequential) trace ID into a
        // uniform u64 before comparing against the rate threshold.
        let hash = splitmix64_mix(trace_id.value());
        let threshold = (self.rate * u64::MAX as f64) as u64;
        hash < threshold
    }
}

// =============================================================================
// Baggage
// =============================================================================

/// Key-value baggage propagated with trace context.
///
/// # Latin Etymology
/// *Impedimenta* = baggage, luggage.
#[derive(Debug, Clone, Default)]
pub struct Impedimenta {
    /// Baggage items.
    items: Vec<(alloc::string::String, alloc::string::String)>,
}

impl Impedimenta {
    /// Create new empty baggage.
    pub fn new() -> Self {
        Impedimenta {
            items: Vec::with_capacity(4),
        }
    }

    /// Set a baggage item.
    pub fn set(
        &mut self,
        key: impl Into<alloc::string::String>,
        value: impl Into<alloc::string::String>,
    ) {
        let key = key.into();
        if let Some(item) = self.items.iter_mut().find(|(k, _)| k == &key) {
            item.1 = value.into();
        } else {
            self.items.push((key, value.into()));
        }
    }

    /// Get a baggage item.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.items
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Remove a baggage item.
    #[inline]
    pub fn remove(&mut self, key: &str) {
        self.items.retain(|(k, _)| k != key);
    }

    /// Iterate over all items.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.items.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_root() {
        let ctx = ContextusVestigium::root();
        assert!(ctx.parent_span_id().is_none());
        assert!(ctx.is_sampled());
    }

    #[test]
    fn test_context_child() {
        let parent = ContextusVestigium::root();
        let child = parent.child();

        assert_eq!(child.trace_id(), parent.trace_id());
        assert_eq!(child.parent_span_id(), Some(parent.span_id()));
        assert_ne!(child.span_id(), parent.span_id());
    }

    #[test]
    fn test_context_stack() {
        let mut stack = ContextusStack::new();

        let ctx1 = stack.enter();
        assert_eq!(stack.depth(), 1);
        assert!(ctx1.parent_span_id().is_none());

        let ctx2 = stack.enter();
        assert_eq!(stack.depth(), 2);
        assert_eq!(ctx2.parent_span_id(), Some(ctx1.span_id()));

        stack.exit();
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_sampler_semper() {
        let sampler = SamplatorSemper;
        assert!(sampler.should_sample(VestigiumId::generate()));
    }

    #[test]
    fn test_sampler_nunquam() {
        let sampler = SamplatorNunquam;
        assert!(!sampler.should_sample(VestigiumId::generate()));
    }

    #[test]
    fn test_sampler_probabilis_half_rate_distribution() {
        let sampler = SamplatorProbabilis::new(0.5);
        let sampled = (1..=1000u64)
            .filter(|&id| sampler.should_sample(VestigiumId::new(id)))
            .count();
        let fraction = sampled as f64 / 1000.0;
        assert!(
            fraction > 0.35 && fraction < 0.65,
            "sampled fraction {fraction} outside (0.35, 0.65)"
        );
    }

    #[test]
    fn test_sampler_probabilis_low_rate_not_all_early_ids() {
        let sampler = SamplatorProbabilis::new(0.01);
        let all_sampled = (1..=100u64).all(|id| sampler.should_sample(VestigiumId::new(id)));
        assert!(
            !all_sampled,
            "every id in 1..=100 sampled at rate 0.01 (sequential ids not mixed)"
        );
    }

    #[test]
    fn test_impedimenta() {
        let mut baggage = Impedimenta::new();
        baggage.set("user_id", "123");
        baggage.set("session", "abc");

        assert_eq!(baggage.get("user_id"), Some("123"));
        assert_eq!(baggage.len(), 2);

        baggage.remove("session");
        assert_eq!(baggage.len(), 1);
    }
}
