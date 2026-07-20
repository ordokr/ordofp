//! Higher-Kinded Type abstraction.
//!
//! Rust does not natively support HKTs (e.g., `f a` where `f` is a type parameter).
//! We use the "witness" pattern (simulating HKTs via GATs) to allow types like `Fix<F>`.

/// A trait for types that represent a type constructor `* -> *`.
///
/// # Example
///
/// ```
/// use ordofp_core::typeclasses::hkt::HKT;
///
/// struct OptionF;
/// impl HKT for OptionF {
///     type Target<T> = Option<T>;
/// }
/// ```
pub trait HKT {
    /// The type applied to `T`.
    type Target<T>;
}

/// A Functor for HKTs.
///
/// This is distinct from `Functor` in `ordofp::typeclasses::functor`, which is implemented
/// on the *applied* type (e.g. `Option<A>`). This trait is implemented on the
/// *witness* type (e.g. `OptionF`).
pub trait FunctorHKT: HKT {
    /// Maps a function over the HKT.
    fn map<A, B, F>(fa: Self::Target<A>, f: F) -> Self::Target<B>
    where
        F: FnMut(A) -> B;
}

/// A trait for cloning HKTs.
///
/// This allows cloning the target type without requiring `Target<T>: Clone` in bounds,
/// which helps avoid infinite recursion in type checking for recursive types like `Cofree`.
pub trait CloneHKT: HKT {
    /// Clones the target type.
    fn clone_hkt<T: Clone>(t: &Self::Target<T>) -> Self::Target<T>;
}
