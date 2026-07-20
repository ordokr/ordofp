//! `ReaderT` Monad Transformer
//!
//! `ReaderT` adds environment/configuration reading capabilities to any base monad.
//! It represents computations that can read from a shared environment while
//! supporting the effects of the underlying monad.
//!
//! # Universalis over any Monad
//!
//! `ReaderT` provides Universalis `map`, `flat_map`, and `pure` that work with
//! any type implementing the library's `Functor`, `Monad`, or `Applicatio` traits.
//! Monad-specific convenience methods (`none`, `ok`, `err`, `map_err`) are provided
//! for common base monads.
//!
//! # Design note
//!
//! The generic `map`, `flat_map`, and `pure` live on the `impl<E, M: Functor>`,
//! `impl<E, M: Monad>`, and `impl<E, M: Applicatio>` blocks respectively. These
//! cover every base monad that implements the relevant typeclass (including
//! `Option`, `Result`, and `Vec`), so callers should prefer these generic
//! methods.
//!
//! The concrete `impl<E, A> ReaderT<E, Option<A>>` / `ReaderT<E, Result<A, Err>>`
//! / `ReaderT<E, Vec<A>>` blocks only exist for two purposes:
//!
//! 1. **Irreducibly concrete constructors** — `none`, `ok`, `err`, `singleton`,
//!    and `empty` must name concrete variants (`None`, `Ok(..)`, `Err(..)`,
//!    `vec![..]`) and cannot be written generically without additional typeclass
//!    machinery (e.g. `MonadError`, `MonadPlus`, `Pointed`).
//! 2. **Shape-specific combinators** — `apply`, `map2`, and `map2_ok` case-split
//!    on the concrete outer shape (e.g. `(Some, Some) -> Some(..)`) in ways the
//!    generic `Functor`/`Monad` traits cannot express without a full
//!    `Applicatio::ap` over `ReaderT`. `map_err` maps the error channel of
//!    `Result`, which the `Functor` trait (parameterised by the success channel)
//!    does not cover. `flat_map_vec` has a strictly looser bound than the
//!    generic `flat_map` — `Monad for Vec<A>` requires `A: Clone`, the concrete
//!    variant does not.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use ordofp_core::transformers::ReaderT;
//!
//! // Define a configuration type
//! struct Config {
//!     multiplier: i32,
//!     offset: i32,
//! }
//!
//! // ReaderT over Option - computation that reads config and may fail
//! let computation: ReaderT<Config, Option<i32>> = ReaderT::new(|cfg: &Config| {
//!     if cfg.multiplier != 0 {
//!         Some(42 * cfg.multiplier + cfg.offset)
//!     } else {
//!         None
//!     }
//! });
//!
//! let config = Config { multiplier: 2, offset: 10 };
//! assert_eq!(computation.run(&config), Some(94));
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::typeclasses::{Applicatio, Functor, Monad};

/// The `ReaderT` monad transformer adds environment reading to any base monad `M`.
///
/// `ReaderT<E, M>` represents a computation that can read from an environment
/// of type `E` and produces a value in the monad `M`.
///
/// # Type Parameters
///
/// - `E`: The environment type (configuration, dependencies, etc.)
/// - `M`: The base monad (e.g., `Option<A>`, `Result<A, Err>`)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::ReaderT;
///
/// // Configuration for our computation
/// struct DbConfig {
///     connection_string: String,
///     timeout: u32,
/// }
///
/// // A computation that reads config
/// let reader: ReaderT<DbConfig, Option<String>> = ReaderT::new(|cfg: &DbConfig| {
///     if cfg.timeout > 0 {
///         Some(cfg.connection_string.clone())
///     } else {
///         None
///     }
/// });
///
/// let config = DbConfig {
///     connection_string: "localhost:5432".to_string(),
///     timeout: 30,
/// };
/// assert_eq!(reader.run(&config), Some("localhost:5432".to_string()));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
///
/// ## Chaining Computations
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::ReaderT;
///
/// struct Config { factor: i32 }
///
/// let multiply: ReaderT<Config, Option<i32>> = ReaderT::new(|cfg: &Config| {
///     Some(10 * cfg.factor)
/// });
///
/// let add_offset = multiply.flat_map(|val| {
///     ReaderT::new(move |cfg: &Config| Some(val + cfg.factor))
/// });
///
/// let config = Config { factor: 5 };
/// assert_eq!(add_offset.run(&config), Some(55)); // 10*5 + 5 = 55
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
#[cfg(feature = "alloc")]
pub struct ReaderT<E, M> {
    /// The reader function
    run_fn: Box<dyn Fn(&E) -> M + Send + Sync>,
}

// ============================================================================
// Core operations (work for any M)
// ============================================================================

#[cfg(feature = "alloc")]
impl<E, M> ReaderT<E, M> {
    /// Creates a new `ReaderT` from a function.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<i32, Option<i32>> = ReaderT::new(|env: &i32| Some(*env * 2));
    /// assert_eq!(reader.run(&21), Some(42));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&E) -> M + Send + Sync + 'static,
    {
        ReaderT {
            run_fn: Box::new(f),
        }
    }

    /// Runs the reader with a specific environment.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<String, Option<usize>> = ReaderT::new(|s: &String| Some(s.len()));
    /// assert_eq!(reader.run(&"hello".to_string()), Some(5));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn run(&self, env: &E) -> M {
        (self.run_fn)(env)
    }

    /// Creates a `ReaderT` that returns the environment itself.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let ask: ReaderT<i32, Option<i32>> = ReaderT::ask();
    /// assert_eq!(ask.run(&42), Some(42));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn ask() -> Self
    where
        E: Clone + 'static,
        M: From<E> + 'static,
    {
        ReaderT::new(|env: &E| M::from(env.clone()))
    }

    /// Modifies the environment before running another reader.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<i32, Option<i32>> = ReaderT::new(|n: &i32| Some(*n * 2));
    /// // Transform environment: add 10 before running
    /// let modified = reader.local(|n: &i32| *n + 10);
    /// assert_eq!(modified.run(&5), Some(30)); // (5 + 10) * 2 = 30
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn local<F>(self, f: F) -> Self
    where
        F: Fn(&E) -> E + Send + Sync + 'static,
        E: 'static,
        M: 'static,
    {
        ReaderT::new(move |env: &E| {
            let modified_env = f(env);
            (self.run_fn)(&modified_env)
        })
    }
}

// ============================================================================
// Universalis Functor/Monad operations (work for any M: Functor/Monad)
// ============================================================================

#[cfg(feature = "alloc")]
impl<E: 'static, M: Functor + 'static> ReaderT<E, M> {
    /// Maps a function over the inner value.
    ///
    /// Works Universalisally for any base monad implementing `Functor`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<(), Option<i32>> = ReaderT::new(|_| Some(21));
    /// let doubled = reader.map(|x| x * 2);
    /// assert_eq!(doubled.run(&()), Some(42));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> ReaderT<E, M::Target<B>>
    where
        F: Fn(M::Inner) -> B + Send + Sync + 'static,
        M::Target<B>: 'static,
    {
        ReaderT::new(move |env: &E| {
            let m = (self.run_fn)(env);
            Functor::map(m, &f)
        })
    }
}

#[cfg(feature = "alloc")]
impl<E: 'static, M: Monad + 'static> ReaderT<E, M> {
    /// Chains a computation that returns a `ReaderT`.
    ///
    /// Works Universalisally for any base monad implementing `Monad`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<i32, Option<i32>> = ReaderT::new(|env: &i32| Some(*env));
    /// let chained = reader.flat_map(|val| {
    ///     ReaderT::new(move |env: &i32| {
    ///         if *env > 0 { Some(val * 2) } else { None }
    ///     })
    /// });
    /// assert_eq!(chained.run(&5), Some(10));
    /// assert_eq!(chained.run(&0), None);
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> ReaderT<E, M::Target<B>>
    where
        F: Fn(M::Inner) -> ReaderT<E, M::Target<B>> + Send + Sync + 'static,
        M::Target<B>: 'static,
    {
        ReaderT::new(move |env: &E| {
            let m = (self.run_fn)(env);
            m.flat_map(|a| f(a).run(env))
        })
    }
}

#[cfg(feature = "alloc")]
impl<E: 'static, M: Applicatio + 'static> ReaderT<E, M>
where
    M::Inner: Clone + Send + Sync,
{
    /// Lifts a value into the `ReaderT` context.
    ///
    /// Works Universalisally for any base monad implementing `Applicatio`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<(), Option<i32>> = ReaderT::pure(42);
    /// assert_eq!(reader.run(&()), Some(42));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn pure(value: M::Inner) -> Self {
        ReaderT::new(move |_: &E| M::pure(value.clone()))
    }
}

// ============================================================================
// Option-specific convenience methods
// ============================================================================

#[cfg(feature = "alloc")]
impl<E: 'static, A: 'static> ReaderT<E, Option<A>> {
    /// Creates a `ReaderT` that always returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<(), Option<i32>> = ReaderT::none();
    /// assert_eq!(reader.run(&()), None);
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn none() -> Self {
        ReaderT::new(|_: &E| None)
    }

    /// Applies a wrapped function to this value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let val: ReaderT<(), Option<i32>> = ReaderT::pure(21);
    /// fn double(x: i32) -> i32 { x * 2 }
    /// let func: ReaderT<(), Option<fn(i32) -> i32>> = ReaderT::pure(double as fn(i32) -> i32);
    /// let result = val.apply(func);
    /// assert_eq!(result.run(&()), Some(42));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn apply<B, F>(self, f: ReaderT<E, Option<F>>) -> ReaderT<E, Option<B>>
    where
        F: FnOnce(A) -> B + Clone + Send + Sync + 'static,
        B: 'static,
    {
        ReaderT::new(move |env: &E| match ((self.run_fn)(env), f.run(env)) {
            (Some(a), Some(func)) => Some(func(a)),
            _ => None,
        })
    }

    /// Combines two readers with a function.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let r1: ReaderT<(), Option<i32>> = ReaderT::pure(10);
    /// let r2: ReaderT<(), Option<i32>> = ReaderT::pure(20);
    /// let combined = r1.map2(r2, |a, b| a + b);
    /// assert_eq!(combined.run(&()), Some(30));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map2<B, C, F>(self, other: ReaderT<E, Option<B>>, f: F) -> ReaderT<E, Option<C>>
    where
        F: Fn(A, B) -> C + Send + Sync + 'static,
        B: 'static,
        C: 'static,
    {
        ReaderT::new(move |env: &E| match ((self.run_fn)(env), other.run(env)) {
            (Some(a), Some(b)) => Some(f(a, b)),
            _ => None,
        })
    }
}

// ============================================================================
// Result-specific convenience methods
// ============================================================================

#[cfg(feature = "alloc")]
impl<E: 'static, A: 'static, Err: 'static> ReaderT<E, Result<A, Err>> {
    /// Creates a `ReaderT` that always returns `Ok(value)`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<(), Result<i32, &str>> = ReaderT::ok(42);
    /// assert_eq!(reader.run(&()), Ok(42));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn ok(value: A) -> Self
    where
        A: Clone + Send + Sync,
    {
        ReaderT::new(move |_: &E| Ok(value.clone()))
    }

    /// Creates a `ReaderT` that always returns `Err(error)`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<(), Result<i32, &str>> = ReaderT::err("error");
    /// assert_eq!(reader.run(&()), Err("error"));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn err(error: Err) -> Self
    where
        Err: Clone + Send + Sync,
    {
        ReaderT::new(move |_: &E| Err(error.clone()))
    }

    /// Maps a function over the error value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::ReaderT;
    ///
    /// let reader: ReaderT<(), Result<i32, i32>> = ReaderT::err(1);
    /// let doubled = reader.map_err(|e| e * 2);
    /// assert_eq!(doubled.run(&()), Err(2));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map_err<Err2, F>(self, f: F) -> ReaderT<E, Result<A, Err2>>
    where
        F: Fn(Err) -> Err2 + Send + Sync + 'static,
        Err2: 'static,
    {
        ReaderT::new(move |env: &E| (self.run_fn)(env).map_err(&f))
    }

    /// Combines two readers with a function.
    #[inline]
    pub fn map2_ok<B, C, F>(
        self,
        other: ReaderT<E, Result<B, Err>>,
        f: F,
    ) -> ReaderT<E, Result<C, Err>>
    where
        F: Fn(A, B) -> C + Send + Sync + 'static,
        B: 'static,
        C: 'static,
    {
        ReaderT::new(move |env: &E| match ((self.run_fn)(env), other.run(env)) {
            (Ok(a), Ok(b)) => Ok(f(a, b)),
            (Err(e), _) | (_, Err(e)) => Err(e),
        })
    }
}

// ============================================================================
// Vec-specific convenience methods
// ============================================================================

#[cfg(feature = "alloc")]
impl<E: 'static, A: 'static> ReaderT<E, Vec<A>> {
    /// Creates a `ReaderT` that returns a single-element vector.
    #[inline]
    pub fn singleton(value: A) -> Self
    where
        A: Clone + Send + Sync,
    {
        ReaderT::new(move |_: &E| alloc::vec![value.clone()])
    }

    /// Creates a `ReaderT` that returns an empty vector.
    #[inline]
    pub fn empty() -> Self {
        ReaderT::new(|_: &E| alloc::vec![])
    }

    /// Chains a computation (alias for `flat_map` on Vec-based `ReaderT`).
    #[inline]
    pub fn flat_map_vec<B, F>(self, f: F) -> ReaderT<E, Vec<B>>
    where
        F: Fn(A) -> ReaderT<E, Vec<B>> + Send + Sync + 'static,
        B: 'static,
    {
        ReaderT::new(move |env: &E| {
            (self.run_fn)(env)
                .into_iter()
                .flat_map(|a| f(a).run(env))
                .collect()
        })
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn test_reader_t_new_and_run() {
        let reader: ReaderT<i32, Option<i32>> = ReaderT::new(|env: &i32| Some(*env * 2));
        assert_eq!(reader.run(&21), Some(42));
    }

    #[test]
    fn test_reader_t_pure() {
        let reader: ReaderT<(), Option<i32>> = ReaderT::pure(42);
        assert_eq!(reader.run(&()), Some(42));
    }

    #[test]
    fn test_reader_t_none() {
        let reader: ReaderT<(), Option<i32>> = ReaderT::none();
        assert_eq!(reader.run(&()), None);
    }

    #[test]
    fn test_reader_t_map() {
        let reader: ReaderT<(), Option<i32>> = ReaderT::pure(21);
        let doubled = reader.map(|x| x * 2);
        assert_eq!(doubled.run(&()), Some(42));
    }

    #[test]
    fn test_reader_t_flat_map() {
        let reader: ReaderT<i32, Option<i32>> = ReaderT::new(|env: &i32| Some(*env));
        let chained = reader.flat_map(|val| {
            ReaderT::new(move |env: &i32| if *env > 0 { Some(val * 2) } else { None })
        });
        assert_eq!(chained.run(&5), Some(10));
        assert_eq!(chained.run(&0), None);
    }

    #[test]
    #[allow(clippy::type_complexity)] // spelled-out nested transformer type is the point
    fn test_reader_t_apply() {
        let val: ReaderT<(), Option<i32>> = ReaderT::pure(21);
        let func: ReaderT<(), Option<fn(i32) -> i32>> =
            ReaderT::pure((|x: i32| x * 2) as fn(i32) -> i32);
        let result = val.apply(func);
        assert_eq!(result.run(&()), Some(42));
    }

    #[test]
    fn test_reader_t_local() {
        let reader: ReaderT<i32, Option<i32>> = ReaderT::new(|n: &i32| Some(*n * 2));
        let modified = reader.local(|n: &i32| *n + 10);
        assert_eq!(modified.run(&5), Some(30)); // (5 + 10) * 2 = 30
    }

    #[test]
    fn test_reader_t_result() {
        let reader: ReaderT<(), Result<i32, &str>> = ReaderT::ok(42);
        assert_eq!(reader.run(&()), Ok(42));

        let err_reader: ReaderT<(), Result<i32, &str>> = ReaderT::err("error");
        assert_eq!(err_reader.run(&()), Err("error"));
    }

    #[test]
    fn test_reader_t_result_universalis_map() {
        // Test that the Universalis map works with Result
        let reader: ReaderT<(), Result<i32, &str>> = ReaderT::ok(21);
        let doubled = reader.map(|x| x * 2);
        assert_eq!(doubled.run(&()), Ok(42));
    }

    #[test]
    fn test_reader_t_result_universalis_flat_map() {
        // Test that the Universalis flat_map works with Result
        let reader: ReaderT<i32, Result<i32, &str>> = ReaderT::new(|env: &i32| Ok(*env));
        let chained = reader.flat_map(|val| {
            ReaderT::new(move |env: &i32| {
                if *env > 0 {
                    Ok(val * 2)
                } else {
                    Err("negative")
                }
            })
        });
        assert_eq!(chained.run(&5), Ok(10));
        assert_eq!(chained.run(&0), Err("negative"));
    }

    // Monad law tests
    #[test]
    fn test_reader_t_left_identity() {
        // pure(a).flat_map(f) == f(a)
        let a = 5;
        let f = |x: i32| ReaderT::<(), Option<i32>>::pure(x * 2);

        let left = ReaderT::<(), Option<i32>>::pure(a).flat_map(f);
        let right = f(a);
        assert_eq!(left.run(&()), right.run(&()));
    }

    #[test]
    fn test_reader_t_right_identity() {
        // m.flat_map(pure) == m
        let m: ReaderT<(), Option<i32>> = ReaderT::pure(42);
        let result = m.flat_map(ReaderT::pure);
        assert_eq!(result.run(&()), Some(42));
    }

    #[test]
    fn test_reader_t_associativity() {
        // m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
        let f = |x: i32| ReaderT::<(), Option<i32>>::pure(x + 1);
        let g = |x: i32| ReaderT::<(), Option<i32>>::pure(x * 2);

        let left = ReaderT::<(), Option<i32>>::pure(5).flat_map(f).flat_map(g);
        let right = ReaderT::<(), Option<i32>>::pure(5).flat_map(move |x| f(x).flat_map(g));
        assert_eq!(left.run(&()), right.run(&()));
    }

    #[test]
    fn test_reader_t_vec() {
        let reader: ReaderT<(), Vec<i32>> = ReaderT::singleton(42);
        assert_eq!(reader.run(&()), alloc::vec![42]);

        let mapped = ReaderT::singleton(21).map(|x| x * 2);
        assert_eq!(mapped.run(&()), alloc::vec![42]);
    }
}
