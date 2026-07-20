//! Async Monad type class - asynchronous sequential computation.
//!
//! > *"Vinculum substantiale"*
//! > — The substantial chain. (Leibniz, on the binding of monads)
//!
//! `MonadAsync` extends `ApplicatioAsync` with sequential chaining of
//! asynchronous computations where each step can depend on the result
//! of the previous step.
//!
//! # Laws
//!
//! 1. **Left Identity**: `pure_async(a).await.flat_map_async(f).await == f(a).await`
//! 2. **Right Identity**: `m.flat_map_async(|x| pure_async(x)).await == m`
//! 3. **Associativity**: `m.flat_map_async(f).await.flat_map_async(g).await == m.flat_map_async(|x| f(x).await.flat_map_async(g)).await`
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
//! use ordofp_core::async_core::MonadAsync;
//!
//! async fn safe_div(x: i32, y: i32) -> Option<i32> {
//!     if y == 0 { None } else { Some(x / y) }
//! }
//!
//! async fn example() {
//!     let result = Some(10)
//!         .flat_map_async(|x| safe_div(x, 2))
//!         .await
//!         .flat_map_async(|x| safe_div(x, 5))
//!         .await;
//!     assert_eq!(result, Some(1));
//! }
//!
//! block_on(example());
//! ```
//!
//! # Design note: deliberate sync mirror
//!
//! This module is a deliberate, line-for-line mirror of its synchronous
//! counterpart [`ordofp_core::typeclasses::monad::Monad`] (see
//! `core/src/typeclasses/monad.rs`), with each method wrapped in
//! `async move { … }` and returning a `Future`.
//!
//! **Why duplicated rather than macro-generated?** Rust does not support
//! generic-over-async-ness without unstable features (e.g. `maybe_async`,
//! `keyword generics`). The mirror is kept explicit so that tooling
//! (go-to-definition, type-inference error messages, rustdoc output,
//! IDE hover) stays legible. A macro would save lines but impose
//! non-trivial macro-debugging cost whenever an async impl diverges.
//!
//! **Invariant**: any behavioural change here MUST be reviewed against the
//! sync counterpart, and vice versa. The two files are expected to stay
//! in lock-step modulo the `async`/`.await` shape.

use alloc::vec::Vec;
use core::future::Future;

use super::applicative_async::ApplicatioAsync;

/// Async Monad - sequential async computation chaining.
///
/// This trait extends `ApplicatioAsync` with `flat_map_async` (also known as
/// `bind` or `>>=`), which allows chaining async computations where each
/// step can depend on the result of the previous step.
///
/// # Laws
///
/// 1. **Left Identity**: `pure_async(a).flat_map_async(f) == f(a)`
/// 2. **Right Identity**: `m.flat_map_async(pure_async) == m`
/// 3. **Associativity**: `m.flat_map_async(f).flat_map_async(g) == m.flat_map_async(|x| f(x).flat_map_async(g))`
pub trait MonadAsync: ApplicatioAsync {
    /// Chain an async computation that returns a monad.
    ///
    /// Also known as `bind` or `>>=` in Haskell.
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
    /// use ordofp_core::async_core::MonadAsync;
    ///
    /// async fn example() {
    ///     let result = Some(5)
    ///         .flat_map_async(|x| async move {
    ///             if x > 0 { Some(x * 2) } else { None }
    ///         })
    ///         .await;
    ///     assert_eq!(result, Some(10));
    /// }
    ///
    /// block_on(example());
    /// ```
    fn flat_map_async<B, F, Fut>(self, f: F) -> impl Future<Output = Self::Target<B>> + Send
    where
        F: FnMut(Self::Inner) -> Fut + Send,
        Fut: Future<Output = Self::Target<B>> + Send,
        B: Send;

    /// Alias for `flat_map_async` using Haskell naming.
    #[inline]
    fn bind_async<B, F, Fut>(self, f: F) -> impl Future<Output = Self::Target<B>> + Send
    where
        Self: Sized,
        F: FnMut(Self::Inner) -> Fut + Send,
        Fut: Future<Output = Self::Target<B>> + Send,
        B: Send,
    {
        self.flat_map_async(f)
    }

    /// Alias for `flat_map_async` using Scala naming.
    #[inline]
    fn and_then_async<B, F, Fut>(self, f: F) -> impl Future<Output = Self::Target<B>> + Send
    where
        Self: Sized,
        F: FnMut(Self::Inner) -> Fut + Send,
        Fut: Future<Output = Self::Target<B>> + Send,
        B: Send,
    {
        self.flat_map_async(f)
    }
}

// ============================================================================
// Implementation for Option
// ============================================================================

impl<A: Send> MonadAsync for Option<A> {
    #[inline]
    async fn flat_map_async<B, F, Fut>(self, mut f: F) -> Option<B>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = Option<B>> + Send,
        B: Send,
    {
        match self {
            Some(a) => f(a).await,
            None => None,
        }
    }
}

// ============================================================================
// Implementation for Result
// ============================================================================

impl<A: Send, E: Send> MonadAsync for Result<A, E> {
    #[inline]
    async fn flat_map_async<B, F, Fut>(self, mut f: F) -> Result<B, E>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = Result<B, E>> + Send,
        B: Send,
    {
        match self {
            Ok(a) => f(a).await,
            Err(e) => Err(e),
        }
    }
}

// ============================================================================
// Implementation for Vec
// ============================================================================

impl<A: Send> MonadAsync for Vec<A> {
    #[inline]
    async fn flat_map_async<B, F, Fut>(self, mut f: F) -> Vec<B>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = Vec<B>> + Send,
        B: Send,
    {
        // Lower-bound capacity hint: each element yields at least an empty Vec.
        let mut results = Vec::with_capacity(self.len());
        for item in self {
            results.extend(f(item).await);
        }
        results
    }
}

/// Extended `MonadAsync` with `FnMut` support.
pub trait MonadAsyncMut: ApplicatioAsync {
    /// Chain async computations over all elements with `FnMut`.
    fn flat_map_async_mut<B, F, Fut>(self, f: F) -> impl Future<Output = Self::Target<B>> + Send
    where
        F: FnMut(Self::Inner) -> Fut + Send,
        Fut: Future<Output = Self::Target<B>> + Send,
        B: Send;
}

impl<A: Send> MonadAsyncMut for Vec<A> {
    #[inline]
    async fn flat_map_async_mut<B, F, Fut>(self, mut f: F) -> Vec<B>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = Vec<B>> + Send,
        B: Send,
    {
        // Pre-allocate with the input length as a conservative lower-bound
        // hint. Each element expands into a Vec<B>, so the true capacity
        // is unknowable without running f, but avoiding a completely cold
        // Vec::new() saves at least one realloc for non-empty inputs.
        let mut results = Vec::with_capacity(self.len());
        for item in self {
            results.extend(f(item).await);
        }
        results
    }
}

// ============================================================================
// Utility functions
// ============================================================================

/// Flatten a nested Option asynchronously.
#[inline]
pub async fn flatten_option_async<A: Send>(opt: Option<Option<A>>) -> Option<A> {
    opt.flatten()
}

/// Flatten a nested Result asynchronously.
///
/// # Errors
///
/// Returns the outer `Err(e)` unchanged when the outer layer failed;
/// otherwise returns the inner `Result` as-is, so an inner `Err` also
/// propagates. No new error values are constructed here.
#[inline]
pub async fn flatten_result_async<A: Send, E: Send>(res: Result<Result<A, E>, E>) -> Result<A, E> {
    match res {
        Ok(inner) => inner,
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_monad_async_type_check() {
        fn assert_monad_async<T: MonadAsync>() {}
        assert_monad_async::<Option<i32>>();
    }

    #[test]
    fn test_result_monad_async_type_check() {
        fn assert_monad_async<T: MonadAsync>() {}
        assert_monad_async::<Result<i32, &str>>();
    }

    #[test]
    fn test_vec_monad_async_type_check() {
        fn assert_monad_async<T: MonadAsync>() {}
        assert_monad_async::<Vec<i32>>();
    }
}
