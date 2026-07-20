//! Arrow type class.
//!
//! Arrows are a generalization of functions.

use crate::typeclasses::category::Category;

/// The Arrow type class representing computations with input and output.
pub trait Arrow: Category {
    /// Lift a function to an arrow.
    fn arr<A, B, F>(f: F) -> Self::Hom<A, B>
    where
        F: Fn(A) -> B + 'static;

    /// Apply the arrow to the first element of a tuple.
    fn first<A, B, C>(f: Self::Hom<A, B>) -> Self::Hom<(A, C), (B, C)>;

    /// Apply the arrow to the second element of a tuple.
    #[inline]
    fn second<A, B, C>(f: Self::Hom<A, B>) -> Self::Hom<(C, A), (C, B)> {
        let swap_in = Self::arr(|(c, a)| (a, c));
        let swap_out = Self::arr(|(b, c)| (c, b));
        Self::compose(swap_out, Self::compose(Self::first(f), swap_in))
    }

    /// Run two arrows in parallel (split).
    /// `f *** g`
    #[inline]
    fn split<A, B, C, D>(f: Self::Hom<A, B>, g: Self::Hom<C, D>) -> Self::Hom<(A, C), (B, D)> {
        // first f >>> second g
        let f_first = Self::first(f);
        let g_second = Self::second(g);
        Self::compose(g_second, f_first)
    }
}

// The Arrow trait is designed for use with GATs and advanced abstractions.
// A full implementation requires a Category instance that can also implement
// Arrow's function lifting. See the category module for the Category trait.
