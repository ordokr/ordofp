//! `OptionTAsync` - Async Option Monad Transformer
//!
//! > *"Possibilitas sine actualitate"*
//! > — Possibility without actuality. (Scholastic philosophy)
//!
//! `OptionTAsync` wraps async computations that may or may not produce a value,
//! combining optionality with async effects.
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
//! use ordofp_core::transformers::async_transforms::OptionTAsync;
//!
//! struct User {
//!     id: i32,
//! }
//!
//! fn fetch_profile_async(id: i32) -> OptionTAsync<String> {
//!     OptionTAsync::some(format!("profile-{}", id))
//! }
//!
//! let maybe_user: OptionTAsync<User> = OptionTAsync::some(User { id: 1 });
//!
//! let result = block_on(
//!     maybe_user
//!         .fmap(|u| u.id)
//!         .flat_map(|id| fetch_profile_async(id))
//!         .run(),
//! );
//! assert_eq!(result, Some("profile-1".to_string()));
//! ```

use alloc::boxed::Box;
use core::future::Future;

use super::{BoxFuture, MonadTransformerAsync};

/// Async Option Monad Transformer.
///
/// `OptionTAsync<A>` represents an async computation that may or may not
/// produce a value of type `A`.
///
/// This is essentially `Future<Output = Option<A>>` with monadic operations.
///
/// # Scholastic Etymology
///
/// The concept relates to scholastic notions of *possibilitas* (possibility)
/// and *actualitas* (actuality) - a value may potentially exist (Some)
/// or be absent (None).
pub struct OptionTAsync<A> {
    /// The wrapped async computation.
    inner: BoxFuture<Option<A>>,
}

impl<A> OptionTAsync<A>
where
    A: Send + 'static,
{
    /// Create an `OptionTAsync` from a future that produces an Option.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt = OptionTAsync::new(async { Some(42) });
    /// let result = block_on(opt.run());
    /// assert_eq!(result, Some(42));
    /// ```
    #[inline]
    pub fn new<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = Option<A>> + Send + 'static,
    {
        OptionTAsync {
            inner: Box::pin(fut),
        }
    }

    /// Create an `OptionTAsync` containing a value.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt: OptionTAsync<i32> = OptionTAsync::some(42);
    /// assert_eq!(block_on(opt.run()), Some(42));
    /// ```
    #[inline]
    pub fn some(value: A) -> Self {
        OptionTAsync::new(async move { Some(value) })
    }

    /// Create an empty `OptionTAsync`.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt: OptionTAsync<i32> = OptionTAsync::none();
    /// assert_eq!(block_on(opt.run()), None);
    /// ```
    #[inline]
    pub fn none() -> Self {
        OptionTAsync::new(async { None })
    }

    /// Create a pure `OptionTAsync` (alias for `some`).
    #[inline]
    pub fn purus(value: A) -> Self {
        Self::some(value)
    }

    /// Run the async computation and get the Option result.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt = OptionTAsync::some(42);
    /// let result = block_on(opt.run());
    /// assert_eq!(result, Some(42));
    /// ```
    #[inline]
    pub async fn run(self) -> Option<A> {
        self.inner.await
    }

    /// Transform the inner value using a function.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt = OptionTAsync::some(21).fmap(|x| x * 2);
    /// let result = block_on(opt.run());
    /// assert_eq!(result, Some(42));
    /// ```
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> OptionTAsync<B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        B: Send + 'static,
    {
        OptionTAsync::new(async move { self.inner.await.map(f) })
    }

    /// Chain this computation with another async option computation.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt = OptionTAsync::some(5)
    ///     .flat_map(|x| {
    ///         if x > 0 {
    ///             OptionTAsync::some(x * 2)
    ///         } else {
    ///             OptionTAsync::none()
    ///         }
    ///     });
    /// assert_eq!(block_on(opt.run()), Some(10));
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> OptionTAsync<B>
    where
        F: FnOnce(A) -> OptionTAsync<B> + Send + 'static,
        B: Send + 'static,
    {
        OptionTAsync::new(async move {
            match self.inner.await {
                Some(a) => f(a).run().await,
                None => None,
            }
        })
    }

    /// Alias for `flat_map` using traditional Haskell naming.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> OptionTAsync<B>
    where
        F: FnOnce(A) -> OptionTAsync<B> + Send + 'static,
        B: Send + 'static,
    {
        self.flat_map(f)
    }

    /// Alias for `flat_map` using Scala naming.
    #[inline]
    pub fn and_then<B, F>(self, f: F) -> OptionTAsync<B>
    where
        F: FnOnce(A) -> OptionTAsync<B> + Send + 'static,
        B: Send + 'static,
    {
        self.flat_map(f)
    }

    /// Filter the value based on a predicate.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt = OptionTAsync::some(10)
    ///     .filter(|x| *x > 5);
    /// assert_eq!(block_on(opt.run()), Some(10));
    ///
    /// let opt2 = OptionTAsync::some(3)
    ///     .filter(|x| *x > 5);
    /// assert_eq!(block_on(opt2.run()), None);
    /// ```
    #[inline]
    pub fn filter<F>(self, predicate: F) -> OptionTAsync<A>
    where
        F: FnOnce(&A) -> bool + Send + 'static,
    {
        OptionTAsync::new(async move {
            match self.inner.await {
                Some(a) if predicate(&a) => Some(a),
                _ => None,
            }
        })
    }

    /// Provide a default value if None.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt: OptionTAsync<i32> = OptionTAsync::none();
    /// let with_default = opt.or_else(|| OptionTAsync::some(0));
    /// assert_eq!(block_on(with_default.run()), Some(0));
    /// ```
    #[inline]
    pub fn or_else<F>(self, default: F) -> OptionTAsync<A>
    where
        F: FnOnce() -> OptionTAsync<A> + Send + 'static,
    {
        OptionTAsync::new(async move {
            match self.inner.await {
                Some(a) => Some(a),
                None => default().run().await,
            }
        })
    }

    /// Unwrap the value or return a default.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt: OptionTAsync<i32> = OptionTAsync::none();
    /// let result = block_on(opt.unwrap_or(42));
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub async fn unwrap_or(self, default: A) -> A {
        self.inner.await.unwrap_or(default)
    }

    /// Unwrap the value or compute a default.
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
    /// use ordofp_core::transformers::async_transforms::OptionTAsync;
    ///
    /// let opt: OptionTAsync<i32> = OptionTAsync::none();
    /// let result = block_on(opt.unwrap_or_else(|| 42));
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub async fn unwrap_or_else<F>(self, default: F) -> A
    where
        F: FnOnce() -> A,
    {
        self.inner.await.unwrap_or_else(default)
    }

    /// Combine two async options.
    #[inline]
    pub fn map2<B, C, F>(self, other: OptionTAsync<B>, f: F) -> OptionTAsync<C>
    where
        F: FnOnce(A, B) -> C + Send + 'static,
        B: Send + 'static,
        C: Send + 'static,
    {
        OptionTAsync::new(async move {
            match (self.inner.await, other.inner.await) {
                (Some(a), Some(b)) => Some(f(a, b)),
                _ => None,
            }
        })
    }

    /// Sequence this computation before another, discarding the first result.
    #[inline]
    pub fn then<B>(self, next: OptionTAsync<B>) -> OptionTAsync<B>
    where
        B: Send + 'static,
    {
        self.flat_map(move |_| next)
    }

    /// Check if the value is Some.
    #[inline]
    pub async fn is_some(self) -> bool {
        self.inner.await.is_some()
    }

    /// Check if the value is None.
    #[inline]
    pub async fn is_none(self) -> bool {
        self.inner.await.is_none()
    }

    /// Convert to Result, using the provided error for None.
    #[inline]
    pub fn ok_or<E>(self, err: E) -> super::EitherTAsync<E, A>
    where
        E: Send + 'static,
    {
        super::EitherTAsync::new(async move {
            match self.inner.await {
                Some(a) => Ok(a),
                None => Err(err),
            }
        })
    }

    /// Convert to Result, computing the error for None.
    #[inline]
    pub fn ok_or_else<E, F>(self, err: F) -> super::EitherTAsync<E, A>
    where
        E: Send + 'static,
        F: FnOnce() -> E + Send + 'static,
    {
        super::EitherTAsync::new(async move {
            match self.inner.await {
                Some(a) => Ok(a),
                None => Err(err()),
            }
        })
    }
}

/// Lift an Option into `OptionTAsync`.
impl<A: Send + 'static> From<Option<A>> for OptionTAsync<A> {
    #[inline]
    fn from(opt: Option<A>) -> Self {
        OptionTAsync::new(async move { opt })
    }
}

impl<A> MonadTransformerAsync for OptionTAsync<A>
where
    A: Send + 'static,
{
    type Output = Option<A>;

    #[inline]
    fn lift_async<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = Option<A>> + Send + 'static,
    {
        OptionTAsync::new(fut)
    }
}

impl<A> core::fmt::Debug for OptionTAsync<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OptionTAsync")
            .field("inner", &"<future>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_t_async_some() {
        let _opt: OptionTAsync<i32> = OptionTAsync::some(42);
    }

    #[test]
    fn test_option_t_async_none() {
        let _opt: OptionTAsync<i32> = OptionTAsync::none();
    }

    #[test]
    fn test_option_t_async_from_option() {
        let _opt: OptionTAsync<i32> = Some(42).into();
        let _opt2: OptionTAsync<i32> = None.into();
    }

    #[test]
    fn test_option_t_async_debug() {
        let opt: OptionTAsync<i32> = OptionTAsync::some(42);
        let debug = alloc::format!("{opt:?}");
        assert!(debug.contains("OptionTAsync"));
    }
}
