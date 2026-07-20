//! # Bifunctor Laws
//!
//! This module provides property-based laws for testing [`Bifunctor`] implementations.
//!
//! ## Laws
//!
//! 1. **Identity**: `x.bimap(id, id) == x`
//! 2. **Composition**: `x.bimap(f, g).bimap(h, i) == x.bimap(|a| h(f(a)), |b| i(g(b)))`
//!
//! ## Derived Laws (from `map_left` and `map_right`)
//!
//! 3. **`map_left` Identity**: `x.map_left(id) == x`
//! 4. **`map_right` Identity**: `x.map_right(id) == x`
//! 5. **bimap decomposition**: `x.bimap(f, g) == x.map_left(f).map_right(g)`
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::bifunctor_laws;
//!
//! // Test identity law for Result
//! assert!(bifunctor_laws::result_identity(Ok::<i32, String>(42)));
//! assert!(bifunctor_laws::result_identity(Err::<i32, String>("error".into())));
//!
//! // Test identity law for tuple
//! assert!(bifunctor_laws::tuple_identity((42, "hello")));
//! ```

use crate::is_eq::IsEq;
use ordofp::bifunctor::Bifunctor;

/// The identity function.
pub fn id<T>(x: T) -> T {
    x
}

// ==================== Result Laws ====================

/// **Identity Law** for Result: `x.bimap(id, id) == x`
pub fn result_identity<A: Clone + Eq, E: Clone + Eq>(fa: Result<A, E>) -> bool {
    fa.clone().bimap(id, id) == fa
}

/// **Composition Law** for Result:
/// `x.bimap(f, g).bimap(h, i) == x.bimap(|a| h(f(a)), |b| i(g(b)))`
pub fn result_composition<A, B, C, E, E2, E3, F, G, H, I>(
    fa: Result<A, E>,
    f: F,
    g: G,
    h: H,
    i: I,
) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    E: Clone,
    E2: Clone,
    E3: Eq,
    F: FnOnce(A) -> B + Clone,
    G: FnOnce(E) -> E2 + Clone,
    H: FnOnce(B) -> C + Clone,
    I: FnOnce(E2) -> E3 + Clone,
{
    let lhs = fa
        .clone()
        .bimap(f.clone(), g.clone())
        .bimap(h.clone(), i.clone());
    let rhs = fa.bimap(|a| h.clone()(f.clone()(a)), |e| i.clone()(g.clone()(e)));
    lhs == rhs
}

/// **`map_left` Identity Law** for Result: `x.map_left(id) == x`
pub fn result_map_left_identity<A: Clone + Eq, E: Clone + Eq>(fa: Result<A, E>) -> bool {
    fa.clone().map_left(id) == fa
}

/// **`map_right` Identity Law** for Result: `x.map_right(id) == x`
pub fn result_map_right_identity<A: Clone + Eq, E: Clone + Eq>(fa: Result<A, E>) -> bool {
    fa.clone().map_right(id) == fa
}

/// **bimap decomposition** for Result: `x.bimap(f, g) == x.map_left(f).map_right(g)`
pub fn result_bimap_decomposition<A, B, E, E2, F, G>(fa: Result<A, E>, f: F, g: G) -> bool
where
    A: Clone,
    B: Clone + Eq,
    E: Clone,
    E2: Clone + Eq,
    F: FnOnce(A) -> B + Clone,
    G: FnOnce(E) -> E2 + Clone,
{
    let lhs = fa.clone().bimap(f.clone(), g.clone());
    let rhs = fa.map_left(f).map_right(g);
    lhs == rhs
}

/// Returns an [`IsEq`] for the Result identity law.
pub fn result_identity_eq<A: Clone, E: Clone>(fa: Result<A, E>) -> IsEq<Result<A, E>> {
    IsEq::equal_under_law(fa.clone().bimap(id, id), fa)
}

// ==================== Tuple Laws ====================

/// **Identity Law** for tuple: `x.bimap(id, id) == x`
pub fn tuple_identity<A: Clone + Eq, B: Clone + Eq>(fa: (A, B)) -> bool {
    fa.clone().bimap(id, id) == fa
}

/// **Composition Law** for tuple:
/// `x.bimap(f, g).bimap(h, i) == x.bimap(|a| h(f(a)), |b| i(g(b)))`
pub fn tuple_composition<A, A2, A3, B, B2, B3, F, G, H, I>(
    fa: (A, B),
    f: F,
    g: G,
    h: H,
    i: I,
) -> bool
where
    A: Clone,
    A2: Clone,
    A3: Eq,
    B: Clone,
    B2: Clone,
    B3: Eq,
    F: FnOnce(A) -> A2 + Clone,
    G: FnOnce(B) -> B2 + Clone,
    H: FnOnce(A2) -> A3 + Clone,
    I: FnOnce(B2) -> B3 + Clone,
{
    let lhs = fa
        .clone()
        .bimap(f.clone(), g.clone())
        .bimap(h.clone(), i.clone());
    let rhs = fa.bimap(|a| h.clone()(f.clone()(a)), |b| i.clone()(g.clone()(b)));
    lhs == rhs
}

/// **`map_left` Identity Law** for tuple: `x.map_left(id) == x`
pub fn tuple_map_left_identity<A: Clone + Eq, B: Clone + Eq>(fa: (A, B)) -> bool {
    fa.clone().map_left(id) == fa
}

/// **`map_right` Identity Law** for tuple: `x.map_right(id) == x`
pub fn tuple_map_right_identity<A: Clone + Eq, B: Clone + Eq>(fa: (A, B)) -> bool {
    fa.clone().map_right(id) == fa
}

/// **bimap decomposition** for tuple: `x.bimap(f, g) == x.map_left(f).map_right(g)`
pub fn tuple_bimap_decomposition<A, A2, B, B2, F, G>(fa: (A, B), f: F, g: G) -> bool
where
    A: Clone,
    A2: Clone + Eq,
    B: Clone,
    B2: Clone + Eq,
    F: FnOnce(A) -> A2 + Clone,
    G: FnOnce(B) -> B2 + Clone,
{
    let lhs = fa.clone().bimap(f.clone(), g.clone());
    let rhs = fa.map_left(f).map_right(g);
    lhs == rhs
}

/// Returns an [`IsEq`] for the tuple identity law.
pub fn tuple_identity_eq<A: Clone, B: Clone>(fa: (A, B)) -> IsEq<(A, B)> {
    IsEq::equal_under_law(fa.clone().bimap(id, id), fa)
}

// ==================== Universalis Laws ====================

/// **First functor law**: `map_left` should be a valid functor
/// `x.map_left(f).map_left(g) == x.map_left(|a| g(f(a)))`
pub fn result_map_left_composition<A, B, C, E, F, G>(fa: Result<A, E>, f: F, g: G) -> bool
where
    A: Clone,
    B: Clone,
    C: Eq,
    E: Clone + Eq,
    F: FnOnce(A) -> B + Clone,
    G: FnOnce(B) -> C + Clone,
{
    let lhs = fa.clone().map_left(f.clone()).map_left(g.clone());
    let rhs = fa.map_left(|a| g(f(a)));
    lhs == rhs
}

/// **Second functor law**: `map_right` should be a valid functor
/// `x.map_right(f).map_right(g) == x.map_right(|b| g(f(b)))`
pub fn result_map_right_composition<A, E, E2, E3, F, G>(fa: Result<A, E>, f: F, g: G) -> bool
where
    A: Clone + Eq,
    E: Clone,
    E2: Clone,
    E3: Eq,
    F: FnOnce(E) -> E2 + Clone,
    G: FnOnce(E2) -> E3 + Clone,
{
    let lhs = fa.clone().map_right(f.clone()).map_right(g.clone());
    let rhs = fa.map_right(|e| g(f(e)));
    lhs == rhs
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    // ==================== Result Tests ====================

    #[test]
    fn test_result_identity_law() {
        fn test(fa: Result<i32, String>) -> bool {
            result_identity(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    #[test]
    fn test_result_map_left_identity_law() {
        fn test(fa: Result<i32, String>) -> bool {
            result_map_left_identity(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    #[test]
    fn test_result_map_right_identity_law() {
        fn test(fa: Result<i32, String>) -> bool {
            result_map_right_identity(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    #[test]
    fn test_result_composition_law() {
        fn test(fa: Result<i8, i8>) -> bool {
            result_composition(
                fa,
                |x: i8| x.wrapping_add(1),
                |e: i8| e.wrapping_mul(2),
                |x: i8| x.wrapping_mul(3),
                |e: i8| e.wrapping_add(10),
            )
        }
        quickcheck(test as fn(Result<i8, i8>) -> bool);
    }

    #[test]
    fn test_result_bimap_decomposition() {
        fn test(fa: Result<i8, i8>) -> bool {
            result_bimap_decomposition(fa, |x: i8| x.wrapping_add(1), |e: i8| e.wrapping_mul(2))
        }
        quickcheck(test as fn(Result<i8, i8>) -> bool);
    }

    #[test]
    fn test_result_map_left_composition() {
        fn test(fa: Result<i8, String>) -> bool {
            result_map_left_composition(fa, |x: i8| x.wrapping_add(1), |x: i8| x.wrapping_mul(2))
        }
        quickcheck(test as fn(Result<i8, String>) -> bool);
    }

    #[test]
    fn test_result_map_right_composition() {
        fn test(fa: Result<String, i8>) -> bool {
            result_map_right_composition(fa, |e: i8| e.wrapping_add(1), |e: i8| e.wrapping_mul(2))
        }
        quickcheck(test as fn(Result<String, i8>) -> bool);
    }

    // ==================== Tuple Tests ====================

    #[test]
    fn test_tuple_identity_law() {
        fn test(fa: (i32, String)) -> bool {
            tuple_identity(fa)
        }
        quickcheck(test as fn((i32, String)) -> bool);
    }

    #[test]
    fn test_tuple_map_left_identity_law() {
        fn test(fa: (i32, String)) -> bool {
            tuple_map_left_identity(fa)
        }
        quickcheck(test as fn((i32, String)) -> bool);
    }

    #[test]
    fn test_tuple_map_right_identity_law() {
        fn test(fa: (i32, String)) -> bool {
            tuple_map_right_identity(fa)
        }
        quickcheck(test as fn((i32, String)) -> bool);
    }

    #[test]
    fn test_tuple_composition_law() {
        fn test(fa: (i8, i8)) -> bool {
            tuple_composition(
                fa,
                |x: i8| x.wrapping_add(1),
                |y: i8| y.wrapping_mul(2),
                |x: i8| x.wrapping_mul(3),
                |y: i8| y.wrapping_add(10),
            )
        }
        quickcheck(test as fn((i8, i8)) -> bool);
    }

    #[test]
    fn test_tuple_bimap_decomposition() {
        fn test(fa: (i8, i8)) -> bool {
            tuple_bimap_decomposition(fa, |x: i8| x.wrapping_add(1), |y: i8| y.wrapping_mul(2))
        }
        quickcheck(test as fn((i8, i8)) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_result_identity_tests() {
        assert!(result_identity(Ok::<i32, String>(42)));
        assert!(result_identity(Err::<i32, String>("error".into())));
    }

    #[test]
    fn manual_tuple_identity_tests() {
        assert!(tuple_identity((42, "hello")));
        assert!(tuple_identity((0, "")));
    }

    #[test]
    fn manual_composition_tests() {
        // Result composition
        assert!(result_composition(
            Ok::<_, &str>(10),
            |x: i32| x + 1,
            |e: &str| e.len(),
            |x: i32| x * 2,
            |n: usize| n + 5,
        ));

        // Tuple composition
        assert!(tuple_composition(
            (10, 20),
            |x| x + 1,
            |y| y * 2,
            |x| x * 10,
            |y| y - 5,
        ));
    }

    #[test]
    fn manual_bimap_decomposition_tests() {
        // Result: bimap(f, g) == map_left(f).map_right(g)
        let result: Result<i32, &str> = Ok(42);
        let bimap_result = result.bimap(|x| x * 2, str::len);
        let decomposed = result.map_left(|x| x * 2).map_right(|e: &str| e.len());
        assert_eq!(bimap_result, decomposed);

        // Tuple: bimap(f, g) == map_left(f).map_right(g)
        let tuple = (10, "hello");
        let bimap_result = tuple.bimap(|x| x * 2, |s: &str| s.len());
        let decomposed = tuple.map_left(|x| x * 2).map_right(|s: &str| s.len());
        assert_eq!(bimap_result, decomposed);
    }

    #[test]
    fn test_identity_eq() {
        let eq = result_identity_eq(Ok::<_, String>(42));
        assert!(eq.holds());

        let eq = tuple_identity_eq((42, "hello"));
        assert!(eq.holds());
    }
}
