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
/// assert!(associativity(&vec![1], &vec![2], &vec![3]));
/// ```
pub fn associativity<A: Compositio + Eq>(a: &A, b: &A, c: &A) -> bool {
    a.combine(b).combine(c) == a.combine(&b.combine(c))
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
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(a: String, b: String, c: String) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(associativity_prop as fn(String, String, String) -> bool);
    }

    #[test]
    fn option_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(a: Option<String>, b: Option<String>, c: Option<String>) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(
            associativity_prop as fn(Option<String>, Option<String>, Option<String>) -> bool,
        );
    }

    #[test]
    fn vec_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(a: Vec<i8>, b: Vec<i8>, c: Vec<i8>) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(associativity_prop as fn(Vec<i8>, Vec<i8>, Vec<i8>) -> bool);
    }

    #[test]
    fn hashset_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(a: HashSet<i8>, b: HashSet<i8>, c: HashSet<i8>) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(associativity_prop as fn(HashSet<i8>, HashSet<i8>, HashSet<i8>) -> bool);
    }

    #[test]
    fn hashmap_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(
            a: HashMap<i8, String>,
            b: HashMap<i8, String>,
            c: HashMap<i8, String>,
        ) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(
            associativity_prop
                as fn(HashMap<i8, String>, HashMap<i8, String>, HashMap<i8, String>) -> bool,
        );
    }

    #[test]
    fn max_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(
            a: Wrapper<Max<i8>>,
            b: Wrapper<Max<i8>>,
            c: Wrapper<Max<i8>>,
        ) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(
            associativity_prop as fn(Wrapper<Max<i8>>, Wrapper<Max<i8>>, Wrapper<Max<i8>>) -> bool,
        );
    }

    #[test]
    fn min_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(
            a: Wrapper<Min<i8>>,
            b: Wrapper<Min<i8>>,
            c: Wrapper<Min<i8>>,
        ) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(
            associativity_prop as fn(Wrapper<Min<i8>>, Wrapper<Min<i8>>, Wrapper<Min<i8>>) -> bool,
        );
    }

    #[test]
    fn any_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(
            a: Wrapper<Aliquid<bool>>,
            b: Wrapper<Aliquid<bool>>,
            c: Wrapper<Aliquid<bool>>,
        ) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(
            associativity_prop
                as fn(
                    Wrapper<Aliquid<bool>>,
                    Wrapper<Aliquid<bool>>,
                    Wrapper<Aliquid<bool>>,
                ) -> bool,
        );
    }

    #[test]
    fn all_prop() {
        #[allow(
            clippy::needless_pass_by_value,
            reason = "quickcheck implements Testable only for fn items taking owned Arbitrary values"
        )]
        fn associativity_prop(
            a: Wrapper<Omnis<bool>>,
            b: Wrapper<Omnis<bool>>,
            c: Wrapper<Omnis<bool>>,
        ) -> bool {
            associativity(&a, &b, &c)
        }

        quickcheck(
            associativity_prop
                as fn(Wrapper<Omnis<bool>>, Wrapper<Omnis<bool>>, Wrapper<Omnis<bool>>) -> bool,
        );
    }
}
