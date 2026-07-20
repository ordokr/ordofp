//! Async Applicative type class - asynchronous applicative functor.
//!
//! > *"Applicatio formae ad materiam."*
//! > — The application of form to matter. (Scholastic philosophy)
//!
//! `ApplicatioAsync` extends `FunctorAsync` with the ability to lift values
//! into an async context and apply wrapped async functions to wrapped values.
//!
//! # Laws
//!
//! 1. **Identity**: `pure(id).ap_async(v).await == v`
//! 2. **Homomorphism**: `pure(f).ap_async(pure(x)).await == pure(f(x))`
//! 3. **Interchange**: `u.ap_async(pure(y)).await == pure(|f| f(y)).ap_async(u).await`
//! 4. **Composition**: `pure(compose).ap_async(u).ap_async(v).ap_async(w) == u.ap_async(v.ap_async(w))`
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
//! use ordofp_core::async_core::ApplicatioAsync;
//!
//! let x = Some(5);
//! let y = Some(10);
//! let sum = block_on(x.map2_async(y, |a, b| async move { a + b }));
//! assert_eq!(sum, Some(15));
//! ```
//!
//! # Design note: deliberate sync mirror
//!
//! This module is a deliberate, line-for-line mirror of its synchronous
//! counterpart [`ordofp_core::typeclasses::applicative::Applicatio`] (see
//! `core/src/typeclasses/applicative.rs`), with each method wrapped in
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

use super::functor_async::FunctorAsync;

/// Async Applicative Functor - async pure and apply operations.
///
/// This trait extends `FunctorAsync` with:
/// - `pure_async`: Lift a value into the async applicative context
/// - `map2_async`: Combine two wrapped values with an async binary function
///
/// # Laws
///
/// 1. **Identity**: `pure(id).ap_async(v) == v`
/// 2. **Homomorphism**: `pure(f).ap_async(pure(x)) == pure(f(x))`
/// 3. **Interchange**: `u.ap_async(pure(y)) == pure(|f| f(y)).ap_async(u)`
pub trait ApplicatioAsync: FunctorAsync {
    /// Lift a value into the async applicative context.
    ///
    /// This is the async equivalent of `pure` from `Applicatio`.
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
    /// use ordofp_core::async_core::ApplicatioAsync;
    ///
    /// let opt: Option<i32> = block_on(Option::<i32>::pure_async(42));
    /// assert_eq!(opt, Some(42));
    /// ```
    fn pure_async<T: Send>(value: T) -> impl Future<Output = Self::Target<T>> + Send;

    /// Combine two async applicatives with a binary async function.
    ///
    /// This is often more ergonomic than `ap_async` for combining values.
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
    /// use ordofp_core::async_core::ApplicatioAsync;
    ///
    /// let x = Some(5);
    /// let y = Some(10);
    /// let sum = block_on(x.map2_async(y, |a, b| async move { a + b }));
    /// assert_eq!(sum, Some(15));
    /// ```
    /// Per-container pairing semantics: `Option`/`Result` combine their two
    /// (at most single) values; `Vec` combines **pairwise (zip)** up to the
    /// shorter length — for the cartesian product, use
    /// [`ApplicatioAsyncMut::map2_async_mut`], which can clone elements.
    fn map2_async<B, C, F, Fut>(
        self,
        other: Self::Target<B>,
        f: F,
    ) -> impl Future<Output = Self::Target<C>> + Send
    where
        F: FnMut(Self::Inner, B) -> Fut + Send,
        Fut: Future<Output = C> + Send,
        B: Send,
        C: Send;
}

// ============================================================================
// Implementation for Option
// ============================================================================

impl<A: Send> ApplicatioAsync for Option<A> {
    #[inline]
    // `ready` skips the async state machine for this trivially-immediate impl.
    fn pure_async<T: Send>(value: T) -> impl Future<Output = Option<T>> {
        core::future::ready(Some(value))
    }

    #[inline]
    async fn map2_async<B, C, F, Fut>(self, other: Option<B>, mut f: F) -> Option<C>
    where
        F: FnMut(A, B) -> Fut + Send,
        Fut: Future<Output = C> + Send,
        B: Send,
        C: Send,
    {
        match (self, other) {
            (Some(a), Some(b)) => Some(f(a, b).await),
            _ => None,
        }
    }
}

// ============================================================================
// Implementation for Result
// ============================================================================

impl<A: Send, E: Send> ApplicatioAsync for Result<A, E> {
    #[inline]
    // `ready` skips the async state machine for this trivially-immediate impl.
    fn pure_async<T: Send>(value: T) -> impl Future<Output = Result<T, E>> {
        core::future::ready(Ok(value))
    }

    #[inline]
    async fn map2_async<B, C, F, Fut>(self, other: Result<B, E>, mut f: F) -> Result<C, E>
    where
        F: FnMut(A, B) -> Fut + Send,
        Fut: Future<Output = C> + Send,
        B: Send,
        C: Send,
    {
        match (self, other) {
            (Ok(a), Ok(b)) => Ok(f(a, b).await),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

// ============================================================================
// Implementation for Vec
// ============================================================================

impl<A: Send> ApplicatioAsync for Vec<A> {
    #[inline]
    // `ready` skips the async state machine for this trivially-immediate impl.
    fn pure_async<T: Send>(value: T) -> impl Future<Output = Vec<T>> {
        core::future::ready(alloc::vec![value])
    }

    #[inline]
    async fn map2_async<B, C, F, Fut>(self, other: Vec<B>, mut f: F) -> Vec<C>
    where
        F: FnMut(A, B) -> Fut + Send,
        Fut: Future<Output = C> + Send,
        B: Send,
        C: Send,
    {
        // Pairwise (zip) semantics up to the shorter length — see the trait
        // doc; cartesian product lives in map2_async_mut (needs Clone).
        let mut results = Vec::with_capacity(self.len().min(other.len()));
        for (a, b) in self.into_iter().zip(other) {
            results.push(f(a, b).await);
        }
        results
    }
}

/// Extended async applicative with `FnMut` support for Vec operations.
pub trait ApplicatioAsyncMut: FunctorAsync {
    /// Combine all pairs from two containers with an async function.
    fn map2_async_mut<B, C, F, Fut>(
        self,
        other: Self::Target<B>,
        f: F,
    ) -> impl Future<Output = Self::Target<C>> + Send
    where
        F: FnMut(Self::Inner, B) -> Fut + Send,
        Fut: Future<Output = C> + Send,
        B: Send + Clone + Sync,
        C: Send,
        Self::Inner: Clone + Sync;
}

impl<A: Send + Clone + Sync> ApplicatioAsyncMut for Vec<A> {
    #[inline]
    async fn map2_async_mut<B, C, F, Fut>(self, other: Vec<B>, mut f: F) -> Vec<C>
    where
        F: FnMut(A, B) -> Fut + Send,
        Fut: Future<Output = C> + Send,
        B: Send + Clone + Sync,
        C: Send,
    {
        let mut results = Vec::with_capacity(self.len() * other.len());
        // Move self and other into owned vectors to avoid iterator issues
        let self_vec = self;
        let other_vec = other;
        for a in &self_vec {
            for b in &other_vec {
                results.push(f(a.clone(), b.clone()).await);
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_applicative_async_type_check() {
        fn assert_applicative_async<T: ApplicatioAsync>() {}
        assert_applicative_async::<Option<i32>>();
    }

    #[test]
    fn test_result_applicative_async_type_check() {
        fn assert_applicative_async<T: ApplicatioAsync>() {}
        assert_applicative_async::<Result<i32, &str>>();
    }

    #[test]
    fn test_vec_applicative_async_type_check() {
        fn assert_applicative_async<T: ApplicatioAsync>() {}
        assert_applicative_async::<Vec<i32>>();
    }
}
