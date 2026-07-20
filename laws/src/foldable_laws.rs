//! # Foldable Laws
//!
//! This module provides property-based laws for testing [`Foldable`] implementations.
//!
//! ## Laws
//!
//! Foldable has several derived consistency laws relating different operations:
//!
//! 1. **`fold_left/fold_right` consistency**: Both folds should produce the same
//!    result when the combining function is associative and the order doesn't matter.
//!
//! 2. **length consistency**: `length() == fold_left(0, |acc, _| acc + 1)`
//!
//! 3. **all/any duality**: `all(p) == !any(|x| !p(x))`
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::foldable_laws;
//!
//! // Test length consistency for Vec
//! assert!(foldable_laws::vec_length_consistency(vec![1, 2, 3, 4, 5]));
//!
//! // Test all/any duality
//! assert!(foldable_laws::vec_all_any_duality(vec![1, 2, 3, 4, 5], |&x| x > 0));
//! ```

use crate::is_eq::IsEq;
use ordofp::foldable::Foldable;

// ==================== Vec Laws ====================

/// **Length Consistency**: `length() == fold_left(0, |acc, _| acc + 1)`
pub fn vec_length_consistency<A>(fa: Vec<A>) -> bool {
    fa.length() == fa.fold_left(0, |acc, _| acc + 1)
}

/// **All/Any Duality**: `all(p) == !any(|x| !p(x))`
pub fn vec_all_any_duality<A, F>(fa: Vec<A>, mut pred: F) -> bool
where
    F: FnMut(&A) -> bool + Clone,
{
    let all_result = fa.all(pred.clone());
    let any_negated = !fa.any(|x| !pred(x));
    all_result == any_negated
}

/// **`is_empty` consistency**: `is_empty_foldable() == (length() == 0)`
pub fn vec_is_empty_consistency<A>(fa: Vec<A>) -> bool {
    fa.is_empty_foldable() == (fa.length() == 0)
}

/// **Fold left accumulates correctly**: folding with addition matches sum
pub fn vec_fold_left_sum(fa: Vec<i8>) -> bool {
    let folded = fa.fold_left(0i8, |acc, &x| acc.wrapping_add(x));
    let sum: i8 = fa.iter().fold(0i8, |acc, &x| acc.wrapping_add(x));
    folded == sum
}

/// **Fold right accumulates correctly**: folding with addition (for commutative ops)
pub fn vec_fold_right_sum(fa: Vec<i8>) -> bool {
    let folded = fa.fold_right(0i8, |&x, acc| x.wrapping_add(acc));
    let sum: i8 = fa.iter().fold(0i8, |acc, &x| acc.wrapping_add(x));
    folded == sum
}

/// **`contains_elem` consistency**: `contains_elem(x) == any(|e| e == x)`
pub fn vec_contains_consistency<A: PartialEq + Clone>(fa: Vec<A>, elem: A) -> bool {
    let contains = fa.contains_elem(&elem);
    let any_eq = fa.any(|e| e == &elem);
    contains == any_eq
}

/// Returns an [`IsEq`] for the length consistency law.
pub fn vec_length_consistency_eq<A>(fa: Vec<A>) -> IsEq<usize> {
    let computed_length = fa.fold_left(0, |acc, _| acc + 1);
    IsEq::equal_under_law(fa.length(), computed_length)
}

// ==================== Option Laws ====================

/// **Length Consistency**: `length() == fold_left(0, |acc, _| acc + 1)`
pub fn option_length_consistency<A>(fa: Option<A>) -> bool {
    fa.length() == fa.fold_left(0, |acc, _| acc + 1)
}

/// **`is_empty` consistency**: `is_empty_foldable() == (length() == 0)`
pub fn option_is_empty_consistency<A>(fa: Option<A>) -> bool {
    fa.is_empty_foldable() == (fa.length() == 0)
}

/// **Fold left/right equivalence for single element**:
/// For Option, `fold_left` and `fold_right` should give the same result
pub fn option_fold_equivalence<A: Clone + Eq>(fa: Option<A>, init: A) -> bool {
    let left = fa.fold_left(init.clone(), |_, x| x.clone());
    let right = fa.fold_right(init, |x, _| x.clone());
    left == right
}

/// Returns an [`IsEq`] for the Option length consistency law.
pub fn option_length_consistency_eq<A>(fa: Option<A>) -> IsEq<usize> {
    let computed_length = fa.fold_left(0, |acc, _| acc + 1);
    IsEq::equal_under_law(fa.length(), computed_length)
}

// ==================== Slice Laws ====================

/// **Length Consistency for slices**: `length() == fold_left(0, |acc, _| acc + 1)`
pub fn slice_length_consistency<A>(fa: &[A]) -> bool {
    fa.length() == fa.fold_left(0, |acc, _| acc + 1)
}

/// **`is_empty` consistency for slices**: `is_empty_foldable() == (length() == 0)`
pub fn slice_is_empty_consistency<A>(fa: &[A]) -> bool {
    fa.is_empty_foldable() == (fa.length() == 0)
}

// ==================== Result Laws ====================

/// **Length Consistency**: `length() == fold_left(0, |acc, _| acc + 1)`
pub fn result_length_consistency<A, E>(fa: Result<A, E>) -> bool {
    fa.length() == fa.fold_left(0, |acc, _| acc + 1)
}

/// **`is_empty` consistency**: `is_empty_foldable() == (length() == 0)`
pub fn result_is_empty_consistency<A, E>(fa: Result<A, E>) -> bool {
    fa.is_empty_foldable() == (fa.length() == 0)
}

/// Returns an [`IsEq`] for the Result length consistency law.
pub fn result_length_consistency_eq<A, E>(fa: Result<A, E>) -> IsEq<usize> {
    let computed_length = fa.fold_left(0, |acc, _| acc + 1);
    IsEq::equal_under_law(fa.length(), computed_length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_length_consistency() {
        quickcheck(vec_length_consistency::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_is_empty_consistency() {
        quickcheck(vec_is_empty_consistency::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_fold_left_sum() {
        quickcheck(vec_fold_left_sum as fn(Vec<i8>) -> bool);
    }

    #[test]
    fn test_vec_fold_right_sum() {
        quickcheck(vec_fold_right_sum as fn(Vec<i8>) -> bool);
    }

    #[test]
    fn test_vec_all_any_duality() {
        fn test(fa: Vec<i32>) -> bool {
            vec_all_any_duality(fa, |&x| x > 0)
        }
        quickcheck(test as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_contains_consistency() {
        fn test(fa: Vec<i32>, elem: i32) -> bool {
            vec_contains_consistency(fa, elem)
        }
        quickcheck(test as fn(Vec<i32>, i32) -> bool);
    }

    // ==================== Option Tests ====================

    #[test]
    fn test_option_length_consistency() {
        quickcheck(option_length_consistency::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_is_empty_consistency() {
        quickcheck(option_is_empty_consistency::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_fold_equivalence() {
        fn test(fa: Option<i32>, init: i32) -> bool {
            option_fold_equivalence(fa, init)
        }
        quickcheck(test as fn(Option<i32>, i32) -> bool);
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_length_consistency() {
        fn test(fa: Result<i32, String>) -> bool {
            result_length_consistency(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    #[test]
    fn test_result_is_empty_consistency() {
        fn test(fa: Result<i32, String>) -> bool {
            result_is_empty_consistency(fa)
        }
        quickcheck(test as fn(Result<i32, String>) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_vec_length_tests() {
        assert!(vec_length_consistency(vec![1, 2, 3, 4, 5]));
        assert!(vec_length_consistency(Vec::<i32>::new()));
        assert!(vec_length_consistency(vec!["a", "b", "c"]));
    }

    #[test]
    fn manual_vec_fold_tests() {
        // Test that fold_left and fold_right with addition give same result
        let v = vec![1, 2, 3, 4, 5];
        let left_sum = v.fold_left(0, |acc, &x| acc + x);
        let right_sum = v.fold_right(0, |&x, acc| x + acc);
        assert_eq!(left_sum, right_sum);
        assert_eq!(left_sum, 15);
    }

    #[test]
    fn manual_vec_all_any_tests() {
        let v = vec![2, 4, 6, 8];
        assert!(vec_all_any_duality(v.clone(), |&x| x % 2 == 0));
        assert!(vec_all_any_duality(v.clone(), |&x| x > 0));
        assert!(vec_all_any_duality(v.clone(), |&x| x > 10));
    }

    #[test]
    fn manual_option_tests() {
        assert!(option_length_consistency(Some(42)));
        assert!(option_length_consistency(None::<i32>));
        assert!(option_is_empty_consistency(Some(42)));
        assert!(option_is_empty_consistency(None::<i32>));
    }

    #[test]
    fn manual_result_tests() {
        assert!(result_length_consistency(Ok::<_, &str>(42)));
        assert!(result_length_consistency(Err::<i32, _>("error")));
        assert!(result_is_empty_consistency(Ok::<_, &str>(42)));
        assert!(result_is_empty_consistency(Err::<i32, _>("error")));
    }

    #[test]
    fn test_slice_laws() {
        let arr = [1, 2, 3, 4, 5];
        assert!(slice_length_consistency(&arr));
        assert!(slice_is_empty_consistency(&arr));
        assert!(slice_is_empty_consistency(&[] as &[i32]));
    }

    #[test]
    fn test_identity_eq() {
        let eq = vec_length_consistency_eq(vec![1, 2, 3]);
        assert!(eq.holds());

        let eq = option_length_consistency_eq(Some(42));
        assert!(eq.holds());

        let eq = result_length_consistency_eq(Ok::<_, String>(42));
        assert!(eq.holds());
    }
}
