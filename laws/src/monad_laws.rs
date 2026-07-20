//! # Monad Laws
//!
//! This module provides property-based laws for testing [`Monad`] implementations.
//!
//! ## Laws
//!
//! 1. **Left Identity**: `pure(a).flat_map(f) == f(a)`
//! 2. **Right Identity**: `m.flat_map(pure) == m`
//! 3. **Associativity**: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::monad_laws;
//!
//! // Test left identity
//! assert!(monad_laws::option_left_identity(5, |x| Some(x * 2)));
//!
//! // Test right identity
//! assert!(monad_laws::option_right_identity(Some(42)));
//!
//! // Test associativity
//! assert!(monad_laws::option_associativity(
//!     Some(5),
//!     |x| Some(x + 1),
//!     |x| Some(x * 2)
//! ));
//! ```

use crate::is_eq::IsEq;
use ordofp::gat::{Applicative, Monad};

// ==================== Option Laws ====================

/// **Left Identity Law** for Option: `pure(a).flat_map(f) == f(a)`
pub fn option_left_identity<A, B, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    F: FnMut(A) -> Option<B> + Clone,
{
    let lhs: Option<B> = Monad::flat_map(<Option<A>>::pure_target(a.clone()), f.clone());
    let rhs: Option<B> = f(a);
    lhs == rhs
}

/// **Right Identity Law** for Option: `m.flat_map(pure) == m`
pub fn option_right_identity<A: Clone + Eq>(ma: Option<A>) -> bool {
    let lhs: Option<A> = Monad::flat_map(ma.clone(), |x| <Option<A>>::pure_target(x));
    lhs == ma
}

/// **Associativity Law** for Option: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
pub fn option_associativity<A, B, C, F, G>(ma: Option<A>, f: F, g: G) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    F: Fn(A) -> Option<B> + Clone,
    G: Fn(B) -> Option<C> + Clone,
{
    let f_clone = f.clone();
    let g_clone = g.clone();

    let lhs: Option<C> = Monad::flat_map(Monad::flat_map(ma.clone(), f), g);
    let rhs: Option<C> = Monad::flat_map(ma, move |x| Monad::flat_map(f_clone(x), g_clone.clone()));
    lhs == rhs
}

/// Returns an [`IsEq`] for the Option left identity law.
pub fn option_left_identity_eq<A, B, F>(a: A, mut f: F) -> IsEq<Option<B>>
where
    A: Clone,
    F: FnMut(A) -> Option<B> + Clone,
{
    let lhs = Monad::flat_map(<Option<A>>::pure_target(a.clone()), f.clone());
    let rhs = f(a);
    IsEq::equal_under_law(lhs, rhs)
}

/// Returns an [`IsEq`] for the Option right identity law.
pub fn option_right_identity_eq<A: Clone>(ma: Option<A>) -> IsEq<Option<A>> {
    let lhs = Monad::flat_map(ma.clone(), |x| <Option<A>>::pure_target(x));
    IsEq::equal_under_law(lhs, ma)
}

// ==================== Result Laws ====================

/// **Left Identity Law** for Result: `pure(a).flat_map(f) == f(a)`
pub fn result_left_identity<A, B, E, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    E: Clone + Eq,
    F: FnMut(A) -> Result<B, E> + Clone,
{
    let lhs: Result<B, E> = Monad::flat_map(<Result<A, E>>::pure_target(a.clone()), f.clone());
    let rhs: Result<B, E> = f(a);
    lhs == rhs
}

/// **Right Identity Law** for Result: `m.flat_map(pure) == m`
pub fn result_right_identity<A: Clone + Eq, E: Clone + Eq>(ma: Result<A, E>) -> bool {
    let lhs: Result<A, E> = Monad::flat_map(ma.clone(), |x| <Result<A, E>>::pure_target(x));
    lhs == ma
}

/// **Associativity Law** for Result: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
pub fn result_associativity<A, B, C, E, F, G>(ma: Result<A, E>, f: F, g: G) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    E: Clone + Eq,
    F: Fn(A) -> Result<B, E> + Clone,
    G: Fn(B) -> Result<C, E> + Clone,
{
    let f_clone = f.clone();
    let g_clone = g.clone();

    let lhs: Result<C, E> = Monad::flat_map(Monad::flat_map(ma.clone(), f), g);
    let rhs: Result<C, E> =
        Monad::flat_map(ma, move |x| Monad::flat_map(f_clone(x), g_clone.clone()));
    lhs == rhs
}

/// Returns an [`IsEq`] for the Result left identity law.
pub fn result_left_identity_eq<A, B, E, F>(a: A, mut f: F) -> IsEq<Result<B, E>>
where
    A: Clone,
    E: Clone,
    F: FnMut(A) -> Result<B, E> + Clone,
{
    let lhs = Monad::flat_map(<Result<A, E>>::pure_target(a.clone()), f.clone());
    let rhs = f(a);
    IsEq::equal_under_law(lhs, rhs)
}

// ==================== Vec Laws ====================

/// **Left Identity Law** for Vec: `pure(a).flat_map(f) == f(a)`
pub fn vec_left_identity<A, B, F>(a: A, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    F: FnMut(A) -> Vec<B> + Clone,
{
    let lhs: Vec<B> = Monad::flat_map(<Vec<A>>::pure_target(a.clone()), f.clone());
    let rhs: Vec<B> = f(a);
    lhs == rhs
}

/// **Right Identity Law** for Vec: `m.flat_map(pure) == m`
pub fn vec_right_identity<A: Clone + Eq>(ma: Vec<A>) -> bool {
    let lhs: Vec<A> = Monad::flat_map(ma.clone(), |x| <Vec<A>>::pure_target(x));
    lhs == ma
}

/// **Associativity Law** for Vec: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
pub fn vec_associativity<A, B, C, F, G>(ma: Vec<A>, f: F, g: G) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    F: Fn(A) -> Vec<B> + Clone,
    G: Fn(B) -> Vec<C> + Clone,
{
    let f_clone = f.clone();
    let g_clone = g.clone();

    let lhs: Vec<C> = Monad::flat_map(Monad::flat_map(ma.clone(), f), g);
    let rhs: Vec<C> = Monad::flat_map(ma, move |x| Monad::flat_map(f_clone(x), g_clone.clone()));
    lhs == rhs
}

/// Map-FlatMap coherence: `m.map(f) == m.flat_map(|x| pure(f(x)))`
pub fn option_map_flatmap_coherence<A, B, F>(ma: Option<A>, mut f: F) -> bool
where
    A: Clone,
    B: Eq,
    F: FnMut(A) -> B + Clone,
{
    use ordofp::gat::Functor;

    let lhs: Option<B> = Functor::map(ma.clone(), f.clone());
    let rhs: Option<B> = Monad::flat_map(ma, move |x| <Option<B>>::pure_target(f(x)));
    lhs == rhs
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    // ==================== Option Tests ====================

    #[test]
    fn test_option_left_identity() {
        fn test(a: i8) -> bool {
            option_left_identity(a, |x| Some(x.wrapping_mul(2)))
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_option_right_identity() {
        quickcheck(option_right_identity::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_associativity() {
        fn test(m: Option<i8>) -> bool {
            option_associativity(m, |x| Some(x.wrapping_add(1)), |x| Some(x.wrapping_mul(2)))
        }
        quickcheck(test as fn(Option<i8>) -> bool);
    }

    #[test]
    fn test_option_map_flatmap_coherence() {
        fn test(m: Option<i8>) -> bool {
            option_map_flatmap_coherence(m, |x| x.wrapping_mul(3))
        }
        quickcheck(test as fn(Option<i8>) -> bool);
    }

    #[test]
    fn test_option_associativity_with_none() {
        assert!(option_associativity(
            None::<i32>,
            |x| Some(x + 1),
            |x| Some(x * 2),
        ));
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_left_identity() {
        fn test(a: i8) -> bool {
            result_left_identity::<_, _, String, _>(a, |x| Ok(x.wrapping_mul(2)))
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_result_right_identity() {
        fn test(m: Result<i32, String>) -> bool {
            result_right_identity(m)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    #[test]
    fn test_result_associativity() {
        fn test(m: Result<i8, String>) -> bool {
            result_associativity(m, |x| Ok(x.wrapping_add(1)), |x| Ok(x.wrapping_mul(2)))
        }
        quickcheck(test as fn(Result<i8, String>) -> bool);
    }

    #[test]
    fn test_result_associativity_with_error() {
        assert!(result_associativity(
            Err::<i32, _>("error".to_string()),
            |x| Ok(x + 1),
            |x| Ok(x * 2),
        ));
    }

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_left_identity() {
        fn test(a: i8) -> bool {
            vec_left_identity(a, |x| vec![x, x.wrapping_mul(2)])
        }
        quickcheck(test as fn(i8) -> bool);
    }

    #[test]
    fn test_vec_right_identity() {
        quickcheck(vec_right_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_associativity() {
        fn test(m: Vec<i8>) -> bool {
            vec_associativity(
                m,
                |x| vec![x, x.wrapping_add(1)],
                |x| vec![x.wrapping_mul(2)],
            )
        }
        quickcheck(test as fn(Vec<i8>) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_left_identity() {
        assert!(option_left_identity(5, |x| Some(x * 2)));
        assert!(option_left_identity(42, |x| Some(x.to_string())));
        assert!(result_left_identity::<_, _, String, _>(10, |x| Ok(x * 3)));
        assert!(vec_left_identity(5, |x| vec![x, x + 1]));
    }

    #[test]
    fn manual_right_identity() {
        assert!(option_right_identity(Some(100)));
        assert!(option_right_identity(None::<i32>));
        assert!(result_right_identity(Ok::<_, String>(42)));
        assert!(result_right_identity(Err::<i32, _>("fail".to_string())));
        assert!(vec_right_identity(vec![1, 2, 3]));
    }

    #[test]
    fn manual_associativity() {
        // Option
        assert!(option_associativity(
            Some(5),
            |x| Some(x + 10),
            |x| Some(x * 2)
        ));
        assert!(option_associativity(
            Some(10),
            |x| if x > 5 { Some(x) } else { None },
            |x| if x % 2 == 0 { Some(x) } else { None }
        ));

        // Result
        assert!(result_associativity(
            Ok::<_, String>(5),
            |x| Ok(x + 10),
            |x| Ok(x * 2)
        ));

        // Vec
        assert!(vec_associativity(
            vec![1, 2],
            |x| vec![x, x * 10],
            |x| vec![x + 1]
        ));
    }

    #[test]
    fn test_eq_variants() {
        let eq = option_left_identity_eq(5, |x| Some(x * 2));
        assert!(eq.holds());

        let eq = option_right_identity_eq(Some(42));
        assert!(eq.holds());
    }
}
