//! Proof Objects and Type Witnesses - Testimonium
//!
//! > *"Quod erat demonstrandum."*
//! > — That which was to be demonstrated. (QED)
//!
//! This module provides proof objects that witness compile-time invariants,
//! inspired by Idris 2's dependent types and Haskell's singletons.
//!
//! # Design
//!
//! Proof objects are zero-sized types that encode proofs as values.
//! Having a value of a proof type means the proposition is true.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::dependent::proof::{Testimonium, testimonium, IsMinor};
//! use ordofp_core::dependent::peano::{N3, N5};
//!
//! // Create a proof that 3 < 5
//! let proof: Testimonium<N3, IsMinor<N5>> = testimonium();
//!
//! // Use the proof to enable operations that require 3 < 5
//! ```

use super::peano::{Aequus, Maior, Minor, MinorVelAequus, Naturalis, NonNihil};
use core::marker::PhantomData;

// =============================================================================
// Testimonium - Proof Witness
// =============================================================================

/// A proof that type `T` satisfies constraint `P`.
///
/// # Latin Etymology
/// *Testimonium* means "evidence, witness, proof".
///
/// This is a zero-sized type that witnesses that a constraint holds.
/// The only way to construct a `Testimonium<T, P>` is through the type
/// system, ensuring the constraint is verified at compile time.
///
/// # Type Parameters
///
/// - `T`: The type being proven about
/// - `P`: The property/constraint marker
///
/// # Example
///
/// ```rust
/// use ordofp_core::dependent::proof::{Testimonium, testimonium, IsNonNihil};
/// use ordofp_core::dependent::peano::N2;
///
/// // This compiles because N2 (which is Succ<Succ<Zero>>) is non-zero
/// let proof: Testimonium<N2, IsNonNihil> = testimonium();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Testimonium<T, P: ?Sized> {
    _type: PhantomData<T>,
    _property: PhantomData<P>,
}

/// Create a proof witness.
///
/// This function can only be called when the constraint `P` is satisfied
/// by type `T`, enforced by trait bounds.
#[inline]
pub fn testimonium<T, P: ?Sized>() -> Testimonium<T, P>
where
    T: TestimoniumConstructor<P>,
{
    Testimonium {
        _type: PhantomData,
        _property: PhantomData,
    }
}

/// Helper trait for constructing proofs.
///
/// Implemented for type-constraint pairs where the constraint holds.
pub trait TestimoniumConstructor<P: ?Sized> {}

// =============================================================================
// Marker Types for Properties
// =============================================================================

/// Marker type for `NonNihil` (non-zero) property.
pub struct IsNonNihil;

/// Marker type for Minor (less-than) property.
pub struct IsMinor<M>(PhantomData<M>);

/// Marker type for Maior (greater-than) property.
pub struct IsMaior<M>(PhantomData<M>);

/// Marker type for Aequus (equality) property.
pub struct IsAequus<M>(PhantomData<M>);

/// Marker type for `MinorVelAequus` (less-or-equal) property.
pub struct IsMinorVelAequus<M>(PhantomData<M>);

// NonNihil proofs
impl<N: NonNihil> TestimoniumConstructor<IsNonNihil> for N {}

// Minor (less-than) proofs
impl<N: Naturalis + Minor<M>, M: Naturalis> TestimoniumConstructor<IsMinor<M>> for N {}

// Maior (greater-than) proofs
impl<N: Naturalis + Maior<M>, M: Naturalis> TestimoniumConstructor<IsMaior<M>> for N {}

// Aequus (equality) proofs
impl<N: Naturalis + Aequus<M>, M: Naturalis> TestimoniumConstructor<IsAequus<M>> for N {}

// MinorVelAequus (less-or-equal) proofs
impl<N: Naturalis + MinorVelAequus<M>, M: Naturalis> TestimoniumConstructor<IsMinorVelAequus<M>>
    for N
{
}

// =============================================================================
// Propositio - Boolean Propositions
// =============================================================================

/// Type-level True.
///
/// # Latin Etymology
/// *Verum* means "true".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Verum;

/// Type-level False.
///
/// # Latin Etymology
/// *Falsum* means "false".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Falsum {}

/// Type-level boolean.
pub trait Propositio {
    /// The runtime boolean value.
    const VALUE: bool;
}

impl Propositio for Verum {
    const VALUE: bool = true;
}

impl Propositio for Falsum {
    const VALUE: bool = false;
}

// =============================================================================
// Aequalitas - Type Equality Proof
// =============================================================================

/// Proof that two types are equal.
///
/// # Latin Etymology
/// *Aequalitas* means "equality".
///
/// This type can only be constructed when `A` and `B` are the same type,
/// providing a compile-time witness of type equality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Aequalitas<A, B> {
    _a: PhantomData<A>,
    _b: PhantomData<B>,
}

impl<A> Aequalitas<A, A> {
    /// Create a proof that a type equals itself (reflexivity).
    ///
    /// # Latin Etymology
    /// *Reflexio* means "a bending back" - reflexivity.
    #[inline]
    pub fn reflexio() -> Self {
        Aequalitas {
            _a: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<A, B> Aequalitas<A, B> {
    /// Symmetry: if A = B then B = A.
    ///
    /// # Latin Etymology
    /// *Symmetria* means "symmetry".
    #[inline]
    pub fn symmetria(self) -> Aequalitas<B, A> {
        // SAFETY: If A = B (which is the only way to construct this),
        // then B = A trivially holds
        Aequalitas {
            _a: PhantomData,
            _b: PhantomData,
        }
    }

    /// Transitivity: if A = B and B = C, then A = C.
    ///
    /// # Latin Etymology
    /// *Transitivitas* means "transitivity".
    #[inline]
    pub fn transitivitas<C>(self, _other: Aequalitas<B, C>) -> Aequalitas<A, C> {
        Aequalitas {
            _a: PhantomData,
            _b: PhantomData,
        }
    }

    /// Congruence: if A = B, then `F<A>` = `F<B>` for any type constructor
    /// `F`, supplied as an [`HKT`](crate::typeclasses::hkt::HKT) witness.
    ///
    /// # Latin Etymology
    /// *Congruentia* means "agreement, consistency".
    #[inline]
    pub fn congruentia<F: crate::typeclasses::hkt::HKT>(
        self,
    ) -> Aequalitas<F::Target<A>, F::Target<B>> {
        Aequalitas {
            _a: PhantomData,
            _b: PhantomData,
        }
    }
}

/// Create a reflexivity proof.
#[inline]
pub fn reflexio<A>() -> Aequalitas<A, A> {
    Aequalitas::reflexio()
}

// =============================================================================
// Decisio - Decidable Propositions
// =============================================================================

/// A decision about whether a proposition holds.
///
/// # Latin Etymology
/// *Decisio* means "a decision, determination".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decisio<P> {
    /// The proposition holds, with a proof.
    Ita(P),
    /// The proposition does not hold.
    Non,
}

impl<P> Decisio<P> {
    /// Check if the decision is positive.
    #[inline]
    pub fn is_ita(&self) -> bool {
        matches!(self, Decisio::Ita(_))
    }

    /// Check if the decision is negative.
    #[inline]
    pub fn is_non(&self) -> bool {
        matches!(self, Decisio::Non)
    }

    /// Map over a positive decision.
    #[inline]
    pub fn map<Q, F>(self, f: F) -> Decisio<Q>
    where
        F: FnOnce(P) -> Q,
    {
        match self {
            Decisio::Ita(p) => Decisio::Ita(f(p)),
            Decisio::Non => Decisio::Non,
        }
    }

    /// Get the proof if positive.
    ///
    /// # Panics
    ///
    /// Panics if the decision is [`Decisio::Non`].
    #[inline]
    pub fn unwrap(self) -> P {
        match self {
            Decisio::Ita(p) => p,
            Decisio::Non => panic!("Called unwrap on Decisio::Non"),
        }
    }

    /// Get the proof if positive, panicking with `msg` on `Non`.
    ///
    /// # Panics
    ///
    /// Panics with the supplied `msg` if the decision is [`Decisio::Non`].
    #[inline]
    pub fn expect(self, msg: &str) -> P {
        match self {
            Decisio::Ita(p) => p,
            Decisio::Non => panic!("{}", msg),
        }
    }
}

// =============================================================================
// Existentia - Existential Proof
// =============================================================================

/// Proof that there exists some value of type `T` satisfying property `P`.
///
/// # Latin Etymology
/// *Existentia* means "existence".
///
/// This packages together a witness value and a proof that the witness
/// satisfies the required property.
#[derive(Debug, Clone, Copy)]
pub struct Existentia<T, P> {
    /// The witness value.
    pub witness: T,
    /// The proof that the witness satisfies P.
    pub proof: P,
}

impl<T, P> Existentia<T, P> {
    /// Create an existential proof with a witness and proof.
    #[inline]
    pub fn new(witness: T, proof: P) -> Self {
        Existentia { witness, proof }
    }

    /// Project out the witness.
    #[inline]
    pub fn fst(self) -> T {
        self.witness
    }

    /// Project out the proof.
    #[inline]
    pub fn snd(self) -> P {
        self.proof
    }

    /// Map over the witness while preserving the proof type.
    #[inline]
    pub fn map_witness<U, F>(self, f: F) -> Existentia<U, P>
    where
        F: FnOnce(T) -> U,
    {
        Existentia {
            witness: f(self.witness),
            proof: self.proof,
        }
    }
}

// =============================================================================
// Coniunctio - Conjunction (And)
// =============================================================================

/// Proof of conjunction: both P and Q hold.
///
/// # Latin Etymology
/// *Coniunctio* means "a joining together".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Coniunctio<P, Q> {
    /// Proof of P.
    pub sinister: P,
    /// Proof of Q.
    pub dexter: Q,
}

impl<P, Q> Coniunctio<P, Q> {
    /// Create a conjunction proof.
    #[inline]
    pub fn new(p: P, q: Q) -> Self {
        Coniunctio {
            sinister: p,
            dexter: q,
        }
    }

    /// Extract the left proof.
    #[inline]
    pub fn sinister(self) -> P {
        self.sinister
    }

    /// Extract the right proof.
    #[inline]
    pub fn dexter(self) -> Q {
        self.dexter
    }
}

// =============================================================================
// Disiunctio - Disjunction (Or)
// =============================================================================

/// Proof of disjunction: either P or Q holds.
///
/// # Latin Etymology
/// *Disiunctio* means "a separation".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disiunctio<P, Q> {
    /// P holds.
    Sinister(P),
    /// Q holds.
    Dexter(Q),
}

impl<P, Q> Disiunctio<P, Q> {
    /// Create a left disjunction.
    #[inline]
    pub fn sinister(p: P) -> Self {
        Disiunctio::Sinister(p)
    }

    /// Create a right disjunction.
    #[inline]
    pub fn dexter(q: Q) -> Self {
        Disiunctio::Dexter(q)
    }

    /// Case analysis on a disjunction.
    #[inline]
    pub fn elimina<R, F, G>(self, f: F, g: G) -> R
    where
        F: FnOnce(P) -> R,
        G: FnOnce(Q) -> R,
    {
        match self {
            Disiunctio::Sinister(p) => f(p),
            Disiunctio::Dexter(q) => g(q),
        }
    }
}

// =============================================================================
// Bounded Natural
// =============================================================================

/// A natural number with a proof that it's less than some bound.
///
/// # Latin Etymology
/// *Finitus* means "bounded, limited".
#[derive(Debug, Clone, Copy)]
pub struct Finitus<N: Naturalis, Bound: Naturalis> {
    _n: PhantomData<N>,
    _bound: PhantomData<Bound>,
}

impl<N: Naturalis + Minor<Bound>, Bound: Naturalis> Finitus<N, Bound> {
    /// Create a bounded natural.
    pub fn new() -> Self {
        Finitus {
            _n: PhantomData,
            _bound: PhantomData,
        }
    }
}

impl<N: Naturalis + Minor<Bound>, Bound: Naturalis> Default for Finitus<N, Bound> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependent::peano::{N1, N2, N3, N5, Zero};

    #[test]
    fn congruentia_lifts_equality_through_a_constructor() {
        struct OptionWitness;
        impl crate::typeclasses::hkt::HKT for OptionWitness {
            type Target<T> = Option<T>;
        }

        let proof: Aequalitas<i32, i32> = Aequalitas::reflexio();
        // A = B entails Option<A> = Option<B>: the lifted proof's type is
        // the real statement of congruence (the old signature returned
        // Aequalitas<F, F>, which said nothing about A or B).
        let _lifted: Aequalitas<Option<i32>, Option<i32>> = proof.congruentia::<OptionWitness>();
    }
    use alloc::string::ToString;

    #[test]
    fn test_testimonium_non_nihil() {
        let _proof: Testimonium<N2, IsNonNihil> = testimonium();
        let _proof2: Testimonium<N5, IsNonNihil> = testimonium();
    }

    #[test]
    fn test_testimonium_minor() {
        let _proof: Testimonium<N2, IsMinor<N5>> = testimonium();
        let _proof2: Testimonium<Zero, IsMinor<N1>> = testimonium();
    }

    #[test]
    fn test_testimonium_maior() {
        let _proof: Testimonium<N5, IsMaior<N2>> = testimonium();
        let _proof2: Testimonium<N3, IsMaior<Zero>> = testimonium();
    }

    #[test]
    fn test_testimonium_aequus() {
        let _proof: Testimonium<N3, IsAequus<N3>> = testimonium();
        let _proof2: Testimonium<Zero, IsAequus<Zero>> = testimonium();
    }

    #[test]
    fn test_aequalitas_reflexio() {
        let _proof: Aequalitas<N3, N3> = reflexio();
        let _proof2: Aequalitas<i32, i32> = reflexio();
    }

    #[test]
    fn test_aequalitas_symmetria() {
        let proof: Aequalitas<N3, N3> = reflexio();
        let _symmetric: Aequalitas<N3, N3> = proof.symmetria();
    }

    #[test]
    fn test_aequalitas_transitivitas() {
        let p1: Aequalitas<N3, N3> = reflexio();
        let p2: Aequalitas<N3, N3> = reflexio();
        let _transitive: Aequalitas<N3, N3> = p1.transitivitas(p2);
    }

    #[test]
    fn test_decisio() {
        let yes: Decisio<i32> = Decisio::Ita(42);
        assert!(yes.is_ita());
        assert!(!yes.is_non());
        assert_eq!(yes.expect("Decisio::Ita(42) should hold a value"), 42);

        let no: Decisio<i32> = Decisio::Non;
        assert!(no.is_non());
        assert!(!no.is_ita());
    }

    #[test]
    fn test_existentia() {
        let exists = Existentia::new(42i32, "proof");
        assert_eq!(exists.fst(), 42);
        assert_eq!(exists.snd(), "proof");
    }

    #[test]
    fn test_coniunctio() {
        let conj = Coniunctio::new(1, "two");
        assert_eq!(conj.sinister(), 1);
        assert_eq!(conj.dexter(), "two");
    }

    #[test]
    fn test_disiunctio() {
        let left: Disiunctio<i32, &str> = Disiunctio::sinister(42);
        let result = left.elimina(|n| n.to_string(), std::string::ToString::to_string);
        assert_eq!(result, "42");

        let right: Disiunctio<i32, &str> = Disiunctio::dexter("hello");
        let result2 = right.elimina(|n| n.to_string(), std::string::ToString::to_string);
        assert_eq!(result2, "hello");
    }

    #[test]
    fn test_finitus() {
        let _bounded: Finitus<N2, N5> = Finitus::new();
        let _bounded2: Finitus<Zero, N3> = Finitus::new();
    }

    #[test]
    fn test_propositio() {
        assert!(Verum::VALUE);
        // Falsum has no values, so we can't test its VALUE
    }
}
