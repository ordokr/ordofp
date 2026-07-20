//! Futurus - A monadic wrapper around Future.
//!
//! > *"Quod futurum est, in potentia est."*
//! > — What is future, exists in potentiality. (Aristotelian metaphysics)
//!
//! `Futurus` wraps a `Future` and provides monadic operations (`fmap`, `flat_map`,
//! `pure`) that compose asynchronous computations in a functional style.
//!
//! # Overview
//!
//! `Futurus<T>` is to `Future` what `Option<T>` is to nullable values - a wrapper
//! that provides a consistent, composable interface for working with async values.
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
//! use ordofp_core::async_core::Futurus;
//!
//! async fn example() -> i32 {
//!     // Create from a value
//!     let fut = Futurus::purus(42);
//!
//!     // Map over the future
//!     let doubled = fut.fmap(|x| x * 2);
//!
//!     // Chain async computations
//!     doubled
//!         .flat_map(|x| Futurus::purus(x + 1))
//!         .await
//! }
//!
//! assert_eq!(block_on(example()), 85);
//! ```
//!
//! # Design Notes
//!
//! `Futurus` uses an internal enum with two variants for optimization:
//! - `Purus`: A pure value (no async computation needed)
//! - `Effectus`: A boxed async computation
//!
//! The two-variant split avoids boxing in the common pure case.

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// A monadic wrapper around `Future` with functional programming operations.
///
/// `Futurus` (Latin: "about to be") represents an asynchronous computation
/// that will eventually produce a value of type `T`.
///
/// # Scholastic Etymology
///
/// From Latin *futurus*, the future active participle of *esse* (to be),
/// meaning "that which is about to be" or "that which will come to be."
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
/// use ordofp_core::async_core::Futurus;
///
/// async fn example() -> i32 {
///     Futurus::purus(5)
///         .fmap(|x| x * 2)
///         .flat_map(|x| Futurus::purus(x + 1))
///         .await
/// }
///
/// assert_eq!(block_on(example()), 11);
/// ```
pub struct Futurus<T> {
    inner: FuturusInner<T>,
}

/// The internal representation of a Futurus computation.
enum FuturusInner<T> {
    /// A pure value - no async computation needed.
    /// *"Actus purus"* - Pure act, already realized.
    Purus(Option<T>),

    /// An async computation that will produce T.
    /// *"Potentia ad actum"* - Potentiality toward actuality.
    Effectus(Pin<Box<dyn Future<Output = T> + Send + 'static>>),
}

impl<T> Futurus<T> {
    /// Create a Futurus containing a pure value.
    ///
    /// This is the monadic `pure` or `return` operation.
    /// No async computation is performed - the value is immediately available.
    ///
    /// # Scholastic Note
    ///
    /// *"Purus"* (Latin: pure) - a value in its actualized form,
    /// requiring no further transformation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::async_core::Futurus;
    ///
    /// let fut: Futurus<i32> = Futurus::purus(42);
    /// let _ = fut;
    /// ```
    #[inline]
    pub fn purus(value: T) -> Self {
        Futurus {
            inner: FuturusInner::Purus(Some(value)),
        }
    }

    /// Alias for `purus` using English naming.
    #[inline]
    pub fn pure(value: T) -> Self {
        Self::purus(value)
    }

    /// Create a Futurus from an async computation.
    ///
    /// The computation will be executed when the Futurus is awaited.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::async_core::Futurus;
    ///
    /// let fut = Futurus::new(async {
    ///     // Some async work...
    ///     42
    /// });
    /// let _: Futurus<i32> = fut;
    /// ```
    #[inline]
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Futurus {
            inner: FuturusInner::Effectus(Box::pin(future)),
        }
    }

    /// Create a Futurus from a function that produces a Future.
    ///
    /// This allows lazy creation of the underlying computation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::async_core::Futurus;
    ///
    /// let fut = Futurus::delay(|| async { 42 });
    /// let _: Futurus<i32> = fut;
    /// ```
    #[inline]
    pub fn delay<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        Futurus {
            inner: FuturusInner::Effectus(Box::pin(async move { f().await })),
        }
    }
}

impl<T: Send + 'static> Futurus<T> {
    /// Map a function over the result of this Futurus.
    ///
    /// This is the Functor `fmap` operation.
    ///
    /// # Laws
    ///
    /// - Identity: `fut.fmap(|x| x) == fut`
    /// - Composition: `fut.fmap(f).fmap(g) == fut.fmap(|x| g(f(x)))`
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
    /// use ordofp_core::async_core::Futurus;
    ///
    /// let doubled = Futurus::purus(5).fmap(|x| x * 2);
    /// assert_eq!(block_on(doubled), 10);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the internal `Purus` slot was already consumed — an
    /// invariant that cannot be violated through the public API, so firing
    /// indicates a bug in this crate. The panic is deferred: it raises when
    /// the returned `Futurus` is awaited, not inside `fmap` itself.
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> Futurus<B>
    where
        F: FnOnce(T) -> B + Send + 'static,
        B: Send + 'static,
    {
        match self.inner {
            FuturusInner::Purus(Some(value)) => {
                // Fast path: pure value, apply function directly
                Futurus::purus(f(value))
            }
            FuturusInner::Purus(None) => {
                // This shouldn't happen in normal use, but handle gracefully
                Futurus {
                    inner: FuturusInner::Effectus(Box::pin(async move {
                        panic!("Futurus::Purus was already consumed")
                    })),
                }
            }
            FuturusInner::Effectus(fut) => {
                // Wrap the future with the mapping function
                Futurus {
                    inner: FuturusInner::Effectus(Box::pin(async move { f(fut.await) })),
                }
            }
        }
    }

    /// Alias for `fmap` using standard Rust naming.
    #[inline]
    pub fn map<B, F>(self, f: F) -> Futurus<B>
    where
        F: FnOnce(T) -> B + Send + 'static,
        B: Send + 'static,
    {
        self.fmap(f)
    }

    /// Chain this Futurus with a function that returns another Futurus.
    ///
    /// This is the Monad `bind` or `>>=` operation.
    ///
    /// # Laws
    ///
    /// - Left Identity: `Futurus::purus(a).flat_map(f) == f(a)`
    /// - Right Identity: `m.flat_map(Futurus::purus) == m`
    /// - Associativity: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
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
    /// use ordofp_core::async_core::Futurus;
    ///
    /// let result = block_on(
    ///     Futurus::purus(5)
    ///         .flat_map(|x| Futurus::purus(x * 2))
    /// );
    /// assert_eq!(result, 10);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the internal `Purus` slot was already consumed — an
    /// invariant that cannot be violated through the public API, so firing
    /// indicates a bug in this crate. The panic is deferred: it raises when
    /// the returned `Futurus` is awaited, not inside `flat_map` itself.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Futurus<B>
    where
        F: FnOnce(T) -> Futurus<B> + Send + 'static,
        B: Send + Unpin + 'static,
    {
        match self.inner {
            FuturusInner::Purus(Some(value)) => {
                // Fast path: pure value, apply function directly
                f(value)
            }
            FuturusInner::Purus(None) => Futurus {
                inner: FuturusInner::Effectus(Box::pin(async move {
                    panic!("Futurus::Purus was already consumed")
                })),
            },
            FuturusInner::Effectus(fut) => Futurus {
                inner: FuturusInner::Effectus(Box::pin(async move { f(fut.await).await })),
            },
        }
    }

    /// Alias for `flat_map` using Haskell naming.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> Futurus<B>
    where
        F: FnOnce(T) -> Futurus<B> + Send + 'static,
        B: Send + Unpin + 'static,
    {
        self.flat_map(f)
    }

    /// Alias for `flat_map` using Scala/Rust naming.
    #[inline]
    pub fn and_then<B, F>(self, f: F) -> Futurus<B>
    where
        F: FnOnce(T) -> Futurus<B> + Send + 'static,
        B: Send + Unpin + 'static,
    {
        self.flat_map(f)
    }

    /// Apply a Futurus containing a function to this Futurus.
    ///
    /// This is the Applicative `ap` operation.
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
    /// use ordofp_core::async_core::Futurus;
    ///
    /// let f = Futurus::purus(|x: i32| x * 2);
    /// let x = Futurus::purus(5);
    /// let result = block_on(x.ap(f));
    /// assert_eq!(result, 10);
    /// ```
    #[inline]
    pub fn ap<B, F>(self, ff: Futurus<F>) -> Futurus<B>
    where
        F: FnOnce(T) -> B + Send + 'static,
        B: Send + Unpin + 'static,
    {
        // Use flat_map to sequence the computations
        ff.flat_map(move |f| self.fmap(f))
    }

    /// Combine two Futurus values with a binary function.
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
    /// use ordofp_core::async_core::Futurus;
    ///
    /// let sum = Futurus::purus(5).map2(Futurus::purus(10), |a, b| a + b);
    /// assert_eq!(block_on(sum), 15);
    /// ```
    #[inline]
    pub fn map2<B, C, F>(self, other: Futurus<B>, f: F) -> Futurus<C>
    where
        F: FnOnce(T, B) -> C + Send + 'static,
        B: Send + 'static,
        C: Send + Unpin + 'static,
    {
        self.flat_map(move |a| other.fmap(move |b| f(a, b)))
    }

    /// Sequence two Futurus, keeping the result of the second.
    ///
    /// Equivalent to `self.map2(other, |_, b| b)`.
    #[inline]
    pub fn then<B>(self, other: Futurus<B>) -> Futurus<B>
    where
        B: Send + Unpin + 'static,
    {
        self.flat_map(move |_| other)
    }

    /// Sequence two Futurus, keeping the result of the first.
    #[inline]
    pub fn skip<B>(self, other: Futurus<B>) -> Futurus<T>
    where
        B: Send + 'static,
        T: Unpin,
    {
        self.flat_map(move |a| other.fmap(move |_| a))
    }

    /// Replace the result with a constant value.
    #[inline]
    pub fn as_const<B>(self, b: B) -> Futurus<B>
    where
        B: Send + 'static,
    {
        self.fmap(move |_| b)
    }

    /// Discard the result, keeping only the effect.
    #[inline]
    pub fn void(self) -> Futurus<()> {
        self.fmap(|_| ())
    }
}

// ============================================================================
// Future implementation
// ============================================================================

impl<T: Unpin> Future for Futurus<T> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We only access inner mutably, never move out of self while pinned
        // get_mut() requires Self: Unpin, which implies T: Unpin.
        let inner = &mut self.as_mut().get_mut().inner;

        match inner {
            FuturusInner::Purus(opt) => {
                // Take the value out of the Option
                match opt.take() {
                    Some(value) => Poll::Ready(value),
                    None => panic!("Futurus::Purus polled after completion"),
                }
            }
            FuturusInner::Effectus(fut) => {
                // Poll the inner future
                fut.as_mut().poll(cx)
            }
        }
    }
}

// ============================================================================
// Additional utilities
// ============================================================================

impl<T: Send + 'static> Futurus<Futurus<T>> {
    /// Flatten a nested Futurus.
    ///
    /// Equivalent to `self.flat_map(|x| x)`.
    #[inline]
    pub fn flatten(self) -> Futurus<T>
    where
        T: Unpin,
    {
        self.flat_map(|inner| inner)
    }
}

// No `Clone` impl: futures are not Clone, so a `Clone` for Futurus could
// only panic on the Effectus/consumed variants — a footgun, so it was
// removed. To duplicate a pure value, construct a fresh
// `Futurus::purus(value.clone())` explicitly.

impl<T: Default> Default for Futurus<T> {
    fn default() -> Self {
        Futurus::purus(T::default())
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Futurus<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.inner {
            FuturusInner::Purus(Some(value)) => {
                f.debug_tuple("Futurus::Purus").field(value).finish()
            }
            FuturusInner::Purus(None) => f
                .debug_tuple("Futurus::Purus")
                .field(&"<consumed>")
                .finish(),
            FuturusInner::Effectus(_) => f
                .debug_tuple("Futurus::Effectus")
                .field(&"<future>")
                .finish(),
        }
    }
}

// ============================================================================
// Conversions
// ============================================================================

impl<T> From<T> for Futurus<T> {
    #[inline]
    fn from(value: T) -> Self {
        Futurus::purus(value)
    }
}

// ============================================================================
// Send + Sync implementations
// ============================================================================

// Futurus is Send if T is Send (Purus case) or the inner future is Send (Effectus case)
// The Effectus variant always contains a Send future by construction
// We rely on auto-derived Send implementation.
// Purus variant holds Option<T> which is Send if T is Send.

// Futurus is NOT Sync because the Effectus variant contains Pin<Box<dyn Future + Send>>,
// which is !Sync. We cannot conditionally implement Sync based on the runtime variant.
// Therefore, Futurus<T> is !Sync even if T is Sync.

// Futurus is Unpin if T is Unpin.
// - Purus contains Option<T>, which is Unpin if T is Unpin.
// - Effectus contains Pin<Box<...>> which is always Unpin.
// We rely on auto-derived Unpin impl, so Futurus<T> is !Unpin if T is !Unpin.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_futurus_purus_creation() {
        let fut: Futurus<i32> = Futurus::purus(42);
        let _: Futurus<i32> = fut;
    }

    #[test]
    fn test_futurus_fmap_type_check() {
        let fut = Futurus::purus(5);
        let _mapped: Futurus<i32> = fut.fmap(|x| x * 2);
    }

    #[test]
    fn test_futurus_flat_map_type_check() {
        let fut = Futurus::purus(5);
        let _chained: Futurus<i32> = fut.flat_map(|x| Futurus::purus(x * 2));
    }

    #[test]
    fn test_futurus_map2_type_check() {
        let a = Futurus::purus(5);
        let b = Futurus::purus(10);
        let _sum: Futurus<i32> = a.map2(b, |x, y| x + y);
    }

    #[test]
    fn test_futurus_debug() {
        let fut = Futurus::purus(42);
        let debug_str = alloc::format!("{fut:?}");
        assert!(debug_str.contains("Purus"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_futurus_from() {
        let fut: Futurus<i32> = 42.into();
        let _: Futurus<i32> = fut;
    }

    #[test]
    fn test_futurus_default() {
        let fut: Futurus<i32> = Futurus::default();
        let _: Futurus<i32> = fut;
    }

    // Note: `Futurus` deliberately has no `Clone` impl — a clone has no
    // sound meaning for Effectus/consumed states.
}
