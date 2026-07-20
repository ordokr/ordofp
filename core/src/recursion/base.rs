//! Base functor implementations for common types.
//!
//! > *"Fundamentum est id super quod aliquid aedificatur."*
//! > — The foundation is that upon which something is built.

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::typeclasses::hkt::{CloneHKT, FunctorHKT, HKT};

// =============================================================================
// Natural Numbers Base Functor
// =============================================================================

/// Base functor for natural numbers.
///
/// ```text
/// data Nat = Zero | Succ Nat
/// data NatF r = ZeroF | SuccF r
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatF<R> {
    /// Zero - the base case.
    ZeroF,
    /// Successor - wraps the recursive position.
    SuccF(R),
}

/// Witness type for `NatF`.
pub struct NatFWitness;

impl HKT for NatFWitness {
    type Target<T> = NatF<T>;
}

impl FunctorHKT for NatFWitness {
    #[inline]
    fn map<A, B, F>(fa: NatF<A>, mut f: F) -> NatF<B>
    where
        F: FnMut(A) -> B,
    {
        match fa {
            NatF::ZeroF => NatF::ZeroF,
            NatF::SuccF(a) => NatF::SuccF(f(a)),
        }
    }
}

impl CloneHKT for NatFWitness {
    #[inline]
    fn clone_hkt<T: Clone>(t: &NatF<T>) -> NatF<T> {
        match t {
            NatF::ZeroF => NatF::ZeroF,
            NatF::SuccF(a) => NatF::SuccF(a.clone()),
        }
    }
}

impl<R> NatF<R> {
    /// Map a function over the recursive position.
    #[inline]
    pub fn map<B, F>(self, f: F) -> NatF<B>
    where
        F: FnOnce(R) -> B,
    {
        match self {
            NatF::ZeroF => NatF::ZeroF,
            NatF::SuccF(r) => NatF::SuccF(f(r)),
        }
    }
}

// =============================================================================
// List Base Functor
// =============================================================================

/// Base functor for lists.
///
/// ```text
/// data List a = Nil | Cons a (List a)
/// data ListF a r = NilF | ConsF a r
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListF<A, R> {
    /// Nil - empty list.
    NilF,
    /// Cons - element and recursive tail.
    ConsF(A, R),
}

/// Witness type for `ListF`.
pub struct ListFWitness<A>(core::marker::PhantomData<A>);

impl<A> HKT for ListFWitness<A> {
    type Target<T> = ListF<A, T>;
}

impl<A: Clone> FunctorHKT for ListFWitness<A> {
    #[inline]
    fn map<R1, R2, F>(fa: ListF<A, R1>, mut f: F) -> ListF<A, R2>
    where
        F: FnMut(R1) -> R2,
    {
        match fa {
            ListF::NilF => ListF::NilF,
            ListF::ConsF(a, r) => ListF::ConsF(a, f(r)),
        }
    }
}

impl<A: Clone> CloneHKT for ListFWitness<A> {
    #[inline]
    fn clone_hkt<T: Clone>(t: &ListF<A, T>) -> ListF<A, T> {
        match t {
            ListF::NilF => ListF::NilF,
            ListF::ConsF(a, r) => ListF::ConsF(a.clone(), r.clone()),
        }
    }
}

impl<A, R> ListF<A, R> {
    /// Map a function over the recursive position.
    #[inline]
    pub fn map_rec<B, F>(self, f: F) -> ListF<A, B>
    where
        F: FnOnce(R) -> B,
    {
        match self {
            ListF::NilF => ListF::NilF,
            ListF::ConsF(a, r) => ListF::ConsF(a, f(r)),
        }
    }

    /// Map a function over the element.
    #[inline]
    pub fn map_elem<B, F>(self, f: F) -> ListF<B, R>
    where
        F: FnOnce(A) -> B,
    {
        match self {
            ListF::NilF => ListF::NilF,
            ListF::ConsF(a, r) => ListF::ConsF(f(a), r),
        }
    }
}

// =============================================================================
// Binary Tree Base Functor
// =============================================================================

/// Base functor for binary trees.
///
/// ```text
/// data Tree a = Empty | Node a (Tree a) (Tree a)
/// data TreeF a r = EmptyF | NodeF a r r
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeF<A, R> {
    /// Empty tree.
    EmptyF,
    /// Node with value and two recursive children.
    NodeF(A, R, R),
}

/// Witness type for `TreeF`.
pub struct TreeFWitness<A>(core::marker::PhantomData<A>);

impl<A> HKT for TreeFWitness<A> {
    type Target<T> = TreeF<A, T>;
}

impl<A: Clone> FunctorHKT for TreeFWitness<A> {
    #[inline]
    fn map<R1, R2, F>(fa: TreeF<A, R1>, mut f: F) -> TreeF<A, R2>
    where
        F: FnMut(R1) -> R2,
    {
        match fa {
            TreeF::EmptyF => TreeF::EmptyF,
            TreeF::NodeF(a, l, r) => TreeF::NodeF(a, f(l), f(r)),
        }
    }
}

impl<A: Clone> CloneHKT for TreeFWitness<A> {
    #[inline]
    fn clone_hkt<T: Clone>(t: &TreeF<A, T>) -> TreeF<A, T> {
        match t {
            TreeF::EmptyF => TreeF::EmptyF,
            TreeF::NodeF(a, l, r) => TreeF::NodeF(a.clone(), l.clone(), r.clone()),
        }
    }
}

impl<A, R> TreeF<A, R> {
    /// Map a function over the recursive positions.
    #[inline]
    pub fn map_rec<B, F>(self, mut f: F) -> TreeF<A, B>
    where
        F: FnMut(R) -> B,
    {
        match self {
            TreeF::EmptyF => TreeF::EmptyF,
            TreeF::NodeF(a, l, r) => TreeF::NodeF(a, f(l), f(r)),
        }
    }
}

// =============================================================================
// Maybe/Option Base Functor
// =============================================================================

/// Base functor for Maybe/Option that adds a "nothing" case.
///
/// This is useful for representing partial structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaybeF<A, R> {
    /// Nothing - no value.
    NothingF,
    /// Just - contains a value and recurses.
    JustF(A, R),
}

/// Witness type for `MaybeF`.
pub struct MaybeFWitness<A>(core::marker::PhantomData<A>);

impl<A> HKT for MaybeFWitness<A> {
    type Target<T> = MaybeF<A, T>;
}

impl<A: Clone> FunctorHKT for MaybeFWitness<A> {
    #[inline]
    fn map<R1, R2, F>(fa: MaybeF<A, R1>, mut f: F) -> MaybeF<A, R2>
    where
        F: FnMut(R1) -> R2,
    {
        match fa {
            MaybeF::NothingF => MaybeF::NothingF,
            MaybeF::JustF(a, r) => MaybeF::JustF(a, f(r)),
        }
    }
}

impl<A: Clone> CloneHKT for MaybeFWitness<A> {
    #[inline]
    fn clone_hkt<T: Clone>(t: &MaybeF<A, T>) -> MaybeF<A, T> {
        match t {
            MaybeF::NothingF => MaybeF::NothingF,
            MaybeF::JustF(a, r) => MaybeF::JustF(a.clone(), r.clone()),
        }
    }
}

// =============================================================================
// Expression Base Functor (Example DSL)
// =============================================================================

/// Base functor for a simple expression language.
///
/// Demonstrates how to define recursive ASTs for interpretation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprF<R> {
    /// Literal integer value.
    LitF(i64),
    /// Addition of two expressions.
    AddF(R, R),
    /// Multiplication of two expressions.
    MulF(R, R),
    /// Negation of an expression.
    NegF(R),
}

/// Witness type for `ExprF`.
pub struct ExprFWitness;

impl HKT for ExprFWitness {
    type Target<T> = ExprF<T>;
}

impl FunctorHKT for ExprFWitness {
    #[inline]
    fn map<A, B, F>(fa: ExprF<A>, mut f: F) -> ExprF<B>
    where
        F: FnMut(A) -> B,
    {
        match fa {
            ExprF::LitF(n) => ExprF::LitF(n),
            ExprF::AddF(l, r) => ExprF::AddF(f(l), f(r)),
            ExprF::MulF(l, r) => ExprF::MulF(f(l), f(r)),
            ExprF::NegF(e) => ExprF::NegF(f(e)),
        }
    }
}

impl CloneHKT for ExprFWitness {
    #[inline]
    fn clone_hkt<T: Clone>(t: &ExprF<T>) -> ExprF<T> {
        match t {
            ExprF::LitF(n) => ExprF::LitF(*n),
            ExprF::AddF(l, r) => ExprF::AddF(l.clone(), r.clone()),
            ExprF::MulF(l, r) => ExprF::MulF(l.clone(), r.clone()),
            ExprF::NegF(e) => ExprF::NegF(e.clone()),
        }
    }
}

impl<R> ExprF<R> {
    /// Map a function over recursive positions.
    #[inline]
    pub fn map<B, F>(self, mut f: F) -> ExprF<B>
    where
        F: FnMut(R) -> B,
    {
        match self {
            ExprF::LitF(n) => ExprF::LitF(n),
            ExprF::AddF(l, r) => ExprF::AddF(f(l), f(r)),
            ExprF::MulF(l, r) => ExprF::MulF(f(l), f(r)),
            ExprF::NegF(e) => ExprF::NegF(f(e)),
        }
    }
}

// =============================================================================
// Rose Tree Base Functor
// =============================================================================

/// Base functor for rose trees (multi-way trees).
///
/// ```text
/// data Rose a = Rose a [Rose a]
/// data RoseF a r = RoseF a [r]
/// ```
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoseF<A, R> {
    /// The value at this node.
    pub value: A,
    /// The children (list of recursive positions).
    pub children: alloc::vec::Vec<R>,
}

/// Witness type for `RoseF`.
#[cfg(feature = "alloc")]
pub struct RoseFWitness<A>(core::marker::PhantomData<A>);

#[cfg(feature = "alloc")]
impl<A> HKT for RoseFWitness<A> {
    type Target<T> = RoseF<A, T>;
}

#[cfg(feature = "alloc")]
impl<A: Clone> FunctorHKT for RoseFWitness<A> {
    #[inline]
    fn map<R1, R2, F>(fa: RoseF<A, R1>, mut f: F) -> RoseF<A, R2>
    where
        F: FnMut(R1) -> R2,
    {
        RoseF {
            value: fa.value,
            children: fa.children.into_iter().map(&mut f).collect(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> CloneHKT for RoseFWitness<A> {
    #[inline]
    fn clone_hkt<T: Clone>(t: &RoseF<A, T>) -> RoseF<A, T> {
        RoseF {
            value: t.value.clone(),
            children: t.children.clone(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<A, R> RoseF<A, R> {
    /// Create a new rose tree node.
    #[inline]
    pub fn new(value: A, children: alloc::vec::Vec<R>) -> Self {
        RoseF { value, children }
    }

    /// Create a leaf node (no children).
    #[inline]
    pub fn leaf(value: A) -> Self {
        RoseF {
            value,
            children: alloc::vec::Vec::new(),
        }
    }

    /// Map a function over recursive positions.
    #[inline]
    pub fn map_rec<B, F>(self, mut f: F) -> RoseF<A, B>
    where
        F: FnMut(R) -> B,
    {
        RoseF {
            value: self.value,
            children: self.children.into_iter().map(&mut f).collect(),
        }
    }
}
