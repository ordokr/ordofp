//! `TraversableAsync` - Async Traversal of Data Structures
//!
//! > *"Per omnia elementa"*
//! > — Through all elements. (Scholastic philosophy)
//!
//! `TraversableAsync` provides async traversal operations for data structures,
//! enabling parallel or sequential async transformations.
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
//! use ordofp_core::async_core::TraversableAsync;
//!
//! let items = vec![1, 2, 3, 4, 5];
//!
//! // Traverse sequentially, applying an async function to each element
//! let doubled: Vec<i32> = block_on(
//!     items.traverse_async(|x| async move { x * 2 }),
//! );
//!
//! assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
//! ```
//!
//! # Design note: deliberate sync mirror
//!
//! This module is a deliberate, line-for-line mirror of its synchronous
//! counterpart [`ordofp_core::traversable::Traversable`] (see
//! `core/src/traversable.rs`), with each method wrapped in
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

use alloc::vec;
use alloc::vec::Vec;
use core::future::Future;

/// Trait for types that can be traversed with async effects.
///
/// `TraversableAsync` extends the concept of `Traversable` to async contexts,
/// allowing you to apply async functions to each element of a structure while
/// collecting the results.
///
/// # Laws
///
/// Async traversable should satisfy these laws (adapted from Haskell):
///
/// 1. **Identity**: `traverse_async(|x| async { x }) == async { self }`
/// 2. **Naturality**: For natural transformations, traversal commutes
///
/// # Scholastic Etymology
///
/// The operation relates to *transire* (Latin: to go across), representing
/// the passage through all elements of a structure.
pub trait TraversableAsync {
    /// The type of elements in this structure.
    type Elem;

    /// Traverse the structure, applying an async function to each element.
    ///
    /// This processes elements sequentially, awaiting each result before
    /// proceeding to the next.
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
    /// use ordofp_core::async_core::TraversableAsync;
    ///
    /// let results: Vec<String> = block_on(
    ///     vec![1, 2, 3].traverse_async(|x| async move { format!("{}", x) }),
    /// );
    /// assert_eq!(results, vec!["1", "2", "3"]);
    /// ```
    fn traverse_async<B, F, Fut>(self, f: F) -> impl Future<Output = Vec<B>> + Send
    where
        F: Fn(Self::Elem) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send;

    /// Sequence a structure of futures into a future of a structure.
    ///
    /// This is `traverse_async` with the identity function.
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
    /// use ordofp_core::async_core::TraversableAsync;
    /// use core::future::ready;
    ///
    /// // `ready` gives every element the same concrete `Future` type, unlike
    /// // three separate `async {}` blocks, which would each have a distinct type.
    /// let futures = vec![ready(1), ready(2), ready(3)];
    /// let results = block_on(futures.sequence_async());
    /// assert_eq!(results, vec![1, 2, 3]);
    /// ```
    #[inline]
    fn sequence_async(self) -> impl Future<Output = Vec<<Self::Elem as IntoFuture>::Output>> + Send
    where
        Self: Sized,
        Self::Elem: IntoFuture + Send,
        <Self::Elem as IntoFuture>::Output: Send,
        <Self::Elem as IntoFuture>::IntoFuture: Send,
    {
        self.traverse_async(core::future::IntoFuture::into_future)
    }
}

/// Trait for parallel async traversal.
///
/// Intended as a concurrent counterpart to `TraversableAsync`.
///
/// **Current status:** the provided implementations await elements
/// *sequentially* — true concurrency requires a runtime (tokio/smol)
/// and has not been wired in. The trait exists so the signature stays
/// stable when that lands.
pub trait TraversableAsyncParallel {
    /// The element type yielded by the structure being traversed; each
    /// element is fed to the effectful function in order.
    type Elem;

    /// Traverse the structure, collecting results in order.
    ///
    /// Currently awaits each element's future sequentially (see the
    /// trait-level status note).
    fn traverse_async_par<B, F, Fut>(self, f: F) -> impl Future<Output = Vec<B>> + Send
    where
        F: Fn(Self::Elem) -> Fut + Send + Sync,
        Fut: Future<Output = B> + Send,
        B: Send;
}

// ============================================================================
// Implementations for Vec
// ============================================================================

impl<T: Send> TraversableAsync for Vec<T> {
    type Elem = T;

    #[inline]
    async fn traverse_async<B, F, Fut>(self, f: F) -> Vec<B>
    where
        F: Fn(Self::Elem) -> Fut + Send,
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

impl<T: Send> TraversableAsyncParallel for Vec<T> {
    type Elem = T;

    #[inline]
    async fn traverse_async_par<B, F, Fut>(self, f: F) -> Vec<B>
    where
        F: Fn(Self::Elem) -> Fut + Send + Sync,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        // Sequential awaits — see the trait-level status note.
        let mut results = Vec::with_capacity(self.len());
        for item in self {
            results.push(f(item).await);
        }
        results
    }
}

// ============================================================================
// Implementations for Option
// ============================================================================

impl<T: Send> TraversableAsync for Option<T> {
    type Elem = T;

    #[inline]
    async fn traverse_async<B, F, Fut>(self, f: F) -> Vec<B>
    where
        F: Fn(Self::Elem) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        match self {
            Some(item) => vec![f(item).await],
            None => vec![],
        }
    }
}

/// Extension trait for Option-specific async traversal returning Option.
pub trait OptionTraverseAsync<T> {
    /// Traverse Option, preserving the Option structure.
    fn traverse_option_async<B, F, Fut>(self, f: F) -> impl Future<Output = Option<B>> + Send
    where
        F: FnOnce(T) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send;
}

impl<T: Send> OptionTraverseAsync<T> for Option<T> {
    #[inline]
    async fn traverse_option_async<B, F, Fut>(self, f: F) -> Option<B>
    where
        F: FnOnce(T) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        match self {
            Some(item) => Some(f(item).await),
            None => None,
        }
    }
}

// ============================================================================
// Implementations for Result
// ============================================================================

impl<T: Send, E: Send> TraversableAsync for Result<T, E> {
    type Elem = T;

    /// Collects the mapped `Ok` value, like `Option`'s impl collects `Some`.
    ///
    /// An `Err` yields an **empty Vec — the error value is discarded** by
    /// this flattening traversal. Use
    /// [`ResultTraverseAsync::traverse_result_async`] to preserve the error.
    #[inline]
    async fn traverse_async<B, F, Fut>(self, f: F) -> Vec<B>
    where
        F: Fn(Self::Elem) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        match self {
            Ok(item) => vec![f(item).await],
            Err(_) => vec![],
        }
    }
}

/// Extension trait for Result-specific async traversal returning Result.
pub trait ResultTraverseAsync<T, E> {
    /// Traverse Result, preserving the Result structure.
    fn traverse_result_async<B, F, Fut>(self, f: F) -> impl Future<Output = Result<B, E>> + Send
    where
        F: FnOnce(T) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send;
}

impl<T: Send, E: Send> ResultTraverseAsync<T, E> for Result<T, E> {
    #[inline]
    async fn traverse_result_async<B, F, Fut>(self, f: F) -> Result<B, E>
    where
        F: FnOnce(T) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        match self {
            Ok(item) => Ok(f(item).await),
            Err(e) => Err(e),
        }
    }
}

// ============================================================================
// Implementations for slices/arrays
// ============================================================================

impl<T: Send + Clone, const N: usize> TraversableAsync for [T; N] {
    type Elem = T;

    #[inline]
    async fn traverse_async<B, F, Fut>(self, f: F) -> Vec<B>
    where
        F: Fn(Self::Elem) -> Fut + Send,
        Fut: Future<Output = B> + Send,
        B: Send,
    {
        let mut results = Vec::with_capacity(N);
        for item in self {
            results.push(f(item).await);
        }
        results
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Traverse a vector with an async function, returning results in a new vector.
///
/// This is a free function version of `TraversableAsync::traverse_async`.
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
/// use ordofp_core::async_core::traverse_vec_async;
///
/// let results = block_on(traverse_vec_async(vec![1, 2, 3], |x| async move { x * 2 }));
/// assert_eq!(results, vec![2, 4, 6]);
/// ```
#[inline]
pub async fn traverse_vec_async<T, B, F, Fut>(items: Vec<T>, f: F) -> Vec<B>
where
    T: Send,
    F: Fn(T) -> Fut + Send,
    Fut: Future<Output = B> + Send,
    B: Send,
{
    items.traverse_async(f).await
}

/// Map an async function over an Option, preserving the Option structure.
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
/// use ordofp_core::async_core::map_option_async;
///
/// let result = block_on(map_option_async(Some(5), |x| async move { x * 2 }));
/// assert_eq!(result, Some(10));
/// ```
#[inline]
pub async fn map_option_async<T, B, F, Fut>(opt: Option<T>, f: F) -> Option<B>
where
    T: Send,
    F: FnOnce(T) -> Fut + Send,
    Fut: Future<Output = B> + Send,
    B: Send,
{
    opt.traverse_option_async(f).await
}

/// Map an async function over a Result, preserving the Result structure.
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
/// use ordofp_core::async_core::map_result_async;
///
/// let result: Result<i32, &str> = block_on(map_result_async(Ok(5), |x| async move { x * 2 }));
/// assert_eq!(result, Ok(10));
/// ```
///
/// # Errors
///
/// Returns the input's `Err(e)` unchanged; `f` is not called in that
/// case. No new error values are constructed here.
#[inline]
pub async fn map_result_async<T, E, B, F, Fut>(result: Result<T, E>, f: F) -> Result<B, E>
where
    T: Send,
    E: Send,
    F: FnOnce(T) -> Fut + Send,
    Fut: Future<Output = B> + Send,
    B: Send,
{
    result.traverse_result_async(f).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_traversable_async_vec_compiles() {
        fn check<T: TraversableAsync>() {}
        check::<Vec<i32>>();
    }

    #[test]
    fn test_traversable_async_option_compiles() {
        fn check<T: TraversableAsync>() {}
        check::<Option<i32>>();
    }

    #[test]
    fn test_traversable_async_result_compiles() {
        fn check<T: TraversableAsync>() {}
        check::<Result<i32, String>>();
    }
}
