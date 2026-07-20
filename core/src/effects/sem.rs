//! The Sem Monad - Polysemy-style extensible effects
//!
//! > *"Sem est significatio"*
//! > — Meaning is signification. (Scholastic philosophy)
//!
//! This module provides the `Sem` monad, inspired by Haskell's Polysemy library.
//! `Sem` is an extensible effects monad that supports effect interpretation,
//! reinterpretation, and subsumption.
//!
//! # Status: pure-only facade
//!
//! Only the **pure** path and row-level operations ([`subsume`], [`embed`],
//! [`run_io`]) are implemented. The interpretation/reinterpretation machinery
//! this design anticipated (operation downcast + handler dispatch) was never
//! built. [`send_sem`] /
//! [`raise`] construct suspensions for type-level effect tracking, but no
//! interpreter exists to run them.
//!
//! # Design
//!
//! The Sem monad differs from Eff in its focus on interpretation:
//! - Effects can be **subsumed**: merged when already handled
//! - Pure computations can be **run** ([`run_sem`], [`run_io`])
//!
//! # Inspired By
//!
//! - Polysemy's `Sem` monad
//! - fused-effects' effect algebra
//! - freer-simple's `Eff` monad
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Sem | Sem | Short for *semantica* |
//! | Interpret | Interpretari | *interpretari* = to explain |
//! | Reinterpret | Reinterpretari | *re-* + *interpretari* |
//! | Subsume | Subsumere | *subsumere* = to take under |
//! | Embed | Inserere | *inserere* = to insert |

extern crate alloc;

use alloc::boxed::Box;
use core::marker::PhantomData;

use super::algebraic::EffectusAlgebraicus;
use super::row_v2::{EffectId, EffectRow, EffectSet, assert_has_effect_type, assert_subrow};

// =============================================================================
// Sem Monad
// =============================================================================

/// The Sem monad - extensible effects with interpretation.
///
/// `Sem<R, A>` represents a computation that may perform effects from row `R`
/// and produces a value of type `A`. Unlike `Eff`, `Sem` focuses on flexible
/// effect interpretation.
///
/// # Type Parameters
///
/// * `R` - The effect row
/// * `A` - The result type
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::row_v2::EffectSet;
/// use ordofp_core::effects::sem::{Sem, run_sem};
///
/// // Pure computation
/// let pure: Sem<EffectSet<0>, i32> = Sem::purus(42);
/// let result = run_sem(pure);
/// assert_eq!(result, 42);
/// ```
pub struct Sem<R: EffectRow, A> {
    /// The computation, wrapped for dynamic dispatch.
    run: Box<dyn FnOnce() -> SemResult<R, A> + Send>,
}

/// Result of running a Sem computation step.
pub enum SemResult<R: EffectRow, A> {
    /// Pure value.
    Purus(A),
    /// Suspended on effect.
    Suspensus(SemSuspension<R, A>),
}

/// Suspended Sem computation.
pub struct SemSuspension<R: EffectRow, A> {
    /// Type-erased operation.
    operation: Box<dyn core::any::Any + Send>,
    /// Continuation.
    continuation: Box<dyn core::any::Any + Send>,
    _phantom: PhantomData<(R, A)>,
}

impl<R: EffectRow, A: 'static + Send> Sem<R, A> {
    /// Create a pure Sem computation.
    ///
    /// > *"Computatio pura"* — Pure computation.
    #[inline]
    pub fn purus(value: A) -> Self {
        Sem {
            run: Box::new(move || SemResult::Purus(value)),
        }
    }

    /// Map a function over the result.
    #[inline]
    pub fn map<B, F>(self, f: F) -> Sem<R, B>
    where
        B: 'static + Send,
        F: FnOnce(A) -> B + Send + 'static,
    {
        Sem {
            run: Box::new(move || match (self.run)() {
                SemResult::Purus(a) => SemResult::Purus(f(a)),
                SemResult::Suspensus(susp) => SemResult::Suspensus(SemSuspension {
                    operation: susp.operation,
                    continuation: susp.continuation,
                    _phantom: PhantomData,
                }),
            }),
        }
    }

    /// Monadic bind.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Sem<R, B>
    where
        B: 'static + Send,
        F: FnOnce(A) -> Sem<R, B> + Send + 'static,
    {
        Sem {
            run: Box::new(move || match (self.run)() {
                SemResult::Purus(a) => (f(a).run)(),
                SemResult::Suspensus(susp) => SemResult::Suspensus(SemSuspension {
                    operation: susp.operation,
                    continuation: susp.continuation,
                    _phantom: PhantomData,
                }),
            }),
        }
    }

    /// Applicative map2.
    #[inline]
    pub fn map2<B, C, F>(self, other: Sem<R, B>, f: F) -> Sem<R, C>
    where
        B: 'static + Send,
        C: 'static + Send,
        F: FnOnce(A, B) -> C + Send + 'static,
    {
        self.flat_map(move |a| other.map(move |b| f(a, b)))
    }
}

/// Run a pure Sem computation.
///
/// Can only be called on computations with no effects.
///
/// # Panics
///
/// Panics only if a computation typed with the empty effect set
/// `EffectSet<0>` nevertheless suspends on an effect, which the effect-row
/// types make impossible — such a panic indicates a bug in this crate.
#[inline]
pub fn run_sem<A: 'static + Send>(sem: Sem<EffectSet<0>, A>) -> A {
    match (sem.run)() {
        SemResult::Purus(a) => a,
        SemResult::Suspensus(_) => {
            panic!("Pure Sem computation suspended - this is a bug")
        }
    }
}

// Deliberately no interpretation machinery beyond `run_io` and `subsume`:
// running a suspended computation needs an operation downcast that does not
// exist. See the module docs for the pure-only status of this facade.

// =============================================================================
// Effect Subsumption
// =============================================================================

/// Subsume an effect that is already handled.
///
/// If effect `E` is already a member of row `R`, we can subsume it.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::EffectusAlgebraicus;
/// use ordofp_core::effects::row_v2::{EffectId, IoRow, builtin_ids};
/// use ordofp_core::effects::sem::{Sem, subsume};
///
/// struct MyIo;
///
/// impl EffectusAlgebraicus for MyIo {
///     type Result = ();
/// }
///
/// impl EffectId for MyIo {
///     const ID: u64 = builtin_ids::IO;
///     const NAME: &'static str = "MyIo";
/// }
///
/// // If IO is already handled in both the input and output row
/// let computation: Sem<IoRow, i32> = Sem::purus(42);
/// let subsumed: Sem<IoRow, i32> = subsume::<MyIo, IoRow, IoRow, i32>(computation);
/// ```
pub fn subsume<E, RIn, ROut, A>(sem: Sem<RIn, A>) -> Sem<ROut, A>
where
    E: EffectusAlgebraicus + EffectId + Send + 'static,
    RIn: EffectRow,
    ROut: EffectRow,
    A: 'static + Send,
{
    // Compile-time row checks, replacing the old `HasEffectType<E>` bounds.
    assert_has_effect_type::<RIn, E>();
    assert_has_effect_type::<ROut, E>();
    Sem {
        run: Box::new(move || match (sem.run)() {
            SemResult::Purus(a) => SemResult::Purus(a),
            SemResult::Suspensus(susp) => SemResult::Suspensus(SemSuspension {
                operation: susp.operation,
                continuation: susp.continuation,
                _phantom: PhantomData,
            }),
        }),
    }
}

// =============================================================================
// Effect Embedding
// =============================================================================

/// Embed a computation with fewer effects into a larger effect row.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::row_v2::{EffectSet, IoRow};
/// use ordofp_core::effects::sem::{Sem, embed};
///
/// let pure_computation: Sem<EffectSet<0>, i32> = Sem::purus(42);
/// let embedded: Sem<IoRow, i32> = embed(pure_computation);
/// ```
pub fn embed<R1, const SUPER: u128, A>(sem: Sem<R1, A>) -> Sem<EffectSet<SUPER>, A>
where
    R1: EffectRow,
    A: 'static + Send,
{
    // Compile-time subset check, replacing the old `SubRow<SUPER>` bound.
    assert_subrow::<R1, SUPER>();
    Sem {
        run: Box::new(move || match (sem.run)() {
            SemResult::Purus(a) => SemResult::Purus(a),
            SemResult::Suspensus(susp) => SemResult::Suspensus(SemSuspension {
                operation: susp.operation,
                continuation: susp.continuation,
                _phantom: PhantomData,
            }),
        }),
    }
}

// =============================================================================
// Effect Sending
// =============================================================================

/// Send an effect operation.
#[inline]
pub fn send_sem<E, R>(op: E) -> Sem<R, E::Result>
where
    E: EffectusAlgebraicus + EffectId + Send + 'static,
    E::Result: Send + 'static,
    R: EffectRow,
{
    // Compile-time row check, replacing the old `HasEffectType<E>` bound.
    assert_has_effect_type::<R, E>();
    Sem {
        run: Box::new(move || {
            SemResult::Suspensus(SemSuspension {
                operation: Box::new(op),
                continuation: Box::new(()),
                _phantom: PhantomData,
            })
        }),
    }
}

// =============================================================================
// Combinators
// =============================================================================

/// Lift a pure value into Sem.
#[inline]
pub fn pure_sem<R: EffectRow, A: 'static + Send>(a: A) -> Sem<R, A> {
    Sem::purus(a)
}

/// Sequence two Sem computations.
#[inline]
pub fn then_sem<R: EffectRow, A: 'static + Send, B: 'static + Send>(
    first: Sem<R, A>,
    second: Sem<R, B>,
) -> Sem<R, B> {
    first.flat_map(move |_| second)
}

/// Run an IO action (for computations that only have IO effects).
///
/// # Panics
///
/// Panics if the computation actually suspends on an IO operation: no IO
/// runtime exists yet, so only computations that happen to be pure (built
/// via `purus`/`map`/`flat_map` without performing IO) complete. See the
/// module docs for the pure-only status of this facade.
pub fn run_io<A: 'static + Send>(sem: Sem<super::row_v2::IoRow, A>) -> A {
    // This would actually run IO effects
    // For now, we just panic as IO requires runtime support
    match (sem.run)() {
        SemResult::Purus(a) => a,
        SemResult::Suspensus(_) => {
            panic!("IO effect not handled - requires runtime support")
        }
    }
}

// =============================================================================
// Raise (for effect-polymorphic operations)
// =============================================================================

/// Raise an effect operation.
///
/// This is a type-constrained version of `send_sem` that helps with inference.
#[inline]
pub fn raise<E, R>(op: E) -> Sem<R, E::Result>
where
    E: EffectusAlgebraicus + EffectId + Send + 'static,
    E::Result: Send + 'static,
    R: EffectRow,
{
    send_sem(op)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sem_purus() {
        let sem: Sem<EffectSet<0>, i32> = Sem::purus(42);
        let result = run_sem(sem);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_sem_map() {
        let sem: Sem<EffectSet<0>, i32> = Sem::purus(21);
        let doubled = sem.map(|x| x * 2);
        assert_eq!(run_sem(doubled), 42);
    }

    #[test]
    fn test_sem_flat_map() {
        let sem: Sem<EffectSet<0>, i32> = Sem::purus(20);
        let result = sem.flat_map(|x| Sem::purus(x + 22));
        assert_eq!(run_sem(result), 42);
    }

    #[test]
    fn test_sem_map2() {
        let sem1: Sem<EffectSet<0>, i32> = Sem::purus(20);
        let sem2: Sem<EffectSet<0>, i32> = Sem::purus(22);
        let sum = sem1.map2(sem2, |a, b| a + b);
        assert_eq!(run_sem(sum), 42);
    }

    #[test]
    fn test_pure_sem() {
        let sem: Sem<EffectSet<0>, &str> = pure_sem("hello");
        assert_eq!(run_sem(sem), "hello");
    }

    #[test]
    fn test_then_sem() {
        let first: Sem<EffectSet<0>, i32> = Sem::purus(1);
        let second: Sem<EffectSet<0>, &str> = Sem::purus("result");
        let result = then_sem(first, second);
        assert_eq!(run_sem(result), "result");
    }

    #[test]
    fn test_embed() {
        let pure: Sem<EffectSet<0>, i32> = Sem::purus(42);
        // Embed into a larger row (IO)
        use super::super::row_v2::IoRow;
        let embedded: Sem<IoRow, i32> = embed(pure);
        // Can't run directly without handling IO, but we can test the structure
        match (embedded.run)() {
            SemResult::Purus(a) => assert_eq!(a, 42),
            SemResult::Suspensus(_) => panic!("Should be pure"),
        }
    }
}
