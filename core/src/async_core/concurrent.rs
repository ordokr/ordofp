//! Concurrent Primitives - Cats Effect-style Synchronization
//!
//! > *"Concurrere est simul currere"*
//! > — To concur is to run together. (Latin)
//!
//! This module provides concurrent synchronization primitives inspired by
//! Cats Effect 3's concurrent data structures.
//!
//! # These primitives BLOCK the calling thread
//!
//! Despite the fiber-flavoured naming, every waiting operation here is
//! implemented with `std::sync::Mutex`/`Condvar` — waits **block the OS
//! thread**, they do not yield to an async executor. Calling them from
//! inside an async task can stall or deadlock single-threaded executors.
//! Use them between OS threads (or dedicated blocking pools), not as
//! await-points inside futures.
//!
//! # Overview
//!
//! These primitives enable safe communication and synchronization between
//! concurrent threads:
//!
//! - `Dilatum` - Single-assignment promise (Deferred)
//! - `Referentia` - Atomic mutable reference (Ref)
//! - `Semaphorum` - Counting semaphore
//! - `MVarSync` - Synchronized mutable variable
//! - `CaudaBackpressure` - Bounded queue with backpressure
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Deferred | Dilatum | *dilatum* = postponed, deferred |
//! | Ref | Referentia | *referentia* = reference |
//! | Semaphore | Semaphorum | *semaphorum* = signal bearer |
//! | Queue | Cauda | *cauda* = tail (FIFO order) |
//! | Permit | Permissum | *permissum* = permission |

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::sync::{Condvar, Mutex};

// =============================================================================
// Dilatum - Single-Assignment Deferred Value
// =============================================================================

/// Internal completion state of a [`Dilatum`].
///
/// Both the "is it done" question and the stored value live behind the
/// *same* mutex, so a completion check-and-set is a single critical
/// section — there is no window where one thread can observe "completed"
/// while another hasn't yet published the value (the TOCTOU that existed
/// when the flag and the value were two separate synchronization
/// primitives).
#[cfg(feature = "std")]
enum Status<A> {
    /// Not yet completed.
    Vacuus,
    /// Completed with a value.
    Perfectus(A),
}

/// A single-assignment concurrent variable.
///
/// `Dilatum<A>` can be completed exactly once with a value, after which
/// all waiters receive that value.
///
/// # Latin Etymology
/// *Dilatum* = participle of *differre*, to postpone or defer.
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::concurrent::Dilatum;
///
/// let deferred = Dilatum::<i32>::new();
///
/// // Complete once, from anywhere.
/// assert!(deferred.complete(42));
///
/// // Any waiter blocks until completion; already complete here, so this
/// // returns immediately with the value.
/// let value = deferred.get_blocking();
/// assert_eq!(value, 42);
/// ```
#[cfg(feature = "std")]
pub struct Dilatum<A> {
    /// Completion state (vacant or holding the value), guarded by a single
    /// mutex so check-and-set is one critical section — see [`Status`].
    state: Mutex<Status<A>>,
    /// Condition variable for waiters.
    cond: Condvar,
}

#[cfg(feature = "std")]
impl<A: Clone> Dilatum<A> {
    /// Create a new empty deferred value.
    #[inline]
    pub fn new() -> Self {
        Dilatum {
            state: Mutex::new(Status::Vacuus),
            cond: Condvar::new(),
        }
    }

    /// Try to complete with a value.
    ///
    /// Returns `true` if this call completed the deferred,
    /// `false` if it was already completed.
    ///
    /// The completion check and the value write happen under one lock
    /// acquisition, so concurrent callers race for a single winner: exactly
    /// one `complete` call ever returns `true`, and no other thread can
    /// ever observe the deferred as completed without its value already
    /// being in place.
    ///
    /// # Panics
    ///
    /// Panics if the state mutex is poisoned. The only user code that ever
    /// runs while this lock is held is `A::clone` (inside
    /// [`try_get`](Self::try_get)/[`get_blocking`](Self::get_blocking)), so
    /// absent a panicking `Clone` implementation, poisoning indicates a bug
    /// in this crate.
    #[inline]
    pub fn complete(&self, a: A) -> bool {
        let mut guard = self
            .state
            .lock()
            .expect("Dilatum::complete: state mutex poisoned — library invariant violated");
        match &*guard {
            Status::Perfectus(_) => false,
            Status::Vacuus => {
                *guard = Status::Perfectus(a);
                self.cond.notify_all();
                true
            }
        }
    }

    /// Check if the deferred has been completed.
    ///
    /// # Panics
    ///
    /// Panics if the state mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent
    /// [`try_get`](Self::try_get)/[`get_blocking`](Self::get_blocking) call;
    /// otherwise it indicates a bug in this crate.
    #[inline]
    pub fn is_completed(&self) -> bool {
        let guard = self
            .state
            .lock()
            .expect("Dilatum::is_completed: state mutex poisoned — library invariant violated");
        matches!(&*guard, Status::Perfectus(_))
    }

    /// Try to get the value without blocking.
    ///
    /// Returns `Some` with a clone of the value if the deferred has been
    /// completed, `None` otherwise.
    ///
    /// # Panics
    ///
    /// Panics if the state mutex is poisoned. `A::clone` runs while the lock
    /// is held, so a panicking `Clone` implementation poisons the lock for
    /// all later callers; absent that, poisoning indicates a bug in this
    /// crate.
    #[inline]
    pub fn try_get(&self) -> Option<A> {
        let guard = self
            .state
            .lock()
            .expect("Dilatum::try_get: state mutex poisoned — library invariant violated");
        match &*guard {
            Status::Perfectus(a) => Some(a.clone()),
            Status::Vacuus => None,
        }
    }

    /// Block until the value is available.
    ///
    /// Blocks the calling OS thread (this is a `Condvar` wait, not an async
    /// await-point) until some thread calls [`complete`](Self::complete),
    /// then returns a clone of the completed value. Returns immediately if
    /// already completed. There is no timeout.
    ///
    /// # Panics
    ///
    /// Panics if the state mutex or the condition-variable wait is poisoned.
    /// `A::clone` runs while the lock is held, so a panicking `Clone`
    /// implementation poisons the lock for all later callers; absent that,
    /// poisoning indicates a bug in this crate.
    pub fn get_blocking(&self) -> A {
        let mut guard = self
            .state
            .lock()
            .expect("Dilatum::get_blocking: state mutex poisoned — library invariant violated");
        loop {
            match &*guard {
                Status::Perfectus(a) => return a.clone(),
                Status::Vacuus => {
                    guard = self.cond.wait(guard).expect(
                        "Dilatum::get_blocking: condvar wait poisoned — library invariant violated",
                    );
                }
            }
        }
    }
}

#[cfg(feature = "std")]
impl<A: Clone> Default for Dilatum<A> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Referentia - Atomic Mutable Reference
// =============================================================================

/// An atomic mutable reference.
///
/// `Referentia<A>` provides atomic read-modify-write operations for
/// concurrent access to mutable state.
///
/// # Latin Etymology
/// *Referentia* = the act of referring, reference.
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::concurrent::Referentia;
///
/// let ref_cell = Referentia::new(0);
///
/// // Atomically increment
/// ref_cell.update(|x| x + 1);
///
/// // Get current value
/// let value = ref_cell.get();
/// assert_eq!(value, 1);
/// ```
#[cfg(feature = "std")]
pub struct Referentia<A> {
    /// The mutable value protected by a mutex.
    value: Mutex<A>,
}

#[cfg(feature = "std")]
impl<A: Clone> Referentia<A> {
    /// Create a new reference with initial value.
    #[inline]
    pub fn new(a: A) -> Self {
        Referentia {
            value: Mutex::new(a),
        }
    }

    /// Get the current value.
    ///
    /// # Poison recovery
    /// No user closure runs here, so there is nothing that could leave the
    /// value torn; a poisoned mutex still holds a fully-formed `A`.
    #[inline]
    pub fn get(&self) -> A {
        let guard = self
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.clone()
    }

    /// Set the value, returning the old value.
    ///
    /// # Poison recovery
    /// No user closure runs here either — `mem::replace` is a single atomic
    /// swap, so there is no torn state to observe.
    #[inline]
    pub fn set(&self, a: A) -> A {
        let mut guard = self
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        core::mem::replace(&mut *guard, a)
    }

    /// Update the value with a function.
    ///
    /// # Poison recovery
    /// `f(guard.clone())` runs and completes into the local `new_value`
    /// BEFORE the single `*guard = new_value` assignment. A panic inside `f`
    /// unwinds before that assignment executes, leaving the previous
    /// (fully-consistent) value in place — there is no torn state to
    /// observe, so recovering from poisoning here is sound.
    #[inline]
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(A) -> A,
    {
        let mut guard = self
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let new_value = f(guard.clone());
        *guard = new_value;
    }

    /// Update and return the old value.
    ///
    /// # Poison recovery
    /// `f(old.clone())` runs and completes BEFORE the single
    /// `*guard = ...` assignment. A panic inside `f` leaves `*guard`
    /// untouched — no torn state.
    #[inline]
    pub fn get_and_update<F>(&self, f: F) -> A
    where
        F: FnOnce(A) -> A,
    {
        let mut guard = self
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old = guard.clone();
        *guard = f(old.clone());
        old
    }

    /// Update and return the new value.
    ///
    /// # Poison recovery
    /// `f(guard.clone())` runs into the local `new_value` BEFORE the single
    /// `*guard = new_value.clone()` assignment. A panic inside `f` leaves
    /// `*guard` untouched — no torn state.
    #[inline]
    pub fn update_and_get<F>(&self, f: F) -> A
    where
        F: FnOnce(A) -> A,
    {
        let mut guard = self
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let new_value = f(guard.clone());
        *guard = new_value.clone();
        new_value
    }

    /// Modify with a function that returns both the new value and a result.
    ///
    /// # Poison recovery
    /// `f(guard.clone())` runs and is destructured into the local
    /// `(new_value, result)` BEFORE the single `*guard = new_value`
    /// assignment. A panic inside `f` leaves `*guard` untouched — no torn
    /// state.
    #[inline]
    pub fn modify<B, F>(&self, f: F) -> B
    where
        F: FnOnce(A) -> (A, B),
    {
        let mut guard = self
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (new_value, result) = f(guard.clone());
        *guard = new_value;
        result
    }
}

// =============================================================================
// Semaphorum - Counting Semaphore
// =============================================================================

/// A counting semaphore.
///
/// `Semaphorum` maintains a count of available permits. Acquiring
/// a permit decrements the count, releasing increments it.
///
/// # Latin Etymology
/// *Semaphorum* = signal bearer (from Greek σῆμα + φέρειν).
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::concurrent::{Permissum, Semaphorum};
///
/// let sem = Semaphorum::new(3); // 3 permits
///
/// // Acquire a permit (blocks until one is available)
/// let permit = Permissum::new(&sem);
/// assert_eq!(sem.available(), 2);
///
/// // Release when done (also happens automatically on drop)
/// permit.release();
/// assert_eq!(sem.available(), 3);
/// ```
#[cfg(feature = "std")]
pub struct Semaphorum {
    /// Number of available permits.
    permits: Mutex<usize>,
    /// Condition for waiting acquires.
    cond: Condvar,
}

#[cfg(feature = "std")]
impl Semaphorum {
    /// Create a semaphore with initial permits.
    #[inline]
    pub fn new(permits: usize) -> Self {
        Semaphorum {
            permits: Mutex::new(permits),
            cond: Condvar::new(),
        }
    }

    /// Get the current number of available permits.
    ///
    /// The count is a snapshot: by the time the caller inspects it, other
    /// threads may already have acquired or released permits.
    ///
    /// # Panics
    ///
    /// Panics only if the permits mutex is poisoned. No user code ever runs
    /// while this lock is held, so poisoning indicates a bug in this crate.
    #[inline]
    pub fn available(&self) -> usize {
        *self
            .permits
            .lock()
            .expect("Semaphorum::available: permits mutex poisoned — library invariant violated")
    }

    /// Try to acquire a permit without blocking.
    ///
    /// Returns `true` if a permit was available and has now been taken,
    /// `false` if the count was zero (nothing is taken in that case).
    ///
    /// # Panics
    ///
    /// Panics only if the permits mutex is poisoned. No user code ever runs
    /// while this lock is held, so poisoning indicates a bug in this crate.
    #[inline]
    pub fn try_acquire(&self) -> bool {
        let mut guard = self
            .permits
            .lock()
            .expect("Semaphorum::try_acquire: permits mutex poisoned — library invariant violated");
        if *guard > 0 {
            *guard -= 1;
            true
        } else {
            false
        }
    }

    /// Block until a permit is available.
    ///
    /// Blocks the calling OS thread (a `Condvar` wait, not an async
    /// await-point) until the permit count is non-zero, then takes one
    /// permit. There is no timeout and no fairness guarantee: waiters race
    /// on wake-up, so a late arrival may acquire before a longer-waiting
    /// thread.
    ///
    /// # Panics
    ///
    /// Panics only if the permits mutex or the condition-variable wait is
    /// poisoned. No user code ever runs while this lock is held, so
    /// poisoning indicates a bug in this crate.
    pub fn acquire_blocking(&self) {
        let mut guard = self.permits.lock().expect(
            "Semaphorum::acquire_blocking: permits mutex poisoned — library invariant violated",
        );
        while *guard == 0 {
            guard = self.cond.wait(guard).expect(
                "Semaphorum::acquire_blocking: condvar wait poisoned — library invariant violated",
            );
        }
        *guard -= 1;
    }

    /// Try to acquire multiple permits.
    ///
    /// All-or-nothing: returns `true` and takes exactly `n` permits if at
    /// least `n` are available, otherwise returns `false` and takes none.
    ///
    /// # Panics
    ///
    /// Panics only if the permits mutex is poisoned. No user code ever runs
    /// while this lock is held, so poisoning indicates a bug in this crate.
    #[inline]
    pub fn try_acquire_n(&self, n: usize) -> bool {
        let mut guard = self.permits.lock().expect(
            "Semaphorum::try_acquire_n: permits mutex poisoned — library invariant violated",
        );
        if *guard >= n {
            *guard -= n;
            true
        } else {
            false
        }
    }

    /// Release a permit.
    ///
    /// Increments the permit count and wakes one blocked
    /// [`acquire_blocking`](Self::acquire_blocking) waiter, if any. The
    /// semaphore does not track ownership: releasing without a matching
    /// acquire inflates the count.
    ///
    /// # Panics
    ///
    /// Panics only if the permits mutex is poisoned. No user code ever runs
    /// while this lock is held, so poisoning indicates a bug in this crate.
    #[inline]
    pub fn release(&self) {
        let mut guard = self
            .permits
            .lock()
            .expect("Semaphorum::release: permits mutex poisoned — library invariant violated");
        *guard += 1;
        self.cond.notify_one();
    }

    /// Release multiple permits.
    ///
    /// Increments the permit count by `n` and wakes all blocked waiters
    /// (each re-checks the count, so at most `n` of them proceed). Like
    /// [`release`](Self::release), ownership is not tracked.
    ///
    /// # Panics
    ///
    /// Panics only if the permits mutex is poisoned. No user code ever runs
    /// while this lock is held, so poisoning indicates a bug in this crate.
    #[inline]
    pub fn release_n(&self, n: usize) {
        let mut guard = self
            .permits
            .lock()
            .expect("Semaphorum::release_n: permits mutex poisoned — library invariant violated");
        *guard += n;
        self.cond.notify_all();
    }
}

/// A permit acquired from a semaphore.
///
/// Automatically releases when dropped.
#[cfg(feature = "std")]
pub struct Permissum<'a> {
    semaphore: &'a Semaphorum,
}

#[cfg(feature = "std")]
impl<'a> Permissum<'a> {
    /// Acquire a permit from `semaphore`, blocking until one is available.
    ///
    /// The returned guard holds exactly the one permit it acquired; it is
    /// returned to the semaphore when the guard is dropped (or via
    /// [`release`](Self::release)). Previously this constructor did not
    /// acquire anything — only `Drop` released — so every `Permissum`
    /// created and dropped inflated the semaphore's permit count by one.
    #[inline]
    pub fn new(semaphore: &'a Semaphorum) -> Self {
        semaphore.acquire_blocking();
        Permissum { semaphore }
    }

    /// Manually release the permit.
    #[inline]
    pub fn release(self) {
        // Drop will handle release
    }
}

#[cfg(feature = "std")]
impl Drop for Permissum<'_> {
    fn drop(&mut self) {
        self.semaphore.release();
    }
}

// =============================================================================
// MVarSync - Synchronized Mutable Variable
// =============================================================================

/// A synchronized mutable variable.
///
/// `MVarSync<A>` is a mutable location that can be empty or full.
/// Taking from an empty `MVar` blocks; putting to a full `MVar` blocks.
///
/// # Latin Etymology
/// *Varia Mutabilis Synchrona* = synchronized mutable variable.
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::concurrent::MVarSync;
///
/// let mvar = MVarSync::<i32>::new_empty();
///
/// // Put a value (blocks if already full).
/// mvar.put_blocking(42);
///
/// // Take the value (blocks if empty).
/// let value = mvar.take_blocking();
/// assert_eq!(value, 42);
/// ```
#[cfg(feature = "std")]
pub struct MVarSync<A> {
    /// The stored value.
    value: Mutex<Option<A>>,
    /// Condition for waiting operations.
    cond: Condvar,
}

#[cfg(feature = "std")]
impl<A> MVarSync<A> {
    /// Create a new empty `MVar`.
    #[inline]
    pub fn new_empty() -> Self {
        MVarSync {
            value: Mutex::new(None),
            cond: Condvar::new(),
        }
    }

    /// Create a new `MVar` with an initial value.
    #[inline]
    pub fn new(a: A) -> Self {
        MVarSync {
            value: Mutex::new(Some(a)),
            cond: Condvar::new(),
        }
    }

    /// Check if the `MVar` is empty.
    ///
    /// The answer is a snapshot: another thread may put or take between
    /// this check and any subsequent operation.
    ///
    /// # Panics
    ///
    /// Panics if the value mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent
    /// [`read_blocking`](Self::read_blocking) call; otherwise it indicates
    /// a bug in this crate.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value
            .lock()
            .expect("MVarSync::is_empty: value mutex poisoned — library invariant violated")
            .is_none()
    }

    /// Try to take a value without blocking.
    ///
    /// Returns `Some(value)` and leaves the `MVar` empty if it was full,
    /// `None` if it was already empty. On success, one blocked
    /// [`put_blocking`](Self::put_blocking) waiter is woken.
    ///
    /// # Panics
    ///
    /// Panics if the value mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent
    /// [`read_blocking`](Self::read_blocking) call; otherwise it indicates
    /// a bug in this crate.
    #[inline]
    pub fn try_take(&self) -> Option<A> {
        let mut guard = self
            .value
            .lock()
            .expect("MVarSync::try_take: value mutex poisoned — library invariant violated");
        let result = guard.take();
        if result.is_some() {
            self.cond.notify_one();
        }
        result
    }

    /// Block until a value is available and take it.
    ///
    /// Blocks the calling OS thread (a `Condvar` wait, not an async
    /// await-point) until the `MVar` is full, removes the value (leaving
    /// the `MVar` empty), and wakes one blocked putter. There is no
    /// timeout and no FIFO fairness among competing takers.
    ///
    /// # Panics
    ///
    /// Panics if the value mutex or the condition-variable wait is
    /// poisoned — reachable only via `A::clone` panicking inside a
    /// concurrent [`read_blocking`](Self::read_blocking) call — or if the
    /// internal "full after wait" invariant is violated, which indicates a
    /// bug in this crate.
    pub fn take_blocking(&self) -> A {
        let mut guard = self
            .value
            .lock()
            .expect("MVarSync::take_blocking: value mutex poisoned — library invariant violated");
        while guard.is_none() {
            guard = self.cond.wait(guard).expect(
                "MVarSync::take_blocking: condvar wait poisoned — library invariant violated",
            );
        }
        let result = guard
            .take()
            .expect("MVarSync::take_blocking: value must be Some after wait loop — invariant bug");
        self.cond.notify_one();
        result
    }

    /// Try to put a value without blocking.
    ///
    /// Returns `true` and stores `a` if the `MVar` was empty (waking one
    /// blocked taker), `false` if it was already full — in which case `a`
    /// is dropped.
    ///
    /// # Panics
    ///
    /// Panics if the value mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent
    /// [`read_blocking`](Self::read_blocking) call; otherwise it indicates
    /// a bug in this crate.
    #[inline]
    pub fn try_put(&self, a: A) -> bool {
        let mut guard = self
            .value
            .lock()
            .expect("MVarSync::try_put: value mutex poisoned — library invariant violated");
        if guard.is_none() {
            *guard = Some(a);
            self.cond.notify_one();
            true
        } else {
            false
        }
    }

    /// Block until the `MVar` is empty and put a value.
    ///
    /// Blocks the calling OS thread (a `Condvar` wait, not an async
    /// await-point) until the `MVar` is empty, stores `a`, and wakes one
    /// blocked taker/reader. There is no timeout and no FIFO fairness
    /// among competing putters.
    ///
    /// # Panics
    ///
    /// Panics if the value mutex or the condition-variable wait is
    /// poisoned, which can only happen if `A::clone` panicked inside a
    /// concurrent [`read_blocking`](Self::read_blocking) call; otherwise
    /// it indicates a bug in this crate.
    pub fn put_blocking(&self, a: A) {
        let mut guard = self
            .value
            .lock()
            .expect("MVarSync::put_blocking: value mutex poisoned — library invariant violated");
        while guard.is_some() {
            guard = self.cond.wait(guard).expect(
                "MVarSync::put_blocking: condvar wait poisoned — library invariant violated",
            );
        }
        *guard = Some(a);
        self.cond.notify_one();
    }

    /// Read the value without taking it.
    ///
    /// Blocks the calling OS thread until the `MVar` is full, then returns
    /// a clone of the value, leaving the `MVar` still full. There is no
    /// timeout.
    ///
    /// # Panics
    ///
    /// Panics if the value mutex or the condition-variable wait is
    /// poisoned. `A::clone` runs while the lock is held here, so a
    /// panicking `Clone` implementation poisons the `MVar` for all later
    /// callers; absent that, poisoning (or the internal "full after wait"
    /// invariant failing) indicates a bug in this crate.
    pub fn read_blocking(&self) -> A
    where
        A: Clone,
    {
        let mut guard = self
            .value
            .lock()
            .expect("MVarSync::read_blocking: value mutex poisoned — library invariant violated");
        while guard.is_none() {
            guard = self.cond.wait(guard).expect(
                "MVarSync::read_blocking: condvar wait poisoned — library invariant violated",
            );
        }
        guard
            .clone()
            .expect("MVarSync::read_blocking: value must be Some after wait loop — invariant bug")
    }

    /// Swap the value atomically.
    ///
    /// Blocks the calling OS thread until the `MVar` is full, then replaces
    /// the stored value with `a` and returns the old value. The take and
    /// put happen in one critical section, so no other thread can observe
    /// the `MVar` empty in between.
    ///
    /// # Panics
    ///
    /// Panics if the value mutex or the condition-variable wait is
    /// poisoned — reachable only via `A::clone` panicking inside a
    /// concurrent [`read_blocking`](Self::read_blocking) call — or if the
    /// internal "full after wait" invariant is violated, which indicates a
    /// bug in this crate.
    pub fn swap_blocking(&self, a: A) -> A {
        let mut guard = self
            .value
            .lock()
            .expect("MVarSync::swap_blocking: value mutex poisoned — library invariant violated");
        while guard.is_none() {
            guard = self.cond.wait(guard).expect(
                "MVarSync::swap_blocking: condvar wait poisoned — library invariant violated",
            );
        }
        let old = guard
            .take()
            .expect("MVarSync::swap_blocking: value must be Some after wait loop — invariant bug");
        *guard = Some(a);
        self.cond.notify_one();
        old
    }
}

// =============================================================================
// CaudaBackpressure - Bounded Queue with Backpressure
// =============================================================================

/// A bounded queue with backpressure.
///
/// `CaudaBackpressure<A>` provides a FIFO queue with a maximum capacity.
/// Offering to a full queue blocks; taking from an empty queue blocks.
///
/// # Latin Etymology
/// *Cauda* = tail (referring to FIFO order).
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::concurrent::CaudaBackpressure;
///
/// let queue = CaudaBackpressure::<i32>::new(10); // capacity 10
///
/// // Producer
/// queue.offer_blocking(42);
///
/// // Consumer
/// let item = queue.take_blocking();
/// assert_eq!(item, 42);
/// ```
#[cfg(feature = "std")]
pub struct CaudaBackpressure<A> {
    /// Maximum capacity.
    capacity: usize,
    /// The buffer.
    buffer: Mutex<VecDeque<A>>,
    /// Condition for consumers.
    not_empty: Condvar,
    /// Condition for producers.
    not_full: Condvar,
}

#[cfg(feature = "std")]
impl<A> CaudaBackpressure<A> {
    /// Create a new bounded queue.
    #[inline]
    pub fn new(capacity: usize) -> Self {
        CaudaBackpressure {
            capacity,
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    /// Get the capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the current size.
    ///
    /// The count is a snapshot: producers and consumers may change it
    /// before the caller acts on the answer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent [`peek`](Self::peek) call;
    /// otherwise it indicates a bug in this crate.
    #[inline]
    pub fn size(&self) -> usize {
        self.buffer
            .lock()
            .expect("CaudaBackpressure::size: buffer mutex poisoned — library invariant violated")
            .len()
    }

    /// Check if empty.
    ///
    /// A snapshot only — see [`size`](Self::size).
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent [`peek`](Self::peek) call;
    /// otherwise it indicates a bug in this crate.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer
            .lock()
            .expect(
                "CaudaBackpressure::is_empty: buffer mutex poisoned — library invariant violated",
            )
            .is_empty()
    }

    /// Check if full (size has reached capacity).
    ///
    /// A snapshot only — see [`size`](Self::size).
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent [`peek`](Self::peek) call;
    /// otherwise it indicates a bug in this crate.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.buffer
            .lock()
            .expect(
                "CaudaBackpressure::is_full: buffer mutex poisoned — library invariant violated",
            )
            .len()
            >= self.capacity
    }

    /// Try to offer without blocking.
    ///
    /// Returns `true` and enqueues `a` at the tail (waking one blocked
    /// consumer) if the queue is below capacity, `false` if it is full —
    /// in which case `a` is dropped.
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent [`peek`](Self::peek) call;
    /// otherwise it indicates a bug in this crate.
    #[inline]
    pub fn try_offer(&self, a: A) -> bool {
        let mut guard = self.buffer.lock().expect(
            "CaudaBackpressure::try_offer: buffer mutex poisoned — library invariant violated",
        );
        if guard.len() < self.capacity {
            guard.push_back(a);
            self.not_empty.notify_one();
            true
        } else {
            false
        }
    }

    /// Block until space is available and offer.
    ///
    /// This is the backpressure edge: it blocks the calling OS thread (a
    /// `Condvar` wait, not an async await-point) while the queue is at
    /// capacity, then enqueues `a` at the tail and wakes one blocked
    /// consumer. There is no timeout and no FIFO fairness among competing
    /// producers.
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex or the condition-variable wait is
    /// poisoned, which can only happen if `A::clone` panicked inside a
    /// concurrent [`peek`](Self::peek) call; otherwise it indicates a bug
    /// in this crate.
    pub fn offer_blocking(&self, a: A) {
        let mut guard = self.buffer.lock().expect(
            "CaudaBackpressure::offer_blocking: buffer mutex poisoned — library invariant violated",
        );
        while guard.len() >= self.capacity {
            guard = self.not_full.wait(guard).expect(
                "CaudaBackpressure::offer_blocking: not_full condvar poisoned — library invariant violated",
            );
        }
        guard.push_back(a);
        self.not_empty.notify_one();
    }

    /// Try to take without blocking.
    ///
    /// Returns `Some` with the head (oldest) item if the queue is
    /// non-empty, waking one blocked producer; `None` if it is empty.
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent [`peek`](Self::peek) call;
    /// otherwise it indicates a bug in this crate.
    #[inline]
    pub fn try_take(&self) -> Option<A> {
        let mut guard = self.buffer.lock().expect(
            "CaudaBackpressure::try_take: buffer mutex poisoned — library invariant violated",
        );
        let result = guard.pop_front();
        if result.is_some() {
            self.not_full.notify_one();
        }
        result
    }

    /// Block until an item is available and take it.
    ///
    /// Blocks the calling OS thread (a `Condvar` wait, not an async
    /// await-point) while the queue is empty, then removes and returns the
    /// head (oldest) item and wakes one blocked producer. There is no
    /// timeout and no FIFO fairness among competing consumers.
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex or the condition-variable wait is
    /// poisoned — reachable only via `A::clone` panicking inside a
    /// concurrent [`peek`](Self::peek) call — or if the internal
    /// "non-empty after wait" invariant is violated, which indicates a bug
    /// in this crate.
    pub fn take_blocking(&self) -> A {
        let mut guard = self.buffer.lock().expect(
            "CaudaBackpressure::take_blocking: buffer mutex poisoned — library invariant violated",
        );
        while guard.is_empty() {
            guard = self.not_empty.wait(guard).expect(
                "CaudaBackpressure::take_blocking: not_empty condvar poisoned — library invariant violated",
            );
        }
        let result = guard.pop_front().expect(
            "CaudaBackpressure::take_blocking: buffer must be non-empty after wait loop — invariant bug",
        );
        self.not_full.notify_one();
        result
    }

    /// Peek at the front without removing.
    ///
    /// Returns a clone of the head (oldest) item, or `None` if the queue
    /// is empty. The item stays in the queue, so another consumer may
    /// still take it.
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex is poisoned. `A::clone` runs while the
    /// lock is held here, so a panicking `Clone` implementation poisons
    /// the queue for all later callers; absent that, poisoning indicates
    /// a bug in this crate.
    #[inline]
    pub fn peek(&self) -> Option<A>
    where
        A: Clone,
    {
        let guard = self
            .buffer
            .lock()
            .expect("CaudaBackpressure::peek: buffer mutex poisoned — library invariant violated");
        guard.front().cloned()
    }

    /// Drain all items.
    ///
    /// Atomically removes every queued item and returns them in FIFO
    /// order, leaving the queue empty and waking all blocked producers.
    ///
    /// # Panics
    ///
    /// Panics if the buffer mutex is poisoned, which can only happen if
    /// `A::clone` panicked inside a concurrent [`peek`](Self::peek) call;
    /// otherwise it indicates a bug in this crate.
    #[inline]
    pub fn drain(&self) -> Vec<A> {
        let mut guard = self
            .buffer
            .lock()
            .expect("CaudaBackpressure::drain: buffer mutex poisoned — library invariant violated");
        let mut result = Vec::with_capacity(guard.len());
        result.extend(guard.drain(..));
        self.not_full.notify_all();
        result
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_referentia_new() {
        let r = Referentia::new(42);
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_referentia_set() {
        let r = Referentia::new(0);
        let old = r.set(42);
        assert_eq!(old, 0);
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_referentia_update() {
        let r = Referentia::new(21);
        r.update(|x| x * 2);
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_referentia_modify() {
        let r = Referentia::new(21);
        let result = r.modify(|x| (x * 2, "done"));
        assert_eq!(result, "done");
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn panicking_update_does_not_poison_forever() {
        let r = Referentia::new(1i32);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            r.update(|_| -> i32 { panic!("boom") });
        }));
        // Pre-fix: this second call panics "mutex poisoned".
        assert_eq!(r.get(), 1);
    }

    #[test]
    fn test_semaphorum_new() {
        let sem = Semaphorum::new(3);
        assert_eq!(sem.available(), 3);
    }

    #[test]
    fn test_semaphorum_try_acquire() {
        let sem = Semaphorum::new(1);
        assert!(sem.try_acquire());
        assert!(!sem.try_acquire());
        sem.release();
        assert!(sem.try_acquire());
    }

    #[test]
    fn test_semaphorum_release_n() {
        let sem = Semaphorum::new(0);
        sem.release_n(5);
        assert_eq!(sem.available(), 5);
    }

    #[test]
    fn test_mvar_sync_new_empty() {
        let mvar: MVarSync<i32> = MVarSync::new_empty();
        assert!(mvar.is_empty());
    }

    #[test]
    fn test_mvar_sync_new() {
        let mvar = MVarSync::new(42);
        assert!(!mvar.is_empty());
    }

    #[test]
    fn test_mvar_sync_try_put_take() {
        let mvar: MVarSync<i32> = MVarSync::new_empty();
        assert!(mvar.try_put(42));
        assert!(!mvar.try_put(43)); // Should fail, already full
        assert_eq!(mvar.try_take(), Some(42));
        assert!(mvar.is_empty());
    }

    #[test]
    fn test_cauda_new() {
        let queue: CaudaBackpressure<i32> = CaudaBackpressure::new(10);
        assert_eq!(queue.capacity(), 10);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_cauda_try_offer_take() {
        let queue = CaudaBackpressure::new(2);
        assert!(queue.try_offer(1));
        assert!(queue.try_offer(2));
        assert!(!queue.try_offer(3)); // Full

        assert_eq!(queue.try_take(), Some(1));
        assert_eq!(queue.try_take(), Some(2));
        assert_eq!(queue.try_take(), None);
    }

    #[test]
    fn test_cauda_drain() {
        let queue = CaudaBackpressure::new(10);
        queue.try_offer(1);
        queue.try_offer(2);
        queue.try_offer(3);

        let items = queue.drain();
        assert_eq!(items, vec![1, 2, 3]);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_dilatum_complete() {
        let deferred = Dilatum::<i32>::new();
        assert!(!deferred.is_completed());

        assert!(deferred.complete(42));
        assert!(deferred.is_completed());

        assert!(!deferred.complete(43)); // Already completed
        assert_eq!(deferred.try_get(), Some(42));
    }

    #[test]
    fn test_dilatum_concurrent_complete_runs_exactly_once() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicU32, Ordering};
        use std::thread;

        // Regression test for the completed-flag/value TOCTOU: with the old
        // AtomicBool + separate Mutex<Option<A>> representation, the flag
        // could flip to "completed" before the value was written, letting a
        // racing reader observe a torn state. Racing many threads through
        // `complete` must still yield exactly one winner and a consistent
        // final value under the new single-mutex `Status<A>`.
        let deferred = Arc::new(Dilatum::<u32>::new());
        let counter = Arc::new(AtomicU32::new(0));

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let deferred = Arc::clone(&deferred);
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    let value = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    deferred.complete(value)
                })
            })
            .collect();

        let wins = handles
            .into_iter()
            .map(|h| h.join().expect("thread panicked"))
            .filter(|&won| won)
            .count();

        assert_eq!(wins, 1, "exactly one complete() call must win the race");
        assert!(deferred.is_completed());
        assert!(
            deferred.try_get().is_some(),
            "completed deferred must never yield a torn (empty) read"
        );
    }

    #[test]
    fn test_dilatum_no_torn_read_under_race() {
        use alloc::sync::Arc;
        use std::thread;

        // Stress the is_completed()/try_get() pair concurrently with racing
        // completions: under the old flag+mutex split, a reader could see
        // is_completed() == true while try_get() still returned None.
        let deferred = Arc::new(Dilatum::<u32>::new());

        let writers: Vec<_> = (0..4)
            .map(|i| {
                let deferred = Arc::clone(&deferred);
                thread::spawn(move || {
                    deferred.complete(i);
                })
            })
            .collect();

        let reader_deferred = Arc::clone(&deferred);
        let reader = thread::spawn(move || {
            for _ in 0..5_000 {
                if reader_deferred.is_completed() {
                    assert!(
                        reader_deferred.try_get().is_some(),
                        "torn read: is_completed() true but try_get() returned None"
                    );
                }
            }
        });

        for h in writers {
            h.join().expect("writer thread panicked");
        }
        reader.join().expect("reader thread panicked");
    }

    #[test]
    fn test_permissum_new_consumes_exactly_one_permit() {
        // Regression test for permit inflation: `Permissum::new` used to
        // only store a reference without acquiring, while `Drop` still
        // released — so every construct+drop cycle added a phantom permit.
        let sem = Semaphorum::new(2);
        assert_eq!(sem.available(), 2);

        let permit = Permissum::new(&sem);
        assert_eq!(
            sem.available(),
            1,
            "Permissum::new must consume exactly one permit"
        );

        drop(permit);
        assert_eq!(
            sem.available(),
            2,
            "releasing the permit must restore the original count exactly, no inflation"
        );
    }

    #[test]
    fn test_permissum_new_admits_exactly_requested_concurrent_holders() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
        use std::thread;
        use std::time::Duration;

        let sem = Arc::new(Semaphorum::new(2));
        let current_holders = Arc::new(AtomicI64::new(0));
        let max_holders = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let sem = Arc::clone(&sem);
                let current_holders = Arc::clone(&current_holders);
                let max_holders = Arc::clone(&max_holders);
                thread::spawn(move || {
                    let _permit = Permissum::new(&sem); // blocks until a permit frees up
                    let now = current_holders.fetch_add(1, Ordering::SeqCst) + 1;
                    max_holders.fetch_max(now as usize, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(20));
                    current_holders.fetch_sub(1, Ordering::SeqCst);
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread panicked");
        }

        assert_eq!(
            max_holders.load(Ordering::SeqCst),
            2,
            "Semaphorum::new(2) must admit exactly 2 concurrent Permissum holders"
        );
        assert_eq!(
            sem.available(),
            2,
            "all permits must be returned, no inflation"
        );
    }
}
