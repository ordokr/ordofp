//! Fiber Scheduling Building Blocks - Queues, Policies, Configuration
//!
//! > *"Ordo est parium dispariumque rerum sua cuique loca tribuens dispositio"*
//! > — Order is an arrangement assigning to each thing its proper place. (Augustine)
//!
//! This module provides the *data structures* for a work-stealing fiber
//! scheduler in the style of ZIO's runtime and Tokio's scheduler: local and
//! global task queues, steal policies, priorities, execution hints,
//! configuration, and statistics.
//!
//! # Scope
//!
//! There is **no executor here** — no `Ordinarius` runtime drives these
//! queues, spawns worker threads, or steals work on its own. The pieces are
//! usable building blocks (and are exercised by tests/benches), but wiring
//! them into a running M:N scheduler is left to the caller or future work.
//!
//! # Contention profile (measured) and standing decision
//!
//! `benches/scheduler_contention.rs` measured the `Mutex<VecDeque>` queues
//! collapsing 6.6× at 2 stealers and 22× at 8 — real physics, but in a
//! component no runtime entrypoint reaches (tests and benches only, per the
//! scope note above). The mutex queues are therefore kept. Revisit if an
//! `Ordinarius` executor lands wired to these queues, or any consumer drives
//! ≥2 stealing workers from a runtime entrypoint: at that point migrate
//! `OrdoLocalis`/`OrdoGlobalis` to `crossbeam-deque` (pre-ranked over a
//! custom SPMC ring by the bench evidence).
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Scheduler | Ordinarius | *ordinarius* = orderly arranger |
//! | Worker | Operator | *operator* = worker |
//! | Task | Munus | *munus* = duty, task |
//! | Queue | Ordo | *ordo* = order, rank |
//! | Steal | Furari | *furari* = to steal |
//! | Priority | Prioritas | *prioritas* = precedence |

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use super::fibra::FibraId;

// =============================================================================
// Prioritas - Task Priority
// =============================================================================

/// Priority level for fiber tasks.
///
/// Higher priority tasks are scheduled before lower priority ones.
///
/// # Latin Etymology
/// *Prioritas* = the state of being prior or first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Prioritas {
    /// Background priority - lowest.
    Infima = 0,
    /// Normal priority - default.
    #[default]
    Normalis = 1,
    /// High priority.
    Alta = 2,
    /// Critical priority - highest.
    Critica = 3,
}

// =============================================================================
// ExecutionHint - CPU vs IO Bound
// =============================================================================

/// Hint about the execution characteristics of a task.
///
/// Helps the scheduler make better decisions about where to run tasks.
///
/// # Latin Etymology
/// *Indicium Executionis* = execution indication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IndiciumExecutionis {
    /// CPU-bound computation (prefer local queue).
    #[default]
    Computatio,
    /// IO-bound operation (can be yielded frequently).
    IoOperatio,
    /// Blocking operation (should run on blocking thread pool).
    Obstruens,
}

// =============================================================================
// MunusFibrae - Fiber Task
// =============================================================================

/// A schedulable fiber task.
///
/// `MunusFibrae` wraps a fiber with scheduling metadata.
///
/// # Latin Etymology
/// *Munus* = duty, office, task. *Fibrae* = of fiber (genitive).
pub struct MunusFibrae {
    /// The fiber ID.
    pub fibra_id: FibraId,
    /// Task priority.
    pub prioritas: Prioritas,
    /// Execution hint.
    pub indicium: IndiciumExecutionis,
    /// The task itself (type-erased).
    task: Box<dyn FnOnce() + Send + 'static>,
}

impl MunusFibrae {
    /// Create a new fiber task.
    #[inline]
    pub fn new<F>(fibra_id: FibraId, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        MunusFibrae {
            fibra_id,
            prioritas: Prioritas::default(),
            indicium: IndiciumExecutionis::default(),
            task: Box::new(f),
        }
    }

    /// Create a new fiber task with priority.
    #[inline]
    pub fn with_priority<F>(fibra_id: FibraId, prioritas: Prioritas, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        MunusFibrae {
            fibra_id,
            prioritas,
            indicium: IndiciumExecutionis::default(),
            task: Box::new(f),
        }
    }

    /// Create a new fiber task with full configuration.
    #[inline]
    pub fn with_config<F>(
        fibra_id: FibraId,
        prioritas: Prioritas,
        indicium: IndiciumExecutionis,
        f: F,
    ) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        MunusFibrae {
            fibra_id,
            prioritas,
            indicium,
            task: Box::new(f),
        }
    }

    /// Execute the task.
    #[inline]
    pub fn execute(self) {
        (self.task)();
    }
}

// =============================================================================
// OrdoLocalis - Local Work Queue
// =============================================================================

/// A local work queue for a worker thread.
///
/// Uses a deque for efficient push/pop operations and allows
/// stealing from the opposite end.
///
/// # Latin Etymology
/// *Ordo Localis* = local order/queue.
#[cfg(feature = "std")]
pub struct OrdoLocalis {
    /// The deque of tasks.
    tasks: std::sync::Mutex<VecDeque<MunusFibrae>>,
    /// Number of tasks.
    count: AtomicUsize,
}

#[cfg(feature = "std")]
impl OrdoLocalis {
    /// Create a new local queue.
    pub fn new() -> Self {
        OrdoLocalis {
            tasks: std::sync::Mutex::new(VecDeque::with_capacity(DEFAULT_LOCAL_QUEUE_CAPACITY)),
            count: AtomicUsize::new(0),
        }
    }

    /// Push a task to the local queue.
    ///
    /// Tasks land at the back, where [`pop`](Self::pop) also operates —
    /// the owning worker runs in LIFO order while thieves
    /// ([`steal`](Self::steal)) take from the front.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held (task closures execute after being
    /// removed from the queue), so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn push(&self, task: MunusFibrae) {
        let mut guard = self.tasks.lock().unwrap();
        guard.push_back(task);
        // Release: length counter decoration; Mutex already orders the queue itself
        self.count.fetch_add(1, Ordering::Release);
    }

    /// Pop a task from the local queue.
    ///
    /// Removes from the back (most recently pushed — LIFO for the owning
    /// worker, which favours cache-warm tasks). Returns `None` if the
    /// queue is empty.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn pop(&self) -> Option<MunusFibrae> {
        let mut guard = self.tasks.lock().unwrap();
        let task = guard.pop_back();
        if task.is_some() {
            // Release: length counter decoration; Mutex already orders the queue itself
            self.count.fetch_sub(1, Ordering::Release);
        }
        task
    }

    /// Steal a task from the front (opposite end from pop).
    ///
    /// Thieves take the oldest task, minimising contention with the
    /// owning worker's LIFO end. Returns `None` if the queue is empty.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn steal(&self) -> Option<MunusFibrae> {
        let mut guard = self.tasks.lock().unwrap();
        let task = guard.pop_front();
        if task.is_some() {
            // Release: length counter decoration; Mutex already orders the queue itself
            self.count.fetch_sub(1, Ordering::Release);
        }
        task
    }

    /// Steal multiple tasks (batch steal).
    ///
    /// Takes up to `min(max, len / 2)` tasks from the front in one lock
    /// acquisition — at most half the queue, so the victim worker is never
    /// left empty by a single thief. May return an empty `Vec`.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    pub fn steal_batch(&self, max: usize) -> Vec<MunusFibrae> {
        let mut guard = self.tasks.lock().unwrap();
        let steal_count = core::cmp::min(max, guard.len() / 2);
        let mut stolen = Vec::with_capacity(steal_count);
        for _ in 0..steal_count {
            if let Some(task) = guard.pop_front() {
                stolen.push(task);
            }
        }
        // Release: length counter decoration; Mutex already orders the queue itself
        self.count.fetch_sub(stolen.len(), Ordering::Release);
        stolen
    }

    /// Check if the queue is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        // Acquire: approximate observer of the length counter; not load-bearing for correctness
        self.count.load(Ordering::Acquire) == 0
    }

    /// Get the number of tasks.
    #[inline]
    pub fn len(&self) -> usize {
        // Acquire: approximate observer of the length counter; not load-bearing for correctness
        self.count.load(Ordering::Acquire)
    }
}

#[cfg(feature = "std")]
impl Default for OrdoLocalis {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// OrdoGlobalis - Global Work Queue
// =============================================================================

/// The global work queue for overflow tasks.
///
/// Tasks that can't fit in local queues go here.
///
/// # Latin Etymology
/// *Ordo Globalis* = global order/queue.
#[cfg(feature = "std")]
pub struct OrdoGlobalis {
    /// The queue of tasks.
    tasks: std::sync::Mutex<VecDeque<MunusFibrae>>,
    /// Condition for waiting workers.
    not_empty: std::sync::Condvar,
    /// Shutdown flag.
    shutdown: AtomicBool,
}

#[cfg(feature = "std")]
impl OrdoGlobalis {
    /// Create a new global queue.
    pub fn new() -> Self {
        OrdoGlobalis {
            // Pre-allocate enough room for a typical burst of overflow tasks.
            tasks: std::sync::Mutex::new(VecDeque::with_capacity(DEFAULT_LOCAL_QUEUE_CAPACITY)),
            not_empty: std::sync::Condvar::new(),
            shutdown: AtomicBool::new(false),
        }
    }

    /// Push a task to the global queue.
    ///
    /// Enqueues at the back (the global queue is strictly FIFO) and wakes
    /// one worker blocked in [`pop_blocking`](Self::pop_blocking).
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn push(&self, task: MunusFibrae) {
        let mut guard = self.tasks.lock().unwrap();
        guard.push_back(task);
        self.not_empty.notify_one();
    }

    /// Push multiple tasks.
    ///
    /// Enqueues the whole batch under one lock acquisition, preserving the
    /// `Vec`'s order, then wakes all blocked workers.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn push_batch(&self, tasks: Vec<MunusFibrae>) {
        let mut guard = self.tasks.lock().unwrap();
        for task in tasks {
            guard.push_back(task);
        }
        self.not_empty.notify_all();
    }

    /// Try to pop a task without blocking.
    ///
    /// Removes and returns the oldest (front) task, or `None` if the
    /// queue is empty.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn try_pop(&self) -> Option<MunusFibrae> {
        let mut guard = self.tasks.lock().unwrap();
        guard.pop_front()
    }

    /// Block until a task is available.
    ///
    /// Blocks the calling OS thread (a `Condvar` wait) until a task is
    /// enqueued or [`shutdown`](Self::shutdown) is signalled. Returns the
    /// oldest task, or `None` when woken by shutdown with the queue empty
    /// — the workers' signal to exit their run loop.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex or the condition-variable
    /// wait is poisoned. No user code runs while this lock is held, so
    /// poisoning indicates a bug in this crate.
    #[inline]
    pub fn pop_blocking(&self) -> Option<MunusFibrae> {
        let mut guard = self.tasks.lock().unwrap();
        // Acquire: pairs with Release store in shutdown(); observes the flag set
        while guard.is_empty() && !self.shutdown.load(Ordering::Acquire) {
            guard = self.not_empty.wait(guard).unwrap();
        }
        guard.pop_front()
    }

    /// Check if empty.
    ///
    /// A snapshot: workers may enqueue or dequeue immediately after the
    /// lock is released.
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tasks.lock().unwrap().is_empty()
    }

    /// Get the length.
    ///
    /// A snapshot of the number of queued tasks; see
    /// [`is_empty`](Self::is_empty).
    ///
    /// # Panics
    ///
    /// Panics only if the internal queue mutex is poisoned. No user code
    /// runs while this lock is held, so poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn len(&self) -> usize {
        self.tasks.lock().unwrap().len()
    }

    /// Signal shutdown.
    #[inline]
    pub fn shutdown(&self) {
        // Release: pairs with Acquire loads of the shutdown flag
        self.shutdown.store(true, Ordering::Release);
        self.not_empty.notify_all();
    }

    /// Check if shutdown.
    #[inline]
    pub fn is_shutdown(&self) -> bool {
        // Acquire: pairs with Release store in shutdown()
        self.shutdown.load(Ordering::Acquire)
    }
}

#[cfg(feature = "std")]
impl Default for OrdoGlobalis {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// PolitiaFurti - Work Stealing Policy
// =============================================================================

/// Policy for work stealing.
///
/// Determines how workers steal from each other.
///
/// # Latin Etymology
/// *Politia Furti* = policy of stealing.
pub trait PolitiaFurti: Send + Sync {
    /// Choose a victim to steal from.
    fn select_victim(&self, thief_id: usize, num_workers: usize) -> Option<usize>;

    /// Determine how many tasks to steal.
    fn steal_count(&self, available: usize) -> usize;
}

/// Random victim selection policy.
pub struct PolitiaFortuita {
    /// Simple counter for pseudo-random selection.
    counter: AtomicU64,
}

impl PolitiaFortuita {
    /// Create a new random policy.
    pub fn new() -> Self {
        PolitiaFortuita {
            counter: AtomicU64::new(0),
        }
    }
}

impl Default for PolitiaFortuita {
    fn default() -> Self {
        Self::new()
    }
}

impl PolitiaFurti for PolitiaFortuita {
    #[inline]
    fn select_victim(&self, thief_id: usize, num_workers: usize) -> Option<usize> {
        if num_workers <= 1 {
            return None;
        }

        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        let mut victim = (count as usize) % num_workers;
        if victim == thief_id {
            victim = (victim + 1) % num_workers;
        }
        Some(victim)
    }

    #[inline]
    fn steal_count(&self, available: usize) -> usize {
        // Steal half of available tasks
        core::cmp::max(1, available / 2)
    }
}

/// Round-robin victim selection.
pub struct PolitiaCircularis {
    /// Next victim index per thief.
    next: AtomicUsize,
}

impl PolitiaCircularis {
    /// Create a new round-robin policy.
    pub fn new() -> Self {
        PolitiaCircularis {
            next: AtomicUsize::new(0),
        }
    }
}

impl Default for PolitiaCircularis {
    fn default() -> Self {
        Self::new()
    }
}

impl PolitiaFurti for PolitiaCircularis {
    #[inline]
    fn select_victim(&self, thief_id: usize, num_workers: usize) -> Option<usize> {
        if num_workers <= 1 {
            return None;
        }

        let mut victim = self.next.fetch_add(1, Ordering::Relaxed) % num_workers;
        if victim == thief_id {
            victim = (victim + 1) % num_workers;
        }
        Some(victim)
    }

    #[inline]
    fn steal_count(&self, available: usize) -> usize {
        core::cmp::max(1, available / 2)
    }
}

// =============================================================================
// OrdinariusConfig - Scheduler Configuration
// =============================================================================

/// Configuration for the scheduler.
///
/// # Latin Etymology
/// *Configuratio Ordinarii* = scheduler configuration.
#[derive(Debug, Clone)]
pub struct OrdinariusConfig {
    /// Number of worker threads.
    pub num_workers: usize,
    /// Local queue capacity.
    pub local_queue_capacity: usize,
    /// Enable work stealing.
    pub work_stealing: bool,
    /// Steal batch size.
    pub steal_batch_size: usize,
}

/// Default number of worker threads.
pub const DEFAULT_NUM_WORKERS: usize = 4;
/// Default local queue capacity per worker.
pub const DEFAULT_LOCAL_QUEUE_CAPACITY: usize = 256;
/// Default number of tasks to steal in a batch.
pub const DEFAULT_STEAL_BATCH_SIZE: usize = 32;

impl Default for OrdinariusConfig {
    fn default() -> Self {
        OrdinariusConfig {
            num_workers: DEFAULT_NUM_WORKERS,
            local_queue_capacity: DEFAULT_LOCAL_QUEUE_CAPACITY,
            work_stealing: true,
            steal_batch_size: DEFAULT_STEAL_BATCH_SIZE,
        }
    }
}

impl OrdinariusConfig {
    /// Create a new configuration with custom worker count.
    #[inline]
    pub fn with_workers(num_workers: usize) -> Self {
        OrdinariusConfig {
            num_workers,
            ..Default::default()
        }
    }
}

// =============================================================================
// Statisticae - Scheduler Statistics
// =============================================================================

/// Statistics about scheduler operation.
///
/// # Latin Etymology
/// *Statisticae* = statistics.
#[derive(Debug, Default)]
pub struct Statisticae {
    /// Total tasks scheduled.
    pub tasks_scheduled: AtomicU64,
    /// Total tasks executed.
    pub tasks_executed: AtomicU64,
    /// Total steal attempts.
    pub steal_attempts: AtomicU64,
    /// Successful steals.
    pub steal_successes: AtomicU64,
    /// Total tasks stolen.
    pub tasks_stolen: AtomicU64,
}

impl Statisticae {
    /// Create new empty statistics.
    #[inline]
    pub fn new() -> Self {
        Statisticae::default()
    }

    /// Record a task being scheduled.
    #[inline]
    pub fn record_scheduled(&self) {
        self.tasks_scheduled.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a task being executed.
    #[inline]
    pub fn record_executed(&self) {
        self.tasks_executed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a steal attempt.
    #[inline]
    pub fn record_steal_attempt(&self, success: bool, count: usize) {
        self.steal_attempts.fetch_add(1, Ordering::Relaxed);
        if success {
            self.steal_successes.fetch_add(1, Ordering::Relaxed);
            self.tasks_stolen.fetch_add(count as u64, Ordering::Relaxed);
        }
    }

    /// Get the steal success rate.
    #[inline]
    pub fn steal_success_rate(&self) -> f64 {
        let attempts = self.steal_attempts.load(Ordering::Relaxed);
        if attempts == 0 {
            0.0
        } else {
            let successes = self.steal_successes.load(Ordering::Relaxed);
            successes as f64 / attempts as f64
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use alloc::sync::Arc;

    fn test_fibra_id() -> FibraId {
        FibraId::new()
    }

    #[test]
    fn test_prioritas_ordering() {
        assert!(Prioritas::Critica > Prioritas::Alta);
        assert!(Prioritas::Alta > Prioritas::Normalis);
        assert!(Prioritas::Normalis > Prioritas::Infima);
    }

    #[test]
    fn test_munus_fibrae_new() {
        let id = test_fibra_id();
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        let task = MunusFibrae::new(id, move || {
            executed_clone.store(true, Ordering::SeqCst);
        });

        task.execute();
        assert!(executed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_ordo_localis_push_pop() {
        let queue = OrdoLocalis::new();
        let id = test_fibra_id();

        queue.push(MunusFibrae::new(id, || {}));
        assert_eq!(queue.len(), 1);

        let task = queue.pop();
        assert!(task.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_ordo_localis_steal() {
        let queue = OrdoLocalis::new();

        // Push multiple tasks
        for _ in 0..4 {
            queue.push(MunusFibrae::new(test_fibra_id(), || {}));
        }

        // Steal from front
        let stolen = queue.steal();
        assert!(stolen.is_some());
        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn test_ordo_localis_steal_batch() {
        let queue = OrdoLocalis::new();

        // Push 10 tasks
        for _ in 0..10 {
            queue.push(MunusFibrae::new(test_fibra_id(), || {}));
        }

        // Steal half
        let stolen = queue.steal_batch(10);
        assert_eq!(stolen.len(), 5); // Half of 10
        assert_eq!(queue.len(), 5);
    }

    #[test]
    fn test_ordo_globalis_push_try_pop() {
        let queue = OrdoGlobalis::new();

        queue.push(MunusFibrae::new(test_fibra_id(), || {}));
        assert!(!queue.is_empty());

        let task = queue.try_pop();
        assert!(task.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_politia_fortuita() {
        let policy = PolitiaFortuita::new();

        let victim = policy.select_victim(0, 4);
        assert!(victim.is_some());
        assert_ne!(
            victim.expect("select_victim should return Some when there are valid targets"),
            0
        ); // Should not steal from self
    }

    #[test]
    fn test_politia_circularis() {
        let policy = PolitiaCircularis::new();

        // Should cycle through victims
        let v1 = policy.select_victim(0, 4);
        let v2 = policy.select_victim(0, 4);

        assert!(v1.is_some());
        assert!(v2.is_some());
    }

    #[test]
    fn test_statisticae() {
        let stats = Statisticae::new();

        stats.record_scheduled();
        stats.record_scheduled();
        stats.record_executed();
        stats.record_steal_attempt(true, 3);
        stats.record_steal_attempt(false, 0);

        assert_eq!(stats.tasks_scheduled.load(Ordering::Relaxed), 2);
        assert_eq!(stats.tasks_executed.load(Ordering::Relaxed), 1);
        assert_eq!(stats.steal_attempts.load(Ordering::Relaxed), 2);
        assert_eq!(stats.steal_successes.load(Ordering::Relaxed), 1);
        assert_eq!(stats.tasks_stolen.load(Ordering::Relaxed), 3);
        assert!((stats.steal_success_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_ordinarius_config_default() {
        let config = OrdinariusConfig::default();
        assert_eq!(config.num_workers, 4);
        assert!(config.work_stealing);
    }
}
