//! # Traversable Laws
//!
//! This module provides property-based laws for testing [`Traversable`] implementations.
//!
//! ## Laws
//!
//! 1. **Identity**: `traverse(pure) == pure` (traversing with pure is identity)
//! 2. **Naturality**: `t . traverse(f) == traverse(t . f)` for applicative morphism `t`
//! 3. **Composition**: For nested effects, traversal distributes
//!
//! ## Consistency Laws
//!
//! 4. **traverse/sequence consistency**: `traverse(f) == sequence . fmap(f)`
//! 5. **sequence/traverse consistency**: `sequence == traverse(id)`
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::traversable_laws;
//!
//! // Test identity law for Vec
//! assert!(traversable_laws::vec_traverse_identity(vec![1, 2, 3]));
//!
//! // Test sequence_option consistency
//! assert!(traversable_laws::vec_sequence_option_consistency(vec![Some(1), Some(2), Some(3)]));
//! ```

use crate::is_eq::IsEq;
use ordofp::traversable::Traversable;

// ==================== Vec Laws ====================

/// **Identity Law** for Vec: `traverse(Some) == Some`
///
/// Traversing with pure (Some) should return Some of the same structure.
pub fn vec_traverse_identity<A: Clone + Eq>(fa: Vec<A>) -> bool {
    let traversed = fa.traverse_option(|a| Some(a.clone()));
    traversed == Some(fa)
}

/// **Sequence consistency**: `sequence(map(Some, xs)) == Some(xs)`
pub fn vec_sequence_option_identity<A: Clone + Eq>(fa: Vec<A>) -> bool {
    let mapped: Vec<Option<A>> = fa.iter().map(|a| Some(a.clone())).collect();
    let sequenced: Option<Vec<A>> = ordofp::traversable::sequence_option(mapped);
    sequenced == Some(fa)
}

/// **Traverse with Result identity**: `traverse(Ok) == Ok`
pub fn vec_traverse_result_identity<A: Clone + Eq>(fa: Vec<A>) -> bool {
    let traversed: Result<Vec<A>, ()> = fa.traverse_result(|a| Ok::<_, ()>(a.clone()));
    traversed == Ok(fa)
}

/// **Sequence with all Some**: sequence preserves structure
pub fn vec_sequence_option_consistency<A: Clone + Eq>(fa: Vec<Option<A>>) -> bool {
    // If traverse_option_owned with identity == sequence_option
    let sequenced: Option<Vec<A>> = ordofp::traversable::sequence_option(fa.clone());
    let traversed: Option<Vec<A>> =
        ordofp::traversable::traverse_option(fa, core::convert::identity);
    sequenced == traversed
}

/// **Sequence with all Ok**: sequence preserves structure
pub fn vec_sequence_result_consistency<A: Clone + Eq, E: Clone + Eq>(
    fa: Vec<Result<A, E>>,
) -> bool {
    let sequenced: Result<Vec<A>, E> = ordofp::traversable::sequence_result(fa.clone());
    let traversed: Result<Vec<A>, E> =
        ordofp::traversable::traverse_result(fa, core::convert::identity);
    sequenced == traversed
}

/// **Empty traversal**: traverse over empty returns empty in effect
pub fn vec_traverse_empty_option() -> bool {
    let empty: Vec<i32> = Vec::new();
    let result = empty.traverse_option(|_| None::<i32>);
    result == Some(Vec::new())
}

/// **Short-circuit on None**: traverse returns None if any element fails
pub fn vec_traverse_option_short_circuit<A: Clone + Eq>(fa: Vec<A>, fail_at: usize) -> bool {
    if fa.is_empty() {
        return true; // nothing to fail on
    }
    // Clamp into range so every non-empty input genuinely fails once.
    let fail_at = fail_at % fa.len();

    // Track the position with a counter (a `position(|x| x == a)` lookup
    // would misfire on duplicate values).
    let idx = core::cell::Cell::new(0usize);
    let result = fa.traverse_option(|a| {
        let here = idx.get();
        idx.set(here + 1);
        if here == fail_at {
            None
        } else {
            Some(a.clone())
        }
    });

    result.is_none()
}

/// Returns an [`IsEq`] for the Vec identity law.
pub fn vec_traverse_identity_eq<A: Clone>(fa: Vec<A>) -> IsEq<Option<Vec<A>>> {
    let traversed = fa.traverse_option(|a| Some(a.clone()));
    IsEq::equal_under_law(traversed, Some(fa))
}

// ==================== Option Laws ====================

/// **Identity Law** for Option: `traverse(Some) == Some`
pub fn option_traverse_identity<A: Clone + Eq>(fa: Option<A>) -> bool {
    let traversed = fa.traverse_option(|a| Some(a.clone()));
    traversed == Some(fa)
}

/// **Traverse with None input**: `traverse(f, None) == Some(None)`
pub fn option_traverse_none_input<B: Eq>() -> bool {
    let none: Option<i32> = None;
    let result: Option<Option<B>> = none.traverse_option(|_| None::<B>);
    result == Some(None)
}

/// **Sequence consistency for Option**: `sequence == traverse(identity)`
///
/// The reference `sequence` is hand-rolled from the categorical definition
/// (commute the inner effect outward: `None -> Some(None)`,
/// `Some(inner) -> inner.map(Some)`), and compared against the library's
/// `traverse_option_owned` with the identity function — two genuinely
/// different code paths computing the same distribution law.
pub fn option_sequence_option_consistency<A: Clone + Eq>(fa: Option<Option<A>>) -> bool {
    // Reference implementation of sequence :: Option<Option<A>> -> Option<Option<A>>
    // (outer = effect, inner = structure).
    let sequenced: Option<Option<A>> = match fa.clone() {
        None => Some(None),             // empty structure: pure(None)
        Some(inner) => inner.map(Some), // commute the effect outward
    };
    // Library path: sequence == traverse(identity).
    let traversed: Option<Option<A>> =
        ordofp::traversable::Traversable::traverse_option_owned(fa, core::convert::identity);
    sequenced == traversed
}

/// Returns an [`IsEq`] for the Option identity law.
pub fn option_traverse_identity_eq<A: Clone>(fa: Option<A>) -> IsEq<Option<Option<A>>> {
    let traversed = fa.traverse_option(|a| Some(a.clone()));
    IsEq::equal_under_law(traversed, Some(fa))
}

// ==================== Result Laws ====================

/// **Identity Law** for Result: `traverse(Some, Ok(x)) == Some(Ok(x))`
pub fn result_traverse_identity<A: Clone + Eq, E: Clone + Eq>(fa: Result<A, E>) -> bool {
    let traversed = fa.traverse_option(|a| Some(a.clone()));
    match (&traversed, &fa) {
        (Some(Ok(a)), Ok(b)) => a == b,
        (Some(Err(e1)), Err(e2)) => e1 == e2,
        _ => false,
    }
}

/// **Traverse Err**: `traverse(f, Err(e)) == Some(Err(e))`
pub fn result_traverse_err_passthrough<A: Eq, E: Clone + Eq, F, B>(fa: Result<A, E>, _f: F) -> bool
where
    F: Fn(&A) -> Option<B>,
{
    if let Err(err) = fa {
        let traversed: Option<Result<A, E>> =
            Err::<A, E>(err.clone()).traverse_option(|_| None::<A>);
        traversed == Some(Err(err))
    } else {
        true // Only testing Err case
    }
}

/// Returns an [`IsEq`] for the Result identity law.
pub fn result_traverse_identity_eq<A: Clone, E: Clone>(
    fa: Result<A, E>,
) -> IsEq<Option<Result<A, E>>> {
    let traversed = fa.traverse_option(|a| Some(a.clone()));
    IsEq::equal_under_law(traversed, Some(fa))
}

// ==================== Functor-Traversable Consistency ====================

/// **Traverse/map consistency**: `traverse(Some . f) == Some . map(f)`
pub fn vec_traverse_map_consistency<A: Clone, B: Clone + Eq, F>(fa: Vec<A>, f: F) -> bool
where
    F: Fn(&A) -> B + Clone,
{
    let f2 = f.clone();
    let traversed = fa.traverse_option(|a| Some(f2(a)));
    let mapped: Vec<B> = fa.iter().map(f).collect();
    traversed == Some(mapped)
}

/// **Option traverse/map consistency**
pub fn option_traverse_map_consistency<A: Clone, B: Clone + Eq, F>(fa: Option<A>, f: F) -> bool
where
    F: Fn(&A) -> B + Clone,
{
    let traversed = fa.traverse_option(|a| Some(f(a)));
    let mapped: Option<B> = fa.as_ref().map(f);
    traversed == Some(mapped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_traverse_identity() {
        quickcheck(vec_traverse_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_sequence_option_identity() {
        quickcheck(vec_sequence_option_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_traverse_result_identity() {
        quickcheck(vec_traverse_result_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_sequence_option_consistency() {
        fn test(fa: Vec<Option<i32>>) -> bool {
            vec_sequence_option_consistency(fa)
        }
        quickcheck(test as fn(Vec<Option<i32>>) -> bool);
    }

    #[test]
    fn test_vec_sequence_result_consistency() {
        fn test(fa: Vec<Result<i32, String>>) -> bool {
            vec_sequence_result_consistency(fa)
        }
        quickcheck(test as fn(Vec<Result<i32, String>>) -> bool);
    }

    #[test]
    fn test_vec_traverse_empty_option() {
        assert!(vec_traverse_empty_option());
    }

    #[test]
    fn test_vec_traverse_map_consistency() {
        fn test(fa: Vec<i8>) -> bool {
            vec_traverse_map_consistency(fa, |x| x.wrapping_mul(2))
        }
        quickcheck(test as fn(Vec<i8>) -> bool);
    }

    #[test]
    fn test_vec_traverse_option_short_circuit() {
        // Property: failing at any (clamped) position makes traverse None —
        // duplicates included.
        fn test(fa: Vec<i32>, fail_at: usize) -> bool {
            vec_traverse_option_short_circuit(fa, fail_at)
        }
        quickcheck(test as fn(Vec<i32>, usize) -> bool);

        // Deterministic spot checks, including duplicate values.
        assert!(vec_traverse_option_short_circuit(vec![1, 1, 1], 2));
        assert!(vec_traverse_option_short_circuit(vec![5], 0));
        assert!(vec_traverse_option_short_circuit(Vec::<i32>::new(), 3)); // vacuous
    }

    // ==================== Option Tests ====================

    #[test]
    fn test_option_traverse_identity() {
        quickcheck(option_traverse_identity::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_traverse_none_input() {
        assert!(option_traverse_none_input::<i32>());
    }

    #[test]
    #[allow(clippy::option_option)] // sequencing Option<Option<_>> is the law's shape
    fn test_option_sequence_option_consistency() {
        fn test(fa: Option<Option<i32>>) -> bool {
            option_sequence_option_consistency(fa)
        }
        quickcheck(test as fn(Option<Option<i32>>) -> bool);
    }

    #[test]
    fn test_option_traverse_map_consistency() {
        fn test(fa: Option<i8>) -> bool {
            option_traverse_map_consistency(fa, |x| x.wrapping_mul(2))
        }
        quickcheck(test as fn(Option<i8>) -> bool);
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_traverse_identity() {
        fn test(fa: Result<i32, String>) -> bool {
            result_traverse_identity(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_vec_tests() {
        // Identity law
        assert!(vec_traverse_identity(vec![1, 2, 3]));
        assert!(vec_traverse_identity(Vec::<i32>::new()));

        // Sequence with all Some
        assert!(vec_sequence_option_consistency(vec![
            Some(1),
            Some(2),
            Some(3)
        ]));

        // Sequence with None using helper function
        let with_none = vec![Some(1), None, Some(3)];
        let sequenced: Option<Vec<i32>> = ordofp::traversable::sequence_option(with_none);
        assert_eq!(sequenced, None);
    }

    #[test]
    fn manual_option_tests() {
        // Identity law
        assert!(option_traverse_identity(Some(42)));
        assert!(option_traverse_identity(None::<i32>));

        // For Option<Option<A>>, sequence flattens: Some(Some(x)) -> Some(x)
        let nested = Some(Some(42));
        let sequenced = nested.and_then(core::convert::identity);
        assert_eq!(sequenced, Some(42));

        // Sequence Some(None) -> None
        let nested: Option<Option<i32>> = Some(None);
        let sequenced = nested.and_then(core::convert::identity);
        assert_eq!(sequenced, None);

        // Sequence None -> None
        let nested: Option<Option<i32>> = None;
        let sequenced = nested.and_then(core::convert::identity);
        assert_eq!(sequenced, None);
    }

    #[test]
    fn manual_result_tests() {
        // Identity law
        assert!(result_traverse_identity(Ok::<i32, String>(42)));
        assert!(result_traverse_identity(Err::<i32, String>("error".into())));

        // Err passthrough
        let err: Result<i32, &str> = Err("error");
        let traversed = err.traverse_option(|x| Some(x * 2));
        assert_eq!(traversed, Some(Err("error")));
    }

    #[test]
    fn test_traverse_short_circuit() {
        // Verify that traverse returns None when encountering None
        let v = vec![1, 2, 3, 4, 5];
        let result = v.traverse_option(|&x| if x == 3 { None } else { Some(x) });

        assert_eq!(result, None);
        // Note: In eager evaluation, all elements may still be visited
        // The key property is the result being None
    }

    #[test]
    fn test_identity_eq() {
        let eq = vec_traverse_identity_eq(vec![1, 2, 3]);
        assert!(eq.holds());

        let eq = option_traverse_identity_eq(Some(42));
        assert!(eq.holds());

        let eq = result_traverse_identity_eq(Ok::<_, String>(42));
        assert!(eq.holds());
    }

    #[test]
    fn test_complex_traversals() {
        // Parse strings to integers
        let strings = vec!["1", "2", "3"];
        let parsed: Option<Vec<i32>> = strings.traverse_option(|s| s.parse().ok());
        assert_eq!(parsed, Some(vec![1, 2, 3]));

        // With a parse failure
        let strings = vec!["1", "two", "3"];
        let parsed: Option<Vec<i32>> = strings.traverse_option(|s| s.parse().ok());
        assert_eq!(parsed, None);

        // With Result
        let strings = vec!["1", "2", "3"];
        let parsed: Result<Vec<i32>, _> = strings.traverse_result(|s| s.parse::<i32>());
        assert_eq!(parsed, Ok(vec![1, 2, 3]));
    }
}
