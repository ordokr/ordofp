//! # Applicative Laws
//!
//! This module provides property-based laws for testing [`Applicative`] implementations.
//!
//! ## Laws
//!
//! 1. **Identity**: `pure(id).ap(v) == v` (tested via map equivalent)
//! 2. **Homomorphism**: `pure(x).apply(pure(f)) == pure(f(x))` (tested via
//!    the real [`Apply::apply`])
//! 3. **Pure Preservation**: `pure(a).map(f) == pure(f(a))` (the map-level
//!    shadow of homomorphism — distinct from law 2, which goes through
//!    `apply`)
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::applicative_laws;
//!
//! // Test identity law
//! assert!(applicative_laws::option_identity(Some(42)));
//!
//! // Test homomorphism law
//! assert!(applicative_laws::option_homomorphism(5, |x| x * 2));
//!
//! // Test pure preservation
//! assert!(applicative_laws::option_pure_preservation(10, |x| x.to_string()));
//! ```

use crate::is_eq::IsEq;
use ordofp::gat::{Applicative, Apply, Functor};

/// The identity function.
fn id<T>(x: T) -> T {
    x
}

// ==================== Option Laws ====================

/// **Identity Law** for Option (via map): `fa.map(id) == fa`
///
/// This tests the functor identity which is equivalent to `pure(id).ap(v) == v`
/// when combined with the applicative homomorphism law.
pub fn option_identity<A: Clone + Eq>(fa: Option<A>) -> bool {
    Functor::map(fa.clone(), id) == fa
}

/// **Homomorphism Law** for Option: `pure(x).apply(pure(f)) == pure(f(x))`
///
/// Tested through the real [`Apply::apply`], unlike
/// [`option_pure_preservation`], which tests the map-level shadow.
pub fn option_homomorphism<A, B, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    F: FnMut(A) -> B + Clone,
{
    let ff: Option<F> = <Option<A>>::pure_target(f.clone());
    let lhs: Option<B> = <Option<A>>::pure_target(a.clone()).apply(ff);
    let rhs: Option<B> = <Option<B>>::pure_target(f(a));
    lhs == rhs
}

/// **Pure Preservation**: `pure(a).map(f) == pure(f(a))`
pub fn option_pure_preservation<A, B, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    F: FnMut(A) -> B + Clone,
{
    let lhs: Option<B> = Functor::map(<Option<A>>::pure_target(a.clone()), f.clone());
    let rhs: Option<B> = <Option<B>>::pure_target(f(a));
    lhs == rhs
}

/// Returns an [`IsEq`] for the Option identity law.
pub fn option_identity_eq<A: Clone>(fa: Option<A>) -> IsEq<Option<A>> {
    IsEq::equal_under_law(Functor::map(fa.clone(), id), fa)
}

/// Returns an [`IsEq`] for the Option homomorphism law (via [`Apply::apply`]).
pub fn option_homomorphism_eq<A, B, F>(a: A, mut f: F) -> IsEq<Option<B>>
where
    A: Clone,
    F: FnMut(A) -> B + Clone,
{
    let ff: Option<F> = <Option<A>>::pure_target(f.clone());
    let lhs = <Option<A>>::pure_target(a.clone()).apply(ff);
    let rhs = <Option<B>>::pure_target(f(a));
    IsEq::equal_under_law(lhs, rhs)
}

// ==================== Result Laws ====================

/// **Identity Law** for Result (via map): `fa.map(id) == fa`
pub fn result_identity<A: Clone + Eq, E: Clone + Eq>(fa: Result<A, E>) -> bool {
    Functor::map(fa.clone(), id) == fa
}

/// **Homomorphism Law** for Result: `pure(x).apply(pure(f)) == pure(f(x))`
///
/// Tested through the real [`Apply::apply`], unlike
/// [`result_pure_preservation`], which tests the map-level shadow.
pub fn result_homomorphism<A, B, E, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    E: Clone + Eq,
    F: FnMut(A) -> B + Clone,
{
    let ff: Result<F, E> = <Result<A, E>>::pure_target(f.clone());
    let lhs: Result<B, E> = <Result<A, E>>::pure_target(a.clone()).apply(ff);
    let rhs: Result<B, E> = <Result<B, E>>::pure_target(f(a));
    lhs == rhs
}

/// **Pure Preservation** for Result: `pure(a).map(f) == pure(f(a))`
pub fn result_pure_preservation<A, B, E, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    E: Clone + Eq,
    F: FnMut(A) -> B + Clone,
{
    let lhs: Result<B, E> = Functor::map(<Result<A, E>>::pure_target(a.clone()), f.clone());
    let rhs: Result<B, E> = <Result<B, E>>::pure_target(f(a));
    lhs == rhs
}

/// Returns an [`IsEq`] for the Result identity law.
pub fn result_identity_eq<A: Clone, E: Clone>(fa: Result<A, E>) -> IsEq<Result<A, E>> {
    IsEq::equal_under_law(Functor::map(fa.clone(), id), fa)
}

// ==================== Vec Laws ====================

/// **Identity Law** for Vec (via map): `fa.map(id) == fa`
pub fn vec_identity<A: Clone + Eq>(fa: Vec<A>) -> bool {
    Functor::map(fa.clone(), id) == fa
}

/// **Homomorphism Law** for Vec: `pure(x).apply(pure(f)) == pure(f(x))`
///
/// Tested through the real [`Apply::apply`], unlike
/// [`vec_pure_preservation`], which tests the map-level shadow.
pub fn vec_homomorphism<A, B, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Clone + Eq,
    F: FnMut(A) -> B + Clone,
{
    let ff: Vec<F> = <Vec<A>>::pure_target(f.clone());
    let lhs: Vec<B> = <Vec<A>>::pure_target(a.clone()).apply(ff);
    let rhs: Vec<B> = <Vec<B>>::pure_target(f(a));
    lhs == rhs
}

/// **Pure Preservation** for Vec: `pure(a).map(f) == pure(f(a))`
pub fn vec_pure_preservation<A, B, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Clone + Eq,
    F: FnMut(A) -> B + Clone,
{
    let lhs: Vec<B> = Functor::map(<Vec<A>>::pure_target(a.clone()), f.clone());
    let rhs: Vec<B> = <Vec<B>>::pure_target(f(a));
    lhs == rhs
}

/// Returns an [`IsEq`] for the Vec identity law.
pub fn vec_identity_eq<A: Clone>(fa: Vec<A>) -> IsEq<Vec<A>> {
    IsEq::equal_under_law(Functor::map(fa.clone(), id), fa)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    // ==================== Option Tests ====================

    #[test]
    fn test_option_identity() {
        quickcheck(option_identity::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_identity_none() {
        assert!(option_identity(None::<i32>));
    }

    #[test]
    fn test_option_homomorphism() {
        fn test(a: i8) -> bool {
            option_homomorphism(a, |x| x.wrapping_mul(2))
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_option_pure_preservation() {
        fn test(a: i32) -> bool {
            option_pure_preservation(a, |x| x.to_string())
        }
        quickcheck(test as fn(i32) -> bool);
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_identity() {
        fn test(fa: Result<i32, String>) -> bool {
            result_identity(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    #[test]
    fn test_result_homomorphism() {
        fn test(a: i8) -> bool {
            result_homomorphism::<_, _, String, _>(a, |x| x.wrapping_mul(2))
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_result_pure_preservation() {
        fn test(a: i32) -> bool {
            result_pure_preservation::<_, _, String, _>(a, |x| format!("{x}"))
        }
        quickcheck(test as fn(i32) -> bool);
    }

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_identity() {
        quickcheck(vec_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_homomorphism() {
        fn test(a: i8) -> bool {
            vec_homomorphism(a, |x| x.wrapping_mul(2))
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_vec_pure_preservation() {
        fn test(a: i32) -> bool {
            vec_pure_preservation(a, |x| x.to_string())
        }
        quickcheck(test as fn(i32) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_identity() {
        assert!(option_identity(Some(42)));
        assert!(option_identity(None::<String>));
        assert!(result_identity(Ok::<_, String>(100)));
        assert!(result_identity(Err::<i32, _>("error".to_string())));
        assert!(vec_identity(vec![1, 2, 3]));
    }

    #[test]
    fn manual_homomorphism() {
        assert!(option_homomorphism(5, |x| x * 10));
        assert!(option_homomorphism(42, |x| x.to_string()));
        assert!(result_homomorphism::<_, _, String, _>(10, |x| x * 2));
        assert!(vec_homomorphism(5, |x| x * 3));
    }

    #[test]
    fn manual_pure_preservation() {
        assert!(option_pure_preservation(5, |x| x + 100));
        assert!(result_pure_preservation::<_, _, String, _>(10, |x| x * 2));
        assert!(vec_pure_preservation(7, |x| x.to_string()));
    }

    #[test]
    fn test_eq_variants() {
        let eq = option_identity_eq(Some(42));
        assert!(eq.holds());

        let eq = option_homomorphism_eq(5, |x| x * 2);
        assert!(eq.holds());

        let eq = result_identity_eq(Ok::<_, String>(42));
        assert!(eq.holds());

        let eq = vec_identity_eq(vec![1, 2, 3]);
        assert!(eq.holds());
    }
}
