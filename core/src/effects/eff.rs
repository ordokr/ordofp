//! The Eff Monad - Type-safe effectful computations
//!
//! > *"Effectus in forma monádica"*
//! > — Effects in monadic form. (Neo-Latin)
//!
//! This module provides the `Eff` monad, inspired by Haskell's `effectful` library.
//! `Eff` is a monad that tracks effects at the type level, enabling type-safe
//! composition of effectful computations.
//!
//! # Design
//!
//! The `Eff` monad uses type-level effect lists to track which effects a computation
//! may perform. Handlers eliminate effects from the list, and a computation can only
//! be run when all effects have been handled.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Effect | Effectus | *effectus* = result |
//! | Pure | Purus | *purus* = clean, pure |
//! | Member | Membrum | *membrum* = member |
//! | Send | Mittere | *mittere* = to send |
//! | Run | Currere | *currere* = to run |
//!
//! # Status: pure-only facade
//!
//! Only the **pure** path of this monad is implemented. [`send`] constructs a
//! suspended computation for type-level effect tracking, but no interpreter
//! exists to run it: the operation-downcast handler machinery was never
//! implemented. `map` /
//! `flat_map` transform `Purus` values and merely propagate suspensions
//! without composing the supplied function into the continuation. Use this
//! facade for compile-time effect-row checking; use `effects::builtin` /
//! `handler_multi` for runnable handlers.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::effects::eff::{Eff, ENil, run_purus};
//!
//! // Pure computation (no effects)
//! let pure_comp: Eff<ENil, i32> = Eff::purus(42);
//! let result = run_purus(pure_comp);
//! assert_eq!(result, 42);
//! ```

extern crate alloc;

use alloc::boxed::Box;
use core::marker::PhantomData;

use super::algebraic::EffectusAlgebraicus;
use super::row_v2::{EffectId, EffectRow, EffectSet, assert_has_effect_type};

// =============================================================================
// Eff Monad
// =============================================================================

/// The Eff monad - effectful computation with type-level effect tracking.
///
/// `Eff<R, A>` represents a computation that:
/// - May perform effects from the effect row `R`
/// - Produces a value of type `A` when run
///
/// # Type Parameters
///
/// * `R` - The effect row (type-level list of effects)
/// * `A` - The result type
///
/// # Example
///
/// (pseudo-code — not compilable by design: this predates the v1-to-v2
/// effect-row migration, where rows were type-level lists. The current
/// encoding writes effect membership as a single `EffectSet<MASK>` bitmask
/// const, and the `/* ... */` placeholder for `effectful` was never a real
/// expression to begin with.)
///
/// ```ignore
/// use ordofp::effects::eff::{Eff, ENil};
///
/// // Pure computation
/// let pure: Eff<ENil, i32> = Eff::purus(42);
///
/// // Computation with effects
/// let effectful: Eff<EffectSet<{ 1 << builtin_ids::IO }>, String> = /* ... */;
/// ```
pub struct Eff<R: EffectRow, A> {
    /// The computation wrapped in a box for dynamic dispatch.
    run: Box<dyn FnOnce() -> EffResult<R, A> + Send>,
}

/// The result of running an Eff computation step.
pub enum EffResult<R: EffectRow, A> {
    /// Pure value - computation complete.
    Purus(A),
    /// Suspended on an effect - needs handling.
    Suspensus(EffSuspension<R, A>),
}

/// A suspended effect computation.
pub struct EffSuspension<R: EffectRow, A> {
    /// Type-erased effect operation.
    operation: Box<dyn core::any::Any + Send>,
    /// Type-erased continuation.
    continuation: Box<dyn core::any::Any + Send>,
    _phantom: PhantomData<(R, A)>,
}

impl<R: EffectRow, A: 'static + Send> Eff<R, A> {
    /// Create a pure computation that immediately returns a value.
    ///
    /// > *"Computatio pura"* — Pure computation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::eff::{Eff, ENil};
    ///
    /// let pure = Eff::<ENil, i32>::purus(42);
    /// ```
    #[inline]
    pub fn purus(value: A) -> Self {
        Eff {
            run: Box::new(move || EffResult::Purus(value)),
        }
    }

    /// Map a function over the result.
    ///
    /// Pure-only: a suspended computation (from [`send`]) is propagated
    /// unchanged and `f` is **not** composed into its continuation — see the
    /// module-level status note.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::eff::{Eff, ENil, run_purus};
    ///
    /// let eff: Eff<ENil, i32> = Eff::purus(21);
    /// let doubled = eff.map(|x| x * 2);
    /// assert_eq!(run_purus(doubled), 42);
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> Eff<R, B>
    where
        B: 'static + Send,
        F: FnOnce(A) -> B + Send + 'static,
    {
        Eff {
            run: Box::new(move || {
                match (self.run)() {
                    EffResult::Purus(a) => EffResult::Purus(f(a)),
                    EffResult::Suspensus(susp) => {
                        // We need to wrap the continuation to apply f
                        EffResult::Suspensus(EffSuspension {
                            operation: susp.operation,
                            continuation: susp.continuation,
                            _phantom: PhantomData,
                        })
                    }
                }
            }),
        }
    }

    /// Monadic bind (`flat_map`).
    ///
    /// Pure-only: a suspended computation (from [`send`]) is propagated
    /// unchanged and `f` is **not** composed into its continuation — see the
    /// module-level status note.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::eff::{Eff, ENil, run_purus};
    ///
    /// let eff: Eff<ENil, i32> = Eff::purus(41);
    /// let result = eff.flat_map(|x| Eff::purus(x + 1));
    /// assert_eq!(run_purus(result), 42);
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Eff<R, B>
    where
        B: 'static + Send,
        F: FnOnce(A) -> Eff<R, B> + Send + 'static,
    {
        Eff {
            run: Box::new(move || match (self.run)() {
                EffResult::Purus(a) => (f(a).run)(),
                EffResult::Suspensus(susp) => EffResult::Suspensus(EffSuspension {
                    operation: susp.operation,
                    continuation: susp.continuation,
                    _phantom: PhantomData,
                }),
            }),
        }
    }

    /// Applicative map2.
    #[inline]
    pub fn map2<B, C, F>(self, other: Eff<R, B>, f: F) -> Eff<R, C>
    where
        B: 'static + Send,
        C: 'static + Send,
        F: FnOnce(A, B) -> C + Send + 'static,
    {
        self.flat_map(move |a| other.map(move |b| f(a, b)))
    }
}

/// Run a pure computation (no effects).
///
/// This can only be called on `Eff<EffectSet<0>, A>` - computations with no effects.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::eff::{Eff, run_purus};
///
/// let result = run_purus(Eff::purus(42));
/// assert_eq!(result, 42);
/// ```
///
/// # Panics
///
/// Panics only if a computation typed with the empty effect set
/// `EffectSet<0>` nevertheless suspends on an effect, which the effect-row
/// types make impossible — such a panic indicates a bug in this crate.
#[inline]
pub fn run_purus<A: 'static + Send>(eff: Eff<EffectSet<0>, A>) -> A {
    match (eff.run)() {
        EffResult::Purus(a) => a,
        EffResult::Suspensus(_) => {
            // This should be unreachable for an empty effect set
            panic!("Pure computation suspended on effect - this is a bug")
        }
    }
}

// =============================================================================
// Effect Membership
// =============================================================================

/// Marker trait for effect membership in an effect row.
///
/// `E: Membrum<R>` names the intent that effect `E` is a member of effect
/// row `R`. Actual membership is enforced where operations are sent
/// (`assert_has_effect_type` in [`send`]), since the conditional impl that
/// used to check the row bit here required `generic_const_exprs`.
pub trait Membrum<R: EffectRow>: EffectusAlgebraicus {}

impl<E, R> Membrum<R> for E
where
    E: EffectusAlgebraicus + EffectId,
    R: EffectRow,
{
}

// =============================================================================
// Effect Operations
// =============================================================================

/// Send an effect operation to be handled.
///
/// This creates a computation that performs the given effect operation.
///
/// # Type Parameters
///
/// * `E` - The effect type
/// * `R` - The effect row (must contain `E`)
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::EffectusAlgebraicus;
/// use ordofp_core::effects::eff::{Eff, Membrum, send};
/// use ordofp_core::effects::row_v2::{EffectId, EffectRow, EffectSet};
///
/// struct GetCounter;
///
/// impl EffectusAlgebraicus for GetCounter {
///     type Result = i32;
/// }
///
/// impl EffectId for GetCounter {
///     const ID: u64 = 100;
///     const NAME: &'static str = "GetCounter";
/// }
///
/// fn get_state<R>() -> Eff<R, i32>
/// where
///     GetCounter: Membrum<R>,
///     R: EffectRow,
/// {
///     send(GetCounter)
/// }
///
/// let _eff: Eff<EffectSet<{ 1 << 100 }>, i32> = get_state::<EffectSet<{ 1 << 100 }>>();
/// ```
#[inline]
pub fn send<E, R>(op: E) -> Eff<R, E::Result>
where
    E: EffectusAlgebraicus + EffectId + Send + 'static,
    E::Result: Send + 'static,
    R: EffectRow,
{
    // Compile-time row check, replacing the old `HasEffectType<E>` bound.
    assert_has_effect_type::<R, E>();
    Eff {
        run: Box::new(move || {
            EffResult::Suspensus(EffSuspension {
                operation: Box::new(op),
                continuation: Box::new(()),
                _phantom: PhantomData,
            })
        }),
    }
}

// Deliberately no handler machinery (`interpret`, handler traits): running a
// suspended computation needs an operation-downcast step that does not exist.
// See the module docs for the pure-only status of this facade.

// =============================================================================
// Effect List Type Constructors
// =============================================================================

/// Empty effect list.
///
/// Represents a computation with no effects (pure). Retained as a type alias
/// for readability after the v1 row removal.
pub type ENil = EffectSet<0>;

// Deliberately no `ECons` cons-cell alias: the bitset row encoding has no
// type-level head effect, so effect membership is written directly as
// `EffectSet<MASK>` with an explicit const mask
// (e.g. `EffectSet<{ 1 << builtin_ids::IO }>`).

// =============================================================================
// Utility Functions
// =============================================================================

/// Lift a pure value into the Eff monad.
#[inline]
pub fn pure_eff<R: EffectRow, A: 'static + Send>(a: A) -> Eff<R, A> {
    Eff::purus(a)
}

/// Sequence two Eff computations, discarding the first result.
#[inline]
pub fn then<R: EffectRow, A: 'static + Send, B: 'static + Send>(
    first: Eff<R, A>,
    second: Eff<R, B>,
) -> Eff<R, B> {
    first.flat_map(move |_| second)
}

/// Sequence a vector of Eff computations.
pub fn sequence_eff<R: EffectRow, A: 'static + Send + Clone>(
    effs: alloc::vec::Vec<Eff<R, A>>,
) -> Eff<R, alloc::vec::Vec<A>> {
    effs.into_iter()
        .fold(Eff::purus(alloc::vec::Vec::new()), |acc, eff| {
            acc.flat_map(move |mut vec| {
                eff.map(move |a| {
                    vec.push(a);
                    vec
                })
            })
        })
}

/// Traverse a collection with an effectful function.
pub fn traverse_eff<R, A, B, F>(items: alloc::vec::Vec<A>, f: F) -> Eff<R, alloc::vec::Vec<B>>
where
    R: EffectRow,
    A: 'static + Send,
    B: 'static + Send + Clone,
    F: Fn(A) -> Eff<R, B> + Clone + Send + 'static,
{
    items
        .into_iter()
        .fold(Eff::purus(alloc::vec::Vec::new()), move |acc, a| {
            let f = f.clone();
            acc.flat_map(move |mut vec| {
                f(a).map(move |b| {
                    vec.push(b);
                    vec
                })
            })
        })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_eff_purus() {
        let eff: Eff<ENil, i32> = Eff::purus(42);
        let result = run_purus(eff);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_eff_map() {
        let eff: Eff<ENil, i32> = Eff::purus(21);
        let doubled = eff.map(|x| x * 2);
        let result = run_purus(doubled);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_eff_flat_map() {
        let eff: Eff<ENil, i32> = Eff::purus(20);
        let result = eff.flat_map(|x| Eff::purus(x + 22));
        assert_eq!(run_purus(result), 42);
    }

    #[test]
    fn test_eff_map2() {
        let eff1: Eff<ENil, i32> = Eff::purus(20);
        let eff2: Eff<ENil, i32> = Eff::purus(22);
        let sum = eff1.map2(eff2, |a, b| a + b);
        assert_eq!(run_purus(sum), 42);
    }

    #[test]
    fn test_pure_eff() {
        let eff: Eff<ENil, &str> = pure_eff("hello");
        assert_eq!(run_purus(eff), "hello");
    }

    #[test]
    fn test_then() {
        let first: Eff<ENil, i32> = Eff::purus(1);
        let second: Eff<ENil, &str> = Eff::purus("result");
        let result = then(first, second);
        assert_eq!(run_purus(result), "result");
    }

    #[test]
    fn test_sequence_eff() {
        let effs: alloc::vec::Vec<Eff<ENil, i32>> =
            vec![Eff::purus(1), Eff::purus(2), Eff::purus(3)];
        let sequenced = sequence_eff(effs);
        assert_eq!(run_purus(sequenced), vec![1, 2, 3]);
    }

    #[test]
    fn test_traverse_eff() {
        let items = vec![1, 2, 3];
        let result = traverse_eff(items, |x| Eff::<ENil, _>::purus(x * 2));
        assert_eq!(run_purus(result), vec![2, 4, 6]);
    }
}
