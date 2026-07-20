//! Category type class.
//!
//! A Category consists of objects and morphisms (arrows) between them, satisfying
//! identity and composition laws.

/// A Category is defined by a collection of objects and morphisms between them.
///
/// In Rust, "objects" are types, and "morphisms" are represented by the `Hom` GAT.
///
/// # Laws
///
/// 1. **Identity**: `id <<< f == f` and `f <<< id == f`
/// 2. **Associativity**: `(h <<< g) <<< f == h <<< (g <<< f)`
pub trait Category {
    /// The type of morphisms (arrows) from A to B.
    type Hom<A, B>;

    /// The identity morphism for an object A.
    fn id<A>() -> Self::Hom<A, A>;

    /// Composition of morphisms.
    /// `compose(f, g)` corresponds to `f . g` (math) or `g >>> f` (flow).
    /// Here we use the standard math order: (B -> C) -> (A -> B) -> (A -> C).
    fn compose<A, B, C>(f: Self::Hom<B, C>, g: Self::Hom<A, B>) -> Self::Hom<A, C>;
}
