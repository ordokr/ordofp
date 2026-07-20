//! `MonasLinearis` - Linear monadic operations
//!
//! > *"Ligatio in ordine lineari"*
//! > — Binding in linear order. (Neo-Latin)
//!
//! This module provides monadic abstractions for quantitative types,
//! enabling sequencing of linear computations.

extern crate alloc;

use alloc::vec::Vec;

use super::multiplicitas::Usage;
use super::qtt::Qtt;

// =============================================================================
// MonasLinearis - Linear Monad Trait
// =============================================================================

/// A monad that operates on values with explicit multiplicity.
///
/// `MonasLinearis` extends monadic operations to quantitative types,
/// preserving multiplicity annotations through bind operations.
///
/// # Latin Etymology
///
/// *Monas* = unit, monad (from Greek μονάς)
/// *Linearis* = linear
///
/// # Laws
///
/// Linear monads satisfy the standard monad laws:
///
/// 1. **Left Identity**: `pure(a).bind(f) ≡ f(a)`
/// 2. **Right Identity**: `m.bind(pure) ≡ m`
/// 3. **Associativity**: `m.bind(f).bind(g) ≡ m.bind(|x| f(x).bind(g))`
pub trait MonasLinearis<M: Usage>: Sized {
    /// The element type.
    type Elem;

    /// The output type with potentially different element type.
    type Output<B>: MonasLinearis<M, Elem = B>;

    /// Wrap a value in the monadic context (pure/return).
    fn purus(value: Self::Elem) -> Self;

    /// Sequentially compose two computations (bind/flatMap).
    fn bind<B, F>(self, f: F) -> Self::Output<B>
    where
        F: FnOnce(Self::Elem) -> Self::Output<B>;

    /// Map a function over the value.
    fn fmap<B, F>(self, f: F) -> Self::Output<B>
    where
        F: FnOnce(Self::Elem) -> B,
    {
        self.bind(|a| Self::Output::<B>::purus(f(a)))
    }

    /// Sequence two computations, discarding the first result.
    fn then<B>(self, other: Self::Output<B>) -> Self::Output<B> {
        self.bind(|_| other)
    }
}

// =============================================================================
// Qtt MonasLinearis Implementation
// =============================================================================

impl<A, M: Usage> MonasLinearis<M> for Qtt<A, M> {
    type Elem = A;
    type Output<B> = Qtt<B, M>;

    #[inline]
    fn purus(value: A) -> Self {
        Qtt::new(value)
    }

    #[inline]
    fn bind<B, F>(self, f: F) -> Qtt<B, M>
    where
        F: FnOnce(A) -> Qtt<B, M>,
    {
        f(self.consume())
    }
}

// =============================================================================
// QttMonad - Newtype for Monadic Operations
// =============================================================================

/// A monadic wrapper for Qtt operations with builder-style API.
///
/// This provides a more ergonomic interface for chaining
/// monadic operations on quantitative values.
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::{QttMonad, Semel};
///
/// let result = QttMonad::<_, Semel>::purus(5)
///     .map(|x| x * 2)
///     .flat_map(|x| QttMonad::purus(x + 1))
///     .run();
///
/// assert_eq!(result, 11);
/// ```
pub struct QttMonad<A, M: Usage> {
    value: Qtt<A, M>,
}

impl<A, M: Usage> QttMonad<A, M> {
    /// Create a pure `QttMonad`.
    #[inline]
    pub fn purus(value: A) -> Self {
        QttMonad {
            value: Qtt::new(value),
        }
    }

    /// Create from an existing Qtt.
    #[inline]
    pub fn from_qtt(qtt: Qtt<A, M>) -> Self {
        QttMonad { value: qtt }
    }

    /// Run the monad, extracting the value.
    #[inline]
    pub fn run(self) -> A {
        self.value.consume()
    }

    /// Get the underlying Qtt.
    #[inline]
    pub fn into_qtt(self) -> Qtt<A, M> {
        self.value
    }

    /// Map a function over the value.
    #[inline]
    pub fn map<B, F>(self, f: F) -> QttMonad<B, M>
    where
        F: FnOnce(A) -> B,
    {
        QttMonad {
            value: self.value.fmap(f),
        }
    }

    /// `FlatMap` (bind) over the value.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> QttMonad<B, M>
    where
        F: FnOnce(A) -> QttMonad<B, M>,
    {
        f(self.value.consume())
    }

    /// Sequence, discarding the first result.
    #[inline]
    pub fn then<B>(self, other: QttMonad<B, M>) -> QttMonad<B, M> {
        let _ = self.value.consume();
        other
    }

    /// Apply a function wrapped in a monad.
    #[inline]
    pub fn ap<B, F>(self, mf: QttMonad<F, M>) -> QttMonad<B, M>
    where
        F: FnOnce(A) -> B,
    {
        let f = mf.value.consume();
        QttMonad {
            value: Qtt::new(f(self.value.consume())),
        }
    }
}

impl<A: Clone, M: Usage> QttMonad<A, M>
where
    Qtt<A, M>: Clone,
{
    /// Duplicate the monad if the multiplicity allows.
    pub fn dup(&self) -> QttMonad<A, M> {
        QttMonad {
            value: self.value.clone(),
        }
    }
}

// =============================================================================
// Free Functions
// =============================================================================

/// Create a pure quantitative value.
#[inline]
pub fn purus_qtt<A, M: Usage>(value: A) -> Qtt<A, M> {
    Qtt::new(value)
}

/// Bind operation for quantitative values.
#[inline]
pub fn bind_qtt<A, B, M: Usage, F>(qtt: Qtt<A, M>, f: F) -> Qtt<B, M>
where
    F: FnOnce(A) -> Qtt<B, M>,
{
    f(qtt.consume())
}

/// Sequence a vector of quantitative computations.
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::{Qtt, Semel, sequence_qtt};
///
/// let computations = vec![
///     Qtt::<_, Semel>::new(1),
///     Qtt::new(2),
///     Qtt::new(3),
/// ];
///
/// let result = sequence_qtt(computations);
/// assert_eq!(result.consume(), vec![1, 2, 3]);
/// ```
#[inline]
pub fn sequence_qtt<A, M: Usage>(qts: Vec<Qtt<A, M>>) -> Qtt<Vec<A>, M> {
    let mut values: Vec<A> = Vec::with_capacity(qts.len());
    values.extend(qts.into_iter().map(super::qtt::Qtt::consume));
    Qtt::new(values)
}

/// Traverse with a quantitative function.
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::{Qtt, Semel, traverse_qtt};
///
/// let items = vec![1, 2, 3];
/// let result = traverse_qtt(items, |x| Qtt::<_, Semel>::new(x * 2));
/// assert_eq!(result.consume(), vec![2, 4, 6]);
/// ```
#[inline]
pub fn traverse_qtt<A, B, M: Usage, F>(items: Vec<A>, f: F) -> Qtt<Vec<B>, M>
where
    F: Fn(A) -> Qtt<B, M>,
{
    let mut values: Vec<B> = Vec::with_capacity(items.len());
    values.extend(items.into_iter().map(|a| f(a).consume()));
    Qtt::new(values)
}

/// Join nested quantitative values.
#[inline]
pub fn join_qtt<A, M: Usage>(nested: Qtt<Qtt<A, M>, M>) -> Qtt<A, M> {
    nested.consume()
}

/// Kleisli composition for quantitative functions.
#[inline]
pub fn kleisli_qtt<A, B, C, M: Usage, F, G>(f: F, g: G) -> impl FnOnce(A) -> Qtt<C, M>
where
    F: FnOnce(A) -> Qtt<B, M>,
    G: FnOnce(B) -> Qtt<C, M>,
{
    move |a| {
        let b = f(a).consume();
        g(b)
    }
}

/// Map2 for quantitative values.
#[inline]
pub fn map2_qtt<A, B, C, M: Usage, F>(qa: Qtt<A, M>, qb: Qtt<B, M>, f: F) -> Qtt<C, M>
where
    F: FnOnce(A, B) -> C,
{
    let a = qa.consume();
    let b = qb.consume();
    Qtt::new(f(a, b))
}

/// Map3 for quantitative values.
#[inline]
pub fn map3_qtt<A, B, C, D, M: Usage, F>(
    qa: Qtt<A, M>,
    qb: Qtt<B, M>,
    qc: Qtt<C, M>,
    f: F,
) -> Qtt<D, M>
where
    F: FnOnce(A, B, C) -> D,
{
    let a = qa.consume();
    let b = qb.consume();
    let c = qc.consume();
    Qtt::new(f(a, b, c))
}

/// Lift a binary function to quantitative values.
#[inline]
pub fn lift2_qtt<A, B, C, M: Usage, F>(f: F) -> impl FnOnce(Qtt<A, M>, Qtt<B, M>) -> Qtt<C, M>
where
    F: FnOnce(A, B) -> C,
{
    move |qa, qb| map2_qtt(qa, qb, f)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::multiplicitas::Semel;
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_monas_linearis_purus() {
        let q: Qtt<i32, Semel> = MonasLinearis::purus(42);
        assert_eq!(q.consume(), 42);
    }

    #[test]
    fn test_monas_linearis_bind() {
        let q: Qtt<i32, Semel> = Qtt::new(5);
        let result = q.bind(|x| Qtt::new(x * 2));
        assert_eq!(result.consume(), 10);
    }

    #[test]
    fn test_monas_linearis_fmap() {
        let q: Qtt<i32, Semel> = Qtt::new(5);
        let result = MonasLinearis::fmap(q, |x| x * 2);
        assert_eq!(result.consume(), 10);
    }

    #[test]
    fn test_monas_linearis_then() {
        let q1: Qtt<i32, Semel> = Qtt::new(5);
        let q2: Qtt<&str, Semel> = Qtt::new("done");
        let result = q1.then(q2);
        assert_eq!(result.consume(), "done");
    }

    #[test]
    fn test_qtt_monad_purus() {
        let m = QttMonad::<_, Semel>::purus(42);
        assert_eq!(m.run(), 42);
    }

    #[test]
    fn test_qtt_monad_map() {
        let m = QttMonad::<_, Semel>::purus(5);
        let result = m.map(|x| x * 2);
        assert_eq!(result.run(), 10);
    }

    #[test]
    fn test_qtt_monad_flat_map() {
        let m = QttMonad::<_, Semel>::purus(5);
        let result = m.flat_map(|x| QttMonad::purus(x + 10));
        assert_eq!(result.run(), 15);
    }

    #[test]
    fn test_qtt_monad_chaining() {
        let result = QttMonad::<_, Semel>::purus(5i32)
            .map(|x| x * 2)
            .flat_map(|x| QttMonad::purus(x + 1))
            .map(|x: i32| x.to_string())
            .run();

        assert_eq!(result, "11");
    }

    #[test]
    fn test_purus_qtt() {
        let q: Qtt<i32, Semel> = purus_qtt(42);
        assert_eq!(q.consume(), 42);
    }

    #[test]
    fn test_bind_qtt() {
        let q: Qtt<i32, Semel> = Qtt::new(5);
        let result = bind_qtt(q, |x| Qtt::new(x * 2));
        assert_eq!(result.consume(), 10);
    }

    #[test]
    fn test_sequence_qtt() {
        let qs: Vec<Qtt<i32, Semel>> = vec![Qtt::new(1), Qtt::new(2), Qtt::new(3)];
        let result = sequence_qtt(qs);
        assert_eq!(result.consume(), vec![1, 2, 3]);
    }

    #[test]
    fn test_traverse_qtt() {
        let items = vec![1, 2, 3];
        let result: Qtt<Vec<i32>, Semel> = traverse_qtt(items, |x| Qtt::new(x * 2));
        assert_eq!(result.consume(), vec![2, 4, 6]);
    }

    #[test]
    fn test_join_qtt() {
        let nested: Qtt<Qtt<i32, Semel>, Semel> = Qtt::new(Qtt::new(42));
        let flat = join_qtt(nested);
        assert_eq!(flat.consume(), 42);
    }

    #[test]
    fn test_kleisli_qtt() {
        let f = |x: i32| Qtt::<_, Semel>::new(x + 1);
        let g = |x: i32| Qtt::<_, Semel>::new(x * 2);

        let composed = kleisli_qtt(f, g);
        let result = composed(5);
        assert_eq!(result.consume(), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_map2_qtt() {
        let qa: Qtt<i32, Semel> = Qtt::new(10);
        let qb: Qtt<i32, Semel> = Qtt::new(32);
        let result = map2_qtt(qa, qb, |a, b| a + b);
        assert_eq!(result.consume(), 42);
    }

    #[test]
    fn test_map3_qtt() {
        let qa: Qtt<i32, Semel> = Qtt::new(1);
        let qb: Qtt<i32, Semel> = Qtt::new(2);
        let qc: Qtt<i32, Semel> = Qtt::new(3);
        let result = map3_qtt(qa, qb, qc, |a, b, c| a + b + c);
        assert_eq!(result.consume(), 6);
    }

    #[test]
    fn test_lift2_qtt() {
        let add = lift2_qtt(|a: i32, b: i32| a + b);
        let qa: Qtt<i32, Semel> = Qtt::new(10);
        let qb: Qtt<i32, Semel> = Qtt::new(32);
        let result = add(qa, qb);
        assert_eq!(result.consume(), 42);
    }

    // Monad Laws for Qtt

    #[test]
    fn test_left_identity() {
        // pure(a).bind(f) ≡ f(a)
        let a = 5;
        let f = |x: i32| Qtt::<_, Semel>::new(x * 2);

        let left: Qtt<i32, Semel> = MonasLinearis::purus(a);
        let left_result = left.bind(f);

        let right = f(a);

        assert_eq!(left_result.consume(), right.consume());
    }

    #[test]
    fn test_right_identity() {
        // m.bind(pure) ≡ m
        let m: Qtt<i32, Semel> = Qtt::new(42);
        let result = m.bind(Qtt::<_, Semel>::new);

        assert_eq!(result.consume(), 42);
    }

    #[test]
    fn test_associativity() {
        // m.bind(f).bind(g) ≡ m.bind(|x| f(x).bind(g))
        let f = |x: i32| Qtt::<_, Semel>::new(x + 1);
        let g = |x: i32| Qtt::<_, Semel>::new(x * 2);

        let m1: Qtt<i32, Semel> = Qtt::new(5);
        let left = m1.bind(f).bind(g);

        let m2: Qtt<i32, Semel> = Qtt::new(5);
        let right = m2.bind(|x| {
            let fx: Qtt<i32, Semel> = Qtt::new(x + 1);
            fx.bind(g)
        });

        assert_eq!(left.consume(), right.consume());
    }
}
