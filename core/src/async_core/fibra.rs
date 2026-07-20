//! Fiber-Based Concurrency - Fibra
//!
//! > *"Fibra est filum subtile"*
//! > — A fiber is a fine thread. (Latin)
//!
//! This module provides lightweight fiber abstractions for structured concurrency,
//! inspired by ZIO and Cats Effect's fiber-based concurrency models.
//!
//! # Overview
//!
//! Fibers are lightweight virtual threads that can be spawned, forked, joined,
//! and composed. Unlike OS threads, fibers are cheap to create and can be
//! used liberally for concurrent computations.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Fiber | Fibra | *fibra* = fiber, filament |
//! | Handle | Manubrium | *manubrium* = handle, grip |
//! | Fork | Furca | *furca* = fork, pitchfork |
//! | Race | Certamen | *certamen* = contest, race |
//! | Supervisor | Praefectus | *praefectus* = overseer |
//! | Cancel | Abrogare | *abrogare* = to cancel, annul |
//!
//! # Example
//!
//! ```rust
//! # use core::future::Future;
//! # use core::pin::Pin;
//! # use core::task::{Context, Poll, Waker};
//! #
//! # fn block_on<F: Future>(fut: F) -> F::Output {
//! #     let mut fut = Box::pin(fut);
//! #     let mut cx = Context::from_waker(Waker::noop());
//! #     loop {
//! #         if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
//! #             return out;
//! #         }
//! #     }
//! # }
//! #
//! use ordofp_core::async_core::fibra::Fibra;
//!
//! let fiber = Fibra::new(async { 42 });
//!
//! // Wait for the result
//! let result = block_on(fiber);
//! assert_eq!(result.unwrap(), 42);
//! ```

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::task::{Context, Poll};

use super::runtime::{JoinError, JoinManubrium, RuntimeGenerare};

// =============================================================================
// TesseraAbrogationis - Cancellation Token
// =============================================================================

/// Shared cancellation token: a flag plus the parked task's waker, so that
/// `abrogare` actually re-polls a fiber suspended on an inner `Pending`.
///
/// # Latin Etymology
/// *Tessera abrogationis* = token of cancellation.
#[derive(Debug, Default)]
pub struct TesseraAbrogationis {
    flag: AtomicBool,
    waker: std::sync::Mutex<Option<core::task::Waker>>,
}

impl TesseraAbrogationis {
    /// Set the flag and wake the fiber (if parked).
    pub fn abrogare(&self) {
        // Release: pairs with the Acquire load in est_abrogata().
        self.flag.store(true, Ordering::Release);
        let waker = self
            .waker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(w) = waker {
            w.wake();
        }
    }

    /// Check whether cancellation has been requested.
    ///
    /// An `Acquire` load pairing with the `Release` store in
    /// [`abrogare`](Self::abrogare): once this returns `true` it never
    /// reverts. This is a request flag only — the fiber may still be
    /// running until it next observes the flag at a poll point.
    #[inline]
    pub fn est_abrogata(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    /// Park the current task's waker. Called by `Fibra::poll` BEFORE checking
    /// the flag — that ordering makes the wakeup race-free: a cancel arriving
    /// after registration and after the flag check still wakes this waker.
    pub(crate) fn registrare(&self, cx: &Context<'_>) {
        *self
            .waker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cx.waker().clone());
    }
}

// =============================================================================
// FibraId - Unique Fiber Identifier
// =============================================================================

/// Unique identifier for a fiber.
///
/// Each fiber is assigned a unique ID when created.
static FIBRA_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique fiber identifier.
///
/// # Latin Etymology
/// *Fibra* + *Id* = fiber identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FibraId(u64);

impl FibraId {
    /// Generate a new unique fiber ID.
    #[inline]
    pub(crate) fn new() -> Self {
        // Relaxed: monotonic counter, no other atomic depends on its order
        FibraId(FIBRA_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value.
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for FibraId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Fibra({})", self.0)
    }
}

// =============================================================================
// FibraStatus - Fiber Execution Status
// =============================================================================

/// The current status of a fiber.
///
/// # Latin Etymology
/// *Status* = standing, state, condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FibraStatus {
    /// Fiber is waiting to be scheduled.
    ///
    /// *Pendens* = pending, waiting.
    Pendens,
    /// Fiber is currently running.
    ///
    /// *Currens* = running.
    Currens,
    /// Fiber has completed successfully.
    ///
    /// *Perfectus* = completed, finished.
    Perfectus,
    /// Fiber has failed with an error.
    ///
    /// *Defectus* = failed.
    Defectus,
    /// Fiber was cancelled.
    ///
    /// *Abrogatus* = cancelled.
    Abrogatus,
}

// =============================================================================
// FibraError - Fiber Error Type
// =============================================================================

/// Error type for fiber operations.
///
/// # Latin Etymology
/// *Error* = wandering, mistake.
#[derive(Debug, Clone)]
pub enum FibraError {
    /// The fiber was cancelled.
    Abrogatus,
    /// The fiber panicked.
    Panic(String),
    /// The fiber timed out.
    TemporisExcessus,
    /// A child fiber failed.
    InfansDefectus(Box<FibraError>),
    /// Other error.
    Alius(String),
}

impl core::fmt::Display for FibraError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FibraError::Abrogatus => write!(f, "fiber was cancelled"),
            FibraError::Panic(msg) => write!(f, "fiber panicked: {msg}"),
            FibraError::TemporisExcessus => write!(f, "fiber timed out"),
            FibraError::InfansDefectus(e) => write!(f, "child fiber failed: {e}"),
            FibraError::Alius(msg) => write!(f, "fiber error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FibraError {}

// =============================================================================
// FibraExitus - Fiber Outcome
// =============================================================================

/// The outcome of a fiber execution.
///
/// # Latin Etymology
/// *Exitus* = outcome, result, exit.
pub type FibraExitus<A> = Result<A, FibraError>;

// =============================================================================
// Fibra - The Core Fiber Type
// =============================================================================

/// A lightweight fiber abstraction for concurrent computations.
///
/// `Fibra` wraps a future and provides fiber-like semantics including
/// cancellation, composition, and structured concurrency.
///
/// # Latin Etymology
/// *Fibra* = fiber, filament, thread.
///
/// # Type Parameters
///
/// - `A`: The output type of the fiber
///
/// # Example
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # use core::task::{Context, Poll, Waker};
/// #
/// # fn block_on<F: Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = Context::from_waker(Waker::noop());
/// #     loop {
/// #         if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
/// #
/// use ordofp_core::async_core::fibra::Fibra;
///
/// let fiber = Fibra::purus(42);
/// let result = block_on(fiber); // FibraExitus<i32>
/// assert_eq!(result.unwrap(), 42);
/// ```
pub struct Fibra<A> {
    id: FibraId,
    inner: Pin<Box<dyn Future<Output = FibraExitus<A>> + Send>>,
    cancelled: Arc<TesseraAbrogationis>,
}

impl<A> Fibra<A> {
    /// Create a new fiber from a future.
    ///
    /// # Latin Etymology
    /// *Novus* = new.
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = A> + Send + 'static,
        A: Send + 'static,
    {
        let cancelled = Arc::new(TesseraAbrogationis::default());
        let cancelled_clone = cancelled.clone();

        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move {
                if cancelled_clone.est_abrogata() {
                    return Err(FibraError::Abrogatus);
                }
                Ok(future.await)
            }),
            cancelled,
        }
    }

    /// Create a fiber that immediately yields a value (pure/return).
    ///
    /// # Latin Etymology
    /// *Purus* = pure, clean.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::async_core::fibra::Fibra;
    ///
    /// let fiber: Fibra<i32> = Fibra::purus(42);
    /// assert!(!fiber.is_cancelled());
    /// ```
    pub fn purus(value: A) -> Self
    where
        A: Send + 'static,
    {
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move { Ok(value) }),
            cancelled: Arc::new(TesseraAbrogationis::default()),
        }
    }

    /// Create a fiber that immediately fails with an error.
    ///
    /// # Latin Etymology
    /// *Deficere* = to fail.
    pub fn deficere(error: FibraError) -> Self
    where
        A: Send + 'static,
    {
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move { Err(error) }),
            cancelled: Arc::new(TesseraAbrogationis::default()),
        }
    }

    /// Get the fiber's unique ID.
    #[inline]
    pub fn id(&self) -> FibraId {
        self.id
    }

    /// Check if cancellation has been requested.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.est_abrogata()
    }

    /// Get the cancellation token for this fiber.
    #[inline]
    pub fn cancellation_token(&self) -> Arc<TesseraAbrogationis> {
        self.cancelled.clone()
    }

    /// Map over the fiber's output.
    ///
    /// # Latin Etymology
    /// *Mutare* = to change, transform.
    #[inline]
    pub fn mutare<B, F>(self, f: F) -> Fibra<B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        A: Send + 'static,
        B: Send + 'static,
    {
        let cancelled = self.cancelled.clone();
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move {
                let result = self.await;
                result.map(f)
            }),
            cancelled,
        }
    }

    /// Alias for `mutare`.
    #[inline]
    pub fn map<B, F>(self, f: F) -> Fibra<B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        A: Send + 'static,
        B: Send + 'static,
    {
        self.mutare(f)
    }

    /// Flat map (bind) over the fiber's output.
    ///
    /// # Latin Etymology
    /// *Ligare* = to bind.
    #[inline]
    pub fn ligare<B, F>(self, f: F) -> Fibra<B>
    where
        F: FnOnce(A) -> Fibra<B> + Send + 'static,
        A: Send + 'static,
        B: Send + 'static,
    {
        let cancelled = self.cancelled.clone();
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move {
                let result = self.await;
                match result {
                    Ok(a) => f(a).await,
                    Err(e) => Err(e),
                }
            }),
            cancelled,
        }
    }

    /// Alias for `ligare`.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Fibra<B>
    where
        F: FnOnce(A) -> Fibra<B> + Send + 'static,
        A: Send + 'static,
        B: Send + 'static,
    {
        self.ligare(f)
    }

    /// Apply a wrapped function to this fiber.
    ///
    /// # Latin Etymology
    /// *Applicare* = to apply.
    pub fn applicare<B, F>(self, ff: Fibra<F>) -> Fibra<B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        A: Send + 'static,
        B: Send + 'static,
    {
        let cancelled = self.cancelled.clone();
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move {
                let f_result = ff.await;
                let a_result = self.await;
                match (f_result, a_result) {
                    (Ok(f), Ok(a)) => Ok(f(a)),
                    (Err(e), _) => Err(e),
                    (_, Err(e)) => Err(e),
                }
            }),
            cancelled,
        }
    }

    /// Handle errors by transforming them into a success value.
    ///
    /// # Latin Etymology
    /// *Recuperare* = to recover.
    pub fn recuperare<F>(self, handler: F) -> Fibra<A>
    where
        F: FnOnce(FibraError) -> A + Send + 'static,
        A: Send + 'static,
    {
        let cancelled = self.cancelled.clone();
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move {
                let result = self.await;
                Ok(result.unwrap_or_else(handler))
            }),
            cancelled,
        }
    }

    /// Handle errors by transforming them into another fiber.
    ///
    /// # Latin Etymology
    /// *Recuperare* + *Fibra* = recover with fiber.
    pub fn recuperare_cum<F>(self, handler: F) -> Fibra<A>
    where
        F: FnOnce(FibraError) -> Fibra<A> + Send + 'static,
        A: Send + 'static,
    {
        let cancelled = self.cancelled.clone();
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move {
                let result = self.await;
                match result {
                    Ok(a) => Ok(a),
                    Err(e) => handler(e).await,
                }
            }),
            cancelled,
        }
    }

    /// Ensure a finalizer runs regardless of outcome.
    ///
    /// # Latin Etymology
    /// *Assecurare* = to ensure, make secure.
    pub fn assecurare<F, Fut>(self, finalizer: F) -> Fibra<A>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        A: Send + 'static,
    {
        let cancelled = self.cancelled.clone();
        Fibra {
            id: FibraId::new(),
            inner: Box::pin(async move {
                let result = self.await;
                finalizer().await;
                result
            }),
            cancelled,
        }
    }
}

impl<A> Future for Fibra<A> {
    type Output = FibraExitus<A>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.cancelled.registrare(cx);
        if self.cancelled.est_abrogata() {
            return Poll::Ready(Err(FibraError::Abrogatus));
        }
        self.inner.as_mut().poll(cx)
    }
}

// =============================================================================
// FibraManubrium - Fiber Handle
// =============================================================================

/// A handle to a spawned fiber for controlling and observing it.
///
/// `FibraManubrium` provides methods to join (wait for completion),
/// cancel, or query the status of a spawned fiber.
///
/// # Latin Etymology
/// *Manubrium* = handle, grip.
pub struct FibraManubrium<A> {
    id: FibraId,
    join_handle: JoinManubrium<FibraExitus<A>>,
    cancelled: Arc<TesseraAbrogationis>,
}

impl<A> FibraManubrium<A> {
    /// Create a new fiber handle.
    #[inline]
    pub fn new(
        id: FibraId,
        join_handle: JoinManubrium<FibraExitus<A>>,
        cancelled: Arc<TesseraAbrogationis>,
    ) -> Self {
        FibraManubrium {
            id,
            join_handle,
            cancelled,
        }
    }

    /// Get the fiber's unique ID.
    #[inline]
    pub fn id(&self) -> FibraId {
        self.id
    }

    /// Request cancellation of the fiber.
    ///
    /// This sets the cancellation flag, but the fiber may not stop immediately.
    /// Use `abrogare_et_conjungere` to cancel and wait for completion.
    ///
    /// # Latin Etymology
    /// *Abrogare* = to cancel, repeal, annul.
    #[inline]
    pub fn abrogare(&self) {
        self.cancelled.abrogare();
    }

    /// Alias for `abrogare`.
    #[inline]
    pub fn cancel(&self) {
        self.abrogare();
    }

    /// Check if cancellation has been requested.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.est_abrogata()
    }

    /// Cancel and wait for the fiber to complete.
    ///
    /// # Errors
    ///
    /// The usual outcome is `Err(FibraError::Abrogatus)`, since the fiber
    /// is cancelled before joining — but if the fiber had already finished
    /// when the cancellation flag was set, its own result (success or
    /// error) is returned instead. Runtime-level join failures map as in
    /// [`conjungere`](Self::conjungere).
    ///
    /// # Latin Etymology
    /// *Abrogare et conjungere* = cancel and join.
    pub async fn abrogare_et_conjungere(self) -> FibraExitus<A> {
        self.abrogare();
        self.conjungere().await
    }

    /// Wait for the fiber to complete.
    ///
    /// # Errors
    ///
    /// Propagates whatever error the fiber itself completed with (for
    /// example `FibraError::Abrogatus` if it observed cancellation).
    /// Runtime-level join failures are mapped onto the same error type:
    /// `FibraError::Abrogatus` if the runtime cancelled the task,
    /// `FibraError::Panic` if the fiber panicked, and `FibraError::Alius`
    /// for any other runtime-reported failure.
    ///
    /// # Latin Etymology
    /// *Conjungere* = to join together.
    #[inline]
    pub async fn conjungere(self) -> FibraExitus<A> {
        match self.join_handle.await {
            Ok(result) => result,
            Err(JoinError::Cancelled) => Err(FibraError::Abrogatus),
            Err(JoinError::Panic(msg)) => Err(FibraError::Panic(msg)),
            Err(JoinError::Other(msg)) => Err(FibraError::Alius(msg)),
        }
    }

    /// Alias for `conjungere`.
    ///
    /// # Errors
    ///
    /// Identical to [`conjungere`](Self::conjungere): the fiber's own
    /// error, or `Abrogatus`/`Panic`/`Alius` for runtime-level join
    /// failures.
    #[inline]
    pub async fn join(self) -> FibraExitus<A> {
        self.conjungere().await
    }

    /// Poll the fiber's completion without consuming the handle.
    ///
    /// Unlike [`conjungere`](Self::conjungere), which takes `self`, this borrows
    /// the handle so a supervisor can watch N children in a single `poll_fn`
    /// without giving up its handles. Registers the current task's waker with
    /// the underlying join handle, so the caller is woken when the child exits.
    ///
    /// # Latin Etymology
    /// *Conjunctio* = a joining, connection.
    pub fn poll_conjunctio(&mut self, cx: &mut Context<'_>) -> Poll<FibraExitus<A>> {
        match Pin::new(&mut self.join_handle).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(JoinError::Cancelled)) => Poll::Ready(Err(FibraError::Abrogatus)),
            Poll::Ready(Err(JoinError::Panic(msg))) => Poll::Ready(Err(FibraError::Panic(msg))),
            Poll::Ready(Err(JoinError::Other(msg))) => Poll::Ready(Err(FibraError::Alius(msg))),
            Poll::Pending => Poll::Pending,
        }
    }
}

// =============================================================================
// Structured Concurrency Combinators
// =============================================================================

/// Run two fibers in parallel and return both results.
///
/// If either fiber fails, the other is cancelled.
///
/// # Latin Etymology
/// *Parallelos* (from Greek) = side by side.
///
/// # Example
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # use core::task::{Context, Poll, Waker};
/// #
/// # fn block_on<F: Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = Context::from_waker(Waker::noop());
/// #     loop {
/// #         if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
/// #
/// use ordofp_core::async_core::fibra::{Fibra, par};
///
/// let fiber1 = Fibra::purus(1);
/// let fiber2 = Fibra::purus(2);
/// let (a, b) = block_on(par(fiber1, fiber2)).unwrap();
/// assert_eq!((a, b), (1, 2));
/// ```
pub fn par<A, B>(fa: Fibra<A>, fb: Fibra<B>) -> Fibra<(A, B)>
where
    A: Send + 'static,
    B: Send + 'static,
{
    let cancelled = Arc::new(TesseraAbrogationis::default());
    let token_a = fa.cancellation_token();
    let token_b = fb.cancellation_token();
    let mut fa = Some(fa);
    let mut fb = Some(fb);
    let mut ra: Option<A> = None;
    let mut rb: Option<B> = None;
    Fibra {
        id: FibraId::new(),
        inner: Box::pin(core::future::poll_fn(move |cx| {
            if ra.is_none()
                && let Some(f) = fa.as_mut()
            {
                match Pin::new(f).poll(cx) {
                    Poll::Ready(Ok(a)) => {
                        ra = Some(a);
                        fa = None;
                    }
                    Poll::Ready(Err(e)) => {
                        token_b.abrogare(); // doc contract: failure cancels the other
                        return Poll::Ready(Err(e));
                    }
                    Poll::Pending => {}
                }
            }
            if rb.is_none()
                && let Some(f) = fb.as_mut()
            {
                match Pin::new(f).poll(cx) {
                    Poll::Ready(Ok(b)) => {
                        rb = Some(b);
                        fb = None;
                    }
                    Poll::Ready(Err(e)) => {
                        token_a.abrogare();
                        return Poll::Ready(Err(e));
                    }
                    Poll::Pending => {}
                }
            }
            match (ra.take(), rb.take()) {
                (Some(a), Some(b)) => Poll::Ready(Ok((a, b))),
                (a, b) => {
                    ra = a;
                    rb = b;
                    Poll::Pending
                }
            }
        })),
        cancelled,
    }
}

/// Alias for `par`.
#[inline]
pub fn zip_par<A, B>(fa: Fibra<A>, fb: Fibra<B>) -> Fibra<(A, B)>
where
    A: Send + 'static,
    B: Send + 'static,
{
    par(fa, fb)
}

/// Race two fibers and return the first to complete.
///
/// The winner is the first fiber to COMPLETE — success or failure alike —
/// not the first to succeed. The loser is cancelled.
///
/// # Latin Etymology
/// *Certamen* = contest, competition, race.
///
/// # Example
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # use core::task::{Context, Poll, Waker};
/// #
/// # fn block_on<F: Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = Context::from_waker(Waker::noop());
/// #     loop {
/// #         if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
/// #
/// use ordofp_core::async_core::fibra::{Fibra, certamen};
///
/// let fiber1 = Fibra::purus(1);
/// let fiber2 = Fibra::purus(2);
/// // Both are already-ready; the first one polled (fiber1) wins the race.
/// let winner = block_on(certamen(fiber1, fiber2)).unwrap();
/// assert_eq!(winner, 1);
/// ```
pub fn certamen<A>(fa: Fibra<A>, fb: Fibra<A>) -> Fibra<A>
where
    A: Send + 'static,
{
    let cancelled = Arc::new(TesseraAbrogationis::default());
    let token_a = fa.cancellation_token();
    let token_b = fb.cancellation_token();
    let mut fa = Some(fa);
    let mut fb = Some(fb);
    Fibra {
        id: FibraId::new(),
        inner: Box::pin(core::future::poll_fn(move |cx| {
            // First to COMPLETE wins (doc contract) — success or failure —
            // and the loser is cancelled.
            if let Some(f) = fa.as_mut()
                && let Poll::Ready(r) = Pin::new(f).poll(cx)
            {
                fa = None;
                token_b.abrogare();
                return Poll::Ready(r);
            }
            if let Some(f) = fb.as_mut()
                && let Poll::Ready(r) = Pin::new(f).poll(cx)
            {
                fb = None;
                token_a.abrogare();
                return Poll::Ready(r);
            }
            Poll::Pending
        })),
        cancelled,
    }
}

/// Alias for `certamen`.
#[inline]
pub fn race<A>(fa: Fibra<A>, fb: Fibra<A>) -> Fibra<A>
where
    A: Send + 'static,
{
    certamen(fa, fb)
}

/// Race multiple fibers, returning the first to succeed.
///
/// # Latin Etymology
/// *Certamen multorum* = race of many.
pub fn certamen_multi<A>(fibers: Vec<Fibra<A>>) -> Fibra<A>
where
    A: Send + 'static,
{
    if fibers.is_empty() {
        return Fibra::deficere(FibraError::Alius("no fibers to race".into()));
    }

    let cancelled = Arc::new(TesseraAbrogationis::default());
    let cancelled_clone = cancelled.clone();

    Fibra {
        id: FibraId::new(),
        inner: Box::pin(async move {
            // Simple sequential fallback
            for fiber in fibers {
                let result = fiber.await;
                if result.is_ok() {
                    return result;
                }
            }
            Err(FibraError::Alius("all fibers failed".into()))
        }),
        cancelled: cancelled_clone,
    }
}

/// Execute fibers sequentially, collecting all results.
///
/// # Latin Etymology
/// *Sequentia* = sequence, following.
pub fn sequentia<A>(fibers: Vec<Fibra<A>>) -> Fibra<Vec<A>>
where
    A: Send + 'static,
{
    let cancelled = Arc::new(TesseraAbrogationis::default());
    let cancelled_clone = cancelled.clone();

    Fibra {
        id: FibraId::new(),
        inner: Box::pin(async move {
            let mut results = Vec::with_capacity(fibers.len());
            for fiber in fibers {
                if cancelled_clone.est_abrogata() {
                    return Err(FibraError::Abrogatus);
                }
                let result = fiber.await?;
                results.push(result);
            }
            Ok(results)
        }),
        cancelled,
    }
}

/// Alias for `sequentia`.
#[inline]
pub fn sequence<A>(fibers: Vec<Fibra<A>>) -> Fibra<Vec<A>>
where
    A: Send + 'static,
{
    sequentia(fibers)
}

/// Execute all fibers in parallel, collecting all results.
///
/// If any fiber fails, all others are cancelled.
///
/// # Latin Etymology
/// *Omnes parallelos* = all in parallel.
///
/// # Panics
///
/// Panics only if the internal "every result slot is filled before
/// collection" invariant is violated, which would indicate a bug in this
/// crate — the completion check precedes the unwraps.
pub fn par_omnes<A>(fibers: Vec<Fibra<A>>) -> Fibra<Vec<A>>
where
    A: Send + 'static,
{
    let cancelled = Arc::new(TesseraAbrogationis::default());
    let tokens: Vec<Arc<TesseraAbrogationis>> =
        fibers.iter().map(Fibra::cancellation_token).collect();
    let mut slots: Vec<Option<Fibra<A>>> = fibers.into_iter().map(Some).collect();
    let mut results: Vec<Option<A>> = core::iter::repeat_with(|| None).take(slots.len()).collect();

    Fibra {
        id: FibraId::new(),
        inner: Box::pin(core::future::poll_fn(move |cx| {
            for (i, slot) in slots.iter_mut().enumerate() {
                if results[i].is_some() {
                    continue;
                }
                if let Some(f) = slot.as_mut() {
                    match Pin::new(f).poll(cx) {
                        Poll::Ready(Ok(a)) => {
                            results[i] = Some(a);
                            *slot = None;
                        }
                        Poll::Ready(Err(e)) => {
                            // doc contract: any failure cancels all others
                            for token in &tokens {
                                token.abrogare();
                            }
                            return Poll::Ready(Err(e));
                        }
                        Poll::Pending => {}
                    }
                }
            }
            if results.iter().all(Option::is_some) {
                let done = results.iter_mut().map(|r| r.take().unwrap()).collect();
                Poll::Ready(Ok(done))
            } else {
                Poll::Pending
            }
        })),
        cancelled,
    }
}

/// Alias for `par_omnes`.
#[inline]
pub fn par_sequence<A>(fibers: Vec<Fibra<A>>) -> Fibra<Vec<A>>
where
    A: Send + 'static,
{
    par_omnes(fibers)
}

/// Traverse a collection, running a fiber-producing function on each element.
///
/// # Latin Etymology
/// *Transire* = to cross, traverse.
pub fn transire<A, B, F>(items: Vec<A>, f: F) -> Fibra<Vec<B>>
where
    A: Send + 'static,
    B: Send + 'static,
    F: Fn(A) -> Fibra<B> + Send + 'static,
{
    let fibers: Vec<Fibra<B>> = items.into_iter().map(f).collect();
    sequentia(fibers)
}

/// Traverse in parallel.
///
/// # Latin Etymology
/// *Transire parallelos* = traverse in parallel.
pub fn transire_par<A, B, F>(items: Vec<A>, f: F) -> Fibra<Vec<B>>
where
    A: Send + 'static,
    B: Send + 'static,
    F: Fn(A) -> Fibra<B> + Send + 'static,
{
    let fibers: Vec<Fibra<B>> = items.into_iter().map(f).collect();
    par_omnes(fibers)
}

// =============================================================================
// FibraScope - Structured Concurrency Scope
// =============================================================================

/// A scope for structured concurrency.
///
/// Fibers spawned within a scope are automatically cancelled when the scope ends.
///
/// # Latin Etymology
/// *Ambitus* = a going around, circuit, scope.
pub struct FibraAmbitus<R: RuntimeGenerare> {
    /// Cancellation tokens of every fiber spawned in this scope.
    children: Vec<Arc<TesseraAbrogationis>>,
    _runtime: PhantomData<R>,
}

impl<R: RuntimeGenerare> FibraAmbitus<R> {
    /// Create a new fiber scope.
    #[inline]
    pub fn new() -> Self {
        FibraAmbitus {
            // Pre-allocate a small buffer; scopes rarely hold more than a handful of fibers.
            children: Vec::with_capacity(8),
            _runtime: PhantomData,
        }
    }

    /// Spawn a fiber within this scope.
    ///
    /// The fiber will be cancelled when the scope is dropped.
    #[inline]
    pub fn spawn<A, F>(&mut self, future: F) -> FibraManubrium<A>
    where
        F: Future<Output = A> + Send + 'static,
        A: Send + 'static,
    {
        let fibra = Fibra::new(future);
        let id = fibra.id();
        let cancelled = fibra.cancellation_token();
        self.children.push(cancelled.clone());

        let join_handle = R::spawn(fibra);

        FibraManubrium::new(id, join_handle, cancelled)
    }

    /// Cancel all fibers in this scope.
    #[inline]
    pub fn cancel_all(&self) {
        for child in &self.children {
            child.abrogare();
        }
    }
}

impl<R: RuntimeGenerare> Default for FibraAmbitus<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: RuntimeGenerare> Drop for FibraAmbitus<R> {
    fn drop(&mut self) {
        // Cancel all children when scope ends
        self.cancel_all();
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a fiber from a future.
///
/// # Example
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # use core::task::{Context, Poll, Waker};
/// #
/// # fn block_on<F: Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = Context::from_waker(Waker::noop());
/// #     loop {
/// #         if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
/// #
/// use ordofp_core::async_core::fibra::fibra;
///
/// let fiber = fibra(async { 42 });
/// assert_eq!(block_on(fiber).unwrap(), 42);
/// ```
#[inline]
pub fn fibra<A, F>(future: F) -> Fibra<A>
where
    F: Future<Output = A> + Send + 'static,
    A: Send + 'static,
{
    Fibra::new(future)
}

/// Create a pure fiber (immediately yields a value).
#[inline]
pub fn purus<A>(value: A) -> Fibra<A>
where
    A: Send + 'static,
{
    Fibra::purus(value)
}

/// Create a failed fiber.
#[inline]
pub fn deficere<A>(error: FibraError) -> Fibra<A>
where
    A: Send + 'static,
{
    Fibra::deficere(error)
}

/// Spawn a fiber on the given runtime and return a handle.
#[inline]
pub fn spawn<R, A, F>(future: F) -> FibraManubrium<A>
where
    R: RuntimeGenerare,
    F: Future<Output = A> + Send + 'static,
    A: Send + 'static,
{
    let fibra = Fibra::new(future);
    let id = fibra.id();
    let cancelled = fibra.cancellation_token();

    let join_handle = R::spawn(fibra);

    FibraManubrium::new(id, join_handle, cancelled)
}

/// Fork a fiber (spawn and return handle as a fiber).
///
/// # Latin Etymology
/// *Furca* = fork.
#[inline]
pub fn furca<R, A, F>(future: F) -> Fibra<FibraManubrium<A>>
where
    R: RuntimeGenerare,
    F: Future<Output = A> + Send + 'static,
    A: Send + 'static,
{
    Fibra::purus(spawn::<R, A, F>(future))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibra_id_unique() {
        let id1 = FibraId::new();
        let id2 = FibraId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_fibra_id_display() {
        let id = FibraId::new();
        let display = alloc::format!("{id}");
        assert!(display.starts_with("Fibra("));
    }

    #[test]
    fn test_fibra_error_display() {
        let err = FibraError::Abrogatus;
        assert!(alloc::format!("{err}").contains("cancelled"));

        let err = FibraError::Panic("oops".into());
        assert!(alloc::format!("{err}").contains("panic"));

        let err = FibraError::TemporisExcessus;
        assert!(alloc::format!("{err}").contains("timed out"));
    }

    #[test]
    fn test_fibra_purus() {
        let fiber = Fibra::purus(42);
        assert!(!fiber.is_cancelled());
    }

    #[test]
    fn test_fibra_deficere() {
        let fiber: Fibra<i32> = Fibra::deficere(FibraError::Abrogatus);
        assert!(!fiber.is_cancelled());
    }

    #[test]
    fn test_fibra_cancellation_token() {
        let fiber = Fibra::purus(42);
        let token = fiber.cancellation_token();
        assert!(!token.est_abrogata());
        token.abrogare();
        assert!(fiber.is_cancelled());
    }

    #[test]
    fn test_fibra_status() {
        assert_eq!(FibraStatus::Pendens, FibraStatus::Pendens);
        assert_ne!(FibraStatus::Pendens, FibraStatus::Currens);
    }

    // `NullRuntime::spawn` used to silently discard its future, which let
    // these two tests get away with spawning on a runtime that couldn't
    // actually run anything, just to exercise `FibraAmbitus`'s cancellation
    // bookkeeping. Task 16 (M-low fix) made `NullRuntime::spawn` panic
    // instead of lying about running the task, so `scope.spawn(..)` below
    // now panics before a `FibraManubrium` is ever produced. Real
    // spawn-then-cancel coverage (including the wake-on-cancel regression)
    // lives in `core/tests/fibra_cancellation.rs` under `TokioRuntime`; these
    // two now assert the new NullRuntime honesty instead.
    #[test]
    #[should_panic(expected = "no async runtime is enabled")]
    fn test_ambitus_cancel_all_cancels_spawned_fibers() {
        use crate::async_core::runtime::NullRuntime;

        let mut scope: FibraAmbitus<NullRuntime> = FibraAmbitus::new();
        let _handle = scope.spawn(async { 42 });
        scope.cancel_all();
    }

    #[test]
    #[should_panic(expected = "no async runtime is enabled")]
    fn test_ambitus_drop_cancels_spawned_fibers() {
        use crate::async_core::runtime::NullRuntime;

        let mut scope: FibraAmbitus<NullRuntime> = FibraAmbitus::new();
        let _handle = scope.spawn(async { 42 });
        drop(scope);
    }
}
