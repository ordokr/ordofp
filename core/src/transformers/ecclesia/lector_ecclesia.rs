//! CPS `ReaderT` transformer.

#![cfg(feature = "transformers-cps")]

use alloc::boxed::Box;

/// CPS `ReaderT` transformer with constant-cost bind *composition*.
///
/// Uses continuation-passing style to avoid quadratic overhead in
/// left-associated bind chains: each `flat_map` adds a single closure layer
/// instead of re-traversing the existing chain. Composing n binds is O(n)
/// total (O(1) per bind), and running the chain is O(n) — the win over a
/// naive encoding is avoiding the O(n²) left-nested re-traversal, not
/// constant-time execution.
///
/// # Type Parameters
///
/// * `R` - Environment type
/// * `M` - Base monad type (e.g., `Option<A>`, `Result<A, E>`)
/// * `A` - Result type
pub struct LectorEcclesiaT<R, M> {
    /// Continuation function: R -> M
    run: Box<dyn Fn(R) -> M + Send + Sync>,
}

impl<R, M> LectorEcclesiaT<R, M> {
    /// Create a new CPS `ReaderT` from a function.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(R) -> M + Send + Sync + 'static,
    {
        Self { run: Box::new(f) }
    }

    /// Run the computation with an environment.
    #[inline]
    pub fn run(self, env: R) -> M {
        (self.run)(env)
    }

    /// Map over the result (functor operation).
    ///
    /// This is a simplified version that works when M is a simple type.
    /// For full monadic composition, use `flat_map`.
    #[inline]
    pub fn map<B, F>(self, f: F) -> LectorEcclesiaT<R, B>
    where
        F: Fn(M) -> B + Send + Sync + 'static,
        B: Send + Sync + 'static,
        M: Clone + 'static,
        R: Clone + 'static,
    {
        let run_old = self.run;
        LectorEcclesiaT::new(move |env: R| {
            let result = run_old(env);
            f(result)
        })
    }

    /// Flat map (monadic bind) with O(1) *per-bind* composition cost.
    ///
    /// Composes continuations so each bind adds one closure layer (see the
    /// type-level docs for the precise complexity claim).
    ///
    /// Note: like `map`, this is a simplified signature that binds over the
    /// whole monadic value `M`, not the inner `A` — a full implementation
    /// would require `M: Monad` machinery.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> LectorEcclesiaT<R, B>
    where
        F: Fn(M) -> LectorEcclesiaT<R, B> + Send + Sync + 'static,
        B: Send + Sync + 'static,
        M: Clone + 'static,
        R: Clone + 'static,
    {
        let run_old = self.run;
        LectorEcclesiaT::new(move |env: R| {
            let inner = run_old(env.clone());
            let next = f(inner);
            next.run(env)
        })
    }

    /// Ask for the environment (`ReaderT` operation).
    #[inline]
    pub fn ask() -> LectorEcclesiaT<R, R>
    where
        R: Clone + Send + Sync + 'static,
    {
        LectorEcclesiaT::new(|env: R| env)
    }

    /// Local modification of environment (`ReaderT` operation).
    #[inline]
    pub fn local<F>(self, f: F) -> LectorEcclesiaT<R, M>
    where
        F: Fn(R) -> R + Send + Sync + 'static,
        M: 'static,
        R: Clone + 'static,
    {
        let run_old = self.run;
        LectorEcclesiaT::new(move |env: R| {
            let modified_env = f(env);
            run_old(modified_env)
        })
    }
}
