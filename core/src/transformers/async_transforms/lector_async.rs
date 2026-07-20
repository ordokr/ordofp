//! `LectorAsync` - Async Reader Monad Transformer
//!
//! > *"Lector qui legit, intelligat"*
//! > — Let the reader who reads, understand. (Medieval manuscript tradition)
//!
//! `LectorAsync` is the async variant of `ReaderT`, providing environment/configuration
//! reading capabilities in asynchronous contexts.
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
//! use ordofp_core::transformers::async_transforms::LectorAsync;
//!
//! struct Config {
//!     api_key: String,
//!     base_url: String,
//! }
//!
//! // Create a reader that fetches data using config
//! let fetch = LectorAsync::new(|cfg: Config| async move {
//!     format!("Fetching from {} with key {}", cfg.base_url, cfg.api_key)
//! });
//!
//! // Chain readers
//! let processed = fetch.fmap(|s: String| s.len());
//!
//! // Run with config
//! let config = Config {
//!     api_key: "secret".to_string(),
//!     base_url: "https://api.example.com".to_string(),
//! };
//! let result = block_on(processed.run(config));
//! assert_eq!(
//!     result,
//!     "Fetching from https://api.example.com with key secret".len()
//! );
//! ```

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::marker::PhantomData;

use super::{AsyncFn, BoxFuture, FutureSlot, MonadTransformerAsync, SlotCustos, cede_semel};

/// Async Reader Monad Transformer.
///
/// `LectorAsync<E, A>` represents an async computation that can read from an
/// environment of type `E` and produces a value of type `A`.
///
/// # Type Parameters
///
/// - `E`: The environment type (configuration, dependencies, etc.)
/// - `A`: The result type produced by the computation
///
/// # Scholastic Etymology
///
/// *Lector* (Latin: reader) derives from *legere* (to read, to gather).
/// In scholastic tradition, the *lector* was responsible for reading and
/// interpreting texts - similarly, this transformer "reads" from its environment.
pub struct LectorAsync<E, A> {
    /// The async reader function, wrapped in Arc for cloneability.
    run_fn: AsyncFn<E, A>,
    /// Phantom data for the environment type.
    _phantom: PhantomData<fn(E) -> A>,
}

impl<E, A> Clone for LectorAsync<E, A> {
    fn clone(&self) -> Self {
        LectorAsync {
            run_fn: Arc::clone(&self.run_fn),
            _phantom: PhantomData,
        }
    }
}

impl<E, A> LectorAsync<E, A>
where
    E: Send + 'static,
    A: Send + 'static,
{
    /// Create a new `LectorAsync` from an async function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// let reader: LectorAsync<i32, i32> = LectorAsync::new(|env: i32| async move { env * 2 });
    /// ```
    #[inline]
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = A> + Send + 'static,
    {
        LectorAsync {
            run_fn: Arc::new(move |e| Box::pin(f(e))),
            _phantom: PhantomData,
        }
    }

    /// Run the reader with the given environment.
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// let reader = LectorAsync::new(|x: i32| async move { x + 1 });
    /// let result = block_on(reader.run(41));
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub async fn run(self, env: E) -> A {
        (self.run_fn)(env).await
    }

    /// Run the reader with a reference to the environment.
    ///
    /// This is useful when you don't want to consume the environment.
    #[inline]
    pub fn run_ref(&self, env: E) -> BoxFuture<A> {
        (self.run_fn)(env)
    }

    /// Create a pure reader that ignores the environment.
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// let reader: LectorAsync<String, i32> = LectorAsync::purus(42);
    /// let result = block_on(reader.run("ignored".to_string()));
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn purus(value: A) -> Self
    where
        A: Clone + Send + Sync + 'static,
    {
        LectorAsync::new(move |_: E| {
            let v = value.clone();
            async move { v }
        })
    }

    /// Access the environment directly.
    ///
    /// Returns a reader that simply returns the environment as its result.
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Config {
    ///     name: String,
    /// }
    ///
    /// let reader = LectorAsync::<Config, Config>::ask();
    /// let config = Config { name: "prod".to_string() };
    /// let result = block_on(reader.run(config.clone()));
    /// assert_eq!(result, config);
    /// ```
    #[inline]
    pub fn ask() -> LectorAsync<E, E>
    where
        E: Clone,
    {
        LectorAsync::new(|e: E| async move { e })
    }

    /// Access a part of the environment using a selector function.
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// #[derive(Clone)]
    /// struct Config {
    ///     port: u16,
    ///     host: String,
    /// }
    ///
    /// let reader = LectorAsync::<Config, u16>::asks(|c: &Config| c.port);
    /// let config = Config { port: 8080, host: "localhost".to_string() };
    /// let result = block_on(reader.run(config));
    /// assert_eq!(result, 8080);
    /// ```
    #[inline]
    pub fn asks<F, B>(f: F) -> LectorAsync<E, B>
    where
        F: Fn(&E) -> B + Send + Sync + 'static,
        B: Send + 'static,
        E: Clone,
    {
        LectorAsync::new(move |e: E| {
            let result = f(&e);
            async move { result }
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// let reader = LectorAsync::new(|x: i32| async move { x });
    /// let doubled = reader.fmap(|x| x * 2);
    /// let result = block_on(doubled.run(21));
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> LectorAsync<E, B>
    where
        F: Fn(A) -> B + Send + Sync + Clone + 'static,
        B: Send + 'static,
    {
        let run_fn = self.run_fn;
        LectorAsync {
            run_fn: Arc::new(move |e: E| {
                let fut = run_fn(e);
                let f = f.clone();
                Box::pin(async move { f(fut.await) }) as BoxFuture<B>
            }),
            _phantom: PhantomData,
        }
    }

    /// Chain this reader with another async computation.
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// let reader = LectorAsync::new(|x: i32| async move { x });
    /// let chained = reader.flat_map(|x| {
    ///     LectorAsync::new(move |_: i32| async move { x * 2 })
    /// });
    /// let result = block_on(chained.run(21));
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> LectorAsync<E, B>
    where
        F: Fn(A) -> LectorAsync<E, B> + Send + Sync + Clone + 'static,
        B: Send + 'static,
        E: Clone,
    {
        let run_fn = self.run_fn;
        LectorAsync {
            run_fn: Arc::new(move |e: E| {
                let e_clone = e.clone();
                let fut = run_fn(e);
                let f = f.clone();
                Box::pin(async move {
                    let a = fut.await;
                    let next = f(a);
                    next.run(e_clone).await
                }) as BoxFuture<B>
            }),
            _phantom: PhantomData,
        }
    }

    /// Alias for `flat_map` using traditional Haskell naming.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> LectorAsync<E, B>
    where
        F: Fn(A) -> LectorAsync<E, B> + Send + Sync + Clone + 'static,
        B: Send + 'static,
        E: Clone,
    {
        self.flat_map(f)
    }

    /// Alias for `flat_map` using Scala naming.
    #[inline]
    pub fn and_then<B, F>(self, f: F) -> LectorAsync<E, B>
    where
        F: Fn(A) -> LectorAsync<E, B> + Send + Sync + Clone + 'static,
        B: Send + 'static,
        E: Clone,
    {
        self.flat_map(f)
    }

    /// Transform the environment before running.
    ///
    /// Useful for adapting readers to different environment types.
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// #[derive(Clone)]
    /// struct DbConfig {
    ///     host: String,
    /// }
    ///
    /// struct ApiConfig {
    ///     key: String,
    /// }
    ///
    /// struct GlobalConfig {
    ///     db: DbConfig,
    ///     api: ApiConfig,
    /// }
    ///
    /// let db_reader: LectorAsync<DbConfig, String> = LectorAsync::<DbConfig, DbConfig>::ask()
    ///     .fmap(|c: DbConfig| c.host);
    ///
    /// // Adapt to GlobalConfig
    /// let global_reader = db_reader.local(|g: GlobalConfig| g.db);
    ///
    /// let config = GlobalConfig {
    ///     db: DbConfig { host: "localhost".to_string() },
    ///     api: ApiConfig { key: "secret".to_string() },
    /// };
    /// let result = block_on(global_reader.run(config));
    /// assert_eq!(result, "localhost");
    /// ```
    #[inline]
    pub fn local<E2, F>(self, f: F) -> LectorAsync<E2, A>
    where
        F: Fn(E2) -> E + Send + Sync + 'static,
        E2: Send + 'static,
    {
        let run_fn = self.run_fn;
        LectorAsync {
            run_fn: Arc::new(move |e2: E2| {
                let e = f(e2);
                run_fn(e)
            }),
            _phantom: PhantomData,
        }
    }

    /// Combine two readers, running both with the same environment.
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
    /// use ordofp_core::transformers::async_transforms::LectorAsync;
    ///
    /// let r1 = LectorAsync::new(|x: i32| async move { x });
    /// let r2 = LectorAsync::new(|x: i32| async move { x * 2 });
    /// let combined = r1.map2(r2, |a, b| a + b);
    /// let result = block_on(combined.run(10));
    /// assert_eq!(result, 30);
    /// ```
    #[inline]
    pub fn map2<B, C, F>(self, other: LectorAsync<E, B>, f: F) -> LectorAsync<E, C>
    where
        F: Fn(A, B) -> C + Send + Sync + Clone + 'static,
        B: Send + 'static,
        C: Send + 'static,
        E: Clone,
    {
        let run_fn1 = self.run_fn;
        let run_fn2 = other.run_fn;
        LectorAsync {
            run_fn: Arc::new(move |e: E| {
                let e2 = e.clone();
                let fut1 = run_fn1(e);
                let fut2 = run_fn2(e2);
                let f = f.clone();
                Box::pin(async move {
                    let a = fut1.await;
                    let b = fut2.await;
                    f(a, b)
                }) as BoxFuture<C>
            }),
            _phantom: PhantomData,
        }
    }

    /// Sequence this reader before another, discarding the first result.
    #[inline]
    pub fn then<B>(self, next: LectorAsync<E, B>) -> LectorAsync<E, B>
    where
        B: Send + 'static,
        E: Clone,
    {
        self.flat_map(move |_| next.clone())
    }

    /// Sequence this reader before another, keeping only the first result.
    #[inline]
    pub fn skip<B>(self, next: LectorAsync<E, B>) -> LectorAsync<E, A>
    where
        B: Send + 'static,
        E: Clone,
        A: Clone + Send + Sync + 'static,
    {
        self.flat_map(move |a| {
            let a_clone = a.clone();
            next.clone().fmap(move |_| a_clone.clone())
        })
    }
}

impl<E, A> MonadTransformerAsync for LectorAsync<E, A>
where
    E: Send + 'static,
    A: Send + Clone + Sync + 'static,
{
    type Output = A;

    fn lift_async<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = A> + Send + 'static,
    {
        // Use std::sync::OnceLock for caching the future result
        // Note: This approach runs the future on first access and caches the result
        use std::sync::OnceLock;

        // We need to run the future once and store the result
        // Since we can't easily await in a sync context, we provide
        // an alternative: wrap the future and await on first run
        let shared_fut: FutureSlot<A> = Arc::new(std::sync::Mutex::new(Some(Box::pin(fut))));
        let result_cache: Arc<OnceLock<A>> = Arc::new(OnceLock::new());

        LectorAsync {
            run_fn: Arc::new(move |_: E| {
                let shared_fut = Arc::clone(&shared_fut);
                let result_cache = Arc::clone(&result_cache);
                Box::pin(async move {
                    loop {
                        // Check if we already have a cached result
                        if let Some(cached) = result_cache.get() {
                            return cached.clone();
                        }

                        // Try to take and run the future
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
                            // Cache the result (ignore if another thread beat us)
                            let _ = result_cache.set(result.clone());
                            return result;
                        }

                        // Another runner holds the future; yield, then re-check
                        // the cache (or the slot, if that runner was cancelled).
                        cede_semel().await;
                    }
                }) as BoxFuture<A>
            }),
            _phantom: PhantomData,
        }
    }
}

// Provide a better lift implementation
impl<E, A> LectorAsync<E, A>
where
    E: Send + 'static,
    A: Send + Clone + Sync + 'static,
{
    /// Lift a future into the reader context.
    ///
    /// The future will be awaited and its result returned regardless of environment.
    /// The result is cached so subsequent runs return the same value.
    ///
    /// # Panics
    ///
    /// The returned reader's future panics if the internal future-slot mutex
    /// is poisoned, i.e. a previous runner panicked while holding it (which
    /// requires the lifted future itself to have panicked mid-poll).
    pub fn lift<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = A> + Send + 'static,
    {
        // Use OnceLock to cache the result
        use std::sync::OnceLock;

        let shared_fut: FutureSlot<A> = Arc::new(std::sync::Mutex::new(Some(Box::pin(fut))));
        let result_cache: Arc<OnceLock<A>> = Arc::new(OnceLock::new());

        LectorAsync {
            run_fn: Arc::new(move |_: E| {
                let shared_fut = Arc::clone(&shared_fut);
                let result_cache = Arc::clone(&result_cache);
                Box::pin(async move {
                    loop {
                        if let Some(cached) = result_cache.get() {
                            return cached.clone();
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
                            return result;
                        }

                        // Another runner holds the future; yield, then re-check
                        // the cache (or the slot, if that runner was cancelled).
                        cede_semel().await;
                    }
                }) as BoxFuture<A>
            }),
            _phantom: PhantomData,
        }
    }

    /// Lift a value into the reader context (ignoring environment).
    ///
    /// This is the recommended way to lift values.
    #[inline]
    pub fn lift_value(value: A) -> Self {
        LectorAsync::purus(value)
    }
}

impl<E: core::fmt::Debug, A> core::fmt::Debug for LectorAsync<E, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LectorAsync")
            .field("run_fn", &"<async fn>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_lector_async_creation() {
        let _reader: LectorAsync<i32, i32> = LectorAsync::new(|x| async move { x * 2 });
    }

    #[test]
    fn test_lector_async_purus() {
        let _reader: LectorAsync<String, i32> = LectorAsync::purus(42);
    }

    #[test]
    fn test_lector_async_ask() {
        let _reader: LectorAsync<i32, i32> = LectorAsync::<i32, i32>::ask();
    }

    #[test]
    fn test_lector_async_clone() {
        let reader: LectorAsync<i32, i32> = LectorAsync::new(|x| async move { x });
        let _cloned = reader.clone();
    }

    #[test]
    fn test_lector_async_debug() {
        let reader: LectorAsync<i32, i32> = LectorAsync::new(|x| async move { x });
        let debug = alloc::format!("{reader:?}");
        assert!(debug.contains("LectorAsync"));
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
    fn test_lift_survives_cancelled_first_run() {
        let lifted: LectorAsync<(), i32> = LectorAsync::lift(PendetSemel {
            polled: false,
            value: 42,
        });

        // Poll the first run once and drop it mid-await (cancellation).
        {
            let mut first = Box::pin(lifted.clone().run(()));
            let mut cx = Context::from_waker(Waker::noop());
            assert!(first.as_mut().poll(&mut cx).is_pending());
        }

        // A later run must still complete instead of panicking.
        assert_eq!(drive(lifted.run(())), 42);
    }

    #[test]
    fn test_lift_async_survives_cancelled_first_run() {
        let lifted: LectorAsync<(), i32> =
            <LectorAsync<(), i32> as MonadTransformerAsync>::lift_async(PendetSemel {
                polled: false,
                value: 42,
            });

        {
            let mut first = Box::pin(lifted.clone().run(()));
            let mut cx = Context::from_waker(Waker::noop());
            assert!(first.as_mut().poll(&mut cx).is_pending());
        }

        assert_eq!(drive(lifted.run(())), 42);
    }

    #[test]
    fn test_lift_concurrent_runner_waits_for_cache() {
        let lifted: LectorAsync<(), i32> = LectorAsync::lift(PendetSemel {
            polled: false,
            value: 7,
        });
        let mut first = Box::pin(lifted.clone().run(()));
        let mut second = Box::pin(lifted.run(()));
        let mut cx = Context::from_waker(Waker::noop());

        // First runner takes the future from the slot; it is pending once.
        assert!(first.as_mut().poll(&mut cx).is_pending());
        // Second runner finds slot and cache empty: it must wait, not panic.
        assert!(second.as_mut().poll(&mut cx).is_pending());

        assert_eq!(first.as_mut().poll(&mut cx), Poll::Ready(7));
        assert_eq!(second.as_mut().poll(&mut cx), Poll::Ready(7));
    }
}
