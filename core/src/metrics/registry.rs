//! Metrics Registry
//!
//! > *"Registrum est fons omnium mensurarum"*
//! > — The registry is the source of all measures. (Latin)
//!
//! This module provides a central registry for all metrics.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write as FmtWrite;
use core::sync::atomic::Ordering;

use super::{
    MensuraArboris, MensuraEffectus, MensuraFibrae, MetricSnapshot, MetricType, MetricValue,
};

// Simple spin lock for no_std (inline implementation)
struct SpinRwLock<T> {
    data: core::cell::UnsafeCell<T>,
    // Simple flag: 0 = unlocked, 1+ = readers, usize::MAX = writer
    state: core::sync::atomic::AtomicUsize,
}

// SAFETY: SpinRwLock safely coordinates access. It is Send if T is Send.
unsafe impl<T: Send> Send for SpinRwLock<T> {}
// SAFETY: SpinRwLock coordinates readers and writers. It is Sync if T is Sync
// (multiple readers) and Send (writer takes ownership of &mut T).
unsafe impl<T: Send + Sync> Sync for SpinRwLock<T> {}

impl<T> SpinRwLock<T> {
    const fn new(data: T) -> Self {
        SpinRwLock {
            data: core::cell::UnsafeCell::new(data),
            state: core::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn read(&self) -> SpinRwLockReadGuard<'_, T> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state != usize::MAX
                && self
                    .state
                    .compare_exchange_weak(state, state + 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
            {
                return SpinRwLockReadGuard {
                    lock: self,
                    _marker: core::marker::PhantomData,
                };
            }
            core::hint::spin_loop();
        }
    }

    fn write(&self) -> SpinRwLockWriteGuard<'_, T> {
        loop {
            if self
                .state
                .compare_exchange_weak(0, usize::MAX, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return SpinRwLockWriteGuard {
                    lock: self,
                    _marker: core::marker::PhantomData,
                };
            }
            core::hint::spin_loop();
        }
    }
}

struct SpinRwLockReadGuard<'a, T> {
    lock: &'a SpinRwLock<T>,
    _marker: core::marker::PhantomData<T>,
}

// SAFETY: A read guard only provides shared references to T. Sharing the guard
// across threads is safe only if T can be safely shared (T: Sync).
unsafe impl<T: Sync> Sync for SpinRwLockReadGuard<'_, T> {}

// SAFETY: A read guard can be sent to another thread as long as the underlying T can be safely shared across threads (T: Sync).
unsafe impl<T: Sync> Send for SpinRwLockReadGuard<'_, T> {}

impl<T> core::ops::Deref for SpinRwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: A read guard is only constructed after the reader count is
        // atomically incremented (fetch_add with Acquire ordering), which
        // prevents any concurrent writer from acquiring the lock. Multiple
        // readers may coexist, but all hold only shared (&T) references, so
        // there is no aliased mutability. The lifetime 'a ensures the
        // UnsafeCell outlives this reference.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for SpinRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

struct SpinRwLockWriteGuard<'a, T> {
    lock: &'a SpinRwLock<T>,
    _marker: core::marker::PhantomData<T>,
}

// SAFETY: A write guard provides mutable references to T. Sharing the guard
// across threads is safe only if T can be safely shared (T: Sync).
unsafe impl<T: Sync> Sync for SpinRwLockWriteGuard<'_, T> {}

// SAFETY: A write guard can be sent to another thread as long as the underlying T can be safely sent to another thread (T: Send) and safely shared across threads (T: Sync) because the lock requires both.
unsafe impl<T: Send + Sync> Send for SpinRwLockWriteGuard<'_, T> {}

impl<T> core::ops::Deref for SpinRwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: A write guard is only constructed after the state is set to
        // usize::MAX via compare_exchange (AcqRel ordering), ensuring exclusive
        // ownership: no other reader or writer can hold the lock simultaneously.
        // This makes the shared reference sound for the guard's lifetime 'a.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> core::ops::DerefMut for SpinRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Same exclusivity guarantee as Deref above. Additionally, the
        // &mut self receiver ensures there is at most one live mutable reference
        // at a time even within a single thread, satisfying Rust's aliasing rules.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}

impl<'a, T> SpinRwLockWriteGuard<'a, T> {
    /// Atomically downgrades a write lock to a read lock.
    ///
    /// This is equivalent to `std::sync::RwLockWriteGuard::downgrade()` (stable 1.92).
    /// The downgrade is atomic: no writer can acquire the lock between releasing
    /// the write lock and acquiring the read lock.
    ///
    /// `SpinRwLock` is a private, no_std-compatible helper internal to this
    /// module, so it has no public path to demonstrate directly. The same
    /// downgrade idiom, using the public std type this mirrors:
    ///
    /// ```rust
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(5);
    /// let write_guard = lock.write().unwrap();
    /// // Perform writes...
    /// let read_guard = std::sync::RwLockWriteGuard::downgrade(write_guard);
    /// // Now we have a read lock, other readers can proceed
    /// assert_eq!(*read_guard, 5);
    /// ```
    #[inline]
    pub(crate) fn downgrade(self) -> SpinRwLockReadGuard<'a, T> {
        // Atomically transition from writer (usize::MAX) to single reader (1).
        // This prevents other writers from acquiring the lock during the transition.
        self.lock.state.store(1, Ordering::Release);

        // Prevent the Drop impl from running (which would set state to 0)
        let lock = self.lock;
        core::mem::forget(self);

        SpinRwLockReadGuard {
            lock,
            _marker: core::marker::PhantomData,
        }
    }
}

// =============================================================================
// Metrics Registry
// =============================================================================

/// Central registry for all metrics.
///
/// # Latin Etymology
/// *Registrum mensurarum* = registry of measures.
pub struct RegistrumMensurarum {
    /// Effect metrics by ID.
    effects: SpinRwLock<BTreeMap<u64, Arc<MensuraEffectus>>>,
    /// Fiber metrics.
    fibers: Arc<MensuraFibrae>,
    /// Supervisor metrics by name.
    supervisors: SpinRwLock<BTreeMap<String, Arc<MensuraArboris>>>,
}

impl RegistrumMensurarum {
    /// Create a new metrics registry.
    pub fn new() -> Self {
        RegistrumMensurarum {
            effects: SpinRwLock::new(BTreeMap::new()),
            fibers: Arc::new(MensuraFibrae::new()),
            supervisors: SpinRwLock::new(BTreeMap::new()),
        }
    }

    /// Get or create effect metrics.
    ///
    /// # Panics
    ///
    /// Panics only if the entry inserted under the write lock is missing
    /// after the atomic downgrade to a read lock — an internal invariant of
    /// this module's lock that cannot be violated absent a bug in this crate.
    #[inline]
    pub fn effect_metrics(&self, effect_id: u64, name: &str) -> Arc<MensuraEffectus> {
        // Fast path: read lock
        {
            let effects = self.effects.read();
            if let Some(metrics) = effects.get(&effect_id) {
                return metrics.clone();
            }
        }

        // Slow path: write lock, then downgrade to read for the clone
        let mut effects = self.effects.write();
        effects
            .entry(effect_id)
            .or_insert_with(|| Arc::new(MensuraEffectus::new(effect_id, name)));
        let effects = effects.downgrade();
        effects.get(&effect_id).unwrap().clone()
    }

    /// Get fiber metrics.
    #[inline]
    pub fn fiber_metrics(&self) -> Arc<MensuraFibrae> {
        self.fibers.clone()
    }

    /// Get or create supervisor metrics.
    #[inline]
    pub fn supervisor_metrics(&self, name: &str) -> Arc<MensuraArboris> {
        // Fast path: read lock
        {
            let supervisors = self.supervisors.read();
            if let Some(metrics) = supervisors.get(name) {
                return metrics.clone();
            }
        }

        // Slow path: write lock
        let mut supervisors = self.supervisors.write();
        supervisors
            .entry(name.into())
            .or_insert_with(|| Arc::new(MensuraArboris::new(name)))
            .clone()
    }

    /// Get all effect metric summaries.
    #[inline]
    pub fn effect_summaries(&self) -> Vec<super::EffectMetricsSummary> {
        let effects = self.effects.read();
        effects.values().map(|m| m.summary()).collect()
    }

    /// Get fiber metrics summary.
    #[inline]
    pub fn fiber_summary(&self) -> super::FiberMetricsSummary {
        self.fibers.summary()
    }

    /// Export all metrics as snapshots.
    // One pass per metric kind; the length comes from enumerating kinds, not
    // from tangled control flow.
    #[allow(clippy::too_many_lines)]
    pub fn export(&self) -> Vec<MetricSnapshot> {
        let mut snapshots = Vec::with_capacity(32);

        // P3: formatting is O(metrics) allocations; do it outside the read lock.
        // Only clone the Arc handles (a cheap refcount bump) while the read
        // guard is held, then drop it before doing any label/string formatting
        // so writers (effect registration) don't block for the full export.
        let effect_metrics: Vec<Arc<MensuraEffectus>> = {
            let effects = self.effects.read();
            effects.values().cloned().collect()
        };

        // Export effect metrics
        for metrics in &effect_metrics {
            // Operations counter
            snapshots.push(MetricSnapshot {
                name: "effect_operations_total".into(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(metrics.total_operations()),
                labels: vec![
                    ("effect_id".into(), metrics.effect_id().to_string()),
                    ("effect_name".into(), metrics.effect_name().into()),
                ],
            });

            // Successes counter
            snapshots.push(MetricSnapshot {
                name: "effect_successes_total".into(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(metrics.success_count()),
                labels: vec![
                    ("effect_id".into(), metrics.effect_id().to_string()),
                    ("effect_name".into(), metrics.effect_name().into()),
                ],
            });

            // Failures counter
            snapshots.push(MetricSnapshot {
                name: "effect_failures_total".into(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(metrics.failure_count()),
                labels: vec![
                    ("effect_id".into(), metrics.effect_id().to_string()),
                    ("effect_name".into(), metrics.effect_name().into()),
                ],
            });

            // In-flight gauge
            snapshots.push(MetricSnapshot {
                name: "effect_in_flight".into(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(metrics.in_flight()),
                labels: vec![
                    ("effect_id".into(), metrics.effect_id().to_string()),
                    ("effect_name".into(), metrics.effect_name().into()),
                ],
            });

            // Latency histogram
            let hist = metrics.latency_histogram();
            let buckets: Vec<(u64, u64)> = hist
                .boundaries()
                .iter()
                .zip(hist.bucket_counts())
                .map(|(&b, c)| (b, c))
                .collect();

            snapshots.push(MetricSnapshot {
                name: "effect_latency_nanoseconds".into(),
                metric_type: MetricType::Histogram,
                value: MetricValue::Histogram {
                    count: hist.count(),
                    sum: hist.sum(),
                    buckets,
                },
                labels: vec![
                    ("effect_id".into(), metrics.effect_id().to_string()),
                    ("effect_name".into(), metrics.effect_name().into()),
                ],
            });
        }

        // Export fiber metrics
        let fibers = &self.fibers;
        snapshots.push(MetricSnapshot {
            name: "fibers_spawned_total".into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(fibers.total_spawned()),
            labels: vec![],
        });

        snapshots.push(MetricSnapshot {
            name: "fibers_completed_total".into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(fibers.completed_count()),
            labels: vec![],
        });

        snapshots.push(MetricSnapshot {
            name: "fibers_failed_total".into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(fibers.failed_count()),
            labels: vec![],
        });

        snapshots.push(MetricSnapshot {
            name: "fibers_cancelled_total".into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(fibers.cancelled_count()),
            labels: vec![],
        });

        snapshots.push(MetricSnapshot {
            name: "fibers_active".into(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(fibers.active_count()),
            labels: vec![],
        });

        snapshots.push(MetricSnapshot {
            name: "fibers_peak_active".into(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(fibers.peak_active()),
            labels: vec![],
        });

        snapshots
    }

    /// Escape a Prometheus label value: backslash, quote, newline.
    fn effuge_valorem(v: &str) -> String {
        let mut out = String::with_capacity(v.len());
        for c in v.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                _ => out.push(c),
            }
        }
        out
    }

    /// Export to Prometheus text format.
    pub fn prometheus_export(&self) -> String {
        let mut output = String::new();

        for snapshot in self.export() {
            // Write help and type
            let type_str = match snapshot.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
            };
            writeln!(output, "# TYPE {} {}", snapshot.name, type_str)
                .expect("writing to String is infallible");

            // Format labels
            let labels = if snapshot.labels.is_empty() {
                String::new()
            } else {
                let label_str: Vec<String> = snapshot
                    .labels
                    .iter()
                    .map(|(k, v)| alloc::format!("{k}=\"{}\"", Self::effuge_valorem(v)))
                    .collect();
                alloc::format!("{{{}}}", label_str.join(","))
            };

            // Write value
            match snapshot.value {
                MetricValue::Counter(v) => {
                    writeln!(output, "{}{} {}", snapshot.name, labels, v)
                        .expect("writing to String is infallible");
                }
                MetricValue::Gauge(v) => {
                    writeln!(output, "{}{} {}", snapshot.name, labels, v)
                        .expect("writing to String is infallible");
                }
                MetricValue::Histogram {
                    count,
                    sum,
                    ref buckets,
                } => {
                    for (boundary, bucket_count) in buckets {
                        let boundary_str = Self::effuge_valorem(&boundary.to_string());
                        let bucket_labels = if labels.is_empty() {
                            alloc::format!("{{le=\"{boundary_str}\"}}")
                        } else {
                            let inner = &labels[1..labels.len() - 1];
                            alloc::format!("{{{inner},le=\"{boundary_str}\"}}")
                        };
                        writeln!(
                            output,
                            "{}_bucket{} {}",
                            snapshot.name, bucket_labels, bucket_count
                        )
                        .expect("writing to String never fails");
                    }
                    writeln!(output, "{}_sum{} {}", snapshot.name, labels, sum)
                        .expect("writing to String never fails");
                    writeln!(output, "{}_count{} {}", snapshot.name, labels, count)
                        .expect("writing to String never fails");
                }
            }
        }

        output
    }
}

impl Default for RegistrumMensurarum {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Global Registry (Optional)
// =============================================================================

/// Get the global metrics registry.
///
/// Uses a simple atomic initialization for `no_std` compatibility.
pub fn global_registry() -> &'static RegistrumMensurarum {
    use alloc::boxed::Box;
    use core::ptr;
    use core::sync::atomic::AtomicPtr;

    static REGISTRY: AtomicPtr<RegistrumMensurarum> = AtomicPtr::new(ptr::null_mut());

    let mut ptr = REGISTRY.load(Ordering::Acquire);
    if ptr.is_null() {
        let new_registry = Box::new(RegistrumMensurarum::new());
        let new_ptr = Box::into_raw(new_registry);

        match REGISTRY.compare_exchange(
            ptr::null_mut(),
            new_ptr,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => ptr = new_ptr,
            Err(existing) => {
                // Another thread initialized first, free our allocation
                // SAFETY: new_ptr was created from Box::into_raw above and the
                // compare_exchange failed, meaning we still own this allocation
                // exclusively (no other thread received it). Reconstituting the
                // Box to drop it is therefore sound.
                unsafe {
                    drop(Box::from_raw(new_ptr));
                }
                ptr = existing;
            }
        }
    }

    // SAFETY: ptr is non-null at this point — either it was loaded from the
    // AtomicPtr after a successful compare_exchange (AcqRel) or via the Acquire
    // load of an already-initialized pointer. The Box was leaked into the static
    // AtomicPtr and is never freed, so the 'static lifetime is valid.
    unsafe { &*ptr }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = RegistrumMensurarum::new();
        assert!(registry.effect_summaries().is_empty());
    }

    #[test]
    fn test_registry_effect_metrics() {
        let registry = RegistrumMensurarum::new();

        let metrics1 = registry.effect_metrics(1, "StateEffect");
        let metrics2 = registry.effect_metrics(1, "StateEffect");

        // Should return the same instance
        assert!(Arc::ptr_eq(&metrics1, &metrics2));
    }

    #[test]
    fn test_registry_different_effects() {
        let registry = RegistrumMensurarum::new();

        let metrics1 = registry.effect_metrics(1, "StateEffect");
        let metrics2 = registry.effect_metrics(2, "IOEffect");

        assert!(!Arc::ptr_eq(&metrics1, &metrics2));
    }

    #[test]
    fn test_registry_fiber_metrics() {
        let registry = RegistrumMensurarum::new();

        let fibers1 = registry.fiber_metrics();
        let fibers2 = registry.fiber_metrics();

        assert!(Arc::ptr_eq(&fibers1, &fibers2));
    }

    #[test]
    fn test_registry_export() {
        let registry = RegistrumMensurarum::new();

        let effect = registry.effect_metrics(1, "Test");
        effect.operation_start();
        effect.record_success(1000);

        let snapshots = registry.export();
        assert!(!snapshots.is_empty());
    }

    #[test]
    fn test_registry_prometheus_export() {
        let registry = RegistrumMensurarum::new();

        let effect = registry.effect_metrics(1, "Test");
        effect.operation_start();
        effect.record_success(1000);

        let output = registry.prometheus_export();
        assert!(output.contains("effect_operations_total"));
        assert!(output.contains("effect_successes_total"));
    }

    #[test]
    fn prometheus_export_escapes_label_values() {
        let registry = RegistrumMensurarum::new();

        // Effect name containing a quote, a backslash, and a newline.
        let effect = registry.effect_metrics(1, "weird\"name\\with\nnewline");
        effect.operation_start();
        effect.record_success(1000);

        let output = registry.prometheus_export();

        assert!(
            output.contains(r#"\"name"#),
            "expected escaped quote: {output}"
        );
        assert!(
            output.contains(r"\\with"),
            "expected escaped backslash: {output}"
        );
        assert!(
            output.contains(r"\nnewline"),
            "expected escaped newline: {output}"
        );

        // A raw (unescaped) newline in a label value would split one logical
        // metric line into two, leaving unbalanced braces somewhere in the
        // output. Every line's braces must balance.
        for line in output.lines() {
            let opens = line.matches('{').count();
            let closes = line.matches('}').count();
            assert_eq!(
                opens, closes,
                "unbalanced braces (raw newline split a label?): {line}"
            );
        }
    }

    #[test]
    fn test_global_registry() {
        let registry1 = global_registry();
        let registry2 = global_registry();

        // Should be the same instance
        assert!(core::ptr::eq(registry1, registry2));
    }

    #[test]
    fn test_spinrwlock_downgrade() {
        // Test the downgrade() method (mirrors RwLockWriteGuard::downgrade from Rust 1.92)
        let lock = SpinRwLock::new(42);

        // Acquire write lock
        let write_guard = lock.write();
        assert_eq!(*write_guard, 42);

        // Downgrade to read lock
        let read_guard = write_guard.downgrade();
        assert_eq!(*read_guard, 42);

        // After downgrade, the read guard still works
        drop(read_guard);

        // Can acquire another read lock now
        let read_guard2 = lock.read();
        assert_eq!(*read_guard2, 42);
    }

    #[test]
    fn test_spinrwlock_downgrade_allows_concurrent_reads() {
        use alloc::sync::Arc;

        let lock = Arc::new(SpinRwLock::new(100));

        // Acquire write lock
        let write_guard = lock.write();

        // Downgrade to read lock
        let read_guard1 = write_guard.downgrade();

        // Now we should be able to get another read lock
        // (in a real concurrent test this would be in another thread)
        let read_guard2 = lock.read();

        assert_eq!(*read_guard1, 100);
        assert_eq!(*read_guard2, 100);
    }
}
