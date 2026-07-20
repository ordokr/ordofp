//! Runtime Abstraction - Runtime-agnostic async execution
//!
//! > *"Generare est producere"*
//! > — To generate is to produce. (Latin)
//!
//! This module provides abstractions for async runtime operations,
//! allowing code to be written in a runtime-agnostic manner.
//!
//! # Overview
//!
//! The runtime module provides:
//!
//! - [`RuntimeGenerare`] - Trait for spawning async tasks
//! - [`JoinManubrium`] - Handle for awaiting spawned tasks
//! - Runtime implementations for tokio and smol (feature-gated)
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Spawn | Generare | *generare* = to bring forth |
//! | Join | Conjungere | *conjungere* = to join together |
//! | Handle | Manubrium | *manubrium* = handle, grip |
//! | Runtime | Cursus | *cursus* = course, running |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::async_core::runtime::{CurrentRuntime, RuntimeGenerare};
//!
//! #[tokio::main]
//! async fn main() {
//!     let handle = CurrentRuntime::spawn(async {
//!         42
//!     });
//!
//!     let result = handle.await;
//!     assert_eq!(result.unwrap(), 42);
//! }
//! ```

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

#[cfg(feature = "tokio")]
use alloc::string::ToString;

/// A handle to a spawned task that can be awaited.
///
/// `JoinManubrium` represents a handle to a background task that was spawned
/// via [`RuntimeGenerare::spawn`]. It can be awaited to get the task's result.
///
/// > *"Manubrium opus"* — Handle of the work.
pub struct JoinManubrium<T> {
    inner: Pin<Box<dyn Future<Output = Result<T, JoinError>> + Send>>,
}

impl<T> JoinManubrium<T> {
    /// Create a new join handle from a future.
    #[inline]
    pub fn new<F>(fut: F) -> Self
    where
        F: Future<Output = Result<T, JoinError>> + Send + 'static,
    {
        JoinManubrium {
            inner: Box::pin(fut),
        }
    }

    /// Create a join handle that immediately returns a value.
    #[inline]
    pub fn ready(value: T) -> Self
    where
        T: Send + 'static,
    {
        JoinManubrium {
            inner: Box::pin(async move { Ok(value) }),
        }
    }

    /// Create a join handle that immediately returns an error.
    #[inline]
    pub fn error(err: JoinError) -> Self
    where
        T: Send + 'static,
    {
        JoinManubrium {
            inner: Box::pin(async move { Err(err) }),
        }
    }
}

impl<T> Future for JoinManubrium<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}

/// Error type for join operations.
///
/// Represents errors that can occur when awaiting a spawned task.
#[derive(Debug, Clone)]
pub enum JoinError {
    /// The task panicked.
    Panic(alloc::string::String),
    /// The task was cancelled.
    Cancelled,
    /// Other error.
    Other(alloc::string::String),
}

impl core::fmt::Display for JoinError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            JoinError::Panic(msg) => write!(f, "task panicked: {msg}"),
            JoinError::Cancelled => write!(f, "task was cancelled"),
            JoinError::Other(msg) => write!(f, "join error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for JoinError {}

/// Runtime-agnostic task spawning trait.
///
/// `RuntimeGenerare` abstracts over different async runtimes (tokio, smol),
/// allowing code to spawn tasks without depending on a specific runtime.
///
/// > *"Generare opus"* — To bring forth work.
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::runtime::{JoinManubrium, RuntimeGenerare, TokioRuntime};
///
/// fn spawn_work<R: RuntimeGenerare>(value: i32) -> JoinManubrium<i32> {
///     R::spawn(async move {
///         value * 2
///     })
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let handle = spawn_work::<TokioRuntime>(21);
///     assert_eq!(handle.await.unwrap(), 42);
/// }
/// ```
pub trait RuntimeGenerare {
    /// Spawn an async task and return a handle to await its result.
    ///
    /// The task will run concurrently with the current task.
    fn spawn<F, T>(future: F) -> JoinManubrium<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;

    /// Spawn a blocking operation on a thread pool.
    ///
    /// This is useful for CPU-bound work or blocking IO operations
    /// that shouldn't block the async runtime.
    fn spawn_blocking<F, T>(f: F) -> JoinManubrium<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;

    /// Yield control back to the runtime.
    ///
    /// This allows other tasks to make progress.
    fn yield_now() -> impl Future<Output = ()> + Send;

    /// Sleep for the specified duration.
    #[cfg(feature = "std")]
    fn sleep(duration: core::time::Duration) -> impl Future<Output = ()> + Send;
}

/// A no-op runtime that runs futures inline.
///
/// This runtime doesn't actually spawn tasks; it runs them synchronously.
/// Useful for testing or when no runtime is available.
///
/// > *"Cursus nullus"* — No runtime.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullRuntime;

impl RuntimeGenerare for NullRuntime {
    /// # Panics
    ///
    /// `NullRuntime` has no executor to run the spawned future on, so
    /// silently dropping it (the old behavior) hides real bugs: fire-and-
    /// forget work compiled under `CurrentRuntime = NullRuntime` (the
    /// default when neither the `tokio` nor `smol` feature is enabled)
    /// would simply never run, with no signal that anything was wrong.
    /// This is now a loud, immediate panic instead.
    fn spawn<F, T>(_future: F) -> JoinManubrium<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        panic!(
            "NullRuntime::spawn: no async runtime is enabled — enable the `tokio` or `smol` feature"
        );
    }

    fn spawn_blocking<F, T>(f: F) -> JoinManubrium<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        // Run synchronously
        let result = f();
        JoinManubrium::ready(result)
    }

    async fn yield_now() {}

    /// **Caveat:** this is a *blocking* sleep — `NullRuntime` has no timer,
    /// so it calls `std::thread::sleep`, blocking the whole OS thread (and
    /// any other tasks on a single-threaded executor) for `duration`.
    #[cfg(feature = "std")]
    async fn sleep(duration: core::time::Duration) {
        std::thread::sleep(duration);
    }
}

/// Tokio runtime implementation.
///
/// Requires the `tokio` feature flag.
#[cfg(feature = "tokio")]
#[derive(Debug, Clone, Copy, Default)]
pub struct TokioRuntime;

#[cfg(feature = "tokio")]
impl RuntimeGenerare for TokioRuntime {
    fn spawn<F, T>(future: F) -> JoinManubrium<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let handle = tokio::spawn(future);
        JoinManubrium::new(async move { handle.await.map_err(|e| JoinError::Panic(e.to_string())) })
    }

    fn spawn_blocking<F, T>(f: F) -> JoinManubrium<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let handle = tokio::task::spawn_blocking(f);
        JoinManubrium::new(async move { handle.await.map_err(|e| JoinError::Panic(e.to_string())) })
    }

    fn yield_now() -> impl Future<Output = ()> + Send {
        tokio::task::yield_now()
    }

    #[cfg(feature = "std")]
    fn sleep(duration: core::time::Duration) -> impl Future<Output = ()> + Send {
        tokio::time::sleep(duration)
    }
}

/// Extract a human-readable message from a caught panic payload, mirroring
/// what tokio's `JoinError::to_string()` does for `&str`/`String` payloads.
#[cfg(feature = "smol")]
fn smol_panic_message(payload: &(dyn core::any::Any + Send)) -> alloc::string::String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        alloc::string::String::from(*s)
    } else if let Some(s) = payload.downcast_ref::<alloc::string::String>() {
        s.clone()
    } else {
        alloc::string::String::from("smol task panicked")
    }
}

/// smol runtime implementation.
///
/// Requires the `smol` feature flag. smol is the actively-maintained
/// successor to the discontinued async-std (async-std was built on smol's
/// executor). Tasks spawned via [`smol::spawn`] run on smol's global executor,
/// whose worker threads start lazily on first use (see the `SMOL_THREADS`
/// environment variable).
#[cfg(feature = "smol")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SmolRuntime;

#[cfg(feature = "smol")]
impl RuntimeGenerare for SmolRuntime {
    fn spawn<F, T>(future: F) -> JoinManubrium<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // smol's `Task` CANCELS the spawned future when the handle drops;
        // tokio's `JoinHandle` keeps the task running in the background.
        // Match tokio: dropping the returned `JoinManubrium` detaches the
        // smol task (lets it keep running) instead of cancelling it.
        struct DetachOnDrop<T: Send + 'static>(Option<smol::Task<T>>);
        impl<T: Send + 'static> Drop for DetachOnDrop<T> {
            fn drop(&mut self) {
                if let Some(t) = self.0.take() {
                    t.detach();
                }
            }
        }

        let mut guard = DetachOnDrop(Some(smol::spawn(future)));
        // Panic parity with tokio's `Err(JoinError::Panic(..))`:
        //
        // `smol::Task` (and its `.fallible()` conversion, `FallibleTask`)
        // resumes the panic on the *awaiting* side the instant a panicked
        // task's output is observed — async-task's `poll_task` calls
        // `std::panic::resume_unwind` on the stored panic payload (see
        // async-task 4.7.1's `task.rs`), it does not hand back a value.
        // `.fallible()` does not change this: its `None` means "the
        // executor's `Runnable` was dropped without ever being polled"
        // (e.g. executor shutdown), a different condition from a user
        // panic, and not something async-task offers a value-based panic
        // signal for. So instead of the `.fallible()` + `None` sketch, we
        // catch the unwind ourselves at the poll boundary (`Task::poll` is
        // an ordinary synchronous fn, so `catch_unwind` around it works)
        // and translate a caught panic into `JoinError::Panic`.
        JoinManubrium::new(core::future::poll_fn(move |cx| {
            let task = guard
                .0
                .as_mut()
                .expect("task present until this future resolves");
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Pin::new(task).poll(cx)))
            {
                Ok(core::task::Poll::Ready(v)) => {
                    guard.0 = None; // completed normally; nothing to detach
                    core::task::Poll::Ready(Ok(v))
                }
                Ok(core::task::Poll::Pending) => core::task::Poll::Pending,
                Err(payload) => {
                    guard.0 = None; // task consumed by the unwind; nothing to detach
                    core::task::Poll::Ready(Err(JoinError::Panic(smol_panic_message(&*payload))))
                }
            }
        }))
    }

    fn spawn_blocking<F, T>(f: F) -> JoinManubrium<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let handle = smol::unblock(f);
        JoinManubrium::new(async move { Ok(handle.await) })
    }

    fn yield_now() -> impl Future<Output = ()> + Send {
        smol::future::yield_now()
    }

    #[cfg(feature = "std")]
    async fn sleep(duration: core::time::Duration) {
        smol::Timer::after(duration).await;
    }
}

/// Type alias for the current runtime based on feature flags.
///
/// - If `tokio` is enabled, uses `TokioRuntime`
/// - If `smol` is enabled (and not tokio), uses `SmolRuntime`
/// - Otherwise, uses `NullRuntime`
#[cfg(feature = "tokio")]
pub type CurrentRuntime = TokioRuntime;

#[cfg(all(feature = "smol", not(feature = "tokio")))]
pub type CurrentRuntime = SmolRuntime;

#[cfg(not(any(feature = "tokio", feature = "smol")))]
pub type CurrentRuntime = NullRuntime;

/// Extension trait for futures that adds runtime-aware operations.
pub trait FutureRuntimeExt: Future + Sized {
    /// Spawn this future as a background task.
    #[inline]
    fn spawn_on<R: RuntimeGenerare>(self) -> JoinManubrium<Self::Output>
    where
        Self: Send + 'static,
        Self::Output: Send + 'static,
    {
        R::spawn(self)
    }

    /// Spawn this future on the current runtime.
    #[inline]
    fn spawn_current(self) -> JoinManubrium<Self::Output>
    where
        Self: Send + 'static,
        Self::Output: Send + 'static,
    {
        CurrentRuntime::spawn(self)
    }
}

impl<F: Future> FutureRuntimeExt for F {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_null_runtime_spawn_blocking() {
        // spawn_blocking should work even without a real runtime
        let _handle = NullRuntime::spawn_blocking(|| 42);
        // Note: We can't easily test this without an executor
    }

    #[test]
    fn test_join_error_display() {
        let panic_err = JoinError::Panic("test panic".to_string());
        assert!(panic_err.to_string().contains("panic"));

        let cancelled_err = JoinError::Cancelled;
        assert!(cancelled_err.to_string().contains("cancel"));

        let other_err = JoinError::Other("other".to_string());
        assert!(other_err.to_string().contains("other"));
    }

    #[test]
    fn test_join_manubrium_ready() {
        // JoinManubrium::ready should create a handle that returns the value
        let _handle = JoinManubrium::ready(42);
    }

    #[test]
    #[should_panic(expected = "no async runtime is enabled")]
    fn null_runtime_spawn_panics_instead_of_discarding() {
        let _handle = NullRuntime::spawn(async { 42 });
    }

    #[cfg(feature = "smol")]
    #[test]
    fn smol_spawn_detaches_on_drop_instead_of_cancelling() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicUsize, Ordering};

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_task = counter.clone();

        // Pre-fix: dropping the smol `Task` handle cancels the future, so
        // the counter would never advance. Post-fix: the `JoinManubrium`
        // detaches the smol task on drop, letting it keep running (tokio
        // parity for fire-and-forget spawns).
        let handle = SmolRuntime::spawn(async move {
            smol::Timer::after(core::time::Duration::from_millis(20)).await;
            counter_task.fetch_add(1, Ordering::SeqCst);
        });
        drop(handle);

        smol::block_on(async {
            smol::Timer::after(core::time::Duration::from_millis(200)).await;
        });

        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "detached task must keep running after the JoinManubrium is dropped"
        );
    }

    #[cfg(feature = "smol")]
    #[test]
    fn smol_spawn_panic_becomes_join_error_panic() {
        let handle = SmolRuntime::spawn(async { panic!("boom") });
        let result = smol::block_on(handle);
        assert!(
            matches!(result, Err(JoinError::Panic(_))),
            "a panicking smol task must surface as JoinError::Panic, not resume the unwind \
             into the awaiting context"
        );
    }
}
