//! Invariant functor typeclass for bidirectional mapping.
//!
//! > *"Biformis est quod duabus formis transformari potest."*
//! > — Biform is that which can be transformed by two forms.
//!
//! This module provides invariant functor operations, which allow
//! bidirectional transformation of values within a context when both directions
//! of the mapping are available.
//!
//! # Etymology
//!
//! - **Biformis** (Latin): "having two forms, double-shaped"
//!   - From *bi-* (two) + *forma* (shape, form)
//!   - Reflects the need for both forward and backward transformations
//!
//! # Theory
//!
//! An invariant functor sits between covariant and contravariant functors:
//!
//! - **Covariant** (Functor): needs `A -> B` to transform `F<A>` to `F<B>`
//! - **Contravariant**: needs `B -> A` to transform `F<A>` to `F<B>`
//! - **Invariant**: needs both `A -> B` AND `B -> A`
//!
//! This is useful for types that both produce AND consume values of type `A`,
//! such as codecs, isomorphisms, and bidirectional transformations.
//!
//! # Note on Implementation
//!
//! Due to Rust's type system limitations (lack of HKTs), we provide this as
//! extension traits per type rather than a single unified trait. Each type
//! gets a `BiformisExt` trait with an `imap` method.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::OptionBiformisExt;
//!
//! // Option is Biformis (via Functor - only uses forward direction)
//! let x = Some("42".to_string());
//! let y: Option<i32> = x.imap(
//!     |s| s.parse().unwrap_or(0),  // forward: String -> i32
//!     |i: i32| i.to_string()        // backward: i32 -> String (unused for Option)
//! );
//! assert_eq!(y, Some(42));
//! ```

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, LinkedList, VecDeque};
use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "std")]
use core::hash::Hash;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

// ============================================================================
// Option extension trait
// ============================================================================

/// Extension trait for Option to provide invariant mapping.
pub trait OptionBiformisExt<A> {
    /// Apply an invariant map to Option.
    ///
    /// For covariant functors like Option, only the forward function `f` is used.
    /// The backward function `g` is provided for API consistency with truly
    /// invariant types.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::OptionBiformisExt;
    ///
    /// let x = Some(42);
    /// let y: Option<String> = x.imap(
    ///     |n| n.to_string(),
    ///     |s: String| s.parse().unwrap_or(0)
    /// );
    /// assert_eq!(y, Some("42".to_string()));
    /// ```
    fn imap<B, F, G>(self, f: F, g: G) -> Option<B>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A;
}

impl<A> OptionBiformisExt<A> for Option<A> {
    #[inline]
    fn imap<B, F, G>(self, f: F, _g: G) -> Option<B>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A,
    {
        self.map(f)
    }
}

// ============================================================================
// Result extension trait
// ============================================================================

/// Extension trait for Result to provide invariant mapping.
pub trait ResultBiformisExt<A, E> {
    /// Apply an invariant map to Result.
    ///
    /// # Errors
    ///
    /// Propagates the existing error unchanged when `self` is `Err`;
    /// the mapping introduces no new failure modes.
    fn imap<B, F, G>(self, f: F, g: G) -> Result<B, E>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A;
}

impl<A, E> ResultBiformisExt<A, E> for Result<A, E> {
    #[inline]
    fn imap<B, F, G>(self, f: F, _g: G) -> Result<B, E>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A,
    {
        self.map(f)
    }
}

// ============================================================================
// Collection extension traits (macro-generated)
// ============================================================================

/// Generates the `imap` extension trait + impl for an owned-iteration
/// collection. These are covariant containers, so only the forward function
/// is used; the backward function exists for API consistency with truly
/// invariant types like [`Codec`].
macro_rules! collection_biformis_ext {
    ($(#[$attr:meta])* $trait_name:ident, $container:ident $(, $bound:path)*) => {
        $(#[$attr])*
        #[doc = concat!("Extension trait for ", stringify!($container),
            " to provide invariant mapping.")]
        pub trait $trait_name<A> {
            #[doc = concat!("Apply an invariant map to ", stringify!($container), ".")]
            fn imap<B, F, G>(self, f: F, g: G) -> $container<B>
            where
                B: $($bound +)* Sized,
                F: FnMut(A) -> B,
                G: FnMut(B) -> A;
        }

        $(#[$attr])*
        impl<A> $trait_name<A> for $container<A> {
            #[inline]
            fn imap<B, F, G>(self, f: F, _g: G) -> $container<B>
            where
                B: $($bound +)* Sized,
                F: FnMut(A) -> B,
                G: FnMut(B) -> A,
            {
                self.into_iter().map(f).collect()
            }
        }
    };
}

collection_biformis_ext!(VecBiformisExt, Vec);
collection_biformis_ext!(VecDequeBiformisExt, VecDeque);
collection_biformis_ext!(LinkedListBiformisExt, LinkedList);
collection_biformis_ext!(BTreeSetBiformisExt, BTreeSet, Ord);
collection_biformis_ext!(
    #[cfg(feature = "std")]
    HashSetBiformisExt,
    HashSet,
    Hash,
    Eq
);

// ============================================================================
// Box extension trait
// ============================================================================

/// Extension trait for Box to provide invariant mapping.
pub trait BoxBiformisExt<A> {
    /// Apply an invariant map to Box.
    fn imap<B, F, G>(self, f: F, g: G) -> Box<B>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A;
}

impl<A> BoxBiformisExt<A> for Box<A> {
    #[inline]
    fn imap<B, F, G>(self, f: F, _g: G) -> Box<B>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A,
    {
        Box::new(f(*self))
    }
}

// ============================================================================
// Map extension traits (macro-generated)
// ============================================================================

/// Generates the `imap`-over-values extension trait + impl for a key-value
/// map. Like the collection traits, only the forward function is used.
macro_rules! map_biformis_ext {
    ($(#[$attr:meta])* $trait_name:ident, $container:ident $(, $kbound:path)*) => {
        $(#[$attr])*
        #[doc = concat!("Extension trait for ", stringify!($container),
            " to provide invariant mapping over values.")]
        pub trait $trait_name<K, V> {
            #[doc = concat!("Apply an invariant map to ", stringify!($container), " values.")]
            fn imap<B, F, G>(self, f: F, g: G) -> $container<K, B>
            where
                K: $($kbound +)* Sized,
                F: FnMut(V) -> B,
                G: FnMut(B) -> V;
        }

        $(#[$attr])*
        impl<K: $($kbound +)* Sized, V> $trait_name<K, V> for $container<K, V> {
            #[inline]
            fn imap<B, F, G>(self, mut f: F, _g: G) -> $container<K, B>
            where
                K: $($kbound +)* Sized,
                F: FnMut(V) -> B,
                G: FnMut(B) -> V,
            {
                self.into_iter().map(|(k, v)| (k, f(v))).collect()
            }
        }
    };
}

map_biformis_ext!(BTreeMapBiformisExt, BTreeMap, Ord);
map_biformis_ext!(
    #[cfg(feature = "std")]
    HashMapBiformisExt,
    HashMap,
    Hash,
    Eq
);

// ============================================================================
// PhantomData extension trait
// ============================================================================

/// Extension trait for `PhantomData` to provide invariant mapping.
pub trait PhantomDataBiformisExt<A> {
    /// Apply an invariant map to `PhantomData`.
    fn imap<B, F, G>(self, f: F, g: G) -> PhantomData<B>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A;
}

impl<A> PhantomDataBiformisExt<A> for PhantomData<A> {
    #[inline]
    fn imap<B, F, G>(self, _f: F, _g: G) -> PhantomData<B>
    where
        F: FnOnce(A) -> B,
        G: FnOnce(B) -> A,
    {
        PhantomData
    }
}

// ============================================================================
// Codec type - a truly invariant type that uses both directions
// ============================================================================

/// A codec that can encode to and decode from a representation type.
///
/// This is the motivating example of a *truly invariant* structure: mapping
/// its value type would require both a forward and a backward function
/// (decode needs `A -> B`, encode needs `B -> A`). Note, however, that
/// `Codec` currently has **no `imap` method** — it stores plain `fn`
/// pointers and ships only `decode`/`encode`; it illustrates why the
/// invariant-mapping signature takes two functions rather than providing
/// the operation itself.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::Codec;
///
/// // Codec for i32 <-> String
/// let int_codec = Codec::new(
///     |s: &String| s.parse::<i32>().ok(),
///     |i: &i32| i.to_string()
/// );
///
/// assert_eq!(int_codec.decode(&"42".to_string()), Some(42));
/// assert_eq!(int_codec.encode(&42), "42".to_string());
/// ```
#[derive(Clone)]
pub struct Codec<R, A> {
    decode: fn(&R) -> Option<A>,
    encode: fn(&A) -> R,
}

impl<R, A> Codec<R, A> {
    /// Create a new codec with decode and encode functions.
    #[inline]
    pub fn new(decode: fn(&R) -> Option<A>, encode: fn(&A) -> R) -> Self {
        Self { decode, encode }
    }

    /// Decode a representation to a value.
    #[inline]
    pub fn decode(&self, r: &R) -> Option<A> {
        (self.decode)(r)
    }

    /// Encode a value to a representation.
    #[inline]
    pub fn encode(&self, a: &A) -> R {
        (self.encode)(a)
    }
}

// ============================================================================
// Standalone imap functions for convenience
// ============================================================================

/// Apply an invariant map to an Option.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::imap_option;
///
/// let x = Some(42);
/// let y: Option<String> = imap_option(x, |n| n.to_string(), |s: String| s.parse().unwrap_or(0));
/// assert_eq!(y, Some("42".to_string()));
/// ```
#[inline]
pub fn imap_option<A, B, F, G>(opt: Option<A>, f: F, _g: G) -> Option<B>
where
    F: FnOnce(A) -> B,
    G: FnOnce(B) -> A,
{
    opt.map(f)
}

/// Apply an invariant map to a Result.
///
/// # Errors
///
/// Propagates the existing error unchanged when `res` is `Err`; the
/// mapping introduces no new failure modes.
#[inline]
pub fn imap_result<A, B, E, F, G>(res: Result<A, E>, f: F, _g: G) -> Result<B, E>
where
    F: FnOnce(A) -> B,
    G: FnOnce(B) -> A,
{
    res.map(f)
}

/// Apply an invariant map to a Vec.
#[inline]
pub fn imap_vec<A, B, F, G>(vec: Vec<A>, f: F, _g: G) -> Vec<B>
where
    F: FnMut(A) -> B,
    G: FnMut(B) -> A,
{
    vec.into_iter().map(f).collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec;

    #[test]
    fn test_option_imap() {
        let x = Some(42i32);
        let y: Option<String> = x.imap(|n| n.to_string(), |s: String| s.parse().unwrap_or(0));
        assert_eq!(y, Some("42".to_string()));
    }

    #[test]
    fn test_option_none_imap() {
        let x: Option<i32> = None;
        let y: Option<String> = x.imap(|n| n.to_string(), |s: String| s.parse().unwrap_or(0));
        assert_eq!(y, None);
    }

    #[test]
    fn test_result_imap() {
        let x: Result<i32, &str> = Ok(42);
        let y: Result<String, &str> = x.imap(|n| n.to_string(), |s: String| s.parse().unwrap_or(0));
        assert_eq!(y, Ok("42".to_string()));
    }

    #[test]
    fn test_vec_imap() {
        let x = vec![1i32, 2, 3];
        let y: Vec<String> = x.imap(|n| n.to_string(), |s: String| s.parse().unwrap_or(0));
        assert_eq!(y, vec!["1".to_string(), "2".to_string(), "3".to_string()]);
    }

    #[test]
    fn test_box_imap() {
        let x = Box::new(10i32);
        let y: Box<String> = x.imap(|n| n.to_string(), |s: String| s.parse().unwrap_or(0));
        assert_eq!(*y, "10".to_string());
    }

    #[test]
    fn test_phantom_imap() {
        let x: PhantomData<i32> = PhantomData;
        let _y: PhantomData<String> = x.imap(|_: i32| "hello".to_string(), |_: String| 0);
    }

    #[test]
    fn test_btreemap_imap() {
        let mut x = BTreeMap::new();
        x.insert("a", 1i32);
        x.insert("b", 2);

        let y: BTreeMap<&str, String> =
            x.imap(|n| n.to_string(), |s: String| s.parse().unwrap_or(0));

        assert_eq!(y.get("a"), Some(&"1".to_string()));
        assert_eq!(y.get("b"), Some(&"2".to_string()));
    }

    #[test]
    fn test_identity_law() {
        // imap(fa, id, id) = fa
        let x = Some(42);
        let y: Option<i32> = x.imap(|a| a, |a| a);
        assert_eq!(x, y);
    }

    #[test]
    fn test_composition_law() {
        // imap(imap(fa, f1, g1), f2, g2) = imap(fa, f2 ∘ f1, g1 ∘ g2)
        let x = Some(10);

        // Left side: apply f1, g1 then f2, g2
        let left: Option<i32> = x.imap(|n| n * 2, |n| n / 2).imap(|n| n + 1, |n| n - 1);

        // Right side: compose the functions
        let right: Option<i32> = x.imap(|n| (n * 2) + 1, |n| (n - 1) / 2);

        assert_eq!(left, right);
    }

    #[test]
    fn test_codec_basic() {
        let int_codec = Codec::new(|s: &String| s.parse::<i32>().ok(), |i: &i32| i.to_string());

        assert_eq!(int_codec.decode(&"42".to_string()), Some(42));
        assert_eq!(int_codec.encode(&42), "42".to_string());
    }

    #[test]
    fn test_imap_option_fn() {
        let x = Some(42i32);
        let y: Option<String> =
            imap_option(x, |n| n.to_string(), |s: String| s.parse().unwrap_or(0));
        assert_eq!(y, Some("42".to_string()));
    }
}
