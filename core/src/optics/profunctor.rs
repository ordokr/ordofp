//! Profunctor Optics - `OpticumProfunctor`
//!
//! > *"Per profundum ad veritatem"*
//! > — Through depth to truth. (Latin)
//!
//! This module provides profunctor-based optics, offering a more principled
//! and composable approach to optics based on the profunctor abstraction.
//!
//! # Overview
//!
//! Profunctor optics represent optics as transformations on profunctors,
//! enabling elegant composition and a unified representation of different
//! optic types.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Profunctor | Profunctor | *pro* = before + *functor* = performer |
//! | Strong | Fortis | *fortis* = strong |
//! | Choice | Electio | *electio* = choice |
//! | Optic | Opticum | *opticus* = of sight |

use core::marker::PhantomData;

/// A profunctor is a bifunctor that is contravariant in the first argument
/// and covariant in the second.
///
/// > *"Profunctor est bifunctor inversus in primo, directus in secundo."*
///
/// # Type Parameters
/// - `A` - The contravariant input type
/// - `B` - The covariant output type
pub trait Profunctor<A, B>: Sized {
    /// The result type after applying dimap
    type Mapped<C, D>: Profunctor<C, D>;

    /// Map over both type parameters.
    ///
    /// `dimap f g` is equivalent to `lmap f >>> rmap g`.
    fn dimap<C, D, F, G>(self, f: F, g: G) -> Self::Mapped<C, D>
    where
        F: Fn(C) -> A + 'static,
        G: Fn(B) -> D + 'static;

    /// Map over the left (contravariant) type parameter.
    #[inline]
    fn lmap<C, F>(self, f: F) -> Self::Mapped<C, B>
    where
        F: Fn(C) -> A + 'static,
    {
        self.dimap(f, |b| b)
    }

    /// Map over the right (covariant) type parameter.
    #[inline]
    fn rmap<D, G>(self, g: G) -> Self::Mapped<A, D>
    where
        G: Fn(B) -> D + 'static,
    {
        self.dimap(|a| a, g)
    }
}

/// A strong profunctor can pass through a product type.
///
/// > *"Fortis est qui per copulam transit."*
/// > — Strong is that which passes through a pair.
///
/// This is the characteristic of lenses.
pub trait Fortis<A, B>: Profunctor<A, B> {
    /// The result type after applying first'
    type Primus<C>: Fortis<(A, C), (B, C)>;

    /// Pass the first component through the profunctor.
    fn primus<C>(self) -> Self::Primus<C>;

    /// The result type after applying second'
    type Secundus<C>: Fortis<(C, A), (C, B)>;

    /// Pass the second component through the profunctor.
    fn secundus<C>(self) -> Self::Secundus<C>;
}

/// A choice profunctor can pass through a sum type.
///
/// > *"Electio est qui per disjunctionem transit."*
/// > — Choice is that which passes through a disjunction.
///
/// This is the characteristic of prisms.
pub trait Electio<A, B>: Profunctor<A, B> {
    /// The result type after applying left'
    type Sinister<C>: Electio<Result<A, C>, Result<B, C>>;

    /// Pass the left branch through the profunctor.
    fn sinister<C>(self) -> Self::Sinister<C>;

    /// The result type after applying right'
    type Dexter<C>: Electio<Result<C, A>, Result<C, B>>;

    /// Pass the right branch through the profunctor.
    fn dexter<C>(self) -> Self::Dexter<C>;
}

// =============================================================================
// Function Profunctor
// =============================================================================

/// A simple function profunctor.
///
/// Functions form a profunctor: `Fn(A) -> B`.
pub struct FunctioProf<A, B, F>
where
    F: Fn(A) -> B,
{
    f: F,
    _phantom: PhantomData<fn(A) -> B>,
}

impl<A, B, F> FunctioProf<A, B, F>
where
    F: Fn(A) -> B,
{
    /// Create a new function profunctor.
    #[inline]
    pub fn new(f: F) -> Self {
        FunctioProf {
            f,
            _phantom: PhantomData,
        }
    }

    /// Apply the function.
    #[inline]
    pub fn apply(&self, a: A) -> B {
        (self.f)(a)
    }
}

// =============================================================================
// Optic Types via Profunctors
// =============================================================================

/// A profunctor optic is a polymorphic function over profunctors.
///
/// > *"Opticum profunctoris est transformatio universalis."*
///
/// An optic `Optic<S, T, A, B>` transforms a `P<A, B>` into a `P<S, T>`
/// for any profunctor P satisfying certain constraints.
///
/// This encoding allows all optics to compose uniformly.
pub trait OpticumProfunctor<S, T, A, B> {
    /// Apply the optic to transform a profunctor.
    fn run<P>(&self, pab: P) -> P::Mapped<S, T>
    where
        P: Profunctor<A, B>;
}

/// An iso (isomorphism) as a profunctor optic.
///
/// An iso requires only the basic Profunctor constraint.
pub struct AequivalentiaProfunctor<S, T, A, B, Fwd, Bwd>
where
    Fwd: Fn(S) -> A,
    Bwd: Fn(B) -> T,
{
    forward: Fwd,
    backward: Bwd,
    _phantom: PhantomData<fn(S, T, A, B)>,
}

impl<S, T, A, B, Fwd, Bwd> AequivalentiaProfunctor<S, T, A, B, Fwd, Bwd>
where
    Fwd: Fn(S) -> A,
    Bwd: Fn(B) -> T,
{
    /// Create a new iso profunctor optic.
    #[inline]
    pub fn new(forward: Fwd, backward: Bwd) -> Self {
        AequivalentiaProfunctor {
            forward,
            backward,
            _phantom: PhantomData,
        }
    }

    /// Get the forward function.
    #[inline]
    pub fn forward(&self) -> &Fwd {
        &self.forward
    }

    /// Get the backward function.
    #[inline]
    pub fn backward(&self) -> &Bwd {
        &self.backward
    }
}

impl<S, T, A, B, Fwd, Bwd> Clone for AequivalentiaProfunctor<S, T, A, B, Fwd, Bwd>
where
    Fwd: Fn(S) -> A + Clone,
    Bwd: Fn(B) -> T + Clone,
{
    fn clone(&self) -> Self {
        AequivalentiaProfunctor {
            forward: self.forward.clone(),
            backward: self.backward.clone(),
            _phantom: PhantomData,
        }
    }
}

/// A lens as a profunctor optic.
///
/// A lens requires the Strong (Fortis) profunctor constraint.
pub struct AspectusProfunctor<S, T, A, B, Get, Set>
where
    Get: Fn(&S) -> A,
    Set: Fn(S, B) -> T,
{
    get: Get,
    set: Set,
    _phantom: PhantomData<fn(S, T, A, B)>,
}

impl<S, T, A, B, Get, Set> AspectusProfunctor<S, T, A, B, Get, Set>
where
    Get: Fn(&S) -> A,
    Set: Fn(S, B) -> T,
{
    /// Create a new lens profunctor optic.
    #[inline]
    pub fn new(get: Get, set: Set) -> Self {
        AspectusProfunctor {
            get,
            set,
            _phantom: PhantomData,
        }
    }

    /// Get the focused value.
    #[inline]
    pub fn view(&self, s: &S) -> A {
        (self.get)(s)
    }

    /// Set a new value.
    #[inline]
    pub fn set(&self, s: S, b: B) -> T {
        (self.set)(s, b)
    }

    /// Modify the focused value.
    #[inline]
    pub fn over<F>(&self, s: S, f: F) -> T
    where
        F: FnOnce(A) -> B,
    {
        let a = (self.get)(&s);
        (self.set)(s, f(a))
    }
}

impl<S, T, A, B, Get, Set> Clone for AspectusProfunctor<S, T, A, B, Get, Set>
where
    Get: Fn(&S) -> A + Clone,
    Set: Fn(S, B) -> T + Clone,
{
    fn clone(&self) -> Self {
        AspectusProfunctor {
            get: self.get.clone(),
            set: self.set.clone(),
            _phantom: PhantomData,
        }
    }
}

/// A prism as a profunctor optic.
///
/// A prism requires the Choice (Electio) profunctor constraint.
pub struct DivisioProfunctor<S, T, A, B, Match, Build>
where
    Match: Fn(S) -> Result<A, T>,
    Build: Fn(B) -> T,
{
    matching: Match,
    build: Build,
    _phantom: PhantomData<fn(S, T, A, B)>,
}

impl<S, T, A, B, Match, Build> DivisioProfunctor<S, T, A, B, Match, Build>
where
    Match: Fn(S) -> Result<A, T>,
    Build: Fn(B) -> T,
{
    /// Create a new prism profunctor optic.
    ///
    /// - `matching`: Returns `Ok(a)` if the prism matches, `Err(t)` otherwise
    /// - `build`: Constructs the target from the focus
    #[inline]
    pub fn new(matching: Match, build: Build) -> Self {
        DivisioProfunctor {
            matching,
            build,
            _phantom: PhantomData,
        }
    }

    /// Try to extract the focused value.
    #[inline]
    pub fn preview(&self, s: S) -> Option<A> {
        (self.matching)(s).ok()
    }

    /// Construct the target from a value.
    #[inline]
    pub fn review(&self, b: B) -> T {
        (self.build)(b)
    }

    /// Modify if the prism matches.
    #[inline]
    pub fn over<F>(&self, s: S, f: F) -> T
    where
        F: FnOnce(A) -> B,
    {
        match (self.matching)(s) {
            Ok(a) => (self.build)(f(a)),
            Err(t) => t,
        }
    }
}

impl<S, T, A, B, Match, Build> Clone for DivisioProfunctor<S, T, A, B, Match, Build>
where
    Match: Fn(S) -> Result<A, T> + Clone,
    Build: Fn(B) -> T + Clone,
{
    fn clone(&self) -> Self {
        DivisioProfunctor {
            matching: self.matching.clone(),
            build: self.build.clone(),
            _phantom: PhantomData,
        }
    }
}

// =============================================================================
// Simple (Monomorphic) Optic Type Aliases
// =============================================================================

/// A simple iso where S = T and A = B.
pub type AequivalentiaSimplexProf<S, A, Fwd, Bwd> = AequivalentiaProfunctor<S, S, A, A, Fwd, Bwd>;

/// A simple lens where S = T and A = B.
pub type AspectusSimplexProf<S, A, Get, Set> = AspectusProfunctor<S, S, A, A, Get, Set>;

/// A simple prism where S = T and A = B.
pub type DivisioSimplexProf<S, A, Match, Build> = DivisioProfunctor<S, S, A, A, Match, Build>;

// =============================================================================
// Composition
// =============================================================================

/// Variance marker tying a [`ComposedOptic`] to its six phantom type
/// parameters (`fn`-pointer encoding keeps the optic `Send`/`Sync` and
/// contravariant-safe without storing any data).
type ComposedOpticMarker<S, T, M, N, A, B> = PhantomData<fn(S, T, M, N, A, B)>;

/// Composed profunctor optic.
pub struct ComposedOptic<O1, O2, S, T, M, N, A, B>
where
    O1: Clone,
    O2: Clone,
{
    outer: O1,
    inner: O2,
    _phantom: ComposedOpticMarker<S, T, M, N, A, B>,
}

impl<O1, O2, S, T, M, N, A, B> ComposedOptic<O1, O2, S, T, M, N, A, B>
where
    O1: Clone,
    O2: Clone,
{
    /// Compose two optics.
    #[inline]
    pub fn new(outer: O1, inner: O2) -> Self {
        ComposedOptic {
            outer,
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<O1, O2, S, T, M, N, A, B> Clone for ComposedOptic<O1, O2, S, T, M, N, A, B>
where
    O1: Clone,
    O2: Clone,
{
    fn clone(&self) -> Self {
        ComposedOptic {
            outer: self.outer.clone(),
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a simple (monomorphic) lens profunctor optic.
#[inline]
pub fn aspectus_profunctor<S, A, Get, Set>(
    get: Get,
    set: Set,
) -> AspectusSimplexProf<S, A, Get, Set>
where
    Get: Fn(&S) -> A,
    Set: Fn(S, A) -> S,
{
    AspectusProfunctor::new(get, set)
}

/// Create a polymorphic lens profunctor optic.
#[inline]
pub fn aspectus_profunctor_poly<S, T, A, B, Get, Set>(
    get: Get,
    set: Set,
) -> AspectusProfunctor<S, T, A, B, Get, Set>
where
    Get: Fn(&S) -> A,
    Set: Fn(S, B) -> T,
{
    AspectusProfunctor::new(get, set)
}

/// Create a simple (monomorphic) prism profunctor optic.
#[inline]
pub fn divisio_profunctor<S, A, Match, Build>(
    matching: Match,
    build: Build,
) -> DivisioSimplexProf<S, A, Match, Build>
where
    Match: Fn(S) -> Result<A, S>,
    Build: Fn(A) -> S,
{
    DivisioProfunctor::new(matching, build)
}

/// Create a polymorphic prism profunctor optic.
#[inline]
pub fn divisio_profunctor_poly<S, T, A, B, Match, Build>(
    matching: Match,
    build: Build,
) -> DivisioProfunctor<S, T, A, B, Match, Build>
where
    Match: Fn(S) -> Result<A, T>,
    Build: Fn(B) -> T,
{
    DivisioProfunctor::new(matching, build)
}

/// Create a simple (monomorphic) iso profunctor optic.
#[inline]
pub fn aequivalentia_profunctor<S, A, Fwd, Bwd>(
    forward: Fwd,
    backward: Bwd,
) -> AequivalentiaSimplexProf<S, A, Fwd, Bwd>
where
    Fwd: Fn(S) -> A,
    Bwd: Fn(A) -> S,
{
    AequivalentiaProfunctor::new(forward, backward)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::string::{String, ToString};

    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Shape {
        Circle(f64),
        Rectangle(f64, f64),
    }

    #[test]
    fn test_aspectus_profunctor_view() {
        let name_lens = aspectus_profunctor(
            |p: &Person| p.name.clone(),
            |p: Person, name| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        assert_eq!(name_lens.view(&person), "Alice");
    }

    #[test]
    fn test_aspectus_profunctor_set() {
        let name_lens = aspectus_profunctor(
            |p: &Person| p.name.clone(),
            |p: Person, name| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let updated = name_lens.set(person, "Bob".to_string());
        assert_eq!(updated.name, "Bob");
        assert_eq!(updated.age, 30);
    }

    #[test]
    fn test_aspectus_profunctor_over() {
        let name_lens = aspectus_profunctor(
            |p: &Person| p.name.clone(),
            |p: Person, name| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let modified = name_lens.over(person, |n| n.to_uppercase());
        assert_eq!(modified.name, "ALICE");
    }

    #[test]
    fn test_divisio_profunctor_preview() {
        let circle_prism = divisio_profunctor(
            |s: Shape| match s {
                Shape::Circle(r) => Ok(r),
                other => Err(other),
            },
            Shape::Circle,
        );

        let circle = Shape::Circle(5.0);
        let rect = Shape::Rectangle(3.0, 4.0);

        assert_eq!(circle_prism.preview(circle), Some(5.0));
        assert_eq!(circle_prism.preview(rect), None);
    }

    #[test]
    fn test_divisio_profunctor_review() {
        let circle_prism = divisio_profunctor(
            |s: Shape| match s {
                Shape::Circle(r) => Ok(r),
                other => Err(other),
            },
            Shape::Circle,
        );

        assert_eq!(circle_prism.review(10.0), Shape::Circle(10.0));
    }

    #[test]
    fn test_divisio_profunctor_over() {
        let circle_prism = divisio_profunctor(
            |s: Shape| match s {
                Shape::Circle(r) => Ok(r),
                other => Err(other),
            },
            Shape::Circle,
        );

        let circle = Shape::Circle(5.0);
        let rect = Shape::Rectangle(3.0, 4.0);

        let doubled = circle_prism.over(circle, |r| r * 2.0);
        assert_eq!(doubled, Shape::Circle(10.0));

        let unchanged = circle_prism.over(rect.clone(), |r| r * 2.0);
        assert_eq!(unchanged, rect);
    }

    #[test]
    fn test_aequivalentia_profunctor() {
        let swap_iso = aequivalentia_profunctor(
            |(a, b): (i32, String)| (b, a),
            |(b, a): (String, i32)| (a, b),
        );

        let original = (42, "hello".to_string());
        let swapped = (swap_iso.forward())(original.clone());
        assert_eq!(swapped, ("hello".to_string(), 42));

        let back = (swap_iso.backward())(swapped);
        assert_eq!(back, original);
    }

    #[test]
    fn test_polymorphic_lens() {
        // A lens that can change the type: Person -> PersonWithNickname
        #[derive(Clone, Debug, PartialEq)]
        struct PersonWithNickname {
            name: String,
            nickname: String,
            age: u32,
        }

        let poly_lens = aspectus_profunctor_poly(
            |p: &Person| p.name.clone(),
            |p: Person, nickname: String| PersonWithNickname {
                name: p.name,
                nickname,
                age: p.age,
            },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let with_nickname = poly_lens.set(person, "Ali".to_string());
        assert_eq!(with_nickname.name, "Alice");
        assert_eq!(with_nickname.nickname, "Ali");
        assert_eq!(with_nickname.age, 30);
    }
}
