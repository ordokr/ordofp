//! Async Monad Transformers for composing asynchronous monadic effects.
//!
//! > *"Motus compositus ex pluribus motibus"*
//! > — Composite motion from many motions. (Scholastic physics)
//!
//! This module provides async variants of the standard monad transformers,
//! enabling functional composition of asynchronous effects.
//!
//! # Overview
//!
//! Async monad transformers extend the concept of monad transformers to
//! asynchronous contexts. Each transformer adds a specific capability while
//! preserving async compatibility:
//!
//! | Transformer | Latin Name | Capability |
//! |-------------|------------|------------|
//! | `AsyncReaderT` | `LectorAsync` | Environment/configuration access |
//! | `AsyncStateT` | `StatusAsync` | Stateful computation |
//! | `AsyncOptionT` | `OptionTAsync` | Optional async values |
//! | `AsyncEitherT` | `EitherTAsync` | Async error handling |
//! | `AsyncWriterT` | `ScriptorAsync` | Async logging/accumulation |
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
//! use ordofp_core::transformers::async_transforms::LectorAsync;
//!
//! struct Config {
//!     base_url: String,
//!     timeout: u32,
//! }
//!
//! let fetch_data: LectorAsync<Config, String> = LectorAsync::new(|cfg: Config| async move {
//!     format!("Fetching from {} with timeout {}", cfg.base_url, cfg.timeout)
//! });
//!
//! let config = Config {
//!     base_url: "https://api.example.com".to_string(),
//!     timeout: 30,
//! };
//!
//! let result = block_on(fetch_data.run(config));
//! assert_eq!(result, "Fetching from https://api.example.com with timeout 30");
//! ```
//!
//! # Design Principles
//!
//! 1. **Arc-wrapped closures** - Async transformers use `Arc` for cloneable async functions
//! 2. **`Pin<Box<dyn Future>>`** - Dynamic dispatch for flexibility
//! 3. **Send + Sync bounds** - Thread-safe by default
//! 4. **Optimized enum variants** - Fast paths for pure values (Purus pattern)

mod either_t_async;
mod lector_async;
mod option_t_async;
mod scriptor_async;
mod status_async;

pub use either_t_async::EitherTAsync;
pub use lector_async::LectorAsync;
pub use option_t_async::OptionTAsync;
pub use scriptor_async::ScriptorAsync;
pub use status_async::StatusAsync;

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Type alias for boxed async functions used in transformers.
pub type AsyncFn<A, B> = Arc<dyn Fn(A) -> Pin<Box<dyn Future<Output = B> + Send>> + Send + Sync>;

/// Type alias for a boxed, pinned, sendable future.
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// One-shot slot holding a lifted future until its first run.
///
/// The future is taken out of the slot on first execution and its result is
/// cached. If that run is dropped before completion, [`SlotCustos`] restores
/// the future so a later run can resume it.
pub(crate) type FutureSlot<A> = Arc<std::sync::Mutex<Option<BoxFuture<A>>>>;

/// Drop guard for a future taken out of a [`FutureSlot`].
///
/// While armed, dropping the guard (e.g. when the owning run is cancelled
/// mid-await) puts the future back into the slot instead of losing it.
/// Call [`SlotCustos::disarm`] once the future has completed.
pub(crate) struct SlotCustos<A> {
    fut: Option<BoxFuture<A>>,
    slot: FutureSlot<A>,
}

impl<A> SlotCustos<A> {
    /// Guard `fut`, restoring it to `slot` if dropped while still armed.
    pub(crate) fn new(fut: BoxFuture<A>, slot: FutureSlot<A>) -> Self {
        SlotCustos {
            fut: Some(fut),
            slot,
        }
    }

    /// Mutable access to the held future for polling/awaiting.
    pub(crate) fn fut_mut(&mut self) -> &mut BoxFuture<A> {
        self.fut
            .as_mut()
            .expect("SlotCustos holds the future until disarmed")
    }

    /// The future completed; do not restore it on drop.
    pub(crate) fn disarm(&mut self) {
        self.fut = None;
    }
}

impl<A> Drop for SlotCustos<A> {
    fn drop(&mut self) {
        if let Some(fut) = self.fut.take() {
            // Ignore a poisoned lock: never panic in drop.
            if let Ok(mut slot) = self.slot.lock() {
                *slot = Some(fut);
            }
        }
    }
}

/// A future that yields to the executor exactly once (waking the waker so the
/// task is re-polled). Executor-agnostic; used to wait for another runner of
/// the same lifted future to publish its cached result.
pub(crate) struct CedeSemel {
    yielded: bool,
}

/// Yield to the executor once. *Cede semel* = yield once.
pub(crate) fn cede_semel() -> CedeSemel {
    CedeSemel { yielded: false }
}

impl Future for CedeSemel {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Trait for async monad transformers.
///
/// This trait provides a common interface for lifting async operations
/// into a transformer context.
///
/// # Laws
///
/// 1. **Lift Preserves Identity:**
///    ```text
///    lift_async(async { pure(x) }) == pure_async(x)
///    ```
///
/// 2. **Lift Preserves Bind:**
///    ```text
///    lift_async(m).flat_map_async(|x| lift_async(f(x))) == lift_async(m.flat_map_async(f))
///    ```
pub trait MonadTransformerAsync {
    /// The type produced when running the transformer.
    type Output;

    /// Lift a future into the transformer context.
    fn lift_async<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = Self::Output> + Send + 'static;
}
