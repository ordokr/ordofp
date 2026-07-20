//! Trace Collectors
//!
//! > *"Collector est custos memoriae"*
//! > — The collector is the guardian of memory. (Latin)
//!
//! This module defines the collector trait and implementations for
//! gathering trace events.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::event::EventusVestigium;

// Simple spin mutex for no_std
struct SpinMutex<T> {
    locked: AtomicBool,
    data: core::cell::UnsafeCell<T>,
}

// SAFETY: SpinMutex encapsulates data in an UnsafeCell and guarantees mutual
// exclusion via an atomic locked flag. It is Send if T is Send, and Sync if T is Send.
unsafe impl<T: Send> Send for SpinMutex<T> {}
// SAFETY: SpinMutex provides mutually exclusive access, so Sync requires T: Send,
// identical to std::sync::Mutex.
unsafe impl<T: Send> Sync for SpinMutex<T> {}

impl<T> SpinMutex<T> {
    fn new(data: T) -> Self {
        SpinMutex {
            locked: AtomicBool::new(false),
            data: core::cell::UnsafeCell::new(data),
        }
    }

    fn lock(&self) -> SpinMutexGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        SpinMutexGuard {
            mutex: self,
            _marker: core::marker::PhantomData,
        }
    }
}

struct SpinMutexGuard<'a, T> {
    mutex: &'a SpinMutex<T>,
    _marker: core::marker::PhantomData<T>,
}

// SAFETY: A mutex guard provides shared references to T via Deref. Sharing the
// guard across threads is safe only if T can be safely shared (T: Sync).
unsafe impl<T: Sync> Sync for SpinMutexGuard<'_, T> {}

// SAFETY: A mutex guard provides mutually exclusive access to T. Sending the
// guard across threads transfers ownership of the locked data, which is safe
// only if T can be safely sent across threads (T: Send).
unsafe impl<T: Send> Send for SpinMutexGuard<'_, T> {}

impl<T> core::ops::Deref for SpinMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: The guard is only constructed after acquiring the spin lock
        // (compare_exchange_weak with Acquire ordering), guaranteeing exclusive
        // access to the UnsafeCell contents. The shared reference lifetime is
        // bounded by 'a, which cannot outlive the SpinMutex.
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> core::ops::DerefMut for SpinMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Same as Deref: the spin lock is held, ensuring no other
        // thread holds a reference. The &mut self receiver prevents aliased
        // mutable references within a single thread.
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for SpinMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}

// =============================================================================
// Collector Trait
// =============================================================================

/// Trait for collecting trace events.
///
/// # Latin Etymology
/// *Collector vestigium* = trace collector.
pub trait CollectorVestigium: Send + Sync {
    /// Record a trace event.
    fn record(&self, event: EventusVestigium);

    /// Flush any buffered events.
    fn flush(&self);

    /// Check if the collector is enabled for the given effect.
    #[inline]
    fn is_enabled(&self, effect_id: u64) -> bool {
        let _ = effect_id;
        true
    }
}

// =============================================================================
// Null Collector
// =============================================================================

/// A collector that discards all events.
///
/// Useful for disabling tracing without code changes.
///
/// # Latin Etymology
/// *Collector nullus* = null collector.
pub struct CollectorNullus;

impl CollectorNullus {
    /// Create a new null collector.
    #[inline]
    pub fn new() -> Self {
        CollectorNullus
    }
}

impl Default for CollectorNullus {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectorVestigium for CollectorNullus {
    #[inline]
    fn record(&self, _event: EventusVestigium) {
        // Discard
    }

    #[inline]
    fn flush(&self) {
        // Nothing to flush
    }

    #[inline]
    fn is_enabled(&self, _effect_id: u64) -> bool {
        false
    }
}

// =============================================================================
// Memory Collector
// =============================================================================

/// A collector that stores events in memory.
///
/// Useful for testing and debugging.
///
/// # Latin Etymology
/// *Collector memoriae* = memory collector.
pub struct CollectorMemoriae {
    /// Stored events.
    events: SpinMutex<Vec<EventusVestigium>>,
    /// Maximum number of events to store.
    max_events: usize,
    /// Number of dropped events.
    dropped: AtomicUsize,
}

impl CollectorMemoriae {
    /// Create a new memory collector.
    pub fn new(max_events: usize) -> Self {
        CollectorMemoriae {
            events: SpinMutex::new(Vec::with_capacity(max_events.min(1024))),
            max_events,
            dropped: AtomicUsize::new(0),
        }
    }

    /// Get all collected events.
    #[inline]
    pub fn events(&self) -> Vec<EventusVestigium> {
        self.events.lock().clone()
    }

    /// Get the number of events.
    #[inline]
    pub fn event_count(&self) -> usize {
        self.events.lock().len()
    }

    /// Get the number of dropped events.
    #[inline]
    pub fn dropped_count(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Clear all events.
    #[inline]
    pub fn clear(&self) {
        self.events.lock().clear();
        self.dropped.store(0, Ordering::Relaxed);
    }
}

impl Default for CollectorMemoriae {
    fn default() -> Self {
        Self::new(10000)
    }
}

impl CollectorVestigium for CollectorMemoriae {
    #[inline]
    fn record(&self, event: EventusVestigium) {
        let mut events = self.events.lock();
        if events.len() >= self.max_events {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        } else {
            events.push(event);
        }
    }

    #[inline]
    fn flush(&self) {
        // Nothing to flush for in-memory storage
    }
}

// =============================================================================
// Filtered Collector
// =============================================================================

/// A collector that filters events before forwarding.
///
/// # Latin Etymology
/// *Collector filtrans* = filtering collector.
pub struct CollectorFiltrans<C: CollectorVestigium> {
    /// Inner collector.
    inner: C,
    /// Minimum level to record.
    min_level: super::Gradus,
    /// Effect IDs to include (empty = all).
    include_effects: Vec<u64>,
    /// Effect IDs to exclude.
    exclude_effects: Vec<u64>,
}

impl<C: CollectorVestigium> CollectorFiltrans<C> {
    /// Create a new filtering collector.
    pub fn new(inner: C) -> Self {
        CollectorFiltrans {
            inner,
            min_level: super::Gradus::Vestigium,
            include_effects: Vec::with_capacity(4),
            exclude_effects: Vec::with_capacity(4),
        }
    }

    /// Set the minimum level.
    #[inline]
    pub fn with_min_level(mut self, level: super::Gradus) -> Self {
        self.min_level = level;
        self
    }

    /// Include only specific effects.
    #[inline]
    pub fn include_effects(mut self, effects: Vec<u64>) -> Self {
        self.include_effects = effects;
        self
    }

    /// Exclude specific effects.
    #[inline]
    pub fn exclude_effects(mut self, effects: Vec<u64>) -> Self {
        self.exclude_effects = effects;
        self
    }

    /// Check if an event should be recorded.
    #[inline]
    fn should_record(&self, event: &EventusVestigium) -> bool {
        // Check level
        if event.level() < self.min_level {
            return false;
        }

        // Check exclusions
        if self.exclude_effects.contains(&event.effect_id()) {
            return false;
        }

        // Check inclusions
        if !self.include_effects.is_empty() && !self.include_effects.contains(&event.effect_id()) {
            return false;
        }

        true
    }
}

impl<C: CollectorVestigium> CollectorVestigium for CollectorFiltrans<C> {
    #[inline]
    fn record(&self, event: EventusVestigium) {
        if self.should_record(&event) {
            self.inner.record(event);
        }
    }

    #[inline]
    fn flush(&self) {
        self.inner.flush();
    }

    #[inline]
    fn is_enabled(&self, effect_id: u64) -> bool {
        if self.exclude_effects.contains(&effect_id) {
            return false;
        }
        if !self.include_effects.is_empty() && !self.include_effects.contains(&effect_id) {
            return false;
        }
        self.inner.is_enabled(effect_id)
    }
}

// =============================================================================
// Composite Collector
// =============================================================================

/// A collector that forwards to multiple collectors.
///
/// # Latin Etymology
/// *Collector compositus* = composite collector.
pub struct CollectorCompositus {
    /// Inner collectors.
    collectors: Vec<Arc<dyn CollectorVestigium>>,
}

impl CollectorCompositus {
    /// Create a new composite collector.
    pub fn new() -> Self {
        CollectorCompositus {
            collectors: Vec::with_capacity(4),
        }
    }

    /// Append a collector (builder-style).
    #[inline]
    pub fn with_collector<C: CollectorVestigium + 'static>(mut self, collector: C) -> Self {
        self.collectors.push(Arc::new(collector));
        self
    }

    /// Append an Arc-wrapped collector (builder-style).
    #[inline]
    pub fn with_collector_arc(mut self, collector: Arc<dyn CollectorVestigium>) -> Self {
        self.collectors.push(collector);
        self
    }
}

impl Default for CollectorCompositus {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectorVestigium for CollectorCompositus {
    #[inline]
    fn record(&self, event: EventusVestigium) {
        for collector in &self.collectors {
            collector.record(event.clone());
        }
    }

    #[inline]
    fn flush(&self) {
        for collector in &self.collectors {
            collector.flush();
        }
    }

    #[inline]
    fn is_enabled(&self, effect_id: u64) -> bool {
        self.collectors.iter().any(|c| c.is_enabled(effect_id))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::{SpatiumId, VestigiumId};

    fn make_event(effect_name: &str) -> EventusVestigium {
        EventusVestigium::new(
            VestigiumId::generate(),
            SpatiumId::generate(),
            1,
            effect_name,
            "operation",
        )
    }

    #[test]
    fn test_null_collector() {
        let collector = CollectorNullus::new();
        collector.record(make_event("Test"));
        assert!(!collector.is_enabled(1));
    }

    #[test]
    fn test_memory_collector() {
        let collector = CollectorMemoriae::new(100);

        collector.record(make_event("Effect1"));
        collector.record(make_event("Effect2"));

        assert_eq!(collector.event_count(), 2);
        assert_eq!(collector.dropped_count(), 0);
    }

    #[test]
    fn test_memory_collector_overflow() {
        let collector = CollectorMemoriae::new(2);

        collector.record(make_event("Effect1"));
        collector.record(make_event("Effect2"));
        collector.record(make_event("Effect3"));

        assert_eq!(collector.event_count(), 2);
        assert_eq!(collector.dropped_count(), 1);
    }

    #[test]
    fn test_filtered_collector() {
        let inner = CollectorMemoriae::new(100);
        let collector = CollectorFiltrans::new(inner).with_min_level(super::super::Gradus::Info);

        let event = make_event("Test");
        collector.record(event);

        // Should be recorded (Info >= Info)
        // Note: We can't directly check inner.event_count() without shared access
    }

    #[test]
    fn test_composite_collector() {
        let collector1 = Arc::new(CollectorMemoriae::new(100));
        let collector2 = Arc::new(CollectorMemoriae::new(100));

        let composite = CollectorCompositus::new()
            .with_collector_arc(collector1.clone())
            .with_collector_arc(collector2.clone());

        composite.record(make_event("Test"));

        assert_eq!(collector1.event_count(), 1);
        assert_eq!(collector2.event_count(), 1);
    }
}
