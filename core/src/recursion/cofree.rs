//! Cofree comonad - for histomorphisms and course-of-values recursion.
//!
//! > *"Historia est magistra vitae."*
//! > — History is the teacher of life. (Cicero)
//!
//! The Cofree comonad annotates each node of a functor with a value,
//! allowing access to the "history" of computation during recursion.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::typeclasses::hkt::{CloneHKT, FunctorHKT, HKT};

/// Cofree comonad - an annotated functor structure.
///
/// `Cofree F A` is like `F` but with an `A` value at each node.
/// This allows histomorphisms to see previously computed values.
///
/// ```text
/// data Cofree f a = a :< f (Cofree f a)
/// ```
///
/// # Scholastic Name: Liber Memoriae
///
/// > *"Memoria praeteritorum."*
/// > — The memory of past things.
#[cfg(feature = "alloc")]
#[derive(Debug, PartialEq, Eq)]
pub struct Cofree<F: FunctorHKT, A> {
    /// The annotation at this node.
    pub attribute: A,
    /// The functor containing children.
    pub children: Box<F::Target<Cofree<F, A>>>,
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT + CloneHKT, A: Clone> Clone for Cofree<F, A> {
    fn clone(&self) -> Self {
        Cofree {
            attribute: self.attribute.clone(),
            children: Box::new(F::clone_hkt(&self.children)),
        }
    }
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT, A> Cofree<F, A> {
    /// Create a new Cofree node.
    #[inline]
    pub fn new(attribute: A, children: F::Target<Cofree<F, A>>) -> Self {
        Cofree {
            attribute,
            children: Box::new(children),
        }
    }

    /// Extract the attribute (comonad extract).
    #[inline]
    pub fn extract(&self) -> &A {
        &self.attribute
    }

    /// Get the children functor.
    #[inline]
    pub fn unwrap(self) -> F::Target<Cofree<F, A>> {
        *self.children
    }

    /// Get a reference to the children.
    #[inline]
    pub fn children_ref(&self) -> &F::Target<Cofree<F, A>> {
        &self.children
    }

    /// Map over the attribute.
    #[inline]
    pub fn map_attr<B, G>(self, f: G) -> Cofree<F, B>
    where
        G: Fn(A) -> B + Clone,
        F::Target<Cofree<F, A>>: Clone,
    {
        self.map_attr_impl(f)
    }

    #[inline]
    fn map_attr_impl<B, G>(self, f: G) -> Cofree<F, B>
    where
        G: Fn(A) -> B + Clone,
        F::Target<Cofree<F, A>>: Clone,
    {
        let new_attr = f(self.attribute);
        let new_children = F::map(*self.children, |child| child.map_attr_impl(f.clone()));
        Cofree {
            attribute: new_attr,
            children: Box::new(new_children),
        }
    }
}

/// Witness for Cofree as an HKT.
#[cfg(feature = "alloc")]
pub struct CofreeWitness<F: FunctorHKT>(core::marker::PhantomData<F>);

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> HKT for CofreeWitness<F> {
    type Target<A> = Cofree<F, A>;
}

// =============================================================================
// Cofree Construction Helpers
// =============================================================================

/// Build a Cofree structure by annotating each layer.
///
/// Given a structure and a way to compute annotations, builds
/// the Cofree comonad.
#[cfg(feature = "alloc")]
#[inline]
pub fn cofree_annotate<F, T, A, Ann>(value: T, annotate: Ann) -> Cofree<F, A>
where
    F: FunctorHKT,
    T: crate::recursion::traits::Recursiva<Base = F>,
    Ann: Fn(&F::Target<Cofree<F, A>>) -> A + Clone,
{
    cofree_annotate_impl(value, &annotate)
}

#[cfg(feature = "alloc")]
fn cofree_annotate_impl<F, T, A, Ann>(value: T, annotate: &Ann) -> Cofree<F, A>
where
    F: FunctorHKT,
    T: crate::recursion::traits::Recursiva<Base = F>,
    Ann: Fn(&F::Target<Cofree<F, A>>) -> A,
{
    let layer = value.project();
    let children = F::map(layer, |child| cofree_annotate_impl(child, annotate));
    let attr = annotate(&children);
    Cofree::new(attr, children)
}

// =============================================================================
// Cofree for specific base functors
// =============================================================================

/// Annotated natural number - each node has a computed value.
#[cfg(feature = "alloc")]
pub type CofreeNat<A> = Cofree<crate::recursion::base::NatFWitness, A>;

/// Annotated list - each cons cell has a computed value.
#[cfg(feature = "alloc")]
pub type CofreeList<E, A> = Cofree<crate::recursion::base::ListFWitness<E>, A>;

/// Annotated expression - each AST node has a computed value.
#[cfg(feature = "alloc")]
pub type CofreeExpr<A> = Cofree<crate::recursion::base::ExprFWitness, A>;
