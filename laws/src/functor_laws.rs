//! # Functor Laws
//!
//! This module provides property-based laws for testing [`Functor`] implementations.
//!
//! ## Laws
//!
//! 1. **Identity**: `fa.map(id) == fa`
//! 2. **Composition**: `fa.map(f).map(g) == fa.map(|x| g(f(x)))`
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::functor_laws;
//!
//! // Test identity law for Option
//! assert!(functor_laws::option_identity(Some(42)));
//! assert!(functor_laws::option_identity(None::<i32>));
//!
//! // Test composition law
//! assert!(functor_laws::option_composition(Some(5), |x| x + 1, |x| x * 2));
//! ```

use crate::is_eq::IsEq;
use ordofp::gat::Functor;

/// The identity function.
pub fn id<T>(x: T) -> T {
    x
}

// ==================== Option Laws ====================

/// **Identity Law** for Option: `fa.map(id) == fa`
pub fn option_identity<A: Clone + Eq>(fa: Option<A>) -> bool {
    Functor::map(fa.clone(), id) == fa
}

/// **Composition Law** for Option: `fa.map(f).map(g) == fa.map(|x| g(f(x)))`
pub fn option_composition<A, B, C, F, G>(fa: Option<A>, mut f: F, mut g: G) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    F: FnMut(A) -> B + Clone,
    G: FnMut(B) -> C + Clone,
{
    let lhs: Option<C> = Functor::map(Functor::map(fa.clone(), f.clone()), g.clone());
    let rhs: Option<C> = Functor::map(fa, move |x| g(f(x)));
    lhs == rhs
}

/// Returns an [`IsEq`] for the Option identity law.
pub fn option_identity_eq<A: Clone>(fa: Option<A>) -> IsEq<Option<A>> {
    IsEq::equal_under_law(Functor::map(fa.clone(), id), fa)
}

// ==================== Result Laws ====================

/// **Identity Law** for Result: `fa.map(id) == fa`
pub fn result_identity<A: Clone + Eq, E: Clone + Eq>(fa: Result<A, E>) -> bool {
    Functor::map(fa.clone(), id) == fa
}

/// **Composition Law** for Result: `fa.map(f).map(g) == fa.map(|x| g(f(x)))`
pub fn result_composition<A, B, C, E, F, G>(fa: Result<A, E>, mut f: F, mut g: G) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    E: Clone + Eq,
    F: FnMut(A) -> B + Clone,
    G: FnMut(B) -> C + Clone,
{
    let lhs: Result<C, E> = Functor::map(Functor::map(fa.clone(), f.clone()), g.clone());
    let rhs: Result<C, E> = Functor::map(fa, move |x| g(f(x)));
    lhs == rhs
}

/// Returns an [`IsEq`] for the Result identity law.
pub fn result_identity_eq<A: Clone, E: Clone>(fa: Result<A, E>) -> IsEq<Result<A, E>> {
    IsEq::equal_under_law(Functor::map(fa.clone(), id), fa)
}

// ==================== Vec Laws ====================

/// **Identity Law** for Vec: `fa.map(id) == fa`
pub fn vec_identity<A: Clone + Eq>(fa: Vec<A>) -> bool {
    Functor::map(fa.clone(), id) == fa
}

/// **Composition Law** for Vec: `fa.map(f).map(g) == fa.map(|x| g(f(x)))`
pub fn vec_composition<A, B, C, F, G>(fa: Vec<A>, mut f: F, mut g: G) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    F: FnMut(A) -> B + Clone,
    G: FnMut(B) -> C + Clone,
{
    let lhs: Vec<C> = Functor::map(Functor::map(fa.clone(), f.clone()), g.clone());
    let rhs: Vec<C> = Functor::map(fa, move |x| g(f(x)));
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
    fn test_option_identity_law() {
        quickcheck(option_identity::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_identity_none() {
        assert!(option_identity(None::<String>));
    }

    #[test]
    fn test_option_composition_law() {
        fn test(fa: Option<i8>) -> bool {
            option_composition(fa, |x| x.wrapping_add(1), |x| x.wrapping_mul(2))
        }
        quickcheck(test as fn(Option<i8>) -> bool);
    }

    #[test]
    fn test_option_composition_with_type_change() {
        fn test(fa: Option<i32>) -> bool {
            option_composition(fa, |x| x.to_string(), |s| s.len())
        }
        quickcheck(test as fn(Option<i32>) -> bool);
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_identity_law() {
        fn test(fa: Result<i32, String>) -> bool {
            result_identity(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    #[test]
    fn test_result_composition_law() {
        fn test(fa: Result<i8, String>) -> bool {
            result_composition(fa, |x| x.wrapping_mul(2), |x| x.wrapping_add(10))
        }
        quickcheck(test as fn(Result<i8, String>) -> bool);
    }

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_identity_law() {
        quickcheck(vec_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_composition_law() {
        fn test(fa: Vec<i8>) -> bool {
            vec_composition(fa, |x| x.wrapping_add(1), |x| x.wrapping_mul(2))
        }
        quickcheck(test as fn(Vec<i8>) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_identity_tests() {
        // Option
        assert!(option_identity(Some(42)));
        assert!(option_identity(None::<i32>));
        assert!(option_identity(Some("hello".to_string())));

        // Result
        assert!(result_identity(Ok::<i32, &str>(100)));
        assert!(result_identity(Err::<i32, &str>("error")));

        // Vec
        assert!(vec_identity(vec![1, 2, 3]));
        assert!(vec_identity(Vec::<i32>::new()));
    }

    #[test]
    fn manual_composition_tests() {
        // Some value
        assert!(option_composition(Some(10), |x| x + 5, |x| x * 2));

        // None
        assert!(option_composition(None::<i32>, |x| x + 5, |x| x * 2));

        // Result Ok
        assert!(result_composition(
            Ok::<_, &str>(10),
            |x: i32| x + 5,
            |x| x * 2
        ));

        // Result Err
        assert!(result_composition(
            Err::<i32, _>("err"),
            |x: i32| x + 5,
            |x| x * 2
        ));

        // Vec
        assert!(vec_composition(vec![1, 2, 3], |x| x + 1, |x| x * 10));
    }

    #[test]
    fn test_identity_eq() {
        let eq = option_identity_eq(Some(42));
        assert!(eq.holds());

        let eq = result_identity_eq(Ok::<_, String>(42));
        assert!(eq.holds());

        let eq = vec_identity_eq(vec![1, 2, 3]);
        assert!(eq.holds());
    }
}
