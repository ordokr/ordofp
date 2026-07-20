//! Flumen - Async Stream Wrapper with Monadic Operations
//!
//! > *"Flumen temporis"*
//! > — The stream of time. (Scholastic philosophy)
//!
//! `Flumen` is a monadic wrapper around async streams, providing functional
//! programming operations for working with sequences of async values.
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
//! use ordofp_core::async_core::Flumen;
//!
//! let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5]);
//!
//! let doubled = stream
//!     .fmap(|x| x * 2)
//!     .filter(|x| *x > 4);
//!
//! let results = block_on(doubled.collect_vec());
//! assert_eq!(results, vec![6, 8, 10]);
//! ```
//!
//! # Scholastic Etymology
//!
//! *Flumen* (Latin: stream, river) derives from *fluere* (to flow).
//! In scholastic natural philosophy, *flumen* represented continuous
//! motion and change - fitting for async streams that flow data over time.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_core::Stream;

#[cfg(feature = "fusion")]
use super::flumen_fusus::{FlumenFusus, Gradus, GradusStep};

/// Type alias for a boxed, pinned, sendable stream.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

/// Async Stream Wrapper with monadic operations.
///
/// `Flumen<T>` wraps an async stream of items of type `T`, providing
/// functional programming combinators like `fmap`, `filter`, `fold`, etc.
///
/// # Type Parameters
///
/// - `T`: The type of items yielded by the stream
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
/// use ordofp_core::async_core::Flumen;
///
/// let stream = Flumen::from_iterator(1..=5);
/// let sum = block_on(stream.fold(0, |acc, x| acc + x));
/// assert_eq!(sum, 15);
/// ```
pub struct Flumen<T> {
    inner: BoxStream<T>,
}

impl<T: Send + 'static> Flumen<T> {
    /// Create a new `Flumen` from a stream.
    ///
    /// # Example
    ///
    /// ```rust
    /// use core::pin::Pin;
    /// use core::task::{Context, Poll};
    /// use futures_core::Stream;
    /// use ordofp_core::async_core::Flumen;
    ///
    /// struct Counter(u32);
    ///
    /// impl Stream for Counter {
    ///     type Item = u32;
    ///
    ///     fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<u32>> {
    ///         if self.0 < 3 {
    ///             self.0 += 1;
    ///             Poll::Ready(Some(self.0))
    ///         } else {
    ///             Poll::Ready(None)
    ///         }
    ///     }
    /// }
    ///
    /// let flumen = Flumen::new(Counter(0));
    /// ```
    #[inline]
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        Flumen {
            inner: Box::pin(stream),
        }
    }

    /// Create a `Flumen` from an iterator.
    ///
    /// Named `from_iterator` rather than implementing `FromIterator`: the
    /// trait's signature cannot express the `Send + Unpin + 'static` bounds
    /// this constructor needs on the iterator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let flumen = Flumen::from_iterator(vec![1, 2, 3]);
    /// let flumen = Flumen::from_iterator(1..=10);
    /// # let _ = flumen;
    /// ```
    #[inline]
    pub fn from_iterator<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + Unpin + 'static,
    {
        Flumen::new(IterStream::new(iter.into_iter()))
    }

    /// Create a `Flumen` yielding a single value.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let flumen = Flumen::once(42);
    /// assert_eq!(block_on(flumen.collect_vec()), vec![42]);
    /// ```
    #[inline]
    pub fn once(value: T) -> Self
    where
        T: Unpin,
    {
        Flumen::from_iterator(core::iter::once(value))
    }

    /// Create an empty `Flumen`.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let flumen: Flumen<i32> = Flumen::empty();
    /// assert_eq!(block_on(flumen.collect_vec()), Vec::<i32>::new());
    /// ```
    #[inline]
    pub fn empty() -> Self
    where
        T: Unpin,
    {
        Flumen::from_iterator(core::iter::empty())
    }

    /// Create a pure `Flumen` (alias for `once`).
    ///
    /// *"Purus"* (Latin: pure) - a single-element stream.
    #[inline]
    pub fn purus(value: T) -> Self
    where
        T: Unpin,
    {
        Self::once(value)
    }

    /// Map a function over stream items.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let doubled = Flumen::from_iterator(vec![1, 2, 3])
    ///     .fmap(|x| x * 2);
    /// assert_eq!(block_on(doubled.collect_vec()), vec![2, 4, 6]);
    /// ```
    #[inline(always)]
    pub fn fmap<B, F>(self, f: F) -> Flumen<B>
    where
        F: Fn(T) -> B + Send + Sync + Unpin + 'static,
        B: Send + 'static,
    {
        Flumen::new(MapStream::new(self.inner, f))
    }

    /// Filter stream items based on a predicate.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let evens = Flumen::from_iterator(1..=10)
    ///     .filter(|x| x % 2 == 0);
    /// assert_eq!(block_on(evens.collect_vec()), vec![2, 4, 6, 8, 10]);
    /// ```
    #[inline(always)]
    pub fn filter<F>(self, predicate: F) -> Flumen<T>
    where
        F: Fn(&T) -> bool + Send + Sync + Unpin + 'static,
    {
        Flumen::new(FilterStream::new(self.inner, predicate))
    }

    /// Filter and map stream items simultaneously.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let parsed = Flumen::from_iterator(vec!["1", "two", "3"])
    ///     .filter_map(|s| s.parse::<i32>().ok());
    /// assert_eq!(block_on(parsed.collect_vec()), vec![1, 3]);
    /// ```
    #[inline(always)]
    pub fn filter_map<B, F>(self, f: F) -> Flumen<B>
    where
        F: Fn(T) -> Option<B> + Send + Sync + Unpin + 'static,
        B: Send + 'static,
    {
        Flumen::new(FilterMapStream::new(self.inner, f))
    }

    /// Take only the first `n` items from the stream.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let first_three = Flumen::from_iterator(1..=10).take(3);
    /// assert_eq!(block_on(first_three.collect_vec()), vec![1, 2, 3]);
    /// ```
    #[inline(always)]
    pub fn take(self, n: usize) -> Flumen<T> {
        Flumen::new(TakeStream::new(self.inner, n))
    }

    /// Skip the first `n` items from the stream.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let after_three = Flumen::from_iterator(1..=5).skip(3);
    /// assert_eq!(block_on(after_three.collect_vec()), vec![4, 5]);
    /// ```
    #[inline(always)]
    pub fn skip(self, n: usize) -> Flumen<T> {
        Flumen::new(SkipStream::new(self.inner, n))
    }

    /// Take items while a predicate holds.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let small = Flumen::from_iterator(1..=10)
    ///     .take_while(|x| *x < 5);
    /// assert_eq!(block_on(small.collect_vec()), vec![1, 2, 3, 4]);
    /// ```
    #[inline(always)]
    pub fn take_while<F>(self, predicate: F) -> Flumen<T>
    where
        F: Fn(&T) -> bool + Send + Sync + Unpin + 'static,
    {
        Flumen::new(TakeWhileStream::new(self.inner, predicate))
    }

    /// Skip items while a predicate holds.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let large = Flumen::from_iterator(1..=10)
    ///     .skip_while(|x| *x < 5);
    /// assert_eq!(block_on(large.collect_vec()), vec![5, 6, 7, 8, 9, 10]);
    /// ```
    #[inline(always)]
    pub fn skip_while<F>(self, predicate: F) -> Flumen<T>
    where
        F: Fn(&T) -> bool + Send + Sync + Unpin + 'static,
    {
        Flumen::new(SkipWhileStream::new(self.inner, predicate))
    }

    /// Chain another stream after this one.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let chained = Flumen::from_iterator(vec![1, 2])
    ///     .chain(Flumen::from_iterator(vec![3, 4]));
    /// assert_eq!(block_on(chained.collect_vec()), vec![1, 2, 3, 4]);
    /// ```
    #[inline(always)]
    pub fn chain(self, other: Flumen<T>) -> Flumen<T> {
        Flumen::new(ChainStream::new(self.inner, other.inner))
    }

    /// Zip two streams together.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let zipped = Flumen::from_iterator(vec![1, 2, 3])
    ///     .zip(Flumen::from_iterator(vec!["a", "b", "c"]));
    /// assert_eq!(block_on(zipped.collect_vec()), vec![(1, "a"), (2, "b"), (3, "c")]);
    /// ```
    #[inline(always)]
    pub fn zip<U: Send + 'static>(self, other: Flumen<U>) -> Flumen<(T, U)> {
        Flumen::new(ZipStream::new(self.inner, other.inner))
    }

    /// Enumerate stream items with their index.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let enumerated = Flumen::from_iterator(vec!["a", "b", "c"]).enumerate();
    /// assert_eq!(block_on(enumerated.collect_vec()), vec![(0, "a"), (1, "b"), (2, "c")]);
    /// ```
    #[inline(always)]
    pub fn enumerate(self) -> Flumen<(usize, T)> {
        Flumen::new(EnumerateStream::new(self.inner))
    }

    /// Scan - a lazy fold that yields intermediate accumulator values.
    ///
    /// Like `fold`, but produces a stream of all intermediate states instead
    /// of just the final result. This enables streaming computation of
    /// running totals, moving averages, and other stateful transformations.
    ///
    /// *"Scrutare"* (Latin: to examine carefully) - examining each step.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// // Running sum
    /// let running_sum = Flumen::from_iterator(vec![1, 2, 3, 4])
    ///     .scan(0, |acc, x| acc + x);
    /// assert_eq!(block_on(running_sum.collect_vec()), vec![1, 3, 6, 10]);
    ///
    /// // Running product
    /// let running_prod = Flumen::from_iterator(vec![1, 2, 3, 4])
    ///     .scan(1, |acc, x| acc * x);
    /// assert_eq!(block_on(running_prod.collect_vec()), vec![1, 2, 6, 24]);
    /// ```
    #[inline(always)]
    pub fn scan<B, F>(self, init: B, f: F) -> Flumen<B>
    where
        B: Clone + Send + Unpin + 'static,
        F: Fn(B, T) -> B + Send + Sync + Unpin + 'static,
    {
        Flumen::new(ScanStream::new(self.inner, init, f))
    }

    /// Scan with state that differs from the output type.
    ///
    /// More flexible than `scan` - allows the accumulator type to differ
    /// from the output type. Returns `None` to terminate the stream early.
    ///
    /// *"Scrutare cum statu"* (Latin: to examine with state)
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// // Take running sum until it exceeds 10
    /// let bounded_sum = Flumen::from_iterator(vec![1, 2, 3, 4, 5, 6])
    ///     .scan_with(0, |acc, x| {
    ///         let new_acc = *acc + x;
    ///         if new_acc > 10 {
    ///             None
    ///         } else {
    ///             *acc = new_acc;
    ///             Some(new_acc)
    ///         }
    ///     });
    /// assert_eq!(block_on(bounded_sum.collect_vec()), vec![1, 3, 6, 10]);
    /// ```
    #[inline(always)]
    pub fn scan_with<S, B, F>(self, init: S, f: F) -> Flumen<B>
    where
        S: Send + Unpin + 'static,
        B: Send + 'static,
        F: FnMut(&mut S, T) -> Option<B> + Send + Sync + Unpin + 'static,
    {
        Flumen::new(ScanWithStream::new(self.inner, init, f))
    }

    /// Windowed aggregation over the stream.
    ///
    /// Collects items into fixed-size windows (non-overlapping).
    /// The last window may have fewer elements if the stream doesn't
    /// divide evenly.
    ///
    /// *"Fenestra"* (Latin: window)
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let windows = Flumen::from_iterator(1..=10)
    ///     .chunks(3);
    /// assert_eq!(block_on(windows.collect_vec()), vec![
    ///     vec![1, 2, 3],
    ///     vec![4, 5, 6],
    ///     vec![7, 8, 9],
    ///     vec![10]
    /// ]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero — a zero-sized chunk could never fill and
    /// the stream would spin forever, so it is rejected eagerly at
    /// construction rather than at first poll.
    #[inline(always)]
    pub fn chunks(self, size: usize) -> Flumen<Vec<T>>
    where
        T: Unpin,
    {
        assert!(size > 0, "chunk size must be greater than 0");
        Flumen::new(ChunksStream::new(self.inner, size))
    }

    /// Inspect each item without modifying the stream.
    ///
    /// Useful for debugging.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let stream = Flumen::from_iterator(1..=3)
    ///     .inspect(|x| println!("Got: {}", x));
    /// let items = block_on(stream.collect_vec());
    /// assert_eq!(items, vec![1, 2, 3]);
    /// ```
    #[inline(always)]
    pub fn inspect<F>(self, f: F) -> Flumen<T>
    where
        F: Fn(&T) + Send + Sync + Unpin + 'static,
    {
        self.fmap(move |x| {
            f(&x);
            x
        })
    }

    /// Fold the stream to a single value.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let sum = block_on(Flumen::from_iterator(1..=5)
    ///     .fold(0, |acc, x| acc + x));
    /// assert_eq!(sum, 15);
    /// ```
    #[inline]
    pub async fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: FnMut(B, T) -> B + Send,
        B: Send,
    {
        let mut acc = init;
        while let Some(item) = self.next().await {
            acc = f(acc, item);
        }
        acc
    }

    /// Reduce the stream using a binary operation.
    ///
    /// Returns `None` if the stream is empty.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let sum = block_on(Flumen::from_iterator(1..=5)
    ///     .reduce(|a, b| a + b));
    /// assert_eq!(sum, Some(15));
    /// ```
    #[inline]
    pub async fn reduce<F>(mut self, f: F) -> Option<T>
    where
        F: Fn(T, T) -> T + Send,
    {
        let first = self.next().await?;
        Some(self.fold(first, f).await)
    }

    /// Collect the stream into a Vec.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let items = block_on(Flumen::from_iterator(1..=3).collect_vec());
    /// assert_eq!(items, vec![1, 2, 3]);
    /// ```
    #[inline]
    pub async fn collect_vec(self) -> Vec<T> {
        let (lower, _) = self.inner.size_hint();
        self.fold(Vec::with_capacity(lower), |mut acc, item| {
            acc.push(item);
            acc
        })
        .await
    }

    /// Convert this `Flumen` into a fused `FlumenFusus` representation.
    ///
    /// This is the opt-in entrypoint for stream fusion: once fused, subsequent
    /// combinators can be applied without allocating new boxed stream adapters.
    #[cfg(feature = "fusion")]
    #[inline(always)]
    pub fn fuse(self) -> FlumenFusus<BoxStream<T>, impl GradusStep<BoxStream<T>, T>, T> {
        FlumenFusus::new(
            self.inner,
            |s: &mut BoxStream<T>, cx: &mut Context<'_>| match s.as_mut().poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Some(item)) => Poll::Ready(Gradus::Yield(item)),
                Poll::Ready(None) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Get the first item from the stream.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let first = block_on(Flumen::from_iterator(1..=5).first());
    /// assert_eq!(first, Some(1));
    /// ```
    #[inline]
    pub async fn first(mut self) -> Option<T> {
        self.next().await
    }

    /// Get the last item from the stream.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let last = block_on(Flumen::from_iterator(1..=5).last());
    /// assert_eq!(last, Some(5));
    /// ```
    #[inline]
    pub async fn last(self) -> Option<T> {
        self.fold(None, |_, item| Some(item)).await
    }

    /// Count the number of items in the stream.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let count = block_on(Flumen::from_iterator(1..=5).count());
    /// assert_eq!(count, 5);
    /// ```
    #[inline]
    pub async fn count(self) -> usize {
        self.fold(0, |acc, _| acc + 1).await
    }

    /// Check if any item satisfies a predicate.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let has_even = block_on(Flumen::from_iterator(vec![1, 3, 4, 7])
    ///     .any(|x| x % 2 == 0));
    /// assert!(has_even);
    /// ```
    #[inline]
    pub async fn any<F>(mut self, f: F) -> bool
    where
        F: Fn(&T) -> bool + Send,
    {
        while let Some(item) = self.next().await {
            if f(&item) {
                return true;
            }
        }
        false
    }

    /// Check if all items satisfy a predicate.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let all_positive = block_on(Flumen::from_iterator(vec![1, 2, 3])
    ///     .all(|x| *x > 0));
    /// assert!(all_positive);
    /// ```
    #[inline]
    pub async fn all<F>(mut self, f: F) -> bool
    where
        F: Fn(&T) -> bool + Send,
    {
        while let Some(item) = self.next().await {
            if !f(&item) {
                return false;
            }
        }
        true
    }

    /// Find the first item satisfying a predicate.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let first_even = block_on(Flumen::from_iterator(vec![1, 3, 4, 7])
    ///     .find(|x| x % 2 == 0));
    /// assert_eq!(first_even, Some(4));
    /// ```
    #[inline]
    pub async fn find<F>(mut self, f: F) -> Option<T>
    where
        F: Fn(&T) -> bool + Send,
    {
        while let Some(item) = self.next().await {
            if f(&item) {
                return Some(item);
            }
        }
        None
    }

    /// Get the next item from the stream.
    #[inline]
    async fn next(&mut self) -> Option<T> {
        use core::future::poll_fn;
        poll_fn(|cx| Pin::new(&mut self.inner).poll_next(cx)).await
    }

    /// Run an async function for each item (for side effects).
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// block_on(Flumen::from_iterator(1..=3)
    ///     .for_each(|x| async move { println!("{}", x) }));
    /// ```
    #[inline]
    pub async fn for_each<F, Fut>(mut self, f: F)
    where
        F: Fn(T) -> Fut + Send,
        Fut: Future<Output = ()> + Send,
    {
        while let Some(item) = self.next().await {
            f(item).await;
        }
    }

    /// Flat map with a function returning a Flumen.
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
    /// use ordofp_core::async_core::Flumen;
    ///
    /// let flattened = Flumen::from_iterator(vec![1, 2])
    ///     .flat_map(|x| Flumen::from_iterator(vec![x, x * 10]));
    /// assert_eq!(block_on(flattened.collect_vec()), vec![1, 10, 2, 20]);
    /// ```
    #[inline(always)]
    pub fn flat_map<B, F>(self, f: F) -> Flumen<B>
    where
        F: Fn(T) -> Flumen<B> + Send + Sync + Unpin + 'static,
        B: Send + 'static,
    {
        Flumen::new(FlatMapStream::new(self.inner, f))
    }

    /// Flatten a stream of streams.
    #[inline(always)]
    pub fn flatten(self) -> Flumen<T::Item>
    where
        T: Stream + Send + 'static,
        T::Item: Send + 'static,
    {
        Flumen::new(FlattenStream::new(self.inner))
    }
}

impl<T> Stream for Flumen<T> {
    type Item = T;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl<T> core::fmt::Debug for Flumen<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Flumen")
            .field("inner", &"<stream>")
            .finish()
    }
}

// ============================================================================
// Helper Stream Implementations
// ============================================================================

/// Stream adapter that converts an iterator into a stream.
struct IterStream<I> {
    iter: I,
}

impl<I> IterStream<I> {
    #[inline(always)]
    fn new(iter: I) -> Self {
        IterStream { iter }
    }
}

impl<I: Iterator + Unpin> Stream for IterStream<I> {
    type Item = I::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.iter.next())
    }
}

/// Stream adapter for mapping.
struct MapStream<S, F> {
    stream: S,
    f: F,
}

impl<S, F> MapStream<S, F> {
    #[inline(always)]
    fn new(stream: S, f: F) -> Self {
        MapStream { stream, f }
    }
}

impl<S, F, B> Stream for MapStream<S, F>
where
    S: Stream + Unpin,
    F: Fn(S::Item) -> B + Unpin,
{
    type Item = B;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some((self.f)(item))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream adapter for filtering.
struct FilterStream<S, F> {
    stream: S,
    predicate: F,
}

impl<S, F> FilterStream<S, F> {
    #[inline(always)]
    fn new(stream: S, predicate: F) -> Self {
        FilterStream { stream, predicate }
    }
}

impl<S, F> Stream for FilterStream<S, F>
where
    S: Stream + Unpin,
    F: Fn(&S::Item) -> bool + Unpin,
{
    type Item = S::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    if (self.predicate)(&item) {
                        return Poll::Ready(Some(item));
                    }
                    // Continue to next item
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Stream adapter for `filter_map`.
struct FilterMapStream<S, F> {
    stream: S,
    f: F,
}

impl<S, F> FilterMapStream<S, F> {
    #[inline(always)]
    fn new(stream: S, f: F) -> Self {
        FilterMapStream { stream, f }
    }
}

impl<S, F, B> Stream for FilterMapStream<S, F>
where
    S: Stream + Unpin,
    F: Fn(S::Item) -> Option<B> + Unpin,
{
    type Item = B;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    if let Some(mapped) = (self.f)(item) {
                        return Poll::Ready(Some(mapped));
                    }
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Stream adapter for take.
struct TakeStream<S> {
    stream: S,
    remaining: usize,
}

impl<S> TakeStream<S> {
    #[inline(always)]
    fn new(stream: S, n: usize) -> Self {
        TakeStream {
            stream,
            remaining: n,
        }
    }
}

impl<S: Stream + Unpin> Stream for TakeStream<S> {
    type Item = S::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.remaining == 0 {
            return Poll::Ready(None);
        }
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                self.remaining -= 1;
                Poll::Ready(Some(item))
            }
            other => other,
        }
    }
}

/// Stream adapter for skip.
struct SkipStream<S> {
    stream: S,
    remaining: usize,
}

impl<S> SkipStream<S> {
    #[inline(always)]
    fn new(stream: S, n: usize) -> Self {
        SkipStream {
            stream,
            remaining: n,
        }
    }
}

impl<S: Stream + Unpin> Stream for SkipStream<S> {
    type Item = S::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        while self.remaining > 0 {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(_)) => {
                    self.remaining -= 1;
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
        Pin::new(&mut self.stream).poll_next(cx)
    }
}

/// Stream adapter for `take_while`.
struct TakeWhileStream<S, F> {
    stream: S,
    predicate: F,
    done: bool,
}

impl<S, F> TakeWhileStream<S, F> {
    #[inline(always)]
    fn new(stream: S, predicate: F) -> Self {
        TakeWhileStream {
            stream,
            predicate,
            done: false,
        }
    }
}

impl<S, F> Stream for TakeWhileStream<S, F>
where
    S: Stream + Unpin,
    F: Fn(&S::Item) -> bool + Unpin,
{
    type Item = S::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                if (self.predicate)(&item) {
                    Poll::Ready(Some(item))
                } else {
                    self.done = true;
                    Poll::Ready(None)
                }
            }
            other => other,
        }
    }
}

/// Stream adapter for `skip_while`.
struct SkipWhileStream<S, F> {
    stream: S,
    predicate: Option<F>,
}

impl<S, F> SkipWhileStream<S, F> {
    #[inline(always)]
    fn new(stream: S, predicate: F) -> Self {
        SkipWhileStream {
            stream,
            predicate: Some(predicate),
        }
    }
}

impl<S, F> Stream for SkipWhileStream<S, F>
where
    S: Stream + Unpin,
    F: Fn(&S::Item) -> bool + Unpin,
{
    type Item = S::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    if let Some(ref predicate) = self.predicate {
                        if predicate(&item) {
                            continue;
                        }
                        self.predicate = None;
                    }
                    return Poll::Ready(Some(item));
                }
                other => return other,
            }
        }
    }
}

/// Stream adapter for chain.
struct ChainStream<S1, S2> {
    first: Option<S1>,
    second: S2,
}

impl<S1, S2> ChainStream<S1, S2> {
    #[inline(always)]
    fn new(first: S1, second: S2) -> Self {
        ChainStream {
            first: Some(first),
            second,
        }
    }
}

impl<S1, S2> Stream for ChainStream<S1, S2>
where
    S1: Stream + Unpin,
    S2: Stream<Item = S1::Item> + Unpin,
{
    type Item = S1::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(ref mut first) = self.first {
            match Pin::new(first).poll_next(cx) {
                Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                Poll::Ready(None) => {
                    self.first = None;
                }
                Poll::Pending => return Poll::Pending,
            }
        }
        Pin::new(&mut self.second).poll_next(cx)
    }
}

/// Stream adapter for zip.
struct ZipStream<S1: Stream, S2> {
    stream1: S1,
    stream2: S2,
    /// Item from `stream1` awaiting its pair: when `stream1` is ready before
    /// `stream2`, the item is parked here instead of being dropped.
    stash: Option<S1::Item>,
}

impl<S1: Stream, S2> ZipStream<S1, S2> {
    #[inline(always)]
    fn new(stream1: S1, stream2: S2) -> Self {
        ZipStream {
            stream1,
            stream2,
            stash: None,
        }
    }
}

// `stash` is never pinned, so the adapter is `Unpin` whenever its halves are
// (regardless of whether `S1::Item` is `Unpin`).
impl<S1: Stream + Unpin, S2: Unpin> Unpin for ZipStream<S1, S2> {}

impl<S1, S2> Stream for ZipStream<S1, S2>
where
    S1: Stream + Unpin,
    S2: Stream + Unpin,
{
    type Item = (S1::Item, S2::Item);

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item1 = match self.stash.take() {
            Some(item) => item,
            None => match Pin::new(&mut self.stream1).poll_next(cx) {
                Poll::Ready(Some(item)) => item,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            },
        };
        let item2 = match Pin::new(&mut self.stream2).poll_next(cx) {
            Poll::Ready(Some(item)) => item,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => {
                self.stash = Some(item1);
                return Poll::Pending;
            }
        };
        Poll::Ready(Some((item1, item2)))
    }
}

/// Stream adapter for enumerate.
struct EnumerateStream<S> {
    stream: S,
    index: usize,
}

impl<S> EnumerateStream<S> {
    #[inline(always)]
    fn new(stream: S) -> Self {
        EnumerateStream { stream, index: 0 }
    }
}

impl<S: Stream + Unpin> Stream for EnumerateStream<S> {
    type Item = (usize, S::Item);

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                let idx = self.index;
                self.index += 1;
                Poll::Ready(Some((idx, item)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream adapter for `flat_map`.
struct FlatMapStream<S, F, B>
where
    S: Stream,
    F: Fn(S::Item) -> Flumen<B>,
{
    stream: S,
    f: F,
    current: Option<BoxStream<B>>,
}

impl<S, F, B> FlatMapStream<S, F, B>
where
    S: Stream,
    F: Fn(S::Item) -> Flumen<B>,
{
    #[inline(always)]
    fn new(stream: S, f: F) -> Self {
        FlatMapStream {
            stream,
            f,
            current: None,
        }
    }
}

impl<S, F, B> Stream for FlatMapStream<S, F, B>
where
    S: Stream + Unpin,
    F: Fn(S::Item) -> Flumen<B> + Unpin,
    B: Send + 'static,
{
    type Item = B;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Try to get item from current inner stream
            if let Some(ref mut inner) = self.current {
                match inner.as_mut().poll_next(cx) {
                    Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                    Poll::Ready(None) => {
                        self.current = None;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            // Get next outer item
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    let flumen = (self.f)(item);
                    self.current = Some(flumen.inner);
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Stream adapter for scan - yields intermediate accumulator values.
///
/// `acc` is `Option<B>` so we can move the previous value into the fold
/// function without cloning it; the slot is always `Some(_)` between
/// `poll_next` calls, and is only `None` transiently while `f` is running.
struct ScanStream<S, B, F> {
    stream: S,
    acc: Option<B>,
    f: F,
}

impl<S, B, F> ScanStream<S, B, F> {
    #[inline]
    fn new(stream: S, init: B, f: F) -> Self {
        ScanStream {
            stream,
            acc: Some(init),
            f,
        }
    }
}

impl<S, B, F> Stream for ScanStream<S, B, F>
where
    S: Stream + Unpin,
    B: Clone + Unpin,
    F: Fn(B, S::Item) -> B + Unpin,
{
    type Item = B;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                let prev = self
                    .acc
                    .take()
                    .expect("ScanStream::acc is Some between polls");
                let new_acc = (self.f)(prev, item);
                self.acc = Some(new_acc.clone());
                Poll::Ready(Some(new_acc))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream adapter for `scan_with` - allows separate state and output types.
struct ScanWithStream<S, St, F> {
    stream: S,
    state: St,
    f: F,
    done: bool,
}

impl<S, St, F> ScanWithStream<S, St, F> {
    #[inline(always)]
    fn new(stream: S, init: St, f: F) -> Self {
        ScanWithStream {
            stream,
            state: init,
            f,
            done: false,
        }
    }
}

impl<S, St, B, F> Stream for ScanWithStream<S, St, F>
where
    S: Stream + Unpin,
    St: Unpin,
    F: FnMut(&mut St, S::Item) -> Option<B> + Unpin,
{
    type Item = B;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                let this = self.get_mut();
                if let Some(output) = (this.f)(&mut this.state, item) {
                    Poll::Ready(Some(output))
                } else {
                    this.done = true;
                    Poll::Ready(None)
                }
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream adapter for chunks - collects items into fixed-size windows.
struct ChunksStream<S: Stream> {
    stream: S,
    size: usize,
    buffer: Vec<S::Item>,
    done: bool,
}

impl<S: Stream> ChunksStream<S> {
    #[inline(always)]
    fn new(stream: S, size: usize) -> Self {
        ChunksStream {
            stream,
            size,
            buffer: Vec::with_capacity(size),
            done: false,
        }
    }
}

impl<S> Stream for ChunksStream<S>
where
    S: Stream + Unpin,
    S::Item: Unpin,
{
    type Item = Vec<S::Item>;

    #[inline(always)]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.done {
            return Poll::Ready(None);
        }

        loop {
            match Pin::new(&mut this.stream).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    this.buffer.push(item);
                    if this.buffer.len() >= this.size {
                        let chunk =
                            core::mem::replace(&mut this.buffer, Vec::with_capacity(this.size));
                        return Poll::Ready(Some(chunk));
                    }
                }
                Poll::Ready(None) => {
                    this.done = true;
                    if this.buffer.is_empty() {
                        return Poll::Ready(None);
                    }
                    let final_chunk = core::mem::take(&mut this.buffer);
                    return Poll::Ready(Some(final_chunk));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Stream adapter for flatten.
struct FlattenStream<S>
where
    S: Stream,
    S::Item: Stream,
{
    stream: S,
    current: Option<Pin<Box<S::Item>>>,
}

impl<S> FlattenStream<S>
where
    S: Stream,
    S::Item: Stream,
{
    #[inline(always)]
    fn new(stream: S) -> Self {
        FlattenStream {
            stream,
            current: None,
        }
    }
}

impl<S> Stream for FlattenStream<S>
where
    S: Stream + Unpin,
    S::Item: Stream + Send + 'static,
{
    type Item = <S::Item as Stream>::Item;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(ref mut inner) = self.current {
                match inner.as_mut().poll_next(cx) {
                    Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                    Poll::Ready(None) => {
                        self.current = None;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(inner_stream)) => {
                    self.current = Some(Box::pin(inner_stream));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_flumen_creation() {
        let _stream: Flumen<i32> = Flumen::from_iterator(vec![1, 2, 3]);
    }

    #[test]
    fn test_flumen_once() {
        let _stream: Flumen<i32> = Flumen::once(42);
    }

    #[test]
    fn test_flumen_empty() {
        let _stream: Flumen<i32> = Flumen::empty();
    }

    #[test]
    fn test_flumen_debug() {
        let stream: Flumen<i32> = Flumen::from_iterator(vec![1, 2, 3]);
        let debug = alloc::format!("{stream:?}");
        assert!(debug.contains("Flumen"));
    }

    /// Wraps a stream so it returns `Pending` once before each item.
    struct PendingOnce<S> {
        inner: S,
        pended: bool,
    }

    impl<S: Stream + Unpin> Stream for PendingOnce<S> {
        type Item = S::Item;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if !self.pended {
                self.pended = true;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            self.pended = false;
            Pin::new(&mut self.inner).poll_next(cx)
        }
    }

    /// Busy-poll a future to completion (the test streams wake the noop waker).
    fn drive<F: Future>(fut: F) -> F::Output {
        let mut fut = core::pin::pin!(fut);
        let mut cx = Context::from_waker(core::task::Waker::noop());
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    #[test]
    fn test_zip_keeps_items_when_second_stream_pending() {
        let fast = Flumen::from_iterator(vec![1, 2, 3]);
        let slow = Flumen::new(PendingOnce {
            inner: IterStream::new(vec![10, 20, 30].into_iter()),
            pended: false,
        });
        let pairs = drive(fast.zip(slow).collect_vec());
        assert_eq!(pairs, vec![(1, 10), (2, 20), (3, 30)]);
    }
}
