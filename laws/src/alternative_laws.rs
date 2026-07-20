//! # Alternative Laws
//!
//! This module provides property-based laws for testing [`Alternative`] implementations.
//!
//! ## Laws
//!
//! 1. **Left Identity**: `empty().alt(&x) == x`
//! 2. **Right Identity**: `x.alt(&empty()) == x`
//! 3. **Associativity**: `a.alt(&b).alt(&c) == a.alt(&b.alt(&c))`
//!
//! ## Usage
//!
//! ```
//! use ordofp_laws::alternative_laws;
//!
//! // Test left identity law for Option
//! assert!(alternative_laws::option_left_identity(Some(42)));
//! assert!(alternative_laws::option_left_identity(None::<i32>));
//!
//! // Test associativity
//! assert!(alternative_laws::option_associativity(Some(1), Some(2), Some(3)));
//! ```

use crate::is_eq::IsEq;
use ordofp::alternative::Alternative;

// ==================== Option Laws ====================

/// **Left Identity Law** for Option: `empty().alt(&x) == x`
pub fn option_left_identity<A: Clone + Eq + Default>(x: Option<A>) -> bool {
    Option::<A>::empty().alt(&x) == x
}

/// **Right Identity Law** for Option: `x.alt(&empty()) == x`
pub fn option_right_identity<A: Clone + Eq + Default>(x: Option<A>) -> bool {
    x.clone().alt(&Option::empty()) == x
}

/// **Associativity Law** for Option: `a.alt(&b).alt(&c) == a.alt(&b.alt(&c))`
pub fn option_associativity<A: Clone + Eq + Default>(
    a: Option<A>,
    b: Option<A>,
    c: Option<A>,
) -> bool {
    a.clone().alt(&b).alt(&c) == a.alt(&b.alt(&c))
}

/// Returns an [`IsEq`] for the Option left identity law.
pub fn option_left_identity_eq<A: Clone + Default>(x: Option<A>) -> IsEq<Option<A>> {
    IsEq::equal_under_law(Option::<A>::empty().alt(&x), x)
}

/// Returns an [`IsEq`] for the Option right identity law.
pub fn option_right_identity_eq<A: Clone + Default>(x: Option<A>) -> IsEq<Option<A>> {
    IsEq::equal_under_law(x.clone().alt(&Option::empty()), x)
}

// ==================== Vec Laws ====================

/// **Left Identity Law** for Vec: `empty().alt(&x) == x`
pub fn vec_left_identity<A: Clone + Eq + Default>(x: Vec<A>) -> bool {
    Vec::<A>::empty().alt(&x) == x
}

/// **Right Identity Law** for Vec: `x.alt(&empty()) == x`
pub fn vec_right_identity<A: Clone + Eq + Default>(x: Vec<A>) -> bool {
    x.clone().alt(&Vec::empty()) == x
}

/// **Associativity Law** for Vec: `a.alt(&b).alt(&c) == a.alt(&b.alt(&c))`
pub fn vec_associativity<A: Clone + Eq + Default>(a: Vec<A>, b: Vec<A>, c: Vec<A>) -> bool {
    a.clone().alt(&b).alt(&c) == a.alt(&b.alt(&c))
}

/// Returns an [`IsEq`] for the Vec left identity law.
pub fn vec_left_identity_eq<A: Clone + Default>(x: Vec<A>) -> IsEq<Vec<A>> {
    IsEq::equal_under_law(Vec::<A>::empty().alt(&x), x)
}

/// Returns an [`IsEq`] for the Vec right identity law.
pub fn vec_right_identity_eq<A: Clone + Default>(x: Vec<A>) -> IsEq<Vec<A>> {
    IsEq::equal_under_law(x.clone().alt(&Vec::empty()), x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    // ==================== Option Tests ====================

    #[test]
    fn test_option_left_identity_law() {
        quickcheck(option_left_identity::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_right_identity_law() {
        quickcheck(option_right_identity::<i32> as fn(Option<i32>) -> bool);
    }

    #[test]
    fn test_option_associativity_law() {
        fn test(a: Option<i32>, b: Option<i32>, c: Option<i32>) -> bool {
            option_associativity(a, b, c)
        }
        quickcheck(test as fn(Option<i32>, Option<i32>, Option<i32>) -> bool);
    }

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_left_identity_law() {
        quickcheck(vec_left_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_right_identity_law() {
        quickcheck(vec_right_identity::<i32> as fn(Vec<i32>) -> bool);
    }

    #[test]
    fn test_vec_associativity_law() {
        fn test(a: Vec<i32>, b: Vec<i32>, c: Vec<i32>) -> bool {
            vec_associativity(a, b, c)
        }
        quickcheck(test as fn(Vec<i32>, Vec<i32>, Vec<i32>) -> bool);
    }

    // ==================== Manual Tests ====================

    #[test]
    fn manual_left_identity_tests() {
        // Option with Some
        assert!(option_left_identity(Some(42)));
        // Option with None
        assert!(option_left_identity(None::<i32>));
        // Vec non-empty
        assert!(vec_left_identity(vec![1, 2, 3]));
        // Vec empty
        assert!(vec_left_identity(Vec::<i32>::new()));
    }

    #[test]
    fn manual_right_identity_tests() {
        // Option
        assert!(option_right_identity(Some(42)));
        assert!(option_right_identity(None::<i32>));
        // Vec
        assert!(vec_right_identity(vec![1, 2, 3]));
        assert!(vec_right_identity(Vec::<i32>::new()));
    }

    #[test]
    fn manual_associativity_tests() {
        // Option all Some
        assert!(option_associativity(Some(1), Some(2), Some(3)));
        // Option mixed
        assert!(option_associativity(None, Some(2), Some(3)));
        assert!(option_associativity(Some(1), None, Some(3)));
        assert!(option_associativity(Some(1), Some(2), None));
        // Option all None
        assert!(option_associativity(None::<i32>, None, None));
        // Vec
        assert!(vec_associativity(vec![1], vec![2], vec![3]));
        assert!(vec_associativity(Vec::<i32>::new(), vec![2], vec![3]));
    }

    #[test]
    fn test_identity_eq() {
        let eq = option_left_identity_eq(Some(42));
        assert!(eq.holds());

        let eq = option_right_identity_eq(Some(42));
        assert!(eq.holds());

        let eq = vec_left_identity_eq(vec![1, 2, 3]);
        assert!(eq.holds());
    }
}
