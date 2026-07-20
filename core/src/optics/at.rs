//! At Optics - Indexed Container Access
//!
//! > *"Ad indicem accedere"*
//! > — To access by index. (Latin)
//!
//! This module provides the `At` trait and related types for accessing
//! elements in indexed containers like maps, arrays, and vectors.
//!
//! # Overview
//!
//! The `At` trait provides a uniform interface for accessing and modifying
//! elements in containers by their index or key. It integrates with the
//! optics system to provide lens-like access to container elements.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | At | Ad | *ad* = at, to |
//! | Index | Index | *index* = pointer, indicator |
//! | Contains | Continet | *continere* = to hold together |
//! | Remove | Removere | *removere* = to move back |

use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Trait for types that support indexed access.
///
/// > *"Ad est accessus per indicem."*
/// > — At is access through an index.
///
/// The `At` trait provides an affine traversal to an element at a given index.
/// The element may or may not exist, hence the `Option` return type.
///
/// # Type Parameters
/// - `I` - The index type
pub trait Ad<I> {
    /// The type of values stored in the container.
    type Value;

    /// Get a reference to the value at the given index.
    fn ad(&self, index: I) -> Option<&Self::Value>;

    /// Get a mutable reference to the value at the given index.
    fn ad_mut(&mut self, index: I) -> Option<&mut Self::Value>;

    /// Check if the container contains an element at the given index.
    #[inline]
    fn continet(&self, index: I) -> bool {
        self.ad(index).is_some()
    }
}

/// Trait for containers that support removal by index.
///
/// > *"Removere est indicem delere."*
pub trait AdRemovere<I>: Ad<I> {
    /// Remove the element at the given index, returning it if it existed.
    fn removere(&mut self, index: I) -> Option<Self::Value>;

    /// Remove the element without returning it.
    #[inline]
    fn sine(&mut self, index: I) -> &mut Self
    where
        Self: Sized,
    {
        self.removere(index);
        self
    }
}

/// Trait for containers that support insertion by index.
///
/// > *"Inserere est indicem ponere."*
pub trait AdInserere<I>: Ad<I> {
    /// Insert a value at the given index, returning the old value if any.
    fn inserere(&mut self, index: I, value: Self::Value) -> Option<Self::Value>;
}

// =============================================================================
// Implementations for BTreeMap
// =============================================================================

#[cfg(feature = "alloc")]
impl<K: Ord, V> Ad<K> for BTreeMap<K, V> {
    type Value = V;

    #[inline]
    fn ad(&self, index: K) -> Option<&Self::Value> {
        self.get(&index)
    }

    #[inline]
    fn ad_mut(&mut self, index: K) -> Option<&mut Self::Value> {
        self.get_mut(&index)
    }

    #[inline]
    fn continet(&self, index: K) -> bool {
        self.contains_key(&index)
    }
}

#[cfg(feature = "alloc")]
impl<K: Ord, V> AdRemovere<K> for BTreeMap<K, V> {
    #[inline]
    fn removere(&mut self, index: K) -> Option<Self::Value> {
        self.remove(&index)
    }
}

#[cfg(feature = "alloc")]
impl<K: Ord, V> AdInserere<K> for BTreeMap<K, V> {
    #[inline]
    fn inserere(&mut self, index: K, value: Self::Value) -> Option<Self::Value> {
        self.insert(index, value)
    }
}

// =============================================================================
// Implementations for Vec
// =============================================================================

#[cfg(feature = "alloc")]
impl<T> Ad<usize> for Vec<T> {
    type Value = T;

    #[inline]
    fn ad(&self, index: usize) -> Option<&Self::Value> {
        self.get(index)
    }

    #[inline]
    fn ad_mut(&mut self, index: usize) -> Option<&mut Self::Value> {
        self.get_mut(index)
    }

    #[inline]
    fn continet(&self, index: usize) -> bool {
        index < self.len()
    }
}

// =============================================================================
// Implementations for slices
// =============================================================================

impl<T> Ad<usize> for [T] {
    type Value = T;

    #[inline]
    fn ad(&self, index: usize) -> Option<&Self::Value> {
        self.get(index)
    }

    #[inline]
    fn ad_mut(&mut self, index: usize) -> Option<&mut Self::Value> {
        self.get_mut(index)
    }

    #[inline]
    fn continet(&self, index: usize) -> bool {
        index < self.len()
    }
}

// =============================================================================
// Implementations for arrays
// =============================================================================

impl<T, const N: usize> Ad<usize> for [T; N] {
    type Value = T;

    #[inline]
    fn ad(&self, index: usize) -> Option<&Self::Value> {
        self.get(index)
    }

    #[inline]
    fn ad_mut(&mut self, index: usize) -> Option<&mut Self::Value> {
        self.get_mut(index)
    }

    #[inline]
    fn continet(&self, index: usize) -> bool {
        index < N
    }
}

// =============================================================================
// At Lens - Creates an affine traversal for a specific index
// =============================================================================

/// An affine traversal focusing on a specific index in a container.
///
/// > *"Aspectus Ad est focus in indice specifico."*
pub struct AspectusAd<C, I, V>
where
    C: Ad<I, Value = V>,
    I: Clone,
{
    index: I,
    _phantom: PhantomData<fn(C) -> V>,
}

impl<C, I, V> Clone for AspectusAd<C, I, V>
where
    C: Ad<I, Value = V>,
    I: Clone,
{
    fn clone(&self) -> Self {
        AspectusAd {
            index: self.index.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<C, I, V> AspectusAd<C, I, V>
where
    C: Ad<I, Value = V>,
    I: Clone,
{
    /// Create a new at-optic for the given index.
    #[inline]
    pub fn new(index: I) -> Self {
        AspectusAd {
            index,
            _phantom: PhantomData,
        }
    }

    /// Get the index this optic focuses on.
    #[inline]
    pub fn index(&self) -> &I {
        &self.index
    }

    /// Preview the value at the index.
    #[inline]
    pub fn preview<'a>(&self, container: &'a C) -> Option<&'a V> {
        container.ad(self.index.clone())
    }

    /// Preview the value mutably at the index.
    #[inline]
    pub fn preview_mut<'a>(&self, container: &'a mut C) -> Option<&'a mut V> {
        container.ad_mut(self.index.clone())
    }

    /// Check if the container has a value at this index.
    #[inline]
    pub fn has_value(&self, container: &C) -> bool {
        container.continet(self.index.clone())
    }

    /// Modify the value at the index if it exists.
    #[inline]
    pub fn modify<F>(&self, container: &mut C, f: F)
    where
        F: FnOnce(&mut V),
    {
        if let Some(v) = container.ad_mut(self.index.clone()) {
            f(v);
        }
    }

    /// Set the value at the index if it exists.
    #[inline]
    pub fn set(&self, container: &mut C, value: V) {
        if let Some(v) = container.ad_mut(self.index.clone()) {
            *v = value;
        }
    }
}

/// Additional methods for insertable containers.
impl<C, I, V> AspectusAd<C, I, V>
where
    C: AdInserere<I, Value = V>,
    I: Clone,
{
    /// Set the value at the index, inserting if necessary.
    #[inline]
    pub fn upsert(&self, container: &mut C, value: V) {
        container.inserere(self.index.clone(), value);
    }
}

/// Additional methods for removable containers.
impl<C, I, V> AspectusAd<C, I, V>
where
    C: AdRemovere<I, Value = V>,
    I: Clone,
{
    /// Remove the value at the index.
    #[inline]
    pub fn remove(&self, container: &mut C) -> Option<V> {
        container.removere(self.index.clone())
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create an at-optic for a specific index.
///
/// # Example
///
/// ```rust
/// use ordofp_core::optics::aspectus_ad;
/// use alloc::collections::BTreeMap;
///
/// extern crate alloc;
///
/// let mut map = BTreeMap::new();
/// map.insert("key", 42);
///
/// let at_key = aspectus_ad::<BTreeMap<&str, i32>, &str, i32>("key");
/// assert_eq!(at_key.preview(&map), Some(&42));
///
/// at_key.modify(&mut map, |v| *v *= 2);
/// assert_eq!(map.get("key"), Some(&84));
/// ```
#[inline]
pub fn aspectus_ad<C, I, V>(index: I) -> AspectusAd<C, I, V>
where
    C: Ad<I, Value = V>,
    I: Clone,
{
    AspectusAd::new(index)
}

/// Extension trait for creating at-optics from containers.
pub trait AdExt<I>: Ad<I> + Sized {
    /// Create an at-optic for this container type at the given index.
    #[inline]
    fn aspectus_at(index: I) -> AspectusAd<Self, I, Self::Value>
    where
        I: Clone,
    {
        AspectusAd::new(index)
    }
}

impl<T, I> AdExt<I> for T where T: Ad<I> {}

// =============================================================================
// Ix Trait - For containers that can be indexed into
// =============================================================================

/// Trait for containers that provide an index into themselves.
///
/// This is similar to Haskell's `Ixed` typeclass.
///
/// > *"Ix est accessus sequentialis."*
pub trait Ix<I> {
    /// The type of values in the container.
    type IxValue;

    /// Try to get a reference to a value at the given index.
    fn ix(&self, index: I) -> Option<&Self::IxValue>;

    /// Try to get a mutable reference to a value at the given index.
    fn ix_mut(&mut self, index: I) -> Option<&mut Self::IxValue>;
}

// Blanket implementation: anything that implements Ad also implements Ix
impl<T, I> Ix<I> for T
where
    T: Ad<I>,
{
    type IxValue = T::Value;

    fn ix(&self, index: I) -> Option<&Self::IxValue> {
        self.ad(index)
    }

    fn ix_mut(&mut self, index: I) -> Option<&mut Self::IxValue> {
        self.ad_mut(index)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[test]
    #[cfg(feature = "alloc")]
    fn test_btree_map_ad() {
        let mut map = BTreeMap::new();
        map.insert("foo", 42);
        map.insert("bar", 100);

        assert_eq!(map.ad("foo"), Some(&42));
        assert_eq!(map.ad("baz"), None);
        assert!(map.continet("foo"));
        assert!(!map.continet("baz"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_btree_map_ad_mut() {
        let mut map = BTreeMap::new();
        map.insert("foo", 42);

        if let Some(v) = map.ad_mut("foo") {
            *v = 100;
        }

        assert_eq!(map.ad("foo"), Some(&100));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_btree_map_removere() {
        let mut map = BTreeMap::new();
        map.insert("foo", 42);

        let removed = map.removere("foo");
        assert_eq!(removed, Some(42));
        assert!(!map.continet("foo"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_btree_map_inserere() {
        let mut map = BTreeMap::new();
        map.insert("foo", 42);

        let old = map.inserere("foo", 100);
        assert_eq!(old, Some(42));
        assert_eq!(map.ad("foo"), Some(&100));

        let old2 = map.inserere("bar", 200);
        assert_eq!(old2, None);
        assert_eq!(map.ad("bar"), Some(&200));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_ad() {
        let vec = vec![10, 20, 30];

        assert_eq!(vec.ad(0), Some(&10));
        assert_eq!(vec.ad(1), Some(&20));
        assert_eq!(vec.ad(3), None);
        assert!(vec.continet(0));
        assert!(!vec.continet(10));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_ad_mut() {
        let mut vec = vec![10, 20, 30];

        if let Some(v) = vec.ad_mut(1) {
            *v = 25;
        }

        assert_eq!(vec, vec![10, 25, 30]);
    }

    #[test]
    fn test_slice_ad() {
        let slice: &[i32] = &[10, 20, 30];

        assert_eq!(slice.ad(0), Some(&10));
        assert_eq!(slice.ad(3), None);
    }

    #[test]
    fn test_array_ad() {
        let arr = [10, 20, 30];

        assert_eq!(arr.ad(0), Some(&10));
        assert_eq!(arr.ad(3), None);
        assert!(arr.continet(0));
        assert!(!arr.continet(5));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_aspectus_ad_preview() {
        let mut map = BTreeMap::new();
        map.insert("key", 42);

        let at_key = aspectus_ad::<BTreeMap<&str, i32>, &str, i32>("key");
        assert_eq!(at_key.preview(&map), Some(&42));

        let at_missing = aspectus_ad::<BTreeMap<&str, i32>, &str, i32>("missing");
        assert_eq!(at_missing.preview(&map), None);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_aspectus_ad_modify() {
        let mut map = BTreeMap::new();
        map.insert("key", 42);

        let at_key = aspectus_ad::<BTreeMap<&str, i32>, &str, i32>("key");
        at_key.modify(&mut map, |v| *v *= 2);

        assert_eq!(map.get("key"), Some(&84));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_aspectus_ad_upsert() {
        let mut map: BTreeMap<&str, i32> = BTreeMap::new();

        let at_key = aspectus_ad::<BTreeMap<&str, i32>, &str, i32>("key");
        at_key.upsert(&mut map, 42);

        assert_eq!(map.get("key"), Some(&42));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_aspectus_ad_remove() {
        let mut map = BTreeMap::new();
        map.insert("key", 42);

        let at_key = aspectus_ad::<BTreeMap<&str, i32>, &str, i32>("key");
        let removed = at_key.remove(&mut map);

        assert_eq!(removed, Some(42));
        assert!(!map.contains_key("key"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_ix_trait() {
        let vec = vec![10, 20, 30];

        assert_eq!(vec.ix(0), Some(&10));
        assert_eq!(vec.ix(5), None);
    }
}
