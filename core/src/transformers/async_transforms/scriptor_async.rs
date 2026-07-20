//! `ScriptorAsync` - Async Writer Monad Transformer
//!
//! > *"Verba volant, scripta manent"*
//! > — Spoken words fly away, written words remain. (Latin proverb)
//!
//! `ScriptorAsync` wraps async computations that produce both a value and
//! accumulated output (like logs), combining logging/accumulation with async effects.
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
//! use ordofp_core::transformers::async_transforms::ScriptorAsync;
//!
//! // Create a writer that logs messages
//! let computation = ScriptorAsync::<Vec<String>, ()>::tell(vec!["Starting...".to_string()])
//!     .then(ScriptorAsync::purus(42))
//!     .flat_map(|x| {
//!         ScriptorAsync::<Vec<String>, ()>::tell(vec![format!("Got value: {}", x)])
//!             .then(ScriptorAsync::purus(x * 2))
//!     });
//!
//! let (logs, result) = block_on(computation.run());
//! assert_eq!(result, 84);
//! assert_eq!(logs, vec!["Starting...", "Got value: 42"]);
//! ```

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;

use super::{BoxFuture, MonadTransformerAsync};

/// Async Writer Monad Transformer.
///
/// `ScriptorAsync<W, A>` represents an async computation that produces a value
/// of type `A` along with accumulated output of type `W` (which must be a Monoid).
///
/// # Type Parameters
///
/// - `W`: The output/log type (must implement `Default` for empty and be combinable)
/// - `A`: The result type produced by the computation
///
/// # Scholastic Etymology
///
/// *Scriptor* (Latin: writer, scribe) derives from *scribere* (to write).
/// In scholastic tradition, the *scriptor* was responsible for recording
/// and preserving knowledge - similarly, this transformer "writes" accumulated output.
pub struct ScriptorAsync<W, A> {
    /// The wrapped async computation that produces (output, value).
    inner: BoxFuture<(W, A)>,
}

impl<W, A> ScriptorAsync<W, A>
where
    W: Send + 'static,
    A: Send + 'static,
{
    /// Create a `ScriptorAsync` from a future that produces (output, value).
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
    /// use ordofp_core::transformers::async_transforms::ScriptorAsync;
    ///
    /// let writer = ScriptorAsync::new(async { (vec!["log"], 42) });
    /// let (logs, result) = block_on(writer.run());
    /// assert_eq!(logs, vec!["log"]);
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn new<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = (W, A)> + Send + 'static,
    {
        ScriptorAsync {
            inner: Box::pin(fut),
        }
    }

    /// Create a `ScriptorAsync` with a pure value and empty output.
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
    /// use ordofp_core::transformers::async_transforms::ScriptorAsync;
    ///
    /// let writer: ScriptorAsync<Vec<String>, i32> = ScriptorAsync::purus(42);
    /// let (logs, result) = block_on(writer.run());
    /// assert!(logs.is_empty());
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn purus(value: A) -> Self
    where
        W: Default,
    {
        ScriptorAsync::new(async move { (W::default(), value) })
    }

    /// Write output without producing a meaningful value.
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
    /// use ordofp_core::transformers::async_transforms::ScriptorAsync;
    ///
    /// let writer =
    ///     ScriptorAsync::<Vec<String>, ()>::tell(vec!["Hello".to_string()]);
    /// let (logs, _) = block_on(writer.run());
    /// assert_eq!(logs, vec!["Hello".to_string()]);
    /// ```
    #[inline]
    pub fn tell(output: W) -> ScriptorAsync<W, ()> {
        ScriptorAsync::new(async move { (output, ()) })
    }

    /// Run the async computation and get (output, value).
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
    /// use ordofp_core::transformers::async_transforms::ScriptorAsync;
    ///
    /// let writer = ScriptorAsync::<Vec<&str>, ()>::tell(vec!["log"])
    ///     .then(ScriptorAsync::purus(42));
    /// let (logs, result) = block_on(writer.run());
    /// assert_eq!(logs, vec!["log"]);
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub async fn run(self) -> (W, A) {
        self.inner.await
    }

    /// Run and return only the accumulated output (discard the result).
    #[inline]
    pub async fn exec(self) -> W {
        let (w, _) = self.run().await;
        w
    }

    /// Run and return only the result (discard the output).
    #[inline]
    pub async fn eval(self) -> A {
        let (_, a) = self.run().await;
        a
    }

    /// Transform the result using a function.
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
    /// use ordofp_core::transformers::async_transforms::ScriptorAsync;
    ///
    /// let writer: ScriptorAsync<Vec<String>, i32> = ScriptorAsync::purus(21).fmap(|x| x * 2);
    /// let (_, result) = block_on(writer.run());
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> ScriptorAsync<W, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        B: Send + 'static,
    {
        ScriptorAsync::new(async move {
            let (w, a) = self.inner.await;
            (w, f(a))
        })
    }

    /// Transform the output using a function.
    #[inline]
    pub fn map_output<W2, F>(self, f: F) -> ScriptorAsync<W2, A>
    where
        F: FnOnce(W) -> W2 + Send + 'static,
        W2: Send + 'static,
    {
        ScriptorAsync::new(async move {
            let (w, a) = self.inner.await;
            (f(w), a)
        })
    }

    /// Add additional output to the computation.
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
    /// use ordofp_core::transformers::async_transforms::ScriptorAsync;
    ///
    /// let writer = ScriptorAsync::<Vec<String>, i32>::purus(42)
    ///     .censor(|logs| {
    ///         let mut new_logs = logs;
    ///         new_logs.push("extra log".to_string());
    ///         new_logs
    ///     });
    /// let (logs, result) = block_on(writer.run());
    /// assert_eq!(logs, vec!["extra log".to_string()]);
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn censor<F>(self, f: F) -> ScriptorAsync<W, A>
    where
        F: FnOnce(W) -> W + Send + 'static,
    {
        ScriptorAsync::new(async move {
            let (w, a) = self.inner.await;
            (f(w), a)
        })
    }

    /// Listen to the output produced by this computation.
    ///
    /// Returns both the value and the output as part of the result.
    #[inline]
    pub fn listen(self) -> ScriptorAsync<W, (A, W)>
    where
        W: Clone,
    {
        ScriptorAsync::new(async move {
            let (w, a) = self.inner.await;
            (w.clone(), (a, w))
        })
    }

    /// Access the output and potentially modify it.
    #[inline]
    pub fn listens<B, F>(self, f: F) -> ScriptorAsync<W, (A, B)>
    where
        F: FnOnce(&W) -> B + Send + 'static,
        W: Clone,
        B: Send + 'static,
    {
        ScriptorAsync::new(async move {
            let (w, a) = self.inner.await;
            let b = f(&w);
            (w, (a, b))
        })
    }

    /// Pass the output through a function that can modify it based on the result.
    ///
    /// **No-op stub:** the standard Writer `pass` requires the value type to
    /// be a pair `(a, W -> W)` whose second component rewrites the log; that
    /// shape is not expressible with this signature, so this method
    /// currently returns `self` unchanged and never modifies the log.
    #[inline]
    pub fn pass(self) -> ScriptorAsync<W, A>
    where
        A: Clone,
    {
        // Standard pass implementation would require A = (a, W -> W);
        // until then this is deliberately the identity (see doc above).
        self
    }
}

// Implementations for Vec<T> as the output type (common use case)
impl<T, A> ScriptorAsync<Vec<T>, A>
where
    T: Send + 'static,
    A: Send + 'static,
{
    /// Chain this computation with another, combining outputs.
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
    /// use ordofp_core::transformers::async_transforms::ScriptorAsync;
    ///
    /// let w1 = ScriptorAsync::<Vec<&str>, ()>::tell(vec!["first"]).then(ScriptorAsync::purus(1));
    /// let w2 = ScriptorAsync::<Vec<&str>, ()>::tell(vec!["second"]).then(ScriptorAsync::purus(2));
    /// let combined = w1.flat_map(|x| w2.fmap(move |y| x + y));
    /// let (logs, result) = block_on(combined.run());
    /// assert_eq!(logs, vec!["first", "second"]);
    /// assert_eq!(result, 3);
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> ScriptorAsync<Vec<T>, B>
    where
        F: FnOnce(A) -> ScriptorAsync<Vec<T>, B> + Send + 'static,
        B: Send + 'static,
    {
        ScriptorAsync::new(async move {
            let (mut w1, a) = self.inner.await;
            let (w2, b) = f(a).run().await;
            w1.extend(w2);
            (w1, b)
        })
    }

    /// Alias for `flat_map` using traditional Haskell naming.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> ScriptorAsync<Vec<T>, B>
    where
        F: FnOnce(A) -> ScriptorAsync<Vec<T>, B> + Send + 'static,
        B: Send + 'static,
    {
        self.flat_map(f)
    }

    /// Sequence this computation before another, discarding the first result.
    #[inline]
    pub fn then<B>(self, next: ScriptorAsync<Vec<T>, B>) -> ScriptorAsync<Vec<T>, B>
    where
        B: Send + 'static,
    {
        self.flat_map(move |_| next)
    }

    /// Combine two writers.
    #[inline]
    pub fn map2<B, C, F>(self, other: ScriptorAsync<Vec<T>, B>, f: F) -> ScriptorAsync<Vec<T>, C>
    where
        F: FnOnce(A, B) -> C + Send + 'static,
        B: Send + 'static,
        C: Send + 'static,
    {
        ScriptorAsync::new(async move {
            let (mut w1, a) = self.inner.await;
            let (w2, b) = other.inner.await;
            w1.extend(w2);
            (w1, f(a, b))
        })
    }
}

// Universalis flat_map for any Monoid-like W
impl<W, A> ScriptorAsync<W, A>
where
    W: Send + 'static + Default,
    A: Send + 'static,
{
    /// Chain with Universalis output combination (requires a combine function).
    #[inline]
    pub fn flat_map_with<B, F, C>(self, f: F, combine: C) -> ScriptorAsync<W, B>
    where
        F: FnOnce(A) -> ScriptorAsync<W, B> + Send + 'static,
        C: FnOnce(W, W) -> W + Send + 'static,
        B: Send + 'static,
    {
        ScriptorAsync::new(async move {
            let (w1, a) = self.inner.await;
            let (w2, b) = f(a).run().await;
            (combine(w1, w2), b)
        })
    }
}

impl<W, A> MonadTransformerAsync for ScriptorAsync<W, A>
where
    W: Send + 'static + Default,
    A: Send + 'static,
{
    type Output = (W, A);

    #[inline]
    fn lift_async<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = (W, A)> + Send + 'static,
    {
        ScriptorAsync::new(fut)
    }
}

impl<W, A> core::fmt::Debug for ScriptorAsync<W, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScriptorAsync")
            .field("inner", &"<future>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_scriptor_async_purus() {
        let _writer: ScriptorAsync<Vec<String>, i32> = ScriptorAsync::purus(42);
    }

    #[test]
    fn test_scriptor_async_tell() {
        let _writer: ScriptorAsync<Vec<String>, ()> =
            ScriptorAsync::<Vec<String>, ()>::tell(vec!["log".to_string()]);
    }

    #[test]
    fn test_scriptor_async_debug() {
        let writer: ScriptorAsync<Vec<String>, i32> = ScriptorAsync::purus(42);
        let debug = alloc::format!("{writer:?}");
        assert!(debug.contains("ScriptorAsync"));
    }
}
