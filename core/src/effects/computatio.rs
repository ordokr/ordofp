//! Computatio - Row-polymorphic effectful computations
//!
//! > *"Computatio effectuosa"*
//! > — Effectful computation. (Neo-Latin)
//!
//! This module provides `Computatio<R, A>`, a type representing a computation
//! that produces a value of type `A` and may perform effects described by row `R`.
//!
//! # Design
//!
//! `Computatio` is inspired by:
//! - Koka's row-polymorphic effect system
//! - PureScript's `Eff` monad
//! - ZIO's `ZIO[R, E, A]` (environment/error/result)
//!
//! Unlike full algebraic effects, `Computatio` uses effect row types to track
//! which effects a computation may perform, without requiring runtime effect handlers.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Computation | Computatio | *computatio* = calculation |
//! | Pure | Purus | *purus* = clean, unmixed |
//! | Map | Mappare | Medieval Latin for mapping |
//! | Bind | Ligare | *ligare* = to bind |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::effects::computatio::Computatio;
//! use ordofp_core::effects::row_v2::{EffectSet, IoRow};
//!
//! // Pure computation
//! let pure: Computatio<EffectSet<0>, i32> = Computatio::purus(42);
//!
//! // IO computation
//! let io: Computatio<IoRow, String> = Computatio::effectus(|| "Hello".to_string());
//! assert_eq!(io.run(), "Hello");
//!
//! // Chain computations
//! let result = pure.map(|x| x * 2).bind(|x| Computatio::purus(x.to_string()));
//! assert_eq!(result.run(), "84");
//! ```

use super::row_v2::{EffectRow, EffectSet, assert_subrow};
use alloc::boxed::Box;
use core::marker::PhantomData;

/// A row-polymorphic effectful computation.
///
/// `Computatio<R, A>` represents a computation that:
/// - Produces a value of type `A`
/// - May perform effects described by effect row `R`
///
/// The computation is lazy - it doesn't execute until `run()` is called.
///
/// # Type Parameters
///
/// * `R` - The effect row describing which effects this computation may perform
/// * `A` - The type of value produced by the computation
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::computatio::Computatio;
/// use ordofp_core::effects::row_v2::EffectSetVacuus;
///
/// let comp: Computatio<EffectSetVacuus, i32> = Computatio::purus(42);
/// assert_eq!(comp.run(), 42);
/// ```
pub struct Computatio<R: EffectRow, A> {
    inner: Box<dyn FnOnce() -> A + Send>,
    _row: PhantomData<R>,
}

impl<R: EffectRow, A> Computatio<R, A> {
    /// Create a computation from a thunk.
    ///
    /// The provided function will be called when the computation is run.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    /// use ordofp_core::effects::row_v2::IoRow;
    ///
    /// let comp: Computatio<IoRow, i32> = Computatio::from_thunk(|| {
    ///     println!("Side effect!");
    ///     42
    /// });
    /// assert_eq!(comp.run(), 42);
    /// ```
    pub fn from_thunk<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static,
    {
        Computatio {
            inner: Box::new(f),
            _row: PhantomData,
        }
    }

    /// Run the computation, producing its result.
    ///
    /// This consumes the computation and executes all effects.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    /// use ordofp_core::effects::row_v2::EffectSetVacuus;
    ///
    /// let comp = Computatio::<EffectSetVacuus, _>::purus(42);
    /// assert_eq!(comp.run(), 42);
    /// ```
    #[inline]
    pub fn run(self) -> A {
        (self.inner)()
    }
}

impl<A: Send + 'static> Computatio<EffectSet<0>, A> {
    /// Create a pure computation with no effects.
    ///
    /// This is the `return` or `pure` operation for the Computatio monad.
    ///
    /// > *"Purus computatio"* — Pure computation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let comp = Computatio::purus(42);
    /// assert_eq!(comp.run(), 42);
    /// ```
    #[inline]
    pub fn purus(value: A) -> Self {
        Computatio {
            inner: Box::new(move || value),
            _row: PhantomData,
        }
    }
}

impl<const MASK: u128, A: Send + 'static> Computatio<EffectSet<MASK>, A> {
    /// Create an effectful computation.
    ///
    /// The target effect set is chosen by the caller via the mask type
    /// parameter (typically by annotating the binding with a concrete
    /// `EffectSet<...>` or a v2 type alias such as `IoRow`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    /// use ordofp_core::effects::row_v2::IoRow;
    ///
    /// let io: Computatio<IoRow, String> = Computatio::effectus(|| {
    ///     // Perform IO
    ///     "result".to_string()
    /// });
    /// assert_eq!(io.run(), "result");
    /// ```
    pub fn effectus<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static,
    {
        Computatio {
            inner: Box::new(f),
            _row: PhantomData,
        }
    }
}

impl<R: EffectRow, A: Send + 'static> Computatio<R, A> {
    /// Map a function over the computation's result.
    ///
    /// This is the functor `fmap` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let comp = Computatio::purus(21);
    /// let doubled = comp.map(|x| x * 2);
    /// assert_eq!(doubled.run(), 42);
    /// ```
    pub fn map<B: Send + 'static, F>(self, f: F) -> Computatio<R, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        let inner = self.inner;
        Computatio {
            inner: Box::new(move || f((inner)())),
            _row: PhantomData,
        }
    }

    /// Apply a function inside a computation to this computation's value.
    ///
    /// This is the applicative `ap` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let func: Computatio<_, fn(i32) -> i32> = Computatio::purus(|x| x * 2);
    /// let value = Computatio::purus(21);
    /// let result = value.ap(func);
    /// assert_eq!(result.run(), 42);
    /// ```
    pub fn ap<B: Send + 'static, F: FnOnce(A) -> B + Send + 'static>(
        self,
        func: Computatio<R, F>,
    ) -> Computatio<R, B> {
        let inner_self = self.inner;
        let inner_func = func.inner;
        Computatio {
            inner: Box::new(move || {
                let f = (inner_func)();
                let a = (inner_self)();
                f(a)
            }),
            _row: PhantomData,
        }
    }

    /// Sequence two computations, discarding the result of the first.
    ///
    /// This is the `*>` or `>>` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let first = Computatio::purus("ignored");
    /// let second = Computatio::purus(42);
    /// let result = first.then(second);
    /// assert_eq!(result.run(), 42);
    /// ```
    pub fn then<B: Send + 'static>(self, other: Computatio<R, B>) -> Computatio<R, B> {
        let inner_self = self.inner;
        let inner_other = other.inner;
        Computatio {
            inner: Box::new(move || {
                let _ = (inner_self)();
                (inner_other)()
            }),
            _row: PhantomData,
        }
    }

    /// Sequence two computations, discarding the result of the second.
    ///
    /// This is the `<*` operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let first = Computatio::purus(42);
    /// let second = Computatio::purus("ignored");
    /// let result = first.before(second);
    /// assert_eq!(result.run(), 42);
    /// ```
    pub fn before<B: Send + 'static>(self, other: Computatio<R, B>) -> Computatio<R, A> {
        let inner_self = self.inner;
        let inner_other = other.inner;
        Computatio {
            inner: Box::new(move || {
                let a = (inner_self)();
                let _ = (inner_other)();
                a
            }),
            _row: PhantomData,
        }
    }

    /// Bind a function that produces a computation over this computation.
    ///
    /// This is the monad `>>=` (flatMap) operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let comp = Computatio::purus(21);
    /// let result = comp.bind(|x| Computatio::purus(x * 2));
    /// assert_eq!(result.run(), 42);
    /// ```
    pub fn bind<B: Send + 'static, F>(self, f: F) -> Computatio<R, B>
    where
        F: FnOnce(A) -> Computatio<R, B> + Send + 'static,
    {
        let inner = self.inner;
        Computatio {
            inner: Box::new(move || {
                let a = (inner)();
                f(a).run()
            }),
            _row: PhantomData,
        }
    }

    /// Flatten a nested computation.
    ///
    /// This is the monad `join` operation.
    ///
    /// # Example
    ///
    /// (pseudo-code — not compilable by design: `flatten`'s current bound is
    /// `A: Into<Computatio<R, A>>` on `self: Computatio<R, A>`, not
    /// `self: Computatio<R, Computatio<R, A>>` as this monad-join example
    /// assumes; no `Into` impl exists that would let a concrete `A` such as
    /// `i32` satisfy that bound, so `flatten` cannot actually be called with
    /// the intent shown here.)
    ///
    /// ```ignore
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let nested = Computatio::purus(Computatio::purus(42));
    /// let flat = nested.flatten();
    /// assert_eq!(flat.run(), 42);
    /// ```
    pub fn flatten(self) -> Computatio<R, A>
    where
        A: Into<Computatio<R, A>>,
    {
        self.bind(core::convert::Into::into)
    }

    /// Lift this computation to a larger effect row.
    ///
    /// This allows using a computation in a context that requires
    /// more effects than the computation actually uses.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    /// use ordofp_core::effects::row_v2::{EffectSetVacuus, IoRow};
    ///
    /// let pure: Computatio<EffectSetVacuus, i32> = Computatio::purus(42);
    /// let lifted: Computatio<IoRow, i32> = pure.lift();
    /// assert_eq!(lifted.run(), 42);
    /// ```
    pub fn lift<const SUPER: u128>(self) -> Computatio<EffectSet<SUPER>, A> {
        // Compile-time subset check, replacing the old `SubRow<SUPER>` bound.
        assert_subrow::<R, SUPER>();
        Computatio {
            inner: self.inner,
            _row: PhantomData,
        }
    }

    /// Zip this computation with another, running both and combining results.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let a = Computatio::purus(1);
    /// let b = Computatio::purus(2);
    /// let zipped = a.zip(b);
    /// assert_eq!(zipped.run(), (1, 2));
    /// ```
    pub fn zip<B: Send + 'static>(self, other: Computatio<R, B>) -> Computatio<R, (A, B)> {
        let inner_self = self.inner;
        let inner_other = other.inner;
        Computatio {
            inner: Box::new(move || {
                let a = (inner_self)();
                let b = (inner_other)();
                (a, b)
            }),
            _row: PhantomData,
        }
    }

    /// Zip this computation with another using a combining function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::effects::computatio::Computatio;
    ///
    /// let a = Computatio::purus(10);
    /// let b = Computatio::purus(20);
    /// let sum = a.zip_with(b, |x, y| x + y);
    /// assert_eq!(sum.run(), 30);
    /// ```
    pub fn zip_with<B: Send + 'static, C: Send + 'static, F>(
        self,
        other: Computatio<R, B>,
        f: F,
    ) -> Computatio<R, C>
    where
        F: FnOnce(A, B) -> C + Send + 'static,
    {
        self.zip(other).map(|(a, b)| f(a, b))
    }
}

/// Combine two computations with different effect rows.
///
/// The resulting computation is parameterized by the caller-chosen effect
/// row, which must name a superset of the union of the two input masks. In
/// v1 the return row was inferred as `EffectusUnio<R1, R2>`; v2 cannot name
/// that union in type position without `generic_const_exprs`, so the caller
/// supplies the combined mask explicitly — and a compile-time `assert_subrow`
/// check rejects any `MOUT` that drops effects from `M1` or `M2`.
///
/// Choosing an output row that erases effects fails to compile:
///
/// ```compile_fail
/// use ordofp_core::effects::computatio::{Computatio, combine_effectus};
/// use ordofp_core::effects::row_v2::{EffectSet, IoRow, StateRow};
///
/// let io: Computatio<IoRow, i32> = Computatio::effectus(|| 1);
/// let state: Computatio<StateRow, i32> = Computatio::from_thunk(|| 2);
/// // EffectSet<0> drops both effects — rejected at compile time.
/// let combined: Computatio<EffectSet<0>, _> = combine_effectus(io, state);
/// let _ = combined.run();
/// ```
pub fn combine_effectus<
    const M1: u128,
    const M2: u128,
    const MOUT: u128,
    A: Send + 'static,
    B: Send + 'static,
>(
    comp1: Computatio<EffectSet<M1>, A>,
    comp2: Computatio<EffectSet<M2>, B>,
) -> Computatio<EffectSet<MOUT>, (A, B)> {
    // Compile-time superset checks: MOUT must not erase effects from M1 or M2.
    assert_subrow::<EffectSet<M1>, MOUT>();
    assert_subrow::<EffectSet<M2>, MOUT>();
    let inner1 = comp1.inner;
    let inner2 = comp2.inner;
    Computatio {
        inner: Box::new(move || {
            let a = (inner1)();
            let b = (inner2)();
            (a, b)
        }),
        _row: PhantomData,
    }
}

/// Sequence a vector of computations, collecting results.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::computatio::{Computatio, sequence};
///
/// let comps = vec![
///     Computatio::purus(1),
///     Computatio::purus(2),
///     Computatio::purus(3),
/// ];
/// let sequenced = sequence(comps);
/// assert_eq!(sequenced.run(), vec![1, 2, 3]);
/// ```
pub fn sequence<R: EffectRow, A: Send + 'static>(
    computations: alloc::vec::Vec<Computatio<R, A>>,
) -> Computatio<R, alloc::vec::Vec<A>> {
    Computatio {
        inner: Box::new(move || {
            let mut results = alloc::vec::Vec::with_capacity(computations.len());
            for c in computations {
                results.push(c.run());
            }
            results
        }),
        _row: PhantomData,
    }
}

/// Traverse a collection with an effectful function.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::computatio::{Computatio, traverse};
///
/// let values = vec![1, 2, 3];
/// let result = traverse(values, |x| Computatio::purus(x * 2));
/// assert_eq!(result.run(), vec![2, 4, 6]);
/// ```
pub fn traverse<R: EffectRow, A, B: Send + 'static, F>(
    items: alloc::vec::Vec<A>,
    f: F,
) -> Computatio<R, alloc::vec::Vec<B>>
where
    F: Fn(A) -> Computatio<R, B> + Send + 'static,
    A: Send + 'static,
{
    let computations: alloc::vec::Vec<Computatio<R, B>> = items.into_iter().map(&f).collect();
    sequence(computations)
}

/// Create a computation that performs an effect and returns a value.
///
/// This is a convenience function for creating effectful computations.
/// The caller chooses the resulting effect mask via the const generic.
#[inline]
pub fn perform<const MASK: u128, A: Send + 'static, F>(f: F) -> Computatio<EffectSet<MASK>, A>
where
    F: FnOnce() -> A + Send + 'static,
{
    Computatio::from_thunk(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::row_v2::{IoRow, IoStateRow, StateRow};
    use alloc::string::{String, ToString};
    use alloc::vec;

    #[test]
    fn test_purus() {
        let comp = Computatio::purus(42);
        assert_eq!(comp.run(), 42);
    }

    #[test]
    fn test_map() {
        let comp = Computatio::purus(21);
        let doubled = comp.map(|x| x * 2);
        assert_eq!(doubled.run(), 42);
    }

    #[test]
    fn test_bind() {
        let comp = Computatio::purus(21);
        let result = comp.bind(|x| Computatio::purus(x * 2));
        assert_eq!(result.run(), 42);
    }

    #[test]
    fn test_then() {
        let first = Computatio::purus("ignored");
        let second = Computatio::purus(42);
        let result = first.then(second);
        assert_eq!(result.run(), 42);
    }

    #[test]
    fn test_before() {
        let first = Computatio::purus(42);
        let second = Computatio::purus("ignored");
        let result = first.before(second);
        assert_eq!(result.run(), 42);
    }

    #[test]
    fn test_zip() {
        let a = Computatio::purus(1);
        let b = Computatio::purus(2);
        let zipped = a.zip(b);
        assert_eq!(zipped.run(), (1, 2));
    }

    #[test]
    fn test_zip_with() {
        let a = Computatio::purus(10);
        let b = Computatio::purus(20);
        let sum = a.zip_with(b, |x, y| x + y);
        assert_eq!(sum.run(), 30);
    }

    #[test]
    fn test_sequence() {
        let comps = vec![
            Computatio::purus(1),
            Computatio::purus(2),
            Computatio::purus(3),
        ];
        let sequenced = sequence(comps);
        assert_eq!(sequenced.run(), vec![1, 2, 3]);
    }

    #[test]
    fn test_traverse() {
        let values = vec![1, 2, 3];
        let result = traverse(values, |x| Computatio::purus(x * 2));
        assert_eq!(result.run(), vec![2, 4, 6]);
    }

    #[test]
    fn test_effectus() {
        let io: Computatio<IoRow, String> = Computatio::effectus(|| "Hello".to_string());
        assert_eq!(io.run(), "Hello");
    }

    #[test]
    fn test_lift() {
        let pure: Computatio<EffectSet<0>, i32> = Computatio::purus(42);
        let lifted: Computatio<IoRow, i32> = pure.lift();
        assert_eq!(lifted.run(), 42);
    }

    #[test]
    fn test_combine_effectus() {
        let io: Computatio<IoRow, i32> = Computatio::effectus(|| 1);
        let state: Computatio<StateRow, i32> = Computatio::from_thunk(|| 2);
        let combined: Computatio<IoStateRow, _> = combine_effectus(io, state);
        assert_eq!(combined.run(), (1, 2));
    }

    #[test]
    fn test_monad_laws_left_identity() {
        let a = 42;
        let f = |x: i32| Computatio::purus(x * 2);

        let left = Computatio::purus(a).bind(f);
        let right = f(a);

        assert_eq!(left.run(), right.run());
    }

    #[test]
    fn test_monad_laws_right_identity() {
        let m = Computatio::purus(42);
        let m_clone = Computatio::purus(42);

        let result = m.bind(Computatio::purus);

        assert_eq!(result.run(), m_clone.run());
    }

    #[test]
    fn test_monad_laws_associativity() {
        let f = |x: i32| Computatio::purus(x + 1);
        let g = |x: i32| Computatio::purus(x * 2);

        let m1 = Computatio::purus(5);
        let m2 = Computatio::purus(5);

        let left = m1.bind(f).bind(g);
        let right = m2.bind(move |x| Computatio::purus(x + 1).bind(g));

        assert_eq!(left.run(), right.run());
    }

    #[test]
    fn test_chaining() {
        let result = Computatio::purus(5)
            .bind(|x| Computatio::purus(x + 1))
            .bind(|x| Computatio::purus(x * 2))
            .map(|x: i32| x.to_string())
            .run();

        assert_eq!(result, "12");
    }

    #[test]
    fn test_perform() {
        let comp: Computatio<IoRow, _> = perform(|| 42);
        assert_eq!(comp.run(), 42);
    }
}
