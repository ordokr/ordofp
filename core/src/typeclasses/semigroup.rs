//! Compositio (Semigroup) type class - types with an associative binary operation.
//!
//! > *"Compositio est coniunctio duorum vel plurium in unum."*
//! > — Composition is the joining of two or more into one. (Aquinas, *De Veritate*)
//!
//! A `Compositio` is a type with an associative `combine` operation.
//! This means: `a.combine(&b).combine(&c) == a.combine(&b.combine(&c))`
//!
//! The name derives from Aristotle's σύνθεσις (synthesis) and Aquinas's
//! treatment of *compositio et divisio* in judgment.
//!
//! # Examples
//!
//! ```rust
//! use ordofp_core::typeclasses::Compositio;
//!
//! // Numbers combine via addition
//! assert_eq!(1.combine(&2), 3);
//!
//! // Options combine their contents (or keep the Some)
//! assert_eq!(Some(1).combine(&Some(2)), Some(3));
//! assert_eq!(Some(1).combine(&None), Some(1));
//! ```

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, string::String, vec::Vec};
use core::cell::{Cell, RefCell};
use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr, Deref};

#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::collections::hash_map::Entry;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

use crate::hlist::{Coniunctio, HList, Nihil};
use crate::wrappers::{Aggregatio, Aliquid, Max, Min, Multiplicatio, Omnis, Primus, Ultimus};

/// Compositio - a type with an associative binary operation.
///
/// > *"In compositione intellectus componit et dividit."*
/// > — In composition the intellect joins and separates. (Aquinas, ST I, q.85, a.5)
///
/// Named after the scholastic concept of *compositio*, the joining of
/// concepts in judgment. This is the Rust equivalent of Haskell's `Semigroup`.
///
/// # Laws
///
/// **Associativity** (*Lex Associationis*): For all `a`, `b`, `c`:
/// ```text
/// a.combine(&b).combine(&c) == a.combine(&b.combine(&c))
/// ```
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::Compositio;
///
/// assert_eq!(Some(1).combine(&Some(2)), Some(3));
/// ```
pub trait Compositio {
    /// Associative binary operation.
    ///
    /// Combines `self` with `other` to produce a new value of the same type.
    fn combine(&self, other: &Self) -> Self;
}

// ============================================================================
// HList implementations
// ============================================================================

impl<H: Compositio, T: HList + Compositio> Compositio for Coniunctio<H, T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        self.tail
            .combine(&other.tail)
            .prepend(self.head.combine(&other.head))
    }
}

impl Compositio for Nihil {
    #[inline]
    fn combine(&self, _: &Self) -> Self {
        *self
    }
}

// ============================================================================
// Numeric implementations (additive by default)
// ============================================================================

macro_rules! numeric_semigroup_impl {
    ($($ty:ty),*) => {
        $(
            impl Compositio for $ty {
                #[inline]
                fn combine(&self, other: &Self) -> Self {
                    self + other
                }
            }
        )*
    };
}

numeric_semigroup_impl!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

// f32 and f64 implementations with floating-point associativity caveat
impl Compositio for f32 {
    /// NOTE: floating-point combine is only approximately associative (rounding); the Semigroup laws hold up to epsilon, not exactly.
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        self + other
    }
}

impl Compositio for f64 {
    /// NOTE: floating-point combine is only approximately associative (rounding); the Semigroup laws hold up to epsilon, not exactly.
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        self + other
    }
}

// ============================================================================
// Wrapper type implementations
// ============================================================================

// Multiplicatio: multiplicative Compositio
macro_rules! multiplicatio_semigroup_impl {
    ($($ty:ty),*) => {
        $(
            impl Compositio for Multiplicatio<$ty> {
                #[inline]
                fn combine(&self, other: &Self) -> Self {
                    Multiplicatio(self.0 * other.0)
                }
            }
        )*
    };
}

multiplicatio_semigroup_impl!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

// Aggregatio: explicit additive (same as default, for symmetry)
macro_rules! aggregatio_semigroup_impl {
    ($($ty:ty),*) => {
        $(
            impl Compositio for Aggregatio<$ty> {
                #[inline]
                fn combine(&self, other: &Self) -> Self {
                    Aggregatio(self.0 + other.0)
                }
            }
        )*
    };
}

aggregatio_semigroup_impl!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

// Max: takes the larger value
impl<T: Ord + Clone> Compositio for Max<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        match self.0.cmp(&other.0) {
            Ordering::Less => Max(other.0.clone()),
            _ => Max(self.0.clone()),
        }
    }
}

// Min: takes the smaller value
impl<T: Ord + Clone> Compositio for Min<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        match self.0.cmp(&other.0) {
            Ordering::Less => Min(self.0.clone()),
            _ => Min(other.0.clone()),
        }
    }
}

// Omnis: bitwise AND
macro_rules! omnis_semigroup_impl {
    ($($ty:ty),*) => {
        $(
            impl Compositio for Omnis<$ty> {
                #[inline]
                fn combine(&self, other: &Self) -> Self {
                    Omnis(self.0.bitand(other.0))
                }
            }
        )*
    };
}

omnis_semigroup_impl!(
    bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

// Aliquid: bitwise OR
macro_rules! aliquid_semigroup_impl {
    ($($ty:ty),*) => {
        $(
            impl Compositio for Aliquid<$ty> {
                #[inline]
                fn combine(&self, other: &Self) -> Self {
                    Aliquid(self.0.bitor(other.0))
                }
            }
        )*
    };
}

aliquid_semigroup_impl!(
    bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

// Primus: keeps the first value
impl<T: Clone> Compositio for Primus<T> {
    #[inline]
    fn combine(&self, _other: &Self) -> Self {
        Primus(self.0.clone())
    }
}

// Ultimus: keeps the last value
impl<T: Clone> Compositio for Ultimus<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        Ultimus(other.0.clone())
    }
}

// ============================================================================
// Standard type implementations
// ============================================================================

impl<T: Compositio + Clone> Compositio for Option<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.combine(b)),
            (Some(_), None) => self.clone(),
            (None, Some(_)) => other.clone(),
            (None, None) => None,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Compositio> Compositio for Box<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        Box::new(self.deref().combine(&**other))
    }
}

#[cfg(feature = "alloc")]
impl Compositio for String {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        // Pre-allocate full capacity to avoid reallocations
        let mut result = String::with_capacity(self.len() + other.len());
        result.push_str(self);
        result.push_str(other);
        result
    }
}

#[cfg(feature = "alloc")]
impl<T: Clone> Compositio for Vec<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        // Pre-allocate full capacity to avoid reallocations
        let mut result = Vec::with_capacity(self.len() + other.len());
        result.extend_from_slice(self);
        result.extend_from_slice(other);
        result
    }
}

impl<T: Compositio + Copy> Compositio for Cell<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        Cell::new(self.get().combine(&other.get()))
    }
}

impl<T: Compositio> Compositio for RefCell<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        RefCell::new(self.borrow().deref().combine(other.borrow().deref()))
    }
}

#[cfg(feature = "std")]
impl<T: Eq + Hash + Clone> Compositio for HashSet<T> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        self.union(other).cloned().collect()
    }
}

#[cfg(feature = "std")]
impl<K: Eq + Hash + Clone, V: Compositio + Clone> Compositio for HashMap<K, V> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (k, v) in other {
            match result.entry(k.clone()) {
                Entry::Occupied(mut e) => {
                    let combined = e.get().combine(v);
                    *e.get_mut() = combined;
                }
                Entry::Vacant(e) => {
                    e.insert(v.clone());
                }
            }
        }
        result
    }
}

// ============================================================================
// Tuple implementations
// ============================================================================

macro_rules! tuple_semigroup_impl {
    () => {};
    (($idx:tt => $typ:ident), $(($nidx:tt => $ntyp:ident),)*) => {
        tuple_semigroup_impl!([($idx, $typ);] $(($nidx => $ntyp),)*);
        tuple_semigroup_impl!($(($nidx => $ntyp),)*);
    };
    ([$(($accIdx:tt, $accTyp:ident);)+] ($idx:tt => $typ:ident), $(($nidx:tt => $ntyp:ident),)*) => {
        tuple_semigroup_impl!([($idx, $typ); $(($accIdx, $accTyp);)*] $(($nidx => $ntyp),)*);
    };
    ([($idx:tt, $typ:ident); $(($nidx:tt, $ntyp:ident);)*]) => {
        impl<$typ: Compositio, $($ntyp: Compositio),*> Compositio for ($typ, $($ntyp),*) {
            #[inline]
            fn combine(&self, other: &Self) -> Self {
                (self.$idx.combine(&other.$idx), $(self.$nidx.combine(&other.$nidx)),*)
            }
        }
    };
}

tuple_semigroup_impl! {
    (25 => Z), (24 => Y), (23 => X), (22 => W), (21 => V),
    (20 => U), (19 => T), (18 => S), (17 => R), (16 => Q),
    (15 => P), (14 => O), (13 => N), (12 => M), (11 => L),
    (10 => K), (9 => J), (8 => I), (7 => H), (6 => G),
    (5 => F), (4 => E), (3 => D), (2 => C), (1 => B), (0 => A),
}

// ============================================================================
// Helper functions
// ============================================================================

/// Return `o` combined with itself `n` times.
///
/// Returns `o` if `n == 1`, `o.combine(o)` if `n == 2`, etc.
///
/// # Panics
///
/// Panics if `n == 0`. Use [`monoid::combine_n`](super::combine_n) for n=0 support.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::semigroup::combine_n;
///
/// assert_eq!(combine_n(&2, 3), 6);  // 2 + 2 + 2
/// assert_eq!(combine_n(&Some(2), 4), Some(8));
/// ```
pub fn combine_n<T: Compositio + Clone>(o: &T, times: u32) -> T {
    assert!(times > 0, "combine_n requires times > 0");
    if times == 1 {
        return o.clone();
    }

    // Exponentiation by squaring (O(log n))
    // x^n = x * x^(n-1) if n odd
    // x^n = (x^2)^(n/2) if n even
    let mut acc: Option<T> = None;
    let mut base = o.clone();
    let mut n = times;

    while n > 0 {
        if n % 2 == 1 {
            acc = match acc {
                Some(a) => Some(a.combine(&base)),
                None => Some(base.clone()),
            };
        }
        if n > 1 {
            base = base.combine(&base);
        }
        n /= 2;
    }
    acc.unwrap()
}

/// Combine all elements in a non-empty slice.
///
/// Returns `None` if the slice is empty, otherwise combines all elements.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::combine_all_option;
///
/// assert_eq!(combine_all_option(&[1, 2, 3]), Some(6));
/// assert_eq!(combine_all_option(&[] as &[i32]), None);
/// ```
pub fn combine_all_option<T: Compositio + Clone>(xs: &[T]) -> Option<T> {
    xs.first()
        .map(|head| xs[1..].iter().fold(head.clone(), |acc, x| acc.combine(x)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::{borrow::ToOwned, vec};
    use core::cell::RefCell;
    #[cfg(feature = "std")]
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_numeric() {
        assert_eq!(1i32.combine(&2), 3);
        assert_eq!(1.5f64.combine(&2.5), 4.0);
    }

    // Exercise the numeric Compositio impl for every primitive type generated by
    // `numeric_semigroup_impl!`. `test_numeric` covers `i32` and `f64`; these
    // tests guard against regressions in the macro expansion for the rest.
    #[test]
    fn test_numeric_i8() {
        assert_eq!(1i8.combine(&2i8), 3i8);
    }

    #[test]
    fn test_numeric_i16() {
        assert_eq!(1i16.combine(&2i16), 3i16);
    }

    #[test]
    fn test_numeric_u8() {
        assert_eq!(1u8.combine(&2u8), 3u8);
    }

    #[test]
    fn test_numeric_u16() {
        assert_eq!(1u16.combine(&2u16), 3u16);
    }

    #[test]
    fn test_numeric_u32() {
        assert_eq!(1u32.combine(&2u32), 3u32);
    }

    #[test]
    fn test_numeric_usize() {
        assert_eq!(1usize.combine(&2usize), 3usize);
    }

    #[test]
    fn test_numeric_isize() {
        assert_eq!(1isize.combine(&2isize), 3isize);
    }

    #[test]
    fn test_numeric_f32() {
        assert_eq!(1f32.combine(&2f32), 3f32);
    }

    // Verify that `Multiplicatio` multiplies rather than adds for a narrow integer type.
    // `test_multiplicatio` covers the default integer case; this guards the macro
    // expansion for the i8 instance.
    #[test]
    fn test_multiplicatio_i8() {
        assert_eq!(
            Multiplicatio(1i8).combine(&Multiplicatio(2i8)),
            Multiplicatio(2i8)
        );
    }

    // Option<i16> Compositio behaviour — exercises the Option impl on a non-default numeric
    // parameter to guard the generic `Option<T: Compositio>` impl.
    #[test]
    fn test_option_i16_some_some() {
        assert_eq!(Some(1i16).combine(&Some(2i16)), Some(3i16));
    }

    #[test]
    fn test_option_i16_none_some() {
        let v: Option<i16> = None;
        assert_eq!(v.combine(&Some(2i16)), Some(2i16));
    }

    #[test]
    fn test_option_i16_some_none() {
        assert_eq!(Some(2i16).combine(&None), Some(2i16));
    }

    // RefCell<T>'s Compositio impl borrows both cells and recombines. Ensures
    // the interior-mutability wrapper propagates `combine` to the inner value.
    #[test]
    fn test_refcell() {
        let v1 = RefCell::new(1);
        let v2 = RefCell::new(2);
        assert_eq!(v1.combine(&v2), RefCell::new(3));
    }

    // HashSet's Compositio impl is set-union semantics.
    #[test]
    #[cfg(feature = "std")]
    fn test_hashset() {
        let mut v1 = HashSet::new();
        v1.insert(1);
        v1.insert(2);
        assert!(!v1.contains(&3));
        let mut v2 = HashSet::new();
        v2.insert(3);
        v2.insert(4);
        assert!(!v2.contains(&1));
        let mut expected = HashSet::new();
        expected.insert(1);
        expected.insert(2);
        expected.insert(3);
        expected.insert(4);
        assert_eq!(v1.combine(&v2), expected);
    }

    // 4-tuple Compositio across heterogeneous types exercises the tuple macro
    // on a non-trivial arity with mixed element kinds (numeric, float, String, Option).
    #[test]
    #[cfg(feature = "std")]
    fn test_tuple_4_mixed() {
        let t1 = (1, 2.5f32, String::from("hi"), Some(3));
        let t2 = (1, 2.5f32, String::from(" world"), None);

        let expected = (2, 5.0f32, String::from("hi world"), Some(3));

        assert_eq!(t1.combine(&t2), expected);
    }

    // HashMap<K, V: Compositio> combines by union, recombining values on key collision.
    #[test]
    #[cfg(feature = "std")]
    fn test_hashmap() {
        let mut v1: HashMap<i32, Option<String>> = HashMap::new();
        v1.insert(1, Some("Hello".to_owned()));
        v1.insert(2, Some("Goodbye".to_owned()));
        v1.insert(4, None);
        let mut v2: HashMap<i32, Option<String>> = HashMap::new();
        v2.insert(1, Some(" World".to_owned()));
        v2.insert(4, Some("Nope".to_owned()));
        let mut expected = HashMap::new();
        expected.insert(1, Some("Hello World".to_owned()));
        expected.insert(2, Some("Goodbye".to_owned()));
        expected.insert(4, Some("Nope".to_owned()));
        assert_eq!(v1.combine(&v2), expected);
    }

    // HList Compositio combines element-wise. Exercises the Coniunctio/Nihil impl.
    #[test]
    #[cfg(feature = "alloc")]
    fn test_combine_hlist() {
        let h1 = crate::hlist![Some(1), 3.3, 53i64, "hello".to_owned()];
        let h2 = crate::hlist![Some(2), 1.2, 1i64, " world".to_owned()];
        let h3 = crate::hlist![Some(3), 4.5, 54, "hello world".to_owned()];
        assert_eq!(h1.combine(&h2), h3);
    }

    // combine_all_option over Option-of-Option exercises the recursive propagation
    // of combine through a nested Compositio container.
    #[test]
    fn test_combine_all_option_nested() {
        let v = [Some(1), Some(2), Some(3)];
        assert_eq!(combine_all_option(&v), Some(Some(6)));
    }

    // combine_all_option on Max/Min wrappers verifies that the helper threads
    // wrapper-specific combine semantics (max/min) through the fold.
    #[test]
    fn test_combine_all_option_max_min() {
        let v = [Max(1), Max(2), Max(3)];
        assert_eq!(combine_all_option(&v), Some(Max(3)));
        let v = [Min(1), Min(2), Min(3)];
        assert_eq!(combine_all_option(&v), Some(Min(1)));
    }

    // combine_n with a multiplier of 1 on Some — exercises the fast-path clone on Option.
    #[test]
    fn test_combine_n_some_single() {
        assert_eq!(combine_n(&Some(2), 1), Some(2));
    }

    // Additive combine on f64 with different values than `test_numeric`, to guard
    // against regressions specific to the 1+2 path (no fractional component).
    #[test]
    fn test_numeric_f64_integers() {
        assert_eq!(1f64.combine(&2f64), 3f64);
    }

    // Additional i32 value to broaden coverage of the numeric Compositio impl.
    #[test]
    fn test_numeric_i32_small() {
        assert_eq!(1i32.combine(&2i32), 3i32);
    }

    // String concatenation with a different-cased word to guard against case-sensitive regressions.
    #[test]
    #[cfg(feature = "alloc")]
    fn test_string_lowercase() {
        let v1 = String::from("Hello");
        let v2 = String::from(" world");
        assert_eq!(v1.combine(&v2), "Hello world");
    }

    // Vec<i32> concatenation with a 3+3 pair (complements `test_vec` 2+2 pair)
    // so we keep coverage of mid-size vectors.
    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_i32_three_each() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5, 6];
        assert_eq!(v1.combine(&v2), vec![1, 2, 3, 4, 5, 6]);
    }

    // Max(1)+Max(2)=Max(2): broadens the values exercised beyond `test_max_min`.
    #[test]
    fn test_max_small_values() {
        assert_eq!(Max(1).combine(&Max(2)), Max(2));
    }

    // Min(1)+Min(2)=Min(1): broadens the values exercised beyond `test_max_min`.
    #[test]
    fn test_min_small_values() {
        assert_eq!(Min(1).combine(&Min(2)), Min(1));
    }

    // Omnis<u8> combine with decimal values 3 & 5 — bitand gives 1 (0b011 & 0b101 = 0b001).
    // Complements `test_all_aliquid` which uses 0b1100 & 0b1010 binary literals.
    #[test]
    fn test_omnis_u8_decimal() {
        assert_eq!(Omnis(3u8).combine(&Omnis(5u8)), Omnis(1u8));
        assert_eq!(Omnis(true).combine(&Omnis(false)), Omnis(false));
    }

    // Aliquid<u8> combine with decimal values 3 | 5 = 7 (0b011 | 0b101 = 0b111).
    // Complements `test_all_aliquid` which uses 0b1100 | 0b0011 binary literals.
    #[test]
    fn test_aliquid_u8_decimal() {
        assert_eq!(Aliquid(3u8).combine(&Aliquid(5u8)), Aliquid(7u8));
        assert_eq!(Aliquid(true).combine(&Aliquid(false)), Aliquid(true));
    }

    // combine_all_option on basic i32 slice — preserves an aggregate-style assertion
    // of the sum over [1,2,3], complementing the singleton `[99i32]` case elsewhere.
    #[test]
    fn test_combine_all_option_i32_three() {
        let v1 = [1, 2, 3];
        assert_eq!(combine_all_option(&v1), Some(6));
    }

    // combine_all_option on an empty slice returns None rather than panicking.
    // Named explicitly to document this edge-case contract.
    #[test]
    fn test_combine_all_option_empty_slice_returns_none() {
        let empty: &[i32] = &[];
        assert_eq!(combine_all_option(empty), None);
    }

    // combine_n contract: `combine_n(&1, 3) == 3` (1+1+1), plus Some(2)*4 = Some(8).
    // Complements the &2,3=>6 case already in `test_combine_n`.
    #[test]
    fn test_combine_n_ones_and_option() {
        assert_eq!(combine_n(&1, 3), 3);
        assert_eq!(combine_n(&2, 1), 2);
        assert_eq!(combine_n(&Some(2), 4), Some(8));
    }

    #[test]
    fn test_multiplicatio() {
        assert_eq!(
            Multiplicatio(3).combine(&Multiplicatio(4)),
            Multiplicatio(12)
        );
    }

    #[test]
    fn test_max_min() {
        assert_eq!(Max(3).combine(&Max(5)), Max(5));
        assert_eq!(Min(3).combine(&Min(5)), Min(3));
    }

    #[test]
    fn test_all_aliquid() {
        assert_eq!(Omnis(true).combine(&Omnis(false)), Omnis(false));
        assert_eq!(Aliquid(true).combine(&Aliquid(false)), Aliquid(true));
        assert_eq!(Omnis(0b1100u8).combine(&Omnis(0b1010)), Omnis(0b1000));
        assert_eq!(Aliquid(0b1100u8).combine(&Aliquid(0b0011)), Aliquid(0b1111));
    }

    #[test]
    fn test_first_ultimus() {
        assert_eq!(Primus(1).combine(&Primus(2)), Primus(1));
        assert_eq!(Ultimus(1).combine(&Ultimus(2)), Ultimus(2));
    }

    #[test]
    fn test_option() {
        assert_eq!(Some(1).combine(&Some(2)), Some(3));
        assert_eq!(Some(1).combine(&None), Some(1));
        assert_eq!(None::<i32>.combine(&Some(2)), Some(2));
        assert_eq!(None::<i32>.combine(&None), None);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_string() {
        assert_eq!(
            String::from("Hello").combine(&String::from(" World")),
            "Hello World"
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec() {
        assert_eq!(vec![1, 2].combine(&vec![3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_tuple() {
        let t1 = (1, 2.0f32);
        let t2 = (3, 4.0f32);
        assert_eq!(t1.combine(&t2), (4, 6.0f32));
    }

    #[test]
    fn test_combine_n() {
        assert_eq!(combine_n(&2, 3), 6);
        assert_eq!(combine_n(&Some(2), 4), Some(8));
    }

    #[test]
    fn test_combine_n_times_one_returns_clone_without_combining() {
        // `times == 1` hits the early-return fast path; the value must be
        // returned as-is (a clone) without ever calling `combine`.
        assert_eq!(combine_n(&7i32, 1), 7);
        assert_eq!(combine_n(&Some(42i32), 1), Some(42));
        // Singleton slice: combine_all_option must return Some(element) with no
        // combining, exercising the same "only a head, empty tail" branch.
        assert_eq!(combine_all_option(&[99i32]), Some(99));
    }

    #[test]
    fn test_combine_all_option() {
        assert_eq!(combine_all_option(&[1, 2, 3]), Some(6));
        assert_eq!(combine_all_option(&[] as &[i32]), None);
    }

    #[test]
    fn test_associativity_law() {
        let a = 1;
        let b = 2;
        let c = 3;
        assert_eq!(a.combine(&b).combine(&c), a.combine(&b.combine(&c)));

        let oa = Some(1);
        let ob = Some(2);
        let oc = Some(3);
        assert_eq!(oa.combine(&ob).combine(&oc), oa.combine(&ob.combine(&oc)));
    }
}
