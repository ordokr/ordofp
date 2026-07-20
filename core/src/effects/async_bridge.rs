//! Async Bridge - Integration between Eff monad and async transformers
//!
//! > *"Pons inter effectus et asynchoniam"*
//! > — Bridge between effects and asynchrony. (Neo-Latin)
//!
//! This module provides bridging utilities to connect the `Eff` monad with
//! async transformers like `LectorAsync`, `StatusAsync`, etc.
//!
//! # Design
//!
//! The effectful library from Haskell uses a ReaderT-over-IO pattern under the
//! hood. We adapt this for Rust by providing conversions between:
//!
//! - `Eff<R, A>` (pure effect tracking)
//! - Async transformers (actual async execution)
//!
//! # Example
//!
//! ```rust
//! # use core::future::Future;
//! # use core::pin::Pin;
//! # use core::task::{Context, Poll, Waker};
//! #
//! # // Busy-poll a future to completion with a noop waker (no real executor
//! # // dependency needed for this doctest).
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
//! use ordofp_core::effects::async_bridge::eff_to_async;
//! use ordofp_core::effects::Eff;
//! use ordofp_core::effects::row_v2::EffectSet;
//!
//! // Convert Eff to async
//! let eff: Eff<EffectSet<0>, i32> = Eff::purus(42);
//! let future = eff_to_async(eff);
//!
//! // Run the async computation
//! let result = block_on(future);
//! assert_eq!(result, 42);
//! ```

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;

use super::eff::{Eff, run_purus};
use super::row_v2::EffectSet;

/// A boxed, pinned, sendable future yielding `A`.
type BoxFutureSend<A> = Pin<Box<dyn Future<Output = A> + Send>>;

/// One-shot runner stored by [`EffAsync`]: consumes the environment `E` and
/// returns a boxed future producing `A`.
type EffRunner<E, A> = Box<dyn FnOnce(E) -> BoxFutureSend<A> + Send>;

// =============================================================================
// Eff to Async Conversion
// =============================================================================

/// Convert a pure `Eff` computation to an async future.
///
/// This only works for pure computations (no effects).
/// For effectful computations, use `run_eff_async` with handlers.
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
/// use ordofp_core::effects::async_bridge::eff_to_async;
/// use ordofp_core::effects::Eff;
/// use ordofp_core::effects::row_v2::EffectSet;
///
/// let eff: Eff<EffectSet<0>, i32> = Eff::purus(42);
/// let future = eff_to_async(eff);
/// let result = block_on(future);
/// assert_eq!(result, 42);
/// ```
#[inline]
pub async fn eff_to_async<A: Send + 'static>(eff: Eff<EffectSet<0>, A>) -> A {
    run_purus(eff)
}

/// An async runner for Eff computations with an environment.
///
/// This mirrors the ReaderT-over-IO pattern from effectful.
/// The environment is passed to handlers that interpret effects.
///
/// Unlike `LectorAsync`, this uses `FnOnce` semantics which is more appropriate
/// for effect handling where the continuation is consumed exactly once.
///
/// # Type Parameters
///
/// * `E` - Environment type (like `ReaderT`'s `r`)
/// * `A` - Result type
pub struct EffAsync<E, A> {
    run: EffRunner<E, A>,
}

impl<E: 'static, A: 'static> EffAsync<E, A> {
    /// Create a new async effectful computation.
    #[inline]
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce(E) -> Fut + Send + 'static,
        Fut: Future<Output = A> + Send + 'static,
    {
        EffAsync {
            run: Box::new(move |env| Box::pin(f(env))),
        }
    }

    /// Create a pure async computation (doesn't use environment).
    #[inline]
    pub fn pure(value: A) -> Self
    where
        A: Send + 'static,
    {
        EffAsync {
            run: Box::new(move |_| Box::pin(async move { value })),
        }
    }

    /// Run the async computation with an environment.
    #[inline]
    pub async fn run(self, env: E) -> A {
        (self.run)(env).await
    }

    /// Map a function over the result.
    #[inline]
    pub fn map<B: 'static, F>(self, f: F) -> EffAsync<E, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        E: Send + 'static,
        A: Send,
    {
        EffAsync {
            run: Box::new(move |env| {
                Box::pin(async move {
                    let a = (self.run)(env).await;
                    f(a)
                })
            }),
        }
    }

    /// Monadic bind for async effects.
    #[inline]
    pub fn flat_map<B: 'static, F>(self, f: F) -> EffAsync<E, B>
    where
        F: FnOnce(A) -> EffAsync<E, B> + Send + 'static,
        E: Clone + Send + 'static,
        A: Send,
    {
        EffAsync {
            run: Box::new(move |env| {
                let env2 = env.clone();
                Box::pin(async move {
                    let a = (self.run)(env).await;
                    f(a).run(env2).await
                })
            }),
        }
    }

    /// Access the environment.
    #[inline]
    pub fn ask() -> EffAsync<E, E>
    where
        E: Clone + Send + 'static,
    {
        EffAsync::new(|env: E| async move { env })
    }

    /// Run a local computation with a modified environment.
    #[inline]
    pub fn local<F>(f: F, inner: EffAsync<E, A>) -> EffAsync<E, A>
    where
        F: FnOnce(E) -> E + Send + 'static,
        E: Send + 'static,
        A: Send,
    {
        EffAsync {
            run: Box::new(move |env| {
                let modified_env = f(env);
                (inner.run)(modified_env)
            }),
        }
    }
}

// =============================================================================
// Lifting between Eff and EffAsync
// =============================================================================

/// Lift a pure Eff computation into `EffAsync`.
///
/// The Eff computation is run synchronously, then wrapped in async.
#[inline]
pub fn lift_eff<E: 'static, A: Send + 'static>(eff: Eff<EffectSet<0>, A>) -> EffAsync<E, A> {
    EffAsync::new(move |_| {
        let result = run_purus(eff);
        async move { result }
    })
}

/// Lift an async operation into `EffAsync`.
#[inline]
pub fn lift_async<E: 'static, A: 'static, Fut>(fut: Fut) -> EffAsync<E, A>
where
    Fut: Future<Output = A> + Send + 'static,
{
    EffAsync {
        run: Box::new(move |_| Box::pin(fut)),
    }
}

// =============================================================================
// Integration with Async Transformers
// =============================================================================

use crate::transformers::async_transforms::LectorAsync;

/// Convert a pure Eff computation to `LectorAsync`.
///
/// This creates a `LectorAsync` that ignores its environment and
/// returns the result of the Eff computation.
///
/// Note: Only works for pure computations (`EffectSet`<0>).
#[inline]
pub fn eff_to_lector<E: Clone + Send + Sync + 'static, A: Clone + Send + Sync + 'static>(
    eff: Eff<EffectSet<0>, A>,
) -> LectorAsync<E, A> {
    let result = run_purus(eff);
    LectorAsync::purus(result)
}

/// Create an `EffAsync` from a `LectorAsync` by running it with a shared environment.
///
/// The `LectorAsync` is cloned for each run.
#[inline]
pub fn lector_to_eff_async<E: Clone + Send + Sync + 'static, A: Send + 'static>(
    lector: LectorAsync<E, A>,
) -> EffAsync<E, A>
where
    LectorAsync<E, A>: Clone,
{
    // LectorAsync holds an Arc internally, so cloning is cheap
    EffAsync::new(move |env: E| {
        let lector = lector.clone();
        async move { lector.run(env).await }
    })
}

// =============================================================================
// EffAsync combinators for common patterns
// =============================================================================

/// Sequence two `EffAsync` computations.
pub fn sequence_eff_async<E, A, B>(
    first: EffAsync<E, A>,
    second: EffAsync<E, B>,
) -> EffAsync<E, (A, B)>
where
    E: Clone + Send + 'static,
    A: Send + 'static,
    B: Send + 'static,
{
    first.flat_map(move |a| second.map(move |b| (a, b)))
}

/// Traverse a collection with an EffAsync-producing function.
pub fn traverse_eff_async<E, A, B, I, F>(items: I, f: F) -> EffAsync<E, alloc::vec::Vec<B>>
where
    E: Clone + Send + 'static,
    A: Send + 'static,
    B: Send + 'static,
    I: IntoIterator<Item = A>,
    F: Fn(A) -> EffAsync<E, B> + Send + Sync + 'static,
{
    let items: alloc::vec::Vec<_> = items.into_iter().collect();
    let f = Arc::new(f);

    EffAsync::new(move |env: E| {
        let f = Arc::clone(&f);
        async move {
            let mut results = alloc::vec::Vec::with_capacity(items.len());
            for item in items {
                let eff = f(item);
                let result = eff.run(env.clone()).await;
                results.push(result);
            }
            results
        }
    })
}

// =============================================================================
// Tests
// =============================================================================

// All tests in this module drive their futures through the tokio current-thread
// runtime, so the module only builds when the `tokio` feature is active. Other
// async runtimes (notably `smol`) do not supply a compatible block-on
// runtime builder here; gating the test module avoids a spurious
// `unresolved crate tokio` error when the crate is built with only `smol`.
#[cfg(all(test, feature = "tokio"))]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};

    #[test]
    fn test_eff_async_pure() {
        let eff: EffAsync<(), i32> = EffAsync::pure(42);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let result = rt.block_on(eff.run(()));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_eff_async_map() {
        let eff: EffAsync<(), i32> = EffAsync::pure(21);
        let doubled = eff.map(|x| x * 2);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let result = rt.block_on(doubled.run(()));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_eff_async_ask() {
        let ask_eff: EffAsync<String, String> = EffAsync::<String, String>::ask();

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let result = rt.block_on(ask_eff.run("hello".to_string()));
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_eff_async_flat_map() {
        let eff: EffAsync<i32, i32> = EffAsync::<i32, i32>::ask();
        let result = eff.flat_map(|env| EffAsync::pure(env * 2));

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let value = rt.block_on(result.run(21));
        assert_eq!(value, 42);
    }

    #[test]
    fn test_lift_eff_to_eff_async() {
        let eff: Eff<EffectSet<0>, i32> = Eff::purus(42);
        let eff_async: EffAsync<(), i32> = lift_eff(eff);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let result = rt.block_on(eff_async.run(()));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_eff_to_lector() {
        let eff: Eff<EffectSet<0>, i32> = Eff::purus(42);
        let lector: LectorAsync<String, i32> = eff_to_lector(eff);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let result = rt.block_on(lector.run("ignored".to_string()));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_eff_async_local() {
        let inner: EffAsync<i32, i32> = EffAsync::<i32, i32>::ask();
        let with_local = EffAsync::local(|x: i32| x * 2, inner);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let result = rt.block_on(with_local.run(21));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_sequence_eff_async() {
        let first: EffAsync<(), i32> = EffAsync::pure(1);
        let second: EffAsync<(), i32> = EffAsync::pure(2);
        let combined = sequence_eff_async(first, second);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio current-thread runtime should build successfully");

        let (a, b) = rt.block_on(combined.run(()));
        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }
}
