//! Reader Effect - Read-Only Environment
//!
//! The Reader effect provides access to a read-only environment that is
//! implicitly threaded through computations.
//!
//! # Handler Laws
//!
//! The Reader handler must satisfy these algebraic laws:
//!
//! ## Ask-Ask Law (Idempotence)
//! ```text
//! ask.and_then(|e| ask.map(|_| e)) = ask
//! ```
//! Asking twice and discarding the second is equivalent to asking once.
//!
//! ## Local-Ask Law
//! ```text
//! local(f, ask) = ask.map(f)
//! ```
//! Locally modifying the environment and then asking is equivalent to
//! asking and then applying the modification to the result.
//!
//! ## Local-Pure Law
//! ```text
//! local(f, pure(x)) = pure(x)
//! ```
//! Local modification doesn't affect pure values.
//!
//! # Verification Tier
//!
//! **Tier 1**: Laws are tested via property-based tests in `nexus::laws`.
//!
//! # Performance
//!
//! When a computation uses only the Reader effect, it compiles to
//! the same code as passing an environment reference through functions.

use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::nexus::effect::{Eff, EffectMarker};
use crate::nexus::row::{READER_BIT, Row};

// =============================================================================
// Reader Effect Type
// =============================================================================

/// The Reader effect marker type.
///
/// `ReaderEffect<E>` represents computations that can read from
/// an environment of type `E`.
#[derive(Copy, Clone, Debug)]
pub struct ReaderEffect<E> {
    _marker: PhantomData<E>,
}

impl<E> EffectMarker for ReaderEffect<E> {
    const BIT: u128 = READER_BIT;
    const NAME: &'static str = "Reader";
}

/// Type alias for a row containing only Reader.
pub type ReaderRow = Row<READER_BIT>;

// =============================================================================
// Reader Operations
// =============================================================================

/// Operations that can be performed with the Reader effect.
pub enum ReaderOp<E, A> {
    /// Get the entire environment.
    Ask,
    /// Extract a value from the environment.
    Asks(Box<dyn FnOnce(&E) -> A>),
    /// Run with a modified environment.
    Local(Box<dyn FnOnce(E) -> E>),
}

// =============================================================================
// Reader Effect Functions
// =============================================================================

/// Get the environment.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::reader::ReaderComputation;
///
/// #[derive(Clone, Default)]
/// struct Config {
///     port: u16,
/// }
///
/// let comp = ReaderComputation::<Config, Config>::ask();
/// let config = comp.run(&Config::default());
/// assert_eq!(config.port, 0);
/// ```
pub fn reader_ask<E: Clone + 'static>() -> Eff<ReaderRow, E> {
    Eff::lazy(|| crate::cold_panic!("reader_ask requires Reader handler"))
}

/// Extract a value from the environment.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::reader::ReaderComputation;
///
/// struct Config {
///     port: u16,
/// }
///
/// let comp = ReaderComputation::<Config, u16>::asks(|c: &Config| c.port);
/// let port = comp.run(&Config { port: 8080 });
/// assert_eq!(port, 8080);
/// ```
pub fn reader_asks<E: 'static, A: 'static, F: FnOnce(&E) -> A + 'static>(
    _f: F,
) -> Eff<ReaderRow, A> {
    Eff::lazy(|| crate::cold_panic!("reader_asks requires Reader handler"))
}

// =============================================================================
// Concrete Reader Implementation
// =============================================================================

/// A concrete reader computation that can be run.
///
/// Uses an enum representation to avoid heap allocation for pure values.
#[must_use = "computations do nothing unless run"]
#[repr(u8)]
pub enum ReaderComputation<E, A> {
    /// Pure value - no environment access, no allocation.
    Pure(A),
    /// Boxed computation.
    Boxed(Box<dyn FnOnce(&E) -> A>),
}

impl<E: 'static, A: 'static> ReaderComputation<E, A> {
    /// Create a new reader computation.
    #[inline(always)]
    pub fn new<F: FnOnce(&E) -> A + 'static>(f: F) -> Self {
        ReaderComputation::Boxed(Box::new(f))
    }

    /// Run the computation with an environment.
    #[inline(always)]
    pub fn run(self, env: &E) -> A {
        match self {
            ReaderComputation::Pure(a) => a,
            ReaderComputation::Boxed(f) => f(env),
        }
    }

    /// Pure value in reader context - NO HEAP ALLOCATION.
    #[inline(always)]
    pub fn pure(value: A) -> Self {
        ReaderComputation::Pure(value)
    }

    /// Get the environment.
    #[inline(always)]
    pub fn ask() -> ReaderComputation<E, E>
    where
        E: Clone,
    {
        ReaderComputation::new(|e: &E| e.clone())
    }

    /// Extract from the environment.
    #[inline(always)]
    pub fn asks<F: FnOnce(&E) -> A + 'static>(f: F) -> Self {
        ReaderComputation::new(f)
    }

    /// Map over the result.
    #[inline(always)]
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> ReaderComputation<E, B> {
        match self {
            ReaderComputation::Pure(a) => ReaderComputation::Pure(f(a)),
            ReaderComputation::Boxed(run_fn) => ReaderComputation::new(move |e| f(run_fn(e))),
        }
    }

    /// Chain two reader computations.
    #[inline(always)]
    pub fn and_then<B: 'static, F: FnOnce(A) -> ReaderComputation<E, B> + 'static>(
        self,
        f: F,
    ) -> ReaderComputation<E, B> {
        ReaderComputation::new(move |e| {
            let a = self.run(e);
            f(a).run(e)
        })
    }

    /// Run with a locally modified environment.
    #[inline(always)]
    pub fn local<F: FnOnce(&E) -> E + 'static>(self, modify: F) -> ReaderComputation<E, A> {
        ReaderComputation::new(move |e| {
            let new_env = modify(e);
            self.run(&new_env)
        })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestEnv {
        value: i32,
        multiplier: i32,
    }

    #[test]
    fn test_reader_pure() {
        let comp = ReaderComputation::<TestEnv, i32>::pure(42);
        let result = comp.run(&TestEnv {
            value: 0,
            multiplier: 1,
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_reader_ask() {
        let comp = ReaderComputation::<TestEnv, TestEnv>::ask();
        let env = TestEnv {
            value: 42,
            multiplier: 2,
        };
        let result = comp.run(&env);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_reader_asks() {
        let comp = ReaderComputation::<TestEnv, i32>::asks(|e| e.value);
        let result = comp.run(&TestEnv {
            value: 42,
            multiplier: 2,
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_reader_map() {
        let comp = ReaderComputation::<TestEnv, i32>::asks(|e| e.value).map(|x| x * 2);
        let result = comp.run(&TestEnv {
            value: 21,
            multiplier: 2,
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_reader_and_then() {
        let comp = ReaderComputation::<TestEnv, i32>::asks(|e| e.value).and_then(|v| {
            ReaderComputation::<TestEnv, i32>::asks(move |e: &TestEnv| v * e.multiplier)
        });
        let result = comp.run(&TestEnv {
            value: 21,
            multiplier: 2,
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_reader_local() {
        let comp = ReaderComputation::<TestEnv, i32>::asks(|e| e.value).local(|e| TestEnv {
            value: e.value + 10,
            multiplier: e.multiplier,
        });
        let result = comp.run(&TestEnv {
            value: 32,
            multiplier: 1,
        });
        assert_eq!(result, 42);
    }
}
