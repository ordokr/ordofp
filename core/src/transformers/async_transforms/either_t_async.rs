//! `EitherTAsync` - Async Either/Result Monad Transformer
//!
//! > *"Aut Caesar aut nihil"*
//! > — Either Caesar or nothing. (Latin proverb)
//!
//! `EitherTAsync` wraps async computations that may succeed or fail with an error,
//! combining error handling with async effects.
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
//! use ordofp_core::transformers::async_transforms::EitherTAsync;
//!
//! struct User {
//!     id: i32,
//! }
//!
//! fn fetch_data_async(id: i32) -> EitherTAsync<String, i32> {
//!     EitherTAsync::right(id * 10)
//! }
//!
//! let result: EitherTAsync<String, User> = EitherTAsync::right(User { id: 1 });
//!
//! let processed = result
//!     .fmap(|u| u.id)
//!     .flat_map(|id| fetch_data_async(id))
//!     .map_err(|e: String| format!("Error: {}", e));
//!
//! match block_on(processed.run()) {
//!     Ok(data) => assert_eq!(data, 10),
//!     Err(e) => panic!("unexpected error: {}", e),
//! }
//! ```

use alloc::boxed::Box;
use core::future::Future;

use super::{BoxFuture, MonadTransformerAsync};

/// Async Either/Result Monad Transformer.
///
/// `EitherTAsync<E, A>` represents an async computation that may succeed with
/// a value of type `A` or fail with an error of type `E`.
///
/// This is essentially `Future<Output = Result<A, E>>` with monadic operations.
///
/// # Scholastic Etymology
///
/// *Aut* (Latin: or, either) represents the disjunction - the computation
/// must produce one of two outcomes: success (Right/Ok) or failure (Left/Err).
pub struct EitherTAsync<E, A> {
    /// The wrapped async computation.
    inner: BoxFuture<Result<A, E>>,
}

impl<E, A> EitherTAsync<E, A>
where
    E: Send + 'static,
    A: Send + 'static,
{
    /// Create an `EitherTAsync` from a future that produces a Result.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either = EitherTAsync::new(async { Ok::<_, String>(42) });
    /// let result = block_on(either.run());
    /// assert_eq!(result, Ok(42));
    /// ```
    #[inline]
    pub fn new<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = Result<A, E>> + Send + 'static,
    {
        EitherTAsync {
            inner: Box::pin(fut),
        }
    }

    /// Create an `EitherTAsync` containing a success value.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either: EitherTAsync<String, i32> = EitherTAsync::right(42);
    /// let result = block_on(either.run());
    /// assert_eq!(result, Ok(42));
    /// ```
    #[inline]
    pub fn right(value: A) -> Self {
        EitherTAsync::new(async move { Ok(value) })
    }

    /// Create an `EitherTAsync` containing an error value.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either: EitherTAsync<String, i32> = EitherTAsync::left("error".to_string());
    /// let result = block_on(either.run());
    /// assert_eq!(result, Err("error".to_string()));
    /// ```
    #[inline]
    pub fn left(err: E) -> Self {
        EitherTAsync::new(async move { Err(err) })
    }

    /// Alias for `right` - create a pure success value.
    #[inline]
    pub fn purus(value: A) -> Self {
        Self::right(value)
    }

    /// Alias for `right` using Result terminology.
    #[inline]
    pub fn ok(value: A) -> Self {
        Self::right(value)
    }

    /// Alias for `left` using Result terminology.
    #[inline]
    pub fn err(error: E) -> Self {
        Self::left(error)
    }

    /// Run the async computation and get the Result.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either: EitherTAsync<String, i32> = EitherTAsync::right(42);
    /// let result = block_on(either.run());
    /// assert_eq!(result, Ok(42));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` when the awaited computation resolved to the left
    /// (error) branch — i.e. it was built via `left`, or an earlier step in
    /// the chain short-circuited with an error.
    #[inline]
    pub async fn run(self) -> Result<A, E> {
        self.inner.await
    }

    /// Transform the success value using a function.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either = EitherTAsync::<String, i32>::right(21).fmap(|x| x * 2);
    /// let result = block_on(either.run());
    /// assert_eq!(result, Ok(42));
    /// ```
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> EitherTAsync<E, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        B: Send + 'static,
    {
        EitherTAsync::new(async move {
            match self.inner.await {
                Ok(a) => Ok(f(a)),
                Err(e) => Err(e),
            }
        })
    }

    /// Transform the error value using a function.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either = EitherTAsync::<i32, String>::left(404)
    ///     .map_err(|code| format!("Error {}", code));
    /// let result = block_on(either.run());
    /// assert_eq!(result, Err("Error 404".to_string()));
    /// ```
    #[inline]
    pub fn map_err<E2, F>(self, f: F) -> EitherTAsync<E2, A>
    where
        F: FnOnce(E) -> E2 + Send + 'static,
        E2: Send + 'static,
    {
        EitherTAsync::new(async move {
            match self.inner.await {
                Ok(a) => Ok(a),
                Err(e) => Err(f(e)),
            }
        })
    }

    /// Transform both the success and error values.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either = EitherTAsync::<String, i32>::right(5)
    ///     .bimap(|x| x * 2, |e: String| format!("Error: {}", e));
    /// let result = block_on(either.run());
    /// assert_eq!(result, Ok(10));
    /// ```
    #[inline]
    pub fn bimap<E2, B, F, G>(self, f: F, g: G) -> EitherTAsync<E2, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        G: FnOnce(E) -> E2 + Send + 'static,
        E2: Send + 'static,
        B: Send + 'static,
    {
        EitherTAsync::new(async move {
            match self.inner.await {
                Ok(a) => Ok(f(a)),
                Err(e) => Err(g(e)),
            }
        })
    }

    /// Chain this computation with another async result computation.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either = EitherTAsync::<String, i32>::right(5)
    ///     .flat_map(|x| {
    ///         if x > 0 {
    ///             EitherTAsync::right(x * 2)
    ///         } else {
    ///             EitherTAsync::left("negative".to_string())
    ///         }
    ///     });
    /// let result = block_on(either.run());
    /// assert_eq!(result, Ok(10));
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> EitherTAsync<E, B>
    where
        F: FnOnce(A) -> EitherTAsync<E, B> + Send + 'static,
        B: Send + 'static,
    {
        EitherTAsync::new(async move {
            match self.inner.await {
                Ok(a) => f(a).run().await,
                Err(e) => Err(e),
            }
        })
    }

    /// Alias for `flat_map` using traditional Haskell naming.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> EitherTAsync<E, B>
    where
        F: FnOnce(A) -> EitherTAsync<E, B> + Send + 'static,
        B: Send + 'static,
    {
        self.flat_map(f)
    }

    /// Alias for `flat_map` using Scala naming.
    #[inline]
    pub fn and_then<B, F>(self, f: F) -> EitherTAsync<E, B>
    where
        F: FnOnce(A) -> EitherTAsync<E, B> + Send + 'static,
        B: Send + 'static,
    {
        self.flat_map(f)
    }

    /// Handle an error by providing an alternative computation.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either: EitherTAsync<String, i32> = EitherTAsync::left("error".to_string())
    ///     .handle_error(|_| EitherTAsync::right(0));
    /// assert_eq!(block_on(either.run()), Ok(0));
    /// ```
    #[inline]
    pub fn handle_error<F>(self, handler: F) -> EitherTAsync<E, A>
    where
        F: FnOnce(E) -> EitherTAsync<E, A> + Send + 'static,
    {
        EitherTAsync::new(async move {
            match self.inner.await {
                Ok(a) => Ok(a),
                Err(e) => handler(e).run().await,
            }
        })
    }

    /// Provide a default value for errors.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either: EitherTAsync<String, i32> = EitherTAsync::left("error".to_string());
    /// let with_default = either.or_else(|_| EitherTAsync::right(0));
    /// assert_eq!(block_on(with_default.run()), Ok(0));
    /// ```
    #[inline]
    pub fn or_else<F>(self, default: F) -> EitherTAsync<E, A>
    where
        F: FnOnce(E) -> EitherTAsync<E, A> + Send + 'static,
    {
        self.handle_error(default)
    }

    /// Unwrap the success value or return a default.
    #[inline]
    pub async fn unwrap_or(self, default: A) -> A {
        self.inner.await.unwrap_or(default)
    }

    /// Unwrap the success value or compute a default from the error.
    #[inline]
    pub async fn unwrap_or_else<F>(self, default: F) -> A
    where
        F: FnOnce(E) -> A,
    {
        self.inner.await.unwrap_or_else(default)
    }

    /// Combine two async results.
    #[inline]
    pub fn map2<B, C, F>(self, other: EitherTAsync<E, B>, f: F) -> EitherTAsync<E, C>
    where
        F: FnOnce(A, B) -> C + Send + 'static,
        B: Send + 'static,
        C: Send + 'static,
    {
        EitherTAsync::new(async move {
            match (self.inner.await, other.inner.await) {
                (Ok(a), Ok(b)) => Ok(f(a, b)),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        })
    }

    /// Sequence this computation before another, discarding the first result.
    #[inline]
    pub fn then<B>(self, next: EitherTAsync<E, B>) -> EitherTAsync<E, B>
    where
        B: Send + 'static,
    {
        self.flat_map(move |_| next)
    }

    /// Check if the result is Ok.
    #[inline]
    pub async fn is_ok(self) -> bool {
        self.inner.await.is_ok()
    }

    /// Check if the result is Err.
    #[inline]
    pub async fn is_err(self) -> bool {
        self.inner.await.is_err()
    }

    /// Convert to Option, discarding the error.
    #[inline]
    pub fn to_option(self) -> super::OptionTAsync<A> {
        super::OptionTAsync::new(async move { self.inner.await.ok() })
    }

    /// Ensure a condition holds, or return an error.
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
    /// use ordofp_core::transformers::async_transforms::EitherTAsync;
    ///
    /// let either = EitherTAsync::<String, i32>::right(5)
    ///     .ensure(|x| *x > 0, || "must be positive".to_string());
    /// let result = block_on(either.run());
    /// assert_eq!(result, Ok(5));
    /// ```
    #[inline]
    pub fn ensure<F, G>(self, predicate: F, err: G) -> EitherTAsync<E, A>
    where
        F: FnOnce(&A) -> bool + Send + 'static,
        G: FnOnce() -> E + Send + 'static,
    {
        EitherTAsync::new(async move {
            match self.inner.await {
                Ok(a) if predicate(&a) => Ok(a),
                Ok(_) => Err(err()),
                Err(e) => Err(e),
            }
        })
    }

    /// Swap the error and success types.
    #[inline]
    pub fn swap(self) -> EitherTAsync<A, E> {
        EitherTAsync::new(async move {
            match self.inner.await {
                Ok(a) => Err(a),
                Err(e) => Ok(e),
            }
        })
    }
}

/// Lift a Result into `EitherTAsync`.
impl<E: Send + 'static, A: Send + 'static> From<Result<A, E>> for EitherTAsync<E, A> {
    #[inline]
    fn from(result: Result<A, E>) -> Self {
        EitherTAsync::new(async move { result })
    }
}

impl<E, A> MonadTransformerAsync for EitherTAsync<E, A>
where
    E: Send + 'static,
    A: Send + 'static,
{
    type Output = Result<A, E>;

    #[inline]
    fn lift_async<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = Result<A, E>> + Send + 'static,
    {
        EitherTAsync::new(fut)
    }
}

impl<E, A> core::fmt::Debug for EitherTAsync<E, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EitherTAsync")
            .field("inner", &"<future>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};

    #[test]
    fn test_either_t_async_right() {
        let _either: EitherTAsync<String, i32> = EitherTAsync::right(42);
    }

    #[test]
    fn test_either_t_async_left() {
        let _either: EitherTAsync<String, i32> = EitherTAsync::left("error".to_string());
    }

    #[test]
    fn test_either_t_async_from_result() {
        let _either: EitherTAsync<String, i32> = Ok(42).into();
        let _either2: EitherTAsync<String, i32> = Err("error".to_string()).into();
    }

    #[test]
    fn test_either_t_async_debug() {
        let either: EitherTAsync<String, i32> = EitherTAsync::right(42);
        let debug = alloc::format!("{either:?}");
        assert!(debug.contains("EitherTAsync"));
    }
}
