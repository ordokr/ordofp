//! Async Functor type class - asynchronous mapping over values.
//!
//! > *"Forma in potentia ad actum reducta per motum."*
//! > — Form in potentiality is reduced to actuality through motion. (Aquinas)
//!
//! `FunctorAsync` extends the concept of `Functor` to asynchronous contexts,
//! allowing mapping with async functions over values in a container.
//!
//! # Laws
//!
//! Async Functor must satisfy the same laws as synchronous Functor:
//!
//! 1. **Identity**: `fa.fmap_async(|x| async { x }).await == fa`
//! 2. **Composition**: `fa.fmap_async(f).await.fmap_async(g).await == fa.fmap_async(|x| async { g(f(x).await).await }).await`
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
//! use ordofp_core::async_core::FunctorAsync;
//!
//! let opt = Some(5);
//! let doubled = block_on(opt.fmap_async(|x| async move { x * 2 }));
//! assert_eq!(doubled, Some(10));
//! ```
//!
//! # Design note: deliberate sync mirror
//!
//! This module is a deliberate, line-for-line mirror of its synchronous
//! counterpart [`ordofp_core::typeclasses::functor::Functor`] (see
//! `core/src/typeclasses/functor.rs`), with each method wrapped in
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

/// Async Functor - types that support asynchronous mapping.
///
/// This trait provides async variants of the standard `Functor` operations,
/// allowing the mapping function to be asynchronous.
///
/// # Laws
///
/// 1. **Identity**: `fa.fmap_async(|x| async { x }).await == fa`
/// 2. **Composition**: Async composition preserves structure
///
/// # Type Parameters
///
/// - `Inner`: The type of the value(s) contained in the functor
/// - `Target<T>`: The functor type with a different inner type
pub trait FunctorAsync: Sized {
    /// The inner value type.
    type Inner;

    /// The target type after mapping (type constructor applied to a new type).
    type Target<T: Send>: FunctorAsync<Inner = T>;

    /// Maps an async function over the inner value(s).
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
    /// use ordofp_core::async_core::FunctorAsync;
    ///
    /// let opt = Some(5);
    /// let result = block_on(opt.fmap_async(|x| async move { x * 2 }));
    /// assert_eq!(result, Some(10));
    /// ```
    fn fmap_async<B, F, Fut>(self, f: F) -> impl Future<Output = Self::Target<B>> + Send
    where
        F: FnMut(Self::Inner) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send;

    /// Replace all values with an async-computed constant.
    ///
    /// Equivalent to `self.fmap_async(|_| async { b })`.
    #[inline]
    fn map_const_async<B, Fut>(self, fut: Fut) -> impl Future<Output = Self::Target<B>> + Send
    where
        Self: Send,
        Fut: Future<Output = B> + Send,
        B: Send + Clone,
    {
        async move {
            let b = fut.await;
            self.fmap_async(move |_| {
                let b = b.clone();
                async move { b }
            })
            .await
        }
    }

    /// Async void - map and discard the result asynchronously.
    #[inline]
    fn void_async(self) -> impl Future<Output = Self::Target<()>> + Send
    where
        Self: Send,
    {
        self.fmap_async(|_| async {})
    }
}

// ============================================================================
// Implementation for Option
// ============================================================================

impl<A: Send> FunctorAsync for Option<A> {
    type Inner = A;
    type Target<T: Send> = Option<T>;

    #[inline]
    async fn fmap_async<B, F, Fut>(self, mut f: F) -> Option<B>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        match self {
            Some(a) => Some(f(a).await),
            None => None,
        }
    }
}

// ============================================================================
// Implementation for Result
// ============================================================================

impl<A: Send, E: Send> FunctorAsync for Result<A, E> {
    type Inner = A;
    type Target<T: Send> = Result<T, E>;

    #[inline]
    async fn fmap_async<B, F, Fut>(self, mut f: F) -> Result<B, E>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        match self {
            Ok(a) => Ok(f(a).await),
            Err(e) => Err(e),
        }
    }
}

// ============================================================================
// Implementation for Vec
// ============================================================================

impl<A: Send> FunctorAsync for Vec<A> {
    type Inner = A;
    type Target<T: Send> = Vec<T>;

    #[inline]
    async fn fmap_async<B, F, Fut>(self, mut f: F) -> Vec<B>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        let mut results = Vec::with_capacity(self.len());
        for item in self {
            results.push(f(item).await);
        }
        results
    }
}

/// Async Functor implementation for Vec with `FnMut` support.
///
/// This extension trait provides a version of `fmap_async` that accepts
/// `FnMut` closures, enabling iteration over all elements.
pub trait FunctorAsyncMut: Sized {
    /// The element type being mapped over (the functor's contents).
    type Inner;
    /// The same container shape re-parameterised at element type `T`;
    /// `fmap_async_mut` returns `Self::Target<B>`.
    type Target<T: Send>;

    /// Maps an async `FnMut` over all elements.
    fn fmap_async_mut<B, F, Fut>(self, f: F) -> impl Future<Output = Self::Target<B>> + Send
    where
        F: FnMut(Self::Inner) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send;
}

impl<A: Send> FunctorAsyncMut for Vec<A> {
    type Inner = A;
    type Target<T: Send> = Vec<T>;

    #[inline]
    async fn fmap_async_mut<B, F, Fut>(self, mut f: F) -> Vec<B>
    where
        F: FnMut(A) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        let mut results = Vec::with_capacity(self.len());
        for item in self {
            results.push(f(item).await);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_fmap_async_type_check() {
        fn assert_functor_async<T: FunctorAsync>() {}
        assert_functor_async::<Option<i32>>();
    }

    #[test]
    fn test_result_fmap_async_type_check() {
        fn assert_functor_async<T: FunctorAsync>() {}
        assert_functor_async::<Result<i32, &str>>();
    }

    #[test]
    fn test_vec_fmap_async_type_check() {
        fn assert_functor_async<T: FunctorAsync>() {}
        assert_functor_async::<Vec<i32>>();
    }
}
