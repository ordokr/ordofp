//! Semigroupal typeclass for pairing values in a functor context.
//!
//! > *"Coniungere est duas res in unum componere."*
//! > — To conjoin is to compose two things into one.
//!
//! This module provides the [`OptionProductExt`], [`ResultProductExt`], etc.
//! extension traits (Semigroupal), which allow pairing of values within a functor
//! context. This is a weaker form of `Apply` that only requires the ability to
//! form products, not full function application.
//!
//! # Etymology
//!
//! - **Coniungendum** (Latin): "that which is to be joined together"
//!   - From *coniungere*: "to join together, unite"
//!   - The gerundive form emphasizes the capability of joining
//! - **Productum** (Latin): "that which has been brought forth"
//!
//! # Theory
//!
//! A `Semigroupal` functor `F` provides a way to combine `F<A>` and `F<B>` into
//! `F<(A, B)>`. This must satisfy the associativity law:
//!
//! ```text
//! product(product(fa, fb), fc) ≅ product(fa, product(fb, fc))
//! ```
//!
//! (up to reassociation of tuples)
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::OptionProductExt;
//!
//! // Option is a Semigroupal - pairs succeed only if both are Some
//! let a = Some(1);
//! let b = Some("hello");
//! let paired = a.productum(b);
//! assert_eq!(paired, Some((1, "hello")));
//!
//! // If either is None, the product is None
//! let c: Option<i32> = None;
//! let d = Some(42);
//! assert_eq!(c.productum(d), None);
//! ```

use alloc::collections::{BTreeMap, BTreeSet, LinkedList, VecDeque};
use alloc::vec::Vec;

#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

// ============================================================================
// Option extension trait
// ============================================================================

/// Extension trait for Option to provide Semigroupal product operation.
pub trait OptionProductExt<A> {
    /// Forms a product of two Options.
    ///
    /// Returns `Some((a, b))` if both are `Some`, otherwise `None`.
    ///
    /// # Latin Etymology
    /// *productum*: "that which has been brought forth"
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::OptionProductExt;
    ///
    /// let a = Some(1);
    /// let b = Some("hello");
    /// assert_eq!(a.productum(b), Some((1, "hello")));
    ///
    /// let c: Option<i32> = None;
    /// assert_eq!(c.productum(Some(42)), None);
    /// ```
    fn productum<B>(self, other: Option<B>) -> Option<(A, B)>;
}

impl<A> OptionProductExt<A> for Option<A> {
    #[inline]
    fn productum<B>(self, other: Option<B>) -> Option<(A, B)> {
        match (self, other) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

// ============================================================================
// Result extension trait
// ============================================================================

/// Extension trait for Result to provide Semigroupal product operation.
pub trait ResultProductExt<A, E> {
    /// Forms a product of two Results with the same error type.
    ///
    /// Returns `Ok((a, b))` if both are `Ok`, otherwise returns the first `Err`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::ResultProductExt;
    ///
    /// let a: Result<i32, &str> = Ok(1);
    /// let b: Result<&str, &str> = Ok("hello");
    /// assert_eq!(a.productum(b), Ok((1, "hello")));
    ///
    /// let c: Result<i32, &str> = Err("error1");
    /// let d: Result<i32, &str> = Ok(42);
    /// assert_eq!(c.productum(d), Err("error1"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the error of `self` if it is `Err`, otherwise the error
    /// of `other`; the other side's success value is discarded. Errors
    /// do not accumulate — use `Probatum` for accumulating validation.
    fn productum<B>(self, other: Result<B, E>) -> Result<(A, B), E>;
}

impl<A, E> ResultProductExt<A, E> for Result<A, E> {
    #[inline]
    fn productum<B>(self, other: Result<B, E>) -> Result<(A, B), E> {
        match (self, other) {
            (Ok(a), Ok(b)) => Ok((a, b)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

// ============================================================================
// Vec extension trait - cartesian product
// ============================================================================

/// Extension trait for Vec to provide Semigroupal product operation.
pub trait VecProductExt<A> {
    /// Forms the cartesian product of two Vecs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::VecProductExt;
    ///
    /// let a = vec![1, 2];
    /// let b = vec!["a", "b"];
    /// let result = a.productum(b);
    /// assert_eq!(result, vec![(1, "a"), (1, "b"), (2, "a"), (2, "b")]);
    /// ```
    fn productum<B: Clone>(self, other: Vec<B>) -> Vec<(A, B)>;
}

impl<A: Clone> VecProductExt<A> for Vec<A> {
    #[inline]
    fn productum<B: Clone>(self, other: Vec<B>) -> Vec<(A, B)> {
        let mut result = Vec::with_capacity(self.len() * other.len());
        for a in &self {
            for b in &other {
                result.push((a.clone(), b.clone()));
            }
        }
        result
    }
}

// ============================================================================
// VecDeque extension trait - cartesian product
// ============================================================================

/// Extension trait for `VecDeque` to provide Semigroupal product operation.
pub trait VecDequeProductExt<A> {
    /// Forms the cartesian product of two `VecDeques`.
    fn productum<B: Clone>(self, other: VecDeque<B>) -> VecDeque<(A, B)>;
}

impl<A: Clone> VecDequeProductExt<A> for VecDeque<A> {
    #[inline]
    fn productum<B: Clone>(self, other: VecDeque<B>) -> VecDeque<(A, B)> {
        let mut result = VecDeque::with_capacity(self.len() * other.len());
        for a in &self {
            for b in &other {
                result.push_back((a.clone(), b.clone()));
            }
        }
        result
    }
}

// ============================================================================
// LinkedList extension trait - cartesian product
// ============================================================================

/// Extension trait for `LinkedList` to provide Semigroupal product operation.
pub trait LinkedListProductExt<A> {
    /// Forms the cartesian product of two `LinkedLists`.
    fn productum<B: Clone>(self, other: LinkedList<B>) -> LinkedList<(A, B)>;
}

impl<A: Clone> LinkedListProductExt<A> for LinkedList<A> {
    #[inline]
    fn productum<B: Clone>(self, other: LinkedList<B>) -> LinkedList<(A, B)> {
        let mut result = LinkedList::new();
        for a in &self {
            for b in &other {
                result.push_back((a.clone(), b.clone()));
            }
        }
        result
    }
}

// ============================================================================
// BTreeSet extension trait - cartesian product
// ============================================================================

/// Extension trait for `BTreeSet` to provide Semigroupal product operation.
pub trait BTreeSetProductExt<A> {
    /// Forms the cartesian product of two `BTreeSets`.
    fn productum<B: Clone + Ord>(self, other: BTreeSet<B>) -> BTreeSet<(A, B)>
    where
        A: Ord;
}

impl<A: Clone + Ord> BTreeSetProductExt<A> for BTreeSet<A> {
    #[inline]
    fn productum<B: Clone + Ord>(self, other: BTreeSet<B>) -> BTreeSet<(A, B)>
    where
        A: Ord,
    {
        let mut result = BTreeSet::new();
        for a in &self {
            for b in &other {
                result.insert((a.clone(), b.clone()));
            }
        }
        result
    }
}

// ============================================================================
// HashSet extension trait - cartesian product
// ============================================================================

#[cfg(feature = "std")]
/// Extension trait for `HashSet` to provide Semigroupal product operation.
pub trait HashSetProductExt<A> {
    /// Forms the cartesian product of two `HashSets`.
    fn productum<B: Clone + Hash + Eq>(self, other: HashSet<B>) -> HashSet<(A, B)>
    where
        A: Hash + Eq;
}

#[cfg(feature = "std")]
impl<A: Clone + Hash + Eq> HashSetProductExt<A> for HashSet<A> {
    #[inline]
    fn productum<B: Clone + Hash + Eq>(self, other: HashSet<B>) -> HashSet<(A, B)>
    where
        A: Hash + Eq,
    {
        let mut result = HashSet::with_capacity(self.len() * other.len());
        for a in &self {
            for b in &other {
                result.insert((a.clone(), b.clone()));
            }
        }
        result
    }
}

// ============================================================================
// BTreeMap extension trait - intersection with value pairing
// ============================================================================

/// Extension trait for `BTreeMap` to provide Semigroupal product operation.
pub trait BTreeMapProductExt<K, V> {
    /// Forms a product of two `BTreeMaps` over their common keys.
    ///
    /// For keys that appear in both maps, pairs their values.
    /// Keys that only appear in one map are dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::BTreeMap;
    /// use ordofp_core::typeclasses::BTreeMapProductExt;
    ///
    /// let mut a = BTreeMap::new();
    /// a.insert("x", 1);
    /// a.insert("y", 2);
    ///
    /// let mut b = BTreeMap::new();
    /// b.insert("x", "a");
    /// b.insert("z", "c");
    ///
    /// let result = a.productum(b);
    /// assert_eq!(result.get("x"), Some(&(1, "a")));
    /// assert_eq!(result.get("y"), None);  // not in b
    /// assert_eq!(result.get("z"), None);  // not in a
    /// ```
    fn productum<V2: Clone>(self, other: BTreeMap<K, V2>) -> BTreeMap<K, (V, V2)>
    where
        K: Ord;
}

impl<K: Ord + Clone, V: Clone> BTreeMapProductExt<K, V> for BTreeMap<K, V> {
    #[inline]
    fn productum<V2: Clone>(self, other: BTreeMap<K, V2>) -> BTreeMap<K, (V, V2)>
    where
        K: Ord,
    {
        let mut result = BTreeMap::new();
        for (k, v1) in &self {
            if let Some(v2) = other.get(k) {
                result.insert(k.clone(), (v1.clone(), v2.clone()));
            }
        }
        result
    }
}

// ============================================================================
// HashMap extension trait - intersection with value pairing
// ============================================================================

#[cfg(feature = "std")]
/// Extension trait for `HashMap` to provide Semigroupal product operation.
pub trait HashMapProductExt<K, V> {
    /// Forms a product of two `HashMaps` over their common keys.
    fn productum<V2: Clone>(self, other: HashMap<K, V2>) -> HashMap<K, (V, V2)>
    where
        K: Hash + Eq;
}

#[cfg(feature = "std")]
impl<K: Hash + Eq + Clone, V: Clone> HashMapProductExt<K, V> for HashMap<K, V> {
    #[inline]
    fn productum<V2: Clone>(self, other: HashMap<K, V2>) -> HashMap<K, (V, V2)>
    where
        K: Hash + Eq,
    {
        let mut result = HashMap::new();
        for (k, v1) in &self {
            if let Some(v2) = other.get(k) {
                result.insert(k.clone(), (v1.clone(), v2.clone()));
            }
        }
        result
    }
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Combine three Option values into a triple.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::product3_option;
///
/// let result = product3_option(Some(1), Some(2), Some(3));
/// assert_eq!(result, Some((1, 2, 3)));
///
/// let result2 = product3_option(Some(1), None::<i32>, Some(3));
/// assert_eq!(result2, None);
/// ```
#[inline]
pub fn product3_option<A, B, C>(a: Option<A>, b: Option<B>, c: Option<C>) -> Option<(A, B, C)> {
    match (a, b, c) {
        (Some(a), Some(b), Some(c)) => Some((a, b, c)),
        _ => None,
    }
}

/// Combine four Option values into a quadruple.
#[inline]
pub fn product4_option<A, B, C, D>(
    a: Option<A>,
    b: Option<B>,
    c: Option<C>,
    d: Option<D>,
) -> Option<(A, B, C, D)> {
    match (a, b, c, d) {
        (Some(a), Some(b), Some(c), Some(d)) => Some((a, b, c, d)),
        _ => None,
    }
}

/// Combine three Result values into a triple.
///
/// Returns the first error encountered.
///
/// # Errors
///
/// Returns the leftmost `Err` among `a`, `b`, `c`; the remaining
/// success values are discarded.
#[inline]
pub fn product3_result<A, B, C, E>(
    a: Result<A, E>,
    b: Result<B, E>,
    c: Result<C, E>,
) -> Result<(A, B, C), E> {
    Ok((a?, b?, c?))
}

/// Combine four Result values into a quadruple.
///
/// # Errors
///
/// Returns the leftmost `Err` among `a`, `b`, `c`, `d`; the remaining
/// success values are discarded.
#[inline]
pub fn product4_result<A, B, C, D, E>(
    a: Result<A, E>,
    b: Result<B, E>,
    c: Result<C, E>,
    d: Result<D, E>,
) -> Result<(A, B, C, D), E> {
    Ok((a?, b?, c?, d?))
}

// ============================================================================
// Extension trait for ergonomic usage via `zip_with` alias
// ============================================================================

/// Extension methods providing `zip_with` as an alias for `productum`.
pub trait ZipWithExt<A> {
    /// Alias for `productum` using more familiar terminology.
    fn zip_with<B>(self, other: Option<B>) -> Option<(A, B)>;
}

impl<A> ZipWithExt<A> for Option<A> {
    #[inline]
    fn zip_with<B>(self, other: Option<B>) -> Option<(A, B)> {
        self.productum(other)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_option_productum() {
        let a = Some(1);
        let b = Some("hello");
        assert_eq!(a.productum(b), Some((1, "hello")));

        let c: Option<i32> = None;
        let d = Some(42);
        assert_eq!(c.productum(d), None);

        let e = Some(1);
        let f: Option<i32> = None;
        assert_eq!(e.productum(f), None);
    }

    #[test]
    fn test_result_productum() {
        let a: Result<i32, &str> = Ok(1);
        let b: Result<&str, &str> = Ok("hello");
        assert_eq!(a.productum(b), Ok((1, "hello")));

        let c: Result<i32, &str> = Err("error");
        let d: Result<i32, &str> = Ok(3);
        assert_eq!(c.productum(d), Err("error"));
    }

    #[test]
    fn test_vec_productum() {
        let a = vec![1, 2];
        let b = vec!["a", "b"];
        let result = a.productum(b);
        assert_eq!(result, vec![(1, "a"), (1, "b"), (2, "a"), (2, "b")]);
    }

    #[test]
    fn test_btreemap_productum() {
        let mut a = BTreeMap::new();
        a.insert("x", 1);
        a.insert("y", 2);
        a.insert("z", 3);

        let mut b = BTreeMap::new();
        b.insert("x", "a");
        b.insert("y", "b");
        b.insert("w", "d");

        let result = a.productum(b);
        assert_eq!(result.get("x"), Some(&(1, "a")));
        assert_eq!(result.get("y"), Some(&(2, "b")));
        assert_eq!(result.get("z"), None); // z not in b
        assert_eq!(result.get("w"), None); // w not in a
    }

    #[test]
    fn test_zip_with_ext() {
        let a = Some(1);
        let b = Some(2);
        // Use fully qualified syntax to avoid conflict with Option::zip_with from std
        assert_eq!(ZipWithExt::zip_with(a, b), Some((1, 2)));
    }

    #[test]
    fn test_product3_option() {
        let result = product3_option(Some(1), Some(2), Some(3));
        assert_eq!(result, Some((1, 2, 3)));

        let result2 = product3_option(Some(1), None::<i32>, Some(3));
        assert_eq!(result2, None);
    }

    #[test]
    fn test_product3_result() {
        let a: Result<i32, &str> = Ok(1);
        let b: Result<i32, &str> = Ok(2);
        let c: Result<i32, &str> = Ok(3);
        assert_eq!(product3_result(a, b, c), Ok((1, 2, 3)));
    }

    #[test]
    fn test_associativity_option() {
        // (a product b) product c ≅ a product (b product c)
        let a = Some(1);
        let b = Some(2);
        let c = Some(3);

        // Left association: ((a, b), c)
        let left = a.productum(b).productum(c);

        // Right association: (a, (b, c))
        let right = a.productum(b.productum(c));

        // They should be isomorphic (same information, different structure)
        assert_eq!(left, Some(((1, 2), 3)));
        assert_eq!(right, Some((1, (2, 3))));
    }
}
