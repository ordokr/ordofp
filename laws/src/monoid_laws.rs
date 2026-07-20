//! # Unitas (Monoid) Laws
//!
//! This module provides property-based laws for testing [`Unitas`] implementations.
//!
//! A Unitas (Monoid) extends Compositio with an identity element.
//!
//! ## Laws
//!
//! 1. **Left Identity**: `empty <> x == x`
//! 2. **Right Identity**: `x <> empty == x`
//!
//! Note: Use [`semigroup_laws`](crate::semigroup_laws) to test associativity.
//!
//! ## Usage
//!
//! ```ignore
//! use ordofp_laws::monoid_laws::{left_identity, right_identity};
//! use quickcheck::quickcheck;
//!
//! quickcheck(left_identity as fn(String) -> bool);
//! quickcheck(right_identity as fn(String) -> bool);
//! ```

use ordofp::monoid::Unitas;

/// **Left Identity Law**: Empty combined with any value yields that value.
///
/// ```text
/// empty <> x == x
/// ```
///
/// # Example
///
/// ```ignore
/// use ordofp_laws::monoid_laws::left_identity;
/// assert!(left_identity("hello".to_string()));
/// ```
pub fn left_identity<A: Unitas + Eq>(a: A) -> bool {
    <A as Unitas>::empty().combine(&a) == a
}

/// **Right Identity Law**: Any value combined with empty yields that value.
///
/// ```text
/// x <> empty == x
/// ```
///
/// # Example
///
/// ```ignore
/// use ordofp_laws::monoid_laws::right_identity;
/// assert!(right_identity(vec![1, 2, 3]));
/// ```
pub fn right_identity<A: Unitas + Eq>(a: A) -> bool {
    a.combine(&<A as Unitas>::empty()) == a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrapper::*;
    use ordofp::semigroup::*;
    use quickcheck::quickcheck;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn string_id_prop() {
        quickcheck(left_identity as fn(String) -> bool);
        quickcheck(right_identity as fn(String) -> bool);
    }

    #[test]
    fn option_id_prop() {
        quickcheck(left_identity as fn(Option<String>) -> bool);
        quickcheck(right_identity as fn(Option<String>) -> bool);
    }

    #[test]
    fn vec_id_prop() {
        quickcheck(left_identity as fn(Vec<String>) -> bool);
        quickcheck(right_identity as fn(Vec<String>) -> bool);
    }

    #[test]
    fn hashset_id_prop() {
        quickcheck(left_identity as fn(HashSet<i32>) -> bool);
        quickcheck(right_identity as fn(HashSet<i32>) -> bool);
    }

    #[test]
    fn hashmap_id_prop() {
        quickcheck(left_identity as fn(HashMap<i32, String>) -> bool);
        quickcheck(right_identity as fn(HashMap<i32, String>) -> bool);
    }

    #[test]
    fn any_id_prop() {
        quickcheck(left_identity as fn(Wrapper<Aliquid<i32>>) -> bool);
        quickcheck(right_identity as fn(Wrapper<Aliquid<i32>>) -> bool);
    }

    #[test]
    fn all_id_prop() {
        quickcheck(left_identity as fn(Wrapper<Omnis<i32>>) -> bool);
        quickcheck(right_identity as fn(Wrapper<Omnis<i32>>) -> bool);
    }

    macro_rules! numeric_id_props {
      ($($id: ident; $tr:ty,)*) => {

        $(
            #[test]
            fn $id() {
                quickcheck(left_identity as fn($tr) -> bool);
                quickcheck(right_identity as fn($tr) -> bool);
            }
        )*
      }
    }

    numeric_id_props! {
        i8_id_prop; i8,
        product_i8_id_prop; Wrapper<Multiplicatio<i8>>,
        u8_id_prop; u8,
        product_u8_id_prop; Wrapper<Multiplicatio<u8>>,
        i16_id_prop; i16,
        product_i16_id_prop; Wrapper<Multiplicatio<i16>>,
        u16_id_prop; u16,
        product_u16_id_prop; Wrapper<Multiplicatio<u16>>,
        i32_id_prop; i32,
        product_i32_id_prop; Wrapper<Multiplicatio<i32>>,
        u32_id_prop; u32,
        product_u32_id_prop; Wrapper<Multiplicatio<u32>>,
        i64_id_prop; i64,
        product_i64_id_prop; Wrapper<Multiplicatio<i64>>,
        u64_id_prop; u64,
        product_u64_id_prop; Wrapper<Multiplicatio<u64>>,
        isize_id_prop; isize,
        product_isize_id_prop; Wrapper<Multiplicatio<isize>>,
        usize_id_prop; usize,
        product_usize_id_prop; Wrapper<Multiplicatio<usize>>,
    }
}
