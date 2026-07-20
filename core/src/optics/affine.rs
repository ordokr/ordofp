//! Affine Traversal - `IteratioAffinis`
//!
//! > *"Affinis est qui unum vel nullum tangit."*
//! > — Affine is that which touches one or none. (Latin)
//!
//! This module provides affine traversals, which focus on at most one element.
//! An affine traversal is a combination of a lens (always succeeds) and a prism
//! (may fail), resulting in an optic that may or may not find a focus.
//!
//! # Overview
//!
//! An affine traversal differs from:
//! - **Lens (Aspectus)**: A lens always finds exactly one focus
//! - **Prism (Divisio)**: A prism focuses on a variant and can construct values
//! - **Affine**: An affine may find 0 or 1 focus, but cannot construct values
//!
//! # Use Cases
//!
//! - Accessing optional fields in structs
//! - Accessing elements that may not exist (e.g., map lookups)
//! - Combining lenses with optional paths
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Affine | Affinis | *affinis* = neighboring, related |
//! | Optional | Optionalis | *optio* = choice |
//! | Preview | Prospicere | *prospicere* = to look ahead |

use core::marker::PhantomData;

/// Trait alias for the preview half of an affine traversal: `Fn(&S) -> Option<A>`.
///
/// Blanket-implemented for every matching closure; exists so constructors like
/// [`iteratio_at_key`] can name their opaque return types without spelling out
/// the nested `Fn` sugar.
pub trait AffinePreviewFn<S, A>: Fn(&S) -> Option<A> {}
impl<S, A, T: Fn(&S) -> Option<A>> AffinePreviewFn<S, A> for T {}

/// Trait alias for the set half of an affine traversal: `Fn(&S, A) -> S`.
///
/// Blanket-implemented for every matching closure; the counterpart of
/// [`AffinePreviewFn`].
pub trait AffineSetFn<S, A>: Fn(&S, A) -> S {}
impl<S, A, T: Fn(&S, A) -> S> AffineSetFn<S, A> for T {}

/// Trait alias for the preview half of a keyed-map affine traversal:
/// `Fn(&BTreeMap<K, V>) -> Option<V>`.
///
/// Blanket-implemented for every matching closure; lets [`iteratio_at_key`]
/// name its opaque return type compactly.
#[cfg(feature = "alloc")]
pub trait MapPreviewFn<K, V>: Fn(&alloc::collections::BTreeMap<K, V>) -> Option<V> {}
#[cfg(feature = "alloc")]
impl<K, V, T: Fn(&alloc::collections::BTreeMap<K, V>) -> Option<V>> MapPreviewFn<K, V> for T {}

/// Trait alias for the set half of a keyed-map affine traversal:
/// `Fn(&BTreeMap<K, V>, V) -> BTreeMap<K, V>`.
///
/// Blanket-implemented for every matching closure; the counterpart of
/// [`MapPreviewFn`].
#[cfg(feature = "alloc")]
pub trait MapSetFn<K, V>:
    Fn(&alloc::collections::BTreeMap<K, V>, V) -> alloc::collections::BTreeMap<K, V>
{
}
#[cfg(feature = "alloc")]
impl<K, V, T> MapSetFn<K, V> for T where
    T: Fn(&alloc::collections::BTreeMap<K, V>, V) -> alloc::collections::BTreeMap<K, V>
{
}

/// An affine traversal (optional) focusing on at most one value.
///
/// > *"Iteratio Affinis unum vel nullum videt."*
/// > — An affine traversal sees one or none.
///
/// An affine traversal is defined by:
/// - `preview: &S -> Option<A>` - Try to extract the focused value
/// - `set: (&S, A) -> S` - Set a new value (no-op if preview would return None)
///
/// Unlike a prism, an affine cannot construct the source from the focus alone.
///
/// # Type Parameters
/// - `S` - The source type
/// - `A` - The target/focus type
pub struct IteratioAffinis<S, A, PreviewFn, SetFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    SetFn: Fn(&S, A) -> S,
{
    preview_fn: PreviewFn,
    set_fn: SetFn,
    _phantom: PhantomData<fn(&S) -> Option<A>>,
}

impl<S, A, PreviewFn, SetFn> Clone for IteratioAffinis<S, A, PreviewFn, SetFn>
where
    PreviewFn: Fn(&S) -> Option<A> + Clone,
    SetFn: Fn(&S, A) -> S + Clone,
{
    fn clone(&self) -> Self {
        Self {
            preview_fn: self.preview_fn.clone(),
            set_fn: self.set_fn.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<S, A, PreviewFn, SetFn> IteratioAffinis<S, A, PreviewFn, SetFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    SetFn: Fn(&S, A) -> S,
{
    /// Create a new affine traversal.
    ///
    /// # Arguments
    /// - `preview_fn` - Function to try to extract the focused value
    /// - `set_fn` - Function to set a new value (should be no-op if preview returns None)
    #[inline]
    pub fn new(preview_fn: PreviewFn, set_fn: SetFn) -> Self {
        Self {
            preview_fn,
            set_fn,
            _phantom: PhantomData,
        }
    }

    /// Try to extract the focused value.
    ///
    /// Returns `Some(a)` if the affine has a focus, `None` otherwise.
    #[inline]
    pub fn preview(&self, source: &S) -> Option<A> {
        (self.preview_fn)(source)
    }

    /// Set a new value, returning a modified source.
    ///
    /// If the affine has no focus, returns the original source unchanged.
    #[inline]
    pub fn set(&self, source: &S, value: A) -> S {
        (self.set_fn)(source, value)
    }

    /// Modify the focused value using a function.
    ///
    /// If the affine has no focus, returns the original source unchanged.
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> S
    where
        F: FnOnce(A) -> A,
        S: Clone,
    {
        match self.preview(source) {
            Some(a) => self.set(source, f(a)),
            None => source.clone(),
        }
    }

    /// Check if the affine has a focus for the given source.
    #[inline]
    pub fn has_focus(&self, source: &S) -> bool {
        self.preview(source).is_some()
    }

    /// Get the focused value or a default.
    #[inline]
    pub fn get_or(&self, source: &S, default: A) -> A {
        self.preview(source).unwrap_or(default)
    }

    /// Get the focused value or compute a default.
    #[inline]
    pub fn get_or_else<F>(&self, source: &S, default: F) -> A
    where
        F: FnOnce() -> A,
    {
        self.preview(source).unwrap_or_else(default)
    }

    /// Compose with another affine traversal.
    #[inline]
    pub fn compose<B, PreviewFn2, SetFn2>(
        &self,
        other: &IteratioAffinis<A, B, PreviewFn2, SetFn2>,
    ) -> ComposedIteratioAffinis<S, A, B, PreviewFn, SetFn, PreviewFn2, SetFn2>
    where
        PreviewFn: Clone,
        SetFn: Clone,
        PreviewFn2: Fn(&A) -> Option<B> + Clone,
        SetFn2: Fn(&A, B) -> A + Clone,
    {
        ComposedIteratioAffinis {
            outer: self.clone(),
            inner: other.clone(),
        }
    }
}

/// A composed affine traversal.
#[derive(Clone)]
pub struct ComposedIteratioAffinis<S, A, B, PreviewFn1, SetFn1, PreviewFn2, SetFn2>
where
    PreviewFn1: Fn(&S) -> Option<A>,
    SetFn1: Fn(&S, A) -> S,
    PreviewFn2: Fn(&A) -> Option<B>,
    SetFn2: Fn(&A, B) -> A,
{
    outer: IteratioAffinis<S, A, PreviewFn1, SetFn1>,
    inner: IteratioAffinis<A, B, PreviewFn2, SetFn2>,
}

impl<S, A, B, PreviewFn1, SetFn1, PreviewFn2, SetFn2>
    ComposedIteratioAffinis<S, A, B, PreviewFn1, SetFn1, PreviewFn2, SetFn2>
where
    PreviewFn1: Fn(&S) -> Option<A>,
    SetFn1: Fn(&S, A) -> S,
    PreviewFn2: Fn(&A) -> Option<B>,
    SetFn2: Fn(&A, B) -> A,
{
    /// Try to extract the focused value through both affines.
    #[inline]
    pub fn preview(&self, source: &S) -> Option<B> {
        self.outer
            .preview(source)
            .and_then(|a| self.inner.preview(&a))
    }

    /// Set a new value through both affines.
    #[inline]
    pub fn set(&self, source: &S, value: B) -> S
    where
        A: Clone,
        S: Clone,
    {
        match self.outer.preview(source) {
            Some(a) => {
                let new_a = self.inner.set(&a, value);
                self.outer.set(source, new_a)
            }
            None => {
                // If outer has no focus, we can't set
                // Return source unchanged
                source.clone()
            }
        }
    }

    /// Modify the focused value through both affines.
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> S
    where
        F: FnOnce(B) -> B,
        S: Clone,
        A: Clone,
    {
        match self.outer.preview(source) {
            Some(a) => match self.inner.preview(&a) {
                Some(b) => {
                    let new_b = f(b);
                    let new_a = self.inner.set(&a, new_b);
                    self.outer.set(source, new_a)
                }
                None => source.clone(),
            },
            None => source.clone(),
        }
    }

    /// Check if there is a focus through both affines.
    #[inline]
    pub fn has_focus(&self, source: &S) -> bool {
        self.preview(source).is_some()
    }
}

// =============================================================================
// Polymorphic Affine Traversal
// =============================================================================

/// A polymorphic affine traversal that can change types.
///
/// > *"Iteratio Affinis Polymorphica tipos mutat."*
///
/// # Type Parameters
/// - `S` - The source type
/// - `T` - The target source type (after modification)
/// - `A` - The focus type
/// - `B` - The new focus type
pub struct IteratioAffinisPolymorphica<S, T, A, B, PreviewFn, SetFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    SetFn: Fn(S, B) -> T,
{
    preview_fn: PreviewFn,
    set_fn: SetFn,
    _phantom: PhantomData<fn(S, T, A, B)>,
}

impl<S, T, A, B, PreviewFn, SetFn> Clone
    for IteratioAffinisPolymorphica<S, T, A, B, PreviewFn, SetFn>
where
    PreviewFn: Fn(&S) -> Option<A> + Clone,
    SetFn: Fn(S, B) -> T + Clone,
{
    fn clone(&self) -> Self {
        Self {
            preview_fn: self.preview_fn.clone(),
            set_fn: self.set_fn.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<S, T, A, B, PreviewFn, SetFn> IteratioAffinisPolymorphica<S, T, A, B, PreviewFn, SetFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    SetFn: Fn(S, B) -> T,
{
    /// Create a new polymorphic affine traversal.
    #[inline]
    pub fn new(preview_fn: PreviewFn, set_fn: SetFn) -> Self {
        Self {
            preview_fn,
            set_fn,
            _phantom: PhantomData,
        }
    }

    /// Try to extract the focused value.
    #[inline]
    pub fn preview(&self, source: &S) -> Option<A> {
        (self.preview_fn)(source)
    }

    /// Set a new value, potentially changing the type.
    #[inline]
    pub fn set(&self, source: S, value: B) -> T {
        (self.set_fn)(source, value)
    }

    /// Modify the focused value, potentially changing the type.
    #[inline]
    pub fn over<F>(&self, source: S, f: F) -> T
    where
        F: FnOnce(A) -> B,
        S: Clone,
        T: From<S>,
    {
        match self.preview(&source) {
            Some(a) => self.set(source, f(a)),
            None => T::from(source),
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a new affine traversal (simple/monomorphic).
///
/// # Example
///
/// ```rust
/// use ordofp_core::optics::iteratio_affinis;
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Config {
///     debug_level: Option<u32>,
/// }
///
/// let debug_level = iteratio_affinis(
///     |c: &Config| c.debug_level,
///     |c: &Config, level| Config { debug_level: Some(level) },
/// );
///
/// let config = Config { debug_level: Some(3) };
/// assert_eq!(debug_level.preview(&config), Some(3));
///
/// let updated = debug_level.set(&config, 5);
/// assert_eq!(updated.debug_level, Some(5));
/// ```
#[inline]
pub fn iteratio_affinis<S, A, PreviewFn, SetFn>(
    preview_fn: PreviewFn,
    set_fn: SetFn,
) -> IteratioAffinis<S, A, PreviewFn, SetFn>
where
    PreviewFn: Fn(&S) -> Option<A>,
    SetFn: Fn(&S, A) -> S,
{
    IteratioAffinis::new(preview_fn, set_fn)
}

/// Create an affine traversal from an Option field.
///
/// This is a common pattern for optional struct fields.
#[inline]
pub fn iteratio_option<S, A, GetOpt, Set>(
    get_opt: GetOpt,
    set: Set,
) -> IteratioAffinis<S, A, GetOpt, Set>
where
    GetOpt: Fn(&S) -> Option<A>,
    Set: Fn(&S, A) -> S,
{
    IteratioAffinis::new(get_opt, set)
}

/// Create an affine traversal for a map lookup.
#[cfg(feature = "alloc")]
#[inline]
pub fn iteratio_at_key<'a, K, V>(
    key: K,
) -> IteratioAffinis<
    alloc::collections::BTreeMap<K, V>,
    V,
    impl MapPreviewFn<K, V> + 'a,
    impl MapSetFn<K, V> + 'a,
>
where
    K: Ord + Clone + 'a,
    V: Clone + 'a,
{
    let key_clone = key.clone();
    IteratioAffinis::new(
        move |map: &alloc::collections::BTreeMap<K, V>| map.get(&key).cloned(),
        move |map: &alloc::collections::BTreeMap<K, V>, v: V| {
            let mut new_map = map.clone();
            new_map.insert(key_clone.clone(), v);
            new_map
        },
    )
}

/// Create an affine traversal for a vector index.
#[cfg(feature = "alloc")]
#[inline]
pub fn iteratio_at_index<T>(
    index: usize,
) -> IteratioAffinis<
    alloc::vec::Vec<T>,
    T,
    impl AffinePreviewFn<alloc::vec::Vec<T>, T>,
    impl AffineSetFn<alloc::vec::Vec<T>, T>,
>
where
    T: Clone,
{
    IteratioAffinis::new(
        move |vec: &alloc::vec::Vec<T>| vec.get(index).cloned(),
        move |vec: &alloc::vec::Vec<T>, v: T| {
            let mut new_vec = vec.clone();
            if index < new_vec.len() {
                new_vec[index] = v;
            }
            new_vec
        },
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::string::{String, ToString};
    use alloc::vec;

    #[derive(Clone, Debug, PartialEq)]
    struct Config {
        name: String,
        debug_level: Option<u32>,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Nested {
        config: Option<Config>,
    }

    #[test]
    fn test_affine_preview_some() {
        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let config = Config {
            name: "test".to_string(),
            debug_level: Some(3),
        };

        assert_eq!(debug_affine.preview(&config), Some(3));
    }

    #[test]
    fn test_affine_preview_none() {
        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let config = Config {
            name: "test".to_string(),
            debug_level: None,
        };

        assert_eq!(debug_affine.preview(&config), None);
    }

    #[test]
    fn test_affine_set() {
        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let config = Config {
            name: "test".to_string(),
            debug_level: Some(3),
        };

        let updated = debug_affine.set(&config, 5);
        assert_eq!(updated.debug_level, Some(5));
    }

    #[test]
    fn test_affine_modify() {
        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let config = Config {
            name: "test".to_string(),
            debug_level: Some(3),
        };

        let modified = debug_affine.modify(&config, |x| x * 2);
        assert_eq!(modified.debug_level, Some(6));
    }

    #[test]
    fn test_affine_modify_none() {
        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let config = Config {
            name: "test".to_string(),
            debug_level: None,
        };

        let modified = debug_affine.modify(&config, |x| x * 2);
        assert_eq!(modified.debug_level, None); // Unchanged
    }

    #[test]
    fn test_affine_has_focus() {
        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let with_debug = Config {
            name: "test".to_string(),
            debug_level: Some(3),
        };
        let without_debug = Config {
            name: "test".to_string(),
            debug_level: None,
        };

        assert!(debug_affine.has_focus(&with_debug));
        assert!(!debug_affine.has_focus(&without_debug));
    }

    #[test]
    fn test_affine_get_or() {
        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let with_debug = Config {
            name: "test".to_string(),
            debug_level: Some(3),
        };
        let without_debug = Config {
            name: "test".to_string(),
            debug_level: None,
        };

        assert_eq!(debug_affine.get_or(&with_debug, 0), 3);
        assert_eq!(debug_affine.get_or(&without_debug, 0), 0);
    }

    #[test]
    fn test_affine_composition() {
        let config_affine = iteratio_affinis(
            |n: &Nested| n.config.clone(),
            |_n: &Nested, c: Config| Nested { config: Some(c) },
        );

        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let composed = config_affine.compose(&debug_affine);

        let nested = Nested {
            config: Some(Config {
                name: "test".to_string(),
                debug_level: Some(3),
            }),
        };

        assert_eq!(composed.preview(&nested), Some(3));

        let modified = composed.modify(&nested, |x| x + 1);
        assert_eq!(
            modified.config.as_ref().and_then(|c| c.debug_level),
            Some(4)
        );
    }

    #[test]
    fn test_affine_composition_none_outer() {
        let config_affine = iteratio_affinis(
            |n: &Nested| n.config.clone(),
            |_n: &Nested, c: Config| Nested { config: Some(c) },
        );

        let debug_affine = iteratio_affinis(
            |c: &Config| c.debug_level,
            |c: &Config, level| Config {
                name: c.name.clone(),
                debug_level: Some(level),
            },
        );

        let composed = config_affine.compose(&debug_affine);

        let nested = Nested { config: None };

        assert_eq!(composed.preview(&nested), None);
        assert!(!composed.has_focus(&nested));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_affine_at_index() {
        let at_1 = iteratio_at_index::<i32>(1);

        let vec = vec![10, 20, 30];
        assert_eq!(at_1.preview(&vec), Some(20));

        let updated = at_1.set(&vec, 25);
        assert_eq!(updated, vec![10, 25, 30]);

        let empty: alloc::vec::Vec<i32> = vec![];
        assert_eq!(at_1.preview(&empty), None);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_affine_at_key() {
        use alloc::collections::BTreeMap;

        let at_foo = iteratio_at_key::<&str, i32>("foo");

        let mut map = BTreeMap::new();
        map.insert("foo", 42);
        map.insert("bar", 100);

        assert_eq!(at_foo.preview(&map), Some(42));

        let updated = at_foo.set(&map, 99);
        assert_eq!(updated.get("foo"), Some(&99));

        let empty: BTreeMap<&str, i32> = BTreeMap::new();
        assert_eq!(at_foo.preview(&empty), None);
    }
}
