//! # Compositio (Semigroup) Laws
//!
//! This module provides property-based laws for testing [`Compositio`] implementations.
//!
//! ## Law
//!
//! **Associativity**: `(x <> y) <> z == x <> (y <> z)`
//!
//! A Compositio is a set with an associative binary operation.
//!
//! ## Usage
//!
//! ```ignore
//! use ordofp_laws::semigroup_laws::associativity;
//! use quickcheck::quickcheck;
//!
//! quickcheck(associativity as fn(Vec<i8>, Vec<i8>, Vec<i8>) -> bool);
//! ```

use ordofp::semigroup::Compositio;

/// **Associativity Law**: The combine operation is associative.
///
/// ```text
/// (x <> y) <> z == x <> (y <> z)
/// ```
///
/// # Example
///
/// ```ignore
/// use ordofp_laws::semigroup_laws::associativity;
/// assert!(associativity(vec![1], vec![2], vec![3]));
/// ```
pub fn associativity<A: Compositio + Eq>(a: A, b: A, c: A) -> bool {
    a.combine(&b).combine(&c) == a.combine(&b.combine(&c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrapper::*;
    use ordofp::wrappers::{Aliquid, Max, Min, Omnis};
    use quickcheck::quickcheck;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn string_prop() {
        quickcheck(associativity as fn(String, String, String) -> bool);
    }

    #[test]
    fn option_prop() {
        quickcheck(associativity as fn(Option<String>, Option<String>, Option<String>) -> bool);
    }

    #[test]
    fn vec_prop() {
        quickcheck(associativity as fn(Vec<i8>, Vec<i8>, Vec<i8>) -> bool);
    }

    #[test]
    fn hashset_prop() {
        quickcheck(associativity as fn(HashSet<i8>, HashSet<i8>, HashSet<i8>) -> bool);
    }

    #[test]
    fn hashmap_prop() {
        quickcheck(
            associativity
                as fn(HashMap<i8, String>, HashMap<i8, String>, HashMap<i8, String>) -> bool,
        );
    }

    #[test]
    fn max_prop() {
        quickcheck(
            associativity as fn(Wrapper<Max<i8>>, Wrapper<Max<i8>>, Wrapper<Max<i8>>) -> bool,
        );
    }

    #[test]
    fn min_prop() {
        quickcheck(
            associativity as fn(Wrapper<Min<i8>>, Wrapper<Min<i8>>, Wrapper<Min<i8>>) -> bool,
        );
    }

    #[test]
    fn any_prop() {
        quickcheck(
            associativity
                as fn(
                    Wrapper<Aliquid<bool>>,
                    Wrapper<Aliquid<bool>>,
                    Wrapper<Aliquid<bool>>,
                ) -> bool,
        );
    }

    #[test]
    fn all_prop() {
        quickcheck(
            associativity
                as fn(Wrapper<Omnis<bool>>, Wrapper<Omnis<bool>>, Wrapper<Omnis<bool>>) -> bool,
        );
    }
}
