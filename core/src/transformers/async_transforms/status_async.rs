//! `StatusAsync` - Async State Monad Transformer
//!
//! > *"Status mutatur, essentia manet"*
//! > — The state changes, the essence remains. (Scholastic philosophy)
//!
//! `StatusAsync` is the async variant of `StateT`, providing stateful computation
//! capabilities in asynchronous contexts.
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
//! use ordofp_core::transformers::async_transforms::StatusAsync;
//!
//! // Counter state
//! let increment = StatusAsync::<i32, ()>::modify(|s| s + 1);
//! let get_count = StatusAsync::<i32, i32>::get();
//!
//! // Chain operations
//! let program = increment.clone()
//!     .then(increment.clone())
//!     .then(increment)
//!     .then(get_count);
//!
//! let (final_state, count) = block_on(program.run(0));
//! assert_eq!(final_state, 3);
//! assert_eq!(count, 3);
//! ```

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::marker::PhantomData;

use super::{AsyncFn, BoxFuture, FutureSlot, MonadTransformerAsync, SlotCustos, cede_semel};

/// The state-transition function of a [`StatusAsync`]: an async `S -> (S, A)`
/// shared behind an `Arc` for cloneability.
type StatusFn<S, A> = AsyncFn<S, (S, A)>;

/// Async State Monad Transformer.
///
/// `StatusAsync<S, A>` represents an async computation that maintains state
/// of type `S` and produces a value of type `A`.
///
/// # Type Parameters
///
/// - `S`: The state type
/// - `A`: The result type produced by the computation
///
/// # Scholastic Etymology
///
/// *Status* (Latin: state, condition) derives from *stare* (to stand).
/// In scholastic philosophy, *status* refers to the condition or state
/// of being of a thing at a given moment.
pub struct StatusAsync<S, A> {
    /// The async state function, wrapped in Arc for cloneability.
    /// Takes state S and returns (`new_state`, result).
    run_fn: StatusFn<S, A>,
    _phantom: PhantomData<fn(S) -> (S, A)>,
}

impl<S, A> Clone for StatusAsync<S, A> {
    fn clone(&self) -> Self {
        StatusAsync {
            run_fn: Arc::clone(&self.run_fn),
            _phantom: PhantomData,
        }
    }
}

impl<S, A> StatusAsync<S, A>
where
    S: Send + 'static,
    A: Send + 'static,
{
    /// Create a new `StatusAsync` from an async state function.
    ///
    /// The function takes the current state and returns a future
    /// that produces a tuple of (`new_state`, result).
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let counter = StatusAsync::new(|s: i32| async move { (s + 1, s) });
    /// let (new_state, old_state) = block_on(counter.run(0));
    /// assert_eq!(new_state, 1);
    /// assert_eq!(old_state, 0);
    /// ```
    #[inline]
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(S) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = (S, A)> + Send + 'static,
    {
        StatusAsync {
            run_fn: Arc::new(move |s| Box::pin(f(s))),
            _phantom: PhantomData,
        }
    }

    /// Run the stateful computation with an initial state.
    ///
    /// Returns the final state and the result.
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let counter = StatusAsync::new(|s: i32| async move { (s + 1, s) });
    /// let (new_state, old_state) = block_on(counter.run(0));
    /// assert_eq!(new_state, 1);
    /// assert_eq!(old_state, 0);
    /// ```
    #[inline]
    pub async fn run(self, initial: S) -> (S, A) {
        (self.run_fn)(initial).await
    }

    /// Run and return only the final state (discard the result).
    #[inline]
    pub async fn exec(self, initial: S) -> S {
        let (s, _) = self.run(initial).await;
        s
    }

    /// Run and return only the result (discard the final state).
    #[inline]
    pub async fn eval(self, initial: S) -> A {
        let (_, a) = self.run(initial).await;
        a
    }

    /// Create a pure computation that doesn't modify the state.
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let pure_42: StatusAsync<String, i32> = StatusAsync::purus(42);
    /// let (state, result) = block_on(pure_42.run("unchanged".to_string()));
    /// assert_eq!(state, "unchanged");
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn purus(value: A) -> Self
    where
        A: Clone + Send + Sync + 'static,
    {
        StatusAsync::new(move |s: S| {
            let v = value.clone();
            async move { (s, v) }
        })
    }

    /// Get the current state.
    ///
    /// Returns a computation that produces the current state as its result.
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let get_state = StatusAsync::<i32, i32>::get();
    /// let (state, result) = block_on(get_state.run(42));
    /// assert_eq!(state, 42);
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn get() -> StatusAsync<S, S>
    where
        S: Clone,
    {
        StatusAsync::new(|s: S| async move { (s.clone(), s) })
    }

    /// Set the state to a new value.
    ///
    /// Returns a computation that sets the state and produces `()`.
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let set_state = StatusAsync::<i32, ()>::put(100);
    /// let (state, _) = block_on(set_state.run(0));
    /// assert_eq!(state, 100);
    /// ```
    #[inline]
    pub fn put(new_state: S) -> StatusAsync<S, ()>
    where
        S: Clone + Send + Sync + 'static,
    {
        StatusAsync::new(move |_: S| {
            let s = new_state.clone();
            async move { (s, ()) }
        })
    }

    /// Modify the state using a function.
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let increment = StatusAsync::<i32, ()>::modify(|s| s + 1);
    /// let (state, _) = block_on(increment.run(0));
    /// assert_eq!(state, 1);
    /// ```
    #[inline]
    pub fn modify<F>(f: F) -> StatusAsync<S, ()>
    where
        F: Fn(S) -> S + Send + Sync + Clone + 'static,
    {
        StatusAsync::new(move |s: S| {
            let f = f.clone();
            async move { (f(s), ()) }
        })
    }

    /// Get a value derived from the state.
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// #[derive(Clone)]
    /// struct AppState { count: i32, name: String }
    ///
    /// let get_count = StatusAsync::<AppState, i32>::gets(|s: &AppState| s.count);
    /// let state = AppState { count: 42, name: "test".to_string() };
    /// let (_, count) = block_on(get_count.run(state));
    /// assert_eq!(count, 42);
    /// ```
    #[inline]
    pub fn gets<B, F>(f: F) -> StatusAsync<S, B>
    where
        F: Fn(&S) -> B + Send + Sync + Clone + 'static,
        B: Send + 'static,
        S: Clone,
    {
        StatusAsync::new(move |s: S| {
            let f = f.clone();
            let result = f(&s);
            async move { (s, result) }
        })
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let counter = StatusAsync::<i32, i32>::get().fmap(|x| x * 2);
    /// let (_, result) = block_on(counter.run(21));
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> StatusAsync<S, B>
    where
        F: Fn(A) -> B + Send + Sync + Clone + 'static,
        B: Send + 'static,
    {
        let run_fn = self.run_fn;
        StatusAsync {
            run_fn: Arc::new(move |s: S| {
                let fut = run_fn(s);
                let f = f.clone();
                Box::pin(async move {
                    let (new_s, a) = fut.await;
                    (new_s, f(a))
                }) as BoxFuture<(S, B)>
            }),
            _phantom: PhantomData,
        }
    }

    /// Chain this computation with another state computation.
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
    /// use ordofp_core::transformers::async_transforms::StatusAsync;
    ///
    /// let increment = StatusAsync::<i32, ()>::modify(|s: i32| s + 1);
    /// let get_doubled = StatusAsync::<i32, i32>::get().fmap(|x| x * 2);
    ///
    /// let program = increment.flat_map(move |_| get_doubled.clone());
    /// let (state, result) = block_on(program.run(0));
    /// assert_eq!(state, 1);
    /// assert_eq!(result, 2);
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> StatusAsync<S, B>
    where
        F: Fn(A) -> StatusAsync<S, B> + Send + Sync + Clone + 'static,
        B: Send + 'static,
    {
        let run_fn = self.run_fn;
        StatusAsync {
            run_fn: Arc::new(move |s: S| {
                let fut = run_fn(s);
                let f = f.clone();
                Box::pin(async move {
                    let (new_s, a) = fut.await;
                    let next = f(a);
                    next.run(new_s).await
                }) as BoxFuture<(S, B)>
            }),
            _phantom: PhantomData,
        }
    }

    /// Alias for `flat_map` using traditional Haskell naming.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> StatusAsync<S, B>
    where
        F: Fn(A) -> StatusAsync<S, B> + Send + Sync + Clone + 'static,
        B: Send + 'static,
    {
        self.flat_map(f)
    }

    /// Alias for `flat_map` using Scala naming.
    #[inline]
    pub fn and_then<B, F>(self, f: F) -> StatusAsync<S, B>
    where
        F: Fn(A) -> StatusAsync<S, B> + Send + Sync + Clone + 'static,
        B: Send + 'static,
    {
        self.flat_map(f)
    }

    /// Combine two state computations.
    #[inline]
    pub fn map2<B, C, F>(self, other: StatusAsync<S, B>, f: F) -> StatusAsync<S, C>
    where
        F: Fn(A, B) -> C + Send + Sync + Clone + 'static,
        A: Clone + Sync,
        B: Send + 'static,
        C: Send + 'static,
    {
        self.flat_map(move |a| {
            let f = f.clone();
            other.clone().fmap(move |b| f(a.clone(), b))
        })
    }

    /// Sequence this computation before another, discarding the first result.
    #[inline]
    pub fn then<B>(self, next: StatusAsync<S, B>) -> StatusAsync<S, B>
    where
        B: Send + 'static,
    {
        self.flat_map(move |_| next.clone())
    }

    /// Sequence this computation before another, keeping only the first result.
    #[inline]
    pub fn skip<B>(self, next: StatusAsync<S, B>) -> StatusAsync<S, A>
    where
        B: Send + 'static,
        A: Clone + Send + Sync + 'static,
    {
        self.flat_map(move |a| {
            let a_clone = a.clone();
            next.clone().fmap(move |_| a_clone.clone())
        })
    }

    /// Transform the state type using isomorphism functions.
    ///
    /// Useful for adapting state computations to different state types.
    #[inline]
    pub fn zoom<S2, F, G>(self, get: F, set: G) -> StatusAsync<S2, A>
    where
        F: Fn(&S2) -> S + Send + Sync + Clone + 'static,
        G: Fn(S2, S) -> S2 + Send + Sync + Clone + 'static,
        S2: Send + 'static,
        S: Clone,
    {
        let run_fn = self.run_fn;
        StatusAsync {
            run_fn: Arc::new(move |s2: S2| {
                let s = get(&s2);
                let fut = run_fn(s);
                let set = set.clone();
                Box::pin(async move {
                    let (new_s, a) = fut.await;
                    (set(s2, new_s), a)
                }) as BoxFuture<(S2, A)>
            }),
            _phantom: PhantomData,
        }
    }
}

impl<S, A> MonadTransformerAsync for StatusAsync<S, A>
where
    S: Send + 'static,
    A: Send + Clone + Sync + 'static,
{
    type Output = A;

    #[inline]
    fn lift_async<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = A> + Send + 'static,
    {
        // Use OnceLock to cache the future result
        use std::sync::OnceLock;

        let shared_fut: FutureSlot<A> = Arc::new(std::sync::Mutex::new(Some(Box::pin(fut))));
        let result_cache: Arc<OnceLock<A>> = Arc::new(OnceLock::new());

        StatusAsync {
            run_fn: Arc::new(move |s: S| {
                let shared_fut = Arc::clone(&shared_fut);
                let result_cache = Arc::clone(&result_cache);
                Box::pin(async move {
                    loop {
                        if let Some(cached) = result_cache.get() {
                            return (s, cached.clone());
                        }

                        let maybe_fut = {
                            let mut guard = shared_fut.lock().unwrap();
                            guard.take()
                        };

                        if let Some(f) = maybe_fut {
                            // Guard: if this run is dropped mid-await, the
                            // future is restored to the slot for a later run.
                            let mut custos = SlotCustos::new(f, Arc::clone(&shared_fut));
                            let result = custos.fut_mut().await;
                            custos.disarm();
                            let _ = result_cache.set(result.clone());
                            return (s, result);
                        }

                        // Another runner holds the future; yield, then re-check
                        // the cache (or the slot, if that runner was cancelled).
                        cede_semel().await;
                    }
                }) as BoxFuture<(S, A)>
            }),
            _phantom: PhantomData,
        }
    }
}

impl<S: core::fmt::Debug, A> core::fmt::Debug for StatusAsync<S, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StatusAsync")
            .field("run_fn", &"<async fn>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_status_async_creation() {
        let _state: StatusAsync<i32, i32> = StatusAsync::new(|s| async move { (s + 1, s) });
    }

    #[test]
    fn test_status_async_purus() {
        let _state: StatusAsync<String, i32> = StatusAsync::purus(42);
    }

    #[test]
    fn test_status_async_get() {
        let _state: StatusAsync<i32, i32> = StatusAsync::<i32, i32>::get();
    }

    #[test]
    fn test_status_async_put() {
        let _state: StatusAsync<i32, ()> = StatusAsync::<i32, ()>::put(42);
    }

    #[test]
    fn test_status_async_modify() {
        let _state: StatusAsync<i32, ()> = StatusAsync::<i32, ()>::modify(|s| s + 1);
    }

    #[test]
    fn test_status_async_clone() {
        let state: StatusAsync<i32, i32> = StatusAsync::<i32, i32>::get();
        let _cloned = state.clone();
    }

    #[test]
    fn test_status_async_debug() {
        let state: StatusAsync<i32, i32> = StatusAsync::<i32, i32>::get();
        let debug = alloc::format!("{state:?}");
        assert!(debug.contains("StatusAsync"));
    }

    use core::pin::Pin;
    use core::task::{Context, Poll, Waker};

    /// A future that is `Pending` once (waking the waker) before yielding its value.
    struct PendetSemel {
        polled: bool,
        value: i32,
    }

    impl Future for PendetSemel {
        type Output = i32;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
            if self.polled {
                Poll::Ready(self.value)
            } else {
                self.polled = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    /// Busy-poll a future to completion with a noop waker.
    fn drive<F: Future>(fut: F) -> F::Output {
        let mut fut = Box::pin(fut);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    #[test]
    fn test_lift_async_survives_cancelled_first_run() {
        let lifted: StatusAsync<i32, i32> =
            <StatusAsync<i32, i32> as MonadTransformerAsync>::lift_async(PendetSemel {
                polled: false,
                value: 42,
            });

        // Poll the first run once and drop it mid-await (cancellation).
        {
            let mut first = Box::pin(lifted.clone().run(0));
            let mut cx = Context::from_waker(Waker::noop());
            assert!(first.as_mut().poll(&mut cx).is_pending());
        }

        // A later run must still complete instead of panicking.
        assert_eq!(drive(lifted.run(5)), (5, 42));
    }

    #[test]
    fn test_lift_async_concurrent_runner_waits_for_cache() {
        let lifted: StatusAsync<i32, i32> =
            <StatusAsync<i32, i32> as MonadTransformerAsync>::lift_async(PendetSemel {
                polled: false,
                value: 7,
            });
        let mut first = Box::pin(lifted.clone().run(1));
        let mut second = Box::pin(lifted.run(2));
        let mut cx = Context::from_waker(Waker::noop());

        // First runner takes the future from the slot; it is pending once.
        assert!(first.as_mut().poll(&mut cx).is_pending());
        // Second runner finds slot and cache empty: it must wait, not panic.
        assert!(second.as_mut().poll(&mut cx).is_pending());

        assert_eq!(first.as_mut().poll(&mut cx), Poll::Ready((1, 7)));
        assert_eq!(second.as_mut().poll(&mut cx), Poll::Ready((2, 7)));
    }
}
