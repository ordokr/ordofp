//! Category laws
//!
//! Laws for the [`Category`] type class.
//!
//! 1. **Identity**: `id . f == f` and `f . id == f`
//! 2. **Associativity**: `(h . g) . f == h . (g . f)`

use ordofp::typeclasses::Category;

/// **Identity Law**: `id . f == f` (Left Identity) and `f . id == f` (Right Identity).
///
/// Requires `Hom` to implement `PartialEq` and `Clone`.
pub fn identity<C, A, B>(f: C::Hom<A, B>) -> bool
where
    C: Category,
    C::Hom<A, B>: Clone + PartialEq,
{
    let id_a = C::id::<A>();
    let id_b = C::id::<B>();

    let left = C::compose(id_b, f.clone());
    let right_side = C::compose(f.clone(), id_a);

    left == f && right_side == f
}

/// **Associativity Law**: `(h . g) . f == h . (g . f)`
///
/// Requires `Hom` to implement `PartialEq` and `Clone`.
pub fn associativity<C, A, B, D, E>(f: C::Hom<A, B>, g: C::Hom<B, D>, h: C::Hom<D, E>) -> bool
where
    C: Category,
    C::Hom<A, B>: Clone,
    C::Hom<B, D>: Clone,
    C::Hom<D, E>: Clone,
    C::Hom<A, E>: PartialEq,
{
    let lhs = C::compose(h.clone(), C::compose(g.clone(), f.clone()));
    let rhs = C::compose(C::compose(h, g), f);

    lhs == rhs
}
