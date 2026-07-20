//! Unitas (Monoid) type class - Compositio with an identity element.
//!
//! > *\"Unum convertitur cum ente.\"*
//! > — The One is convertible with Being. (Aquinas, ST I, q.11, a.1)
//!
//! A `Unitas` is a `Compositio` with an `empty` value that acts as identity:
//! - `empty().combine(&x) == x`
//! - `x.combine(&empty()) == x`
//!
//! The name derives from Aquinas's treatment of *unum* (unity) as a
//! transcendental property of being, the principle from which all
//! multiplicity proceeds.
//!
//! # Examples
//!
//! ```rust
//! use ordofp_core::typeclasses::{Unitas, combine_all};
//!
//! // combine_all uses empty() as the starting value
//! assert_eq!(combine_all(&[1, 2, 3, 4]), 10);
//! assert_eq!(combine_all(&[] as &[i32]), 0);  // empty for i32 is 0
//!
//! // Unitas::empty() returns the identity element
//! assert_eq!(i32::empty(), 0);
//! assert_eq!(String::empty(), "");
//! ```

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

use super::Compositio;
use crate::wrappers::{Aggregatio, Aliquid, Multiplicatio, Omnis};

/// Unitas - a Compositio with an identity element.
///
/// > *\"Numerus est multitudo ex unitatibus constituta.\"*
/// > — Number is a multitude constituted from unities. (Euclid, via Boethius)
///
/// Named after the scholastic transcendental *unum* (unity), the principle
/// from which multiplicity proceeds. This is the Rust equivalent of Haskell's `Monoid`.
///
/// # Laws
///
/// **Left identity** (*Lex Identitatis Sinistrae*): `empty().combine(&x) == x`
/// **Right identity** (*Lex Identitatis Dextrae*): `x.combine(&empty()) == x`
/// **Associativity** (*Lex Associationis*): (inherited from Compositio)
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::Unitas;
///
/// assert_eq!(i32::empty(), 0);
/// assert_eq!(String::empty(), "");
/// ```
pub trait Unitas: Compositio {
    /// Returns the identity element for this Unitas.
    ///
    /// The identity element (*elementum identitatis*) satisfies:
    /// - `empty().combine(&x) == x`
    /// - `x.combine(&empty()) == x`
    fn empty() -> Self;

    /// Combine all elements in a slice, using `empty()` for empty slices.
    ///
    /// This method can be overridden for performance optimization.
    /// For example, `String` and `Vec` implementations use `with_capacity`
    /// to avoid repeated reallocations.
    #[inline]
    fn combine_all(xs: &[Self]) -> Self
    where
        Self: Clone,
    {
        xs.iter().fold(Self::empty(), |acc, x| acc.combine(x))
    }
}

// ============================================================================
// Numeric implementations
// ============================================================================

macro_rules! numeric_monoid_impl {
    ($zero:expr; $($ty:ty),*) => {
        $(
            impl Unitas for $ty {
                #[inline]
                fn empty() -> Self {
                    $zero
                }
            }
        )*
    };
}

numeric_monoid_impl!(0; i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// f32 and f64 implementations with floating-point associativity caveat
impl Unitas for f32 {
    /// NOTE: floating-point combine is only approximately associative (rounding); the Semigroup laws hold up to epsilon, not exactly.
    #[inline]
    fn empty() -> Self {
        0.0
    }
}

impl Unitas for f64 {
    /// NOTE: floating-point combine is only approximately associative (rounding); the Semigroup laws hold up to epsilon, not exactly.
    #[inline]
    fn empty() -> Self {
        0.0
    }
}

// ============================================================================
// Wrapper type implementations
// ============================================================================

// Multiplicatio: identity is 1
macro_rules! multiplicatio_monoid_impl {
    ($one:expr; $($ty:ty),*) => {
        $(
            impl Unitas for Multiplicatio<$ty> {
                #[inline]
                fn empty() -> Self {
                    Multiplicatio($one)
                }
            }
        )*
    };
}

multiplicatio_monoid_impl!(1; i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
multiplicatio_monoid_impl!(1.0; f32, f64);

// Aggregatio: identity is 0
macro_rules! aggregatio_monoid_impl {
    ($zero:expr; $($ty:ty),*) => {
        $(
            impl Unitas for Aggregatio<$ty> {
                #[inline]
                fn empty() -> Self {
                    Aggregatio($zero)
                }
            }
        )*
    };
}

aggregatio_monoid_impl!(0; i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
aggregatio_monoid_impl!(0.0; f32, f64);

// Omnis<bool>: identity is true
impl Unitas for Omnis<bool> {
    #[inline]
    fn empty() -> Self {
        Omnis(true)
    }
}

// Omnis<numeric>: identity is all 1s
macro_rules! omnis_numeric_monoid_impl {
    ($($ty:ty),*) => {
        $(
            impl Unitas for Omnis<$ty> {
                #[inline]
                fn empty() -> Self {
                    Omnis(!0)  // All bits set to 1
                }
            }
        )*
    };
}

omnis_numeric_monoid_impl!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

// Aliquid<bool>: identity is false
impl Unitas for Aliquid<bool> {
    #[inline]
    fn empty() -> Self {
        Aliquid(false)
    }
}

// Aliquid<numeric>: identity is 0
macro_rules! aliquid_numeric_monoid_impl {
    ($($ty:ty),*) => {
        $(
            impl Unitas for Aliquid<$ty> {
                #[inline]
                fn empty() -> Self {
                    Aliquid(0)
                }
            }
        )*
    };
}

aliquid_numeric_monoid_impl!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

// Note: Max, Min, First, Last don't have natural Unitas instances without bounds
// Max would need T::MIN, Min would need T::MAX

// ============================================================================
// Standard type implementations
// ============================================================================

impl<T: Compositio + Clone> Unitas for Option<T> {
    #[inline]
    fn empty() -> Self {
        None
    }

    /// Optimized implementation for `Option` that avoids wrapping/unwrapping
    /// `Some` at every step of the fold. It also skips `None` values efficiently.
    #[inline]
    fn combine_all(xs: &[Self]) -> Self {
        let mut iter = xs.iter().flatten();
        match iter.next() {
            None => None,
            Some(first) => {
                let first = first.clone();
                Some(iter.fold(first, |acc, x| acc.combine(x)))
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl Unitas for String {
    #[inline]
    fn empty() -> Self {
        String::new()
    }

    /// Optimized implementation for `String` that pre-calculates the total length
    /// and reserves capacity to avoid repeated reallocations during concatenation.
    #[inline]
    fn combine_all(xs: &[Self]) -> Self {
        let total_len = xs.iter().map(String::len).sum();
        let mut result = String::with_capacity(total_len);
        for x in xs {
            result.push_str(x);
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<T: Clone> Unitas for Vec<T> {
    #[inline]
    fn empty() -> Self {
        Vec::new()
    }

    /// Optimized implementation for `Vec` that pre-calculates the total length
    /// and reserves capacity to avoid repeated reallocations.
    #[inline]
    fn combine_all(xs: &[Self]) -> Self {
        let total_len = xs.iter().map(Vec::len).sum();
        let mut result = Vec::with_capacity(total_len);
        for x in xs {
            result.extend_from_slice(x);
        }
        result
    }
}

#[cfg(feature = "std")]
impl<T: Eq + Hash + Clone> Unitas for HashSet<T> {
    #[inline]
    fn empty() -> Self {
        HashSet::new()
    }
}

#[cfg(feature = "std")]
impl<K: Eq + Hash + Clone, V: Compositio + Clone> Unitas for HashMap<K, V> {
    #[inline]
    fn empty() -> Self {
        HashMap::new()
    }
}

// ============================================================================
// Tuple implementations
// ============================================================================

macro_rules! tuple_monoid_impl {
    () => {};
    (($idx:tt => $typ:ident), $(($nidx:tt => $ntyp:ident),)*) => {
        tuple_monoid_impl!([($idx, $typ);] $(($nidx => $ntyp),)*);
        tuple_monoid_impl!($(($nidx => $ntyp),)*);
    };
    ([$(($accIdx:tt, $accTyp:ident);)+] ($idx:tt => $typ:ident), $(($nidx:tt => $ntyp:ident),)*) => {
        tuple_monoid_impl!([($idx, $typ); $(($accIdx, $accTyp);)*] $(($nidx => $ntyp),)*);
    };
    ([($idx:tt, $typ:ident); $(($nidx:tt, $ntyp:ident);)*]) => {
        impl<$typ: Unitas, $($ntyp: Unitas),*> Unitas for ($typ, $($ntyp),*) {
            #[inline]
            fn empty() -> Self {
                (<$typ as Unitas>::empty(), $(<$ntyp as Unitas>::empty()),*)
            }
        }
    };
}

tuple_monoid_impl! {
    (25 => Z), (24 => Y), (23 => X), (22 => W), (21 => V),
    (20 => U), (19 => T), (18 => S), (17 => R), (16 => Q),
    (15 => P), (14 => O), (13 => N), (12 => M), (11 => L),
    (10 => K), (9 => J), (8 => I), (7 => H), (6 => G),
    (5 => F), (4 => E), (3 => D), (2 => C), (1 => B), (0 => A),
}

// ============================================================================
// Helper functions
// ============================================================================

/// Combine all elements in a slice, using `empty()` for empty slices.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::combine_all;
///
/// assert_eq!(combine_all(&[1, 2, 3, 4]), 10);
/// assert_eq!(combine_all(&[] as &[i32]), 0);
/// ```
#[inline]
pub fn combine_all<T: Unitas + Clone>(xs: &[T]) -> T {
    T::combine_all(xs)
}

/// Return `o` combined with itself `n` times, with n=0 returning `empty()`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::combine_n;
///
/// assert_eq!(combine_n(&2, 0), 0);  // empty for i32
/// assert_eq!(combine_n(&2, 3), 6);  // 2 + 2 + 2
/// ```
#[inline]
pub fn combine_n<T: Unitas + Clone>(o: &T, times: u32) -> T {
    if times == 0 {
        T::empty()
    } else {
        super::semigroup::combine_n(o, times)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::{borrow::ToOwned, string::ToString, vec};
    #[cfg(feature = "std")]
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_empty() {
        assert_eq!(i32::empty(), 0);
        assert_eq!(f64::empty(), 0.0);
        assert_eq!(Option::<i32>::empty(), None);
    }

    // combine_n with Option<i32>: n=0 must yield empty() (None), n>0 folds over Some.
    // Guards the Option-specific combine_n path (which delegates to Unitas::empty on 0).
    #[test]
    fn test_combine_n_option() {
        assert_eq!(combine_n(&Some(2), 0), None);
        assert_eq!(combine_n(&Some(2), 4), Some(8));
    }

    // combine_all on empty Option<i32> slice must return None (Option::empty()).
    // Guards the Unitas::empty fallback path for combine_all on empty slices.
    #[test]
    fn test_combine_all_empty_option() {
        assert_eq!(combine_all(&[] as &[Option<i32>]), None);
    }

    // combine_all over Option<String> slice: delegates to Option's optimised
    // combine_all that flattens None and accumulates Some values via String concat.
    #[test]
    #[cfg(feature = "alloc")]
    fn test_combine_all_option_string() {
        let vec_of_some_strings = vec![Some("Hello".to_owned()), Some(" World".to_owned())];
        assert_eq!(
            combine_all(&vec_of_some_strings),
            Some("Hello World".to_owned())
        );
    }

    // combine_all on HashSet: empty-slice path yields Unitas::empty() (empty set);
    // non-empty slice unions all elements via Compositio::combine.
    #[test]
    #[cfg(feature = "std")]
    fn test_combine_all_hashset() {
        let vec_of_no_hashes: Vec<HashSet<i32>> = Vec::new();
        assert_eq!(
            combine_all(&vec_of_no_hashes),
            <HashSet<i32> as Unitas>::empty()
        );

        let mut h1 = HashSet::new();
        h1.insert(1);
        let mut h2 = HashSet::new();
        h2.insert(2);
        let mut h3 = HashSet::new();
        h3.insert(3);
        let vec_of_hashes = vec![h1, h2, h3];
        let mut h_expected = HashSet::new();
        h_expected.insert(1);
        h_expected.insert(2);
        h_expected.insert(3);
        assert_eq!(combine_all(&vec_of_hashes), h_expected);
    }

    // combine_all on HashMap: empty-slice path yields Unitas::empty() (empty map);
    // non-empty slice unions maps, recombining values on colliding keys via the
    // value type's Compositio (here: String concatenation).
    #[test]
    #[cfg(feature = "std")]
    fn test_combine_all_hashmap() {
        let vec_of_no_hashmaps: Vec<HashMap<i32, String>> = Vec::new();
        assert_eq!(
            combine_all(&vec_of_no_hashmaps),
            <HashMap<i32, String> as Unitas>::empty()
        );

        let mut h1: HashMap<i32, String> = HashMap::new();
        h1.insert(1, String::from("Hello"));
        let mut h2: HashMap<i32, String> = HashMap::new();
        h2.insert(1, String::from(" World"));
        h2.insert(2, String::from("Goodbye"));
        let mut h3: HashMap<i32, String> = HashMap::new();
        h3.insert(3, String::from("Cruel World"));
        let vec_of_hashes = vec![h1, h2, h3];

        let mut h_expected: HashMap<i32, String> = HashMap::new();
        h_expected.insert(1, String::from("Hello World"));
        h_expected.insert(2, String::from("Goodbye"));
        h_expected.insert(3, String::from("Cruel World"));
        assert_eq!(combine_all(&vec_of_hashes), h_expected);
    }

    // combine_all on Omnis<T>: empty slice yields identity (all-ones bitmask / true);
    // non-empty slice intersects via bitwise AND / logical AND.
    #[test]
    fn test_combine_all_omnis() {
        assert_eq!(combine_all(&[] as &[Omnis<i32>]), Omnis(!0));
        assert_eq!(combine_all(&[Omnis(3), Omnis(7)]), Omnis(3));

        assert_eq!(combine_all(&[] as &[Omnis<bool>]), Omnis(true));
        assert_eq!(combine_all(&[Omnis(false), Omnis(false)]), Omnis(false));
        assert_eq!(combine_all(&[Omnis(true), Omnis(true)]), Omnis(true));
    }

    // combine_all on Aliquid<T>: empty slice yields identity (0 / false);
    // non-empty slice unions via bitwise OR / logical OR.
    #[test]
    fn test_combine_all_aliquid() {
        assert_eq!(combine_all(&[] as &[Aliquid<i32>]), Aliquid(0));
        assert_eq!(combine_all(&[Aliquid(3), Aliquid(8)]), Aliquid(11));

        assert_eq!(combine_all(&[] as &[Aliquid<bool>]), Aliquid(false));
        assert_eq!(
            combine_all(&[Aliquid(false), Aliquid(false)]),
            Aliquid(false)
        );
        assert_eq!(combine_all(&[Aliquid(true), Aliquid(false)]), Aliquid(true));
    }

    // combine_all on a 4-tuple with heterogeneous element types — exercises the
    // tuple Unitas macro expansion under folding semantics.
    #[test]
    #[cfg(feature = "alloc")]
    fn test_combine_all_tuple_4_mixed() {
        let t1 = (1, 2.5f32, String::from("hi"), Some(3));
        let t2 = (1, 2.5f32, String::from(" world"), None);
        let t3 = (1, 2.5f32, String::from(", goodbye"), Some(10));
        let tuples = vec![t1, t2, t3];

        let expected = (3, 7.5f32, String::from("hi world, goodbye"), Some(13));
        assert_eq!(combine_all(&tuples), expected);
    }

    // combine_all on Multiplicatio folds via multiplication starting from the
    // multiplicative identity (1).
    #[test]
    fn test_combine_all_multiplicatio() {
        let v = [Multiplicatio(2), Multiplicatio(3), Multiplicatio(4)];
        assert_eq!(combine_all(&v), Multiplicatio(24));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_string_empty() {
        assert_eq!(String::empty(), "");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_empty() {
        assert_eq!(Vec::<i32>::empty(), Vec::<i32>::new());
    }

    #[test]
    fn test_multiplicatio_empty() {
        assert_eq!(Multiplicatio::<i32>::empty(), Multiplicatio(1));
    }

    #[test]
    fn test_omnis_aliquid_empty() {
        assert_eq!(Omnis::<bool>::empty(), Omnis(true));
        assert_eq!(Aliquid::<bool>::empty(), Aliquid(false));
        assert_eq!(Omnis::<u8>::empty(), Omnis(!0));
        assert_eq!(Aliquid::<u8>::empty(), Aliquid(0));
    }

    #[test]
    fn test_combine_all() {
        assert_eq!(combine_all(&[1, 2, 3, 4]), 10);
        assert_eq!(combine_all(&[] as &[i32]), 0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_combine_all_strings() {
        let strings = vec!["Hello".to_string(), " ".to_string(), "World".to_string()];
        assert_eq!(combine_all(&strings), "Hello World");
    }

    #[test]
    fn test_combine_n() {
        assert_eq!(combine_n(&2, 0), 0);
        assert_eq!(combine_n(&2, 1), 2);
        assert_eq!(combine_n(&2, 3), 6);
    }

    #[test]
    fn test_tuple_empty() {
        let empty: (i32, f64) = Unitas::empty();
        assert_eq!(empty, (0, 0.0));
    }

    #[test]
    fn test_left_identity_law() {
        let x = 42i32;
        assert_eq!(i32::empty().combine(&x), x);

        let s = Some(10);
        assert_eq!(Option::<i32>::empty().combine(&s), s);
    }

    #[test]
    fn test_right_identity_law() {
        let x = 42i32;
        assert_eq!(x.combine(&i32::empty()), x);

        let s = Some(10);
        assert_eq!(s.combine(&Option::empty()), s);
    }
}
