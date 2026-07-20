//! Comonad type class - dual of Monad.
//!
//! > *\"Omnis effectus sufficientem rationem habet a qua dependet.\"*
//! > — Every effect has a sufficient reason on which it depends. (Leibniz)
//!
//! While `Monad` wraps values in context and chains computations,
//! `Comonad` extracts values and extends computations.
//!
//! # Scholastic Names
//!
//! - `Identitas` (Identity) - the simple identity comonad
//! - `Thesaurus` (Store) - a storehouse of values indexed by position
//! - `Contextus` (Env) - value with read-only context/environment
//! - `Vestigium` (Traced) - value with write-only trace
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::comonad::{Comonad, Identitas};
//!
//! // Identitas is the simplest comonad: extract unwraps the value,
//! // extend applies a whole-context function and rewraps the result.
//! let w = Identitas(21);
//! assert_eq!(w.extract(), 21);
//!
//! let doubled = w.extend(|w| w.extract() * 2);
//! assert_eq!(doubled.extract(), 42);
//! ```
//!
//! # Laws
//!
//! 1. **Left identity**: `w.extend(extract) == w`
//! 2. **Right identity**: `w.extend(f).extract() == f(w)`
//! 3. **Associativity**: `w.extend(f).extend(g) == w.extend(|w| g(w.extend(f)))`

/// Comonad - the categorical dual of Monad.
///
/// While Monad has:
/// - `pure: A -> M<A>` (wrap a value)
/// - `flat_map: M<A> -> (A -> M<B>) -> M<B>` (chain computations)
///
/// Comonad has:
/// - `extract: W<A> -> A` (unwrap a value)
/// - `extend: W<A> -> (W<A> -> B) -> W<B>` (extend computations)
///
/// # Intuition
///
/// A comonad represents a value in a context. `extract` gets the focused value,
/// while `extend` applies a function to all possible "focus points".
pub trait Comonad: Sized {
    /// The type inside the comonad.
    type Item;

    /// Extract the focused value from the comonad.
    ///
    /// This is the dual of `pure` - instead of wrapping a value,
    /// it unwraps/extracts the focused value.
    fn extract(&self) -> Self::Item;

    /// Extend a function over the comonad.
    ///
    /// Given a function that takes the whole comonad and produces a value,
    /// apply it to all possible focus points.
    ///
    /// This is the dual of `flat_map`.
    fn extend<F, B>(&self, f: F) -> Self::Output<B>
    where
        F: Fn(&Self) -> B;

    /// Output type constructor for extend.
    type Output<B>;

    /// Duplicate the comonad.
    ///
    /// Creates a comonad of comonads where each position contains
    /// the original comonad focused at that position.
    ///
    /// `duplicate` can be defined in terms of `extend`:
    /// `w.duplicate() == w.extend(|x| x.clone())`
    #[inline]
    fn duplicate(&self) -> Self::Output<Self>
    where
        Self: Clone,
    {
        self.extend(core::clone::Clone::clone)
    }

    /// Map a function over the comonad.
    ///
    /// Every comonad is also a functor:
    /// `w.map(f) == w.extend(|w| f(w.extract()))`
    #[inline]
    fn cmap<F, B>(&self, f: F) -> Self::Output<B>
    where
        F: Fn(Self::Item) -> B,
    {
        self.extend(move |w| f(w.extract()))
    }
}

/// Identitas: A simple identity comonad wrapper.
///
/// > *\"Idem est idem sibi.\"*
/// > — The same is the same to itself. (Scholastic axiom)
///
/// The simplest comonad - just a wrapper around a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identitas<A>(pub A);

impl<A> Identitas<A> {
    /// Create a new Identitas.
    #[inline]
    pub fn new(value: A) -> Self {
        Identitas(value)
    }

    /// Get the inner value.
    #[inline]
    pub fn value(&self) -> &A {
        &self.0
    }

    /// Into the inner value.
    #[inline]
    pub fn into_value(self) -> A {
        self.0
    }
}

impl<A: Clone> Comonad for Identitas<A> {
    type Item = A;
    type Output<B> = Identitas<B>;

    #[inline]
    fn extract(&self) -> A {
        self.0.clone()
    }

    #[inline]
    fn extend<F, B>(&self, f: F) -> Identitas<B>
    where
        F: Fn(&Self) -> B,
    {
        Identitas(f(self))
    }
}

/// Thesaurus: Store comonad - represents a value that depends on a position.
///
/// > *\"Thesaurus est locus in quo divitiae reponuntur.\"*
/// > — A storehouse is a place where riches are deposited.
///
/// This is a more interesting comonad that represents a value at a
/// position, where you can peek at other positions.
#[derive(Clone)]
pub struct Thesaurus<S, A, F>
where
    F: Fn(&S) -> A,
{
    /// The current position.
    pub position: S,
    /// Function to get value at any position.
    pub peek_fn: F,
}

impl<S: Clone, A, F> Thesaurus<S, A, F>
where
    F: Fn(&S) -> A,
{
    /// Create a new Thesaurus.
    #[inline]
    pub fn new(position: S, peek_fn: F) -> Self {
        Thesaurus { position, peek_fn }
    }

    /// Peek at a specific position.
    #[inline]
    pub fn peek(&self, pos: &S) -> A {
        (self.peek_fn)(pos)
    }

    /// Get the current position.
    #[inline]
    pub fn pos(&self) -> &S {
        &self.position
    }

    /// Move to a new position.
    #[inline]
    pub fn seek(self, new_pos: S) -> Self {
        Thesaurus {
            position: new_pos,
            peek_fn: self.peek_fn,
        }
    }

    /// Modify the position.
    #[inline]
    pub fn seeks<G>(self, f: G) -> Self
    where
        G: FnOnce(S) -> S,
    {
        Thesaurus {
            position: f(self.position),
            peek_fn: self.peek_fn,
        }
    }
}

/// Contextus: Env comonad - value with read-only environment.
///
/// > *\"Contextus dat intellectum verbis.\"*
/// > — Context gives understanding to words.
///
/// This is a simple comonad where each value has an associated environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contextus<E, A> {
    /// The environment.
    pub env: E,
    /// The value.
    pub value: A,
}

impl<E, A> Contextus<E, A> {
    /// Create a new Contextus.
    #[inline]
    pub fn new(env: E, value: A) -> Self {
        Contextus { env, value }
    }

    /// Get the environment.
    #[inline]
    pub fn ask(&self) -> &E {
        &self.env
    }

    /// Get the value.
    #[inline]
    pub fn val(&self) -> &A {
        &self.value
    }
}

impl<E: Clone, A: Clone> Comonad for Contextus<E, A> {
    type Item = A;
    type Output<B> = Contextus<E, B>;

    #[inline]
    fn extract(&self) -> A {
        self.value.clone()
    }

    #[inline]
    fn extend<F, B>(&self, f: F) -> Contextus<E, B>
    where
        F: Fn(&Self) -> B,
    {
        Contextus {
            env: self.env.clone(),
            value: f(self),
        }
    }
}

/// Vestigium: Traced comonad - value with write-only trace.
///
/// > *\"Vestigium est signum rei praeteritae.\"*
/// > — A trace is a sign of something past.
///
/// This is dual to Reader - instead of reading from environment,
/// we accumulate a trace.
#[derive(Clone)]
pub struct Vestigium<M, A, F>
where
    F: Fn(&M) -> A,
{
    /// Function from trace to value.
    pub run: F,
    _marker: core::marker::PhantomData<M>,
}

impl<M, A, F> Vestigium<M, A, F>
where
    F: Fn(&M) -> A,
{
    /// Create a new Vestigium.
    #[inline]
    pub fn new(run: F) -> Self {
        Vestigium {
            run,
            _marker: core::marker::PhantomData,
        }
    }

    /// Run with a trace.
    #[inline]
    pub fn run_traced(&self, trace: &M) -> A {
        (self.run)(trace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::string::{String, ToString};

    #[test]
    fn test_identitas_extract() {
        let id = Identitas::new(42);
        assert_eq!(id.extract(), 42);
    }

    #[test]
    fn test_identitas_extend() {
        let id = Identitas::new(10);
        let result = id.extend(|w| w.extract() * 2);
        assert_eq!(result.extract(), 20);
    }

    #[test]
    fn test_identitas_duplicate() {
        let id = Identitas::new(42);
        let dup = id.duplicate();
        assert_eq!(dup.extract().extract(), 42);
    }

    #[test]
    fn test_identitas_cmap() {
        let id = Identitas::new(5);
        let result = id.cmap(|x| x * 3);
        assert_eq!(result.extract(), 15);
    }

    #[test]
    fn test_identitas_left_identity_law() {
        // w.extend(extract) == w
        let w = Identitas::new(42);
        let result = w.extend(super::Comonad::extract);
        assert_eq!(result.extract(), w.extract());
    }

    #[test]
    fn test_identitas_right_identity_law() {
        // w.extend(f).extract() == f(w)
        let w = Identitas::new(10);
        let f = |x: &Identitas<i32>| x.extract() * 2;
        assert_eq!(w.extend(f).extract(), f(&w));
    }

    #[test]
    fn test_contextus_extract() {
        let e = Contextus::new("config", 42);
        assert_eq!(e.extract(), 42);
    }

    #[test]
    fn test_contextus_ask() {
        let e = Contextus::new("config", 42);
        assert_eq!(e.ask(), &"config");
    }

    #[test]
    fn test_contextus_extend() {
        let e = Contextus::new(10, 5);
        // Use the environment in the computation
        let result = e.extend(|w| w.extract() + *w.ask());
        assert_eq!(result.extract(), 15);
        assert_eq!(result.ask(), &10); // Environment preserved
    }

    #[test]
    fn test_contextus_left_identity_law() {
        let w = Contextus::new("env", 42);
        let result = w.extend(super::Comonad::extract);
        assert_eq!(result.extract(), w.extract());
    }

    #[test]
    fn test_thesaurus_peek() {
        let store = Thesaurus::new(0, |i: &i32| i * i);
        assert_eq!(store.peek(&0), 0);
        assert_eq!(store.peek(&5), 25);
        assert_eq!(store.peek(&-3), 9);
    }

    #[test]
    fn test_thesaurus_seek() {
        let store = Thesaurus::new(0, |i: &i32| i * 2);
        let moved = store.seek(5);
        assert_eq!(moved.pos(), &5);
        assert_eq!(moved.peek(&5), 10);
    }

    #[test]
    fn test_thesaurus_seeks() {
        let store = Thesaurus::new(5, |i: &i32| i * 2);
        let modified = store.seeks(|x| x + 3);
        assert_eq!(modified.pos(), &8);
    }

    #[test]
    fn test_vestigium_run() {
        let traced = Vestigium::new(|s: &String| s.len());
        assert_eq!(traced.run_traced(&"hello".to_string()), 5);
        assert_eq!(traced.run_traced(&"hi".to_string()), 2);
    }
}
