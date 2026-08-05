//! # Persistent Ordered Map
//!
//! A persistent ordered map using a balanced binary search tree.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::iter::FromIterator;

// ============================================================================
// OrdMap
// ============================================================================

/// A persistent ordered map with O(log n) operations.
///
/// # Example
/// ```
/// use ordofp_core::pfds::OrdMap;
///
/// let map = OrdMap::new()
///     .insert("b", 2)
///     .insert("a", 1)
///     .insert("c", 3);
///
/// assert_eq!(map.get(&"b"), Some(&2));
/// assert_eq!(map.len(), 3);
/// ```
pub struct OrdMap<K, V> {
    root: Option<Arc<Node<K, V>>>,
    len: usize,
}

struct Node<K, V> {
    key: K,
    value: V,
    left: Option<Arc<Node<K, V>>>,
    right: Option<Arc<Node<K, V>>>,
    height: u8,
}

impl<K: Clone, V: Clone> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        Node {
            key: self.key.clone(),
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            height: self.height,
        }
    }
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            left: None,
            right: None,
            height: 1,
        }
    }
}

fn height<K, V>(node: &Option<Arc<Node<K, V>>>) -> u8 {
    node.as_ref().map_or(0, |n| n.height)
}

fn balance_factor<K, V>(node: &Node<K, V>) -> i8 {
    height(&node.right) as i8 - height(&node.left) as i8
}

fn update_height<K, V>(node: &mut Node<K, V>) {
    node.height = 1 + core::cmp::max(height(&node.left), height(&node.right));
}

fn build_balanced_from_sorted_iter<K, V, I>(
    iter: &mut I,
    len: usize,
) -> (Option<Arc<Node<K, V>>>, u8)
where
    I: Iterator<Item = (K, V)>,
{
    if len == 0 {
        return (None, 0);
    }

    let left_len = len / 2;
    let right_len = len - left_len - 1;

    let (left, hl) = build_balanced_from_sorted_iter(iter, left_len);
    let (key, value) = iter.next().expect("iterator length mismatch");
    let (right, hr) = build_balanced_from_sorted_iter(iter, right_len);

    let height = 1 + core::cmp::max(hl, hr);

    (
        Some(Arc::new(Node {
            key,
            value,
            left,
            right,
            height,
        })),
        height,
    )
}

fn ordmap_from_sorted_unique<K, V>(entries: Vec<(K, V)>) -> OrdMap<K, V> {
    let len = entries.len();
    let mut iter = entries.into_iter();
    let (root, _) = build_balanced_from_sorted_iter(&mut iter, len);
    OrdMap { root, len }
}

/// Batch builder for [`OrdMap`] (*structor* = builder).
///
/// Accumulates key-value pairs in a plain `Vec`, then sorts, deduplicates
/// (last insertion wins), and constructs a perfectly balanced persistent map
/// in one pass on `finish`. This is O(n log n) total, versus O(n log n) with
/// far higher constants for n successive persistent inserts, because no
/// intermediate tree versions are allocated.
pub struct OrdMapStructor<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> Default for OrdMapStructor<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> OrdMapStructor<K, V> {
    /// Create an empty builder with no pre-allocated capacity.
    #[inline]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Create an empty builder pre-allocating space for `capacity` entries.
    ///
    /// Use this when the number of entries is known in advance to avoid
    /// repeated reallocations during [`insert`](Self::insert) calls.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Stage a key-value pair for inclusion in the final [`OrdMap`].
    ///
    /// If the same key is inserted more than once, the last value wins
    /// when [`finish`](OrdMapStructor::finish) is called.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.entries.push((key, value));
    }

    /// Stage all key-value pairs from `iter` for inclusion in the final [`OrdMap`].
    ///
    /// Equivalent to calling [`insert`](Self::insert) for each item.
    #[inline]
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        self.entries.extend(iter);
    }
}

impl<K: Ord, V> OrdMapStructor<K, V> {
    /// Consume the builder and produce a balanced [`OrdMap`].
    ///
    /// Entries are sorted by key and deduplicated: if the same key was
    /// inserted multiple times, the last value wins. The resulting tree is
    /// height-balanced so all subsequent O(log n) operations are efficient.
    #[inline]
    pub fn finish(mut self) -> OrdMap<K, V> {
        self.entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

        let mut unique: Vec<(K, V)> = Vec::with_capacity(self.entries.len());
        for (k, v) in self.entries {
            if let Some((last_k, last_v)) = unique.last_mut()
                && Ordering::Equal == (*last_k).cmp(&k)
            {
                *last_v = v;
                continue;
            }
            unique.push((k, v));
        }

        ordmap_from_sorted_unique(unique)
    }

    /// Finish building with parallel sort (requires `rayon` feature).
    ///
    /// Uses Rayon for parallel sorting when the entry count is large.
    #[cfg(feature = "rayon")]
    #[inline]
    pub fn finish_par(mut self) -> OrdMap<K, V>
    where
        K: Send,
        V: Send,
    {
        use rayon::prelude::*;

        // Use parallel sort for large collections
        if self.entries.len() > 1024 {
            self.entries.par_sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        } else {
            self.entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        }

        let mut unique: Vec<(K, V)> = Vec::with_capacity(self.entries.len());
        for (k, v) in self.entries {
            if let Some((last_k, last_v)) = unique.last_mut()
                && Ordering::Equal == (*last_k).cmp(&k)
            {
                *last_v = v;
                continue;
            }
            unique.push((k, v));
        }

        ordmap_from_sorted_unique(unique)
    }
}

impl<K, V> Clone for OrdMap<K, V> {
    fn clone(&self) -> Self {
        OrdMap {
            root: self.root.clone(),
            len: self.len,
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for OrdMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Default for OrdMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for OrdMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        self.iter()
            .zip(other.iter())
            .all(|((k1, v1), (k2, v2))| k1 == k2 && v1 == v2)
    }
}

impl<K: Eq, V: Eq> Eq for OrdMap<K, V> {}

impl<K, V> OrdMap<K, V> {
    /// Create an empty map.
    #[inline]
    pub fn new() -> Self {
        OrdMap { root: None, len: 0 }
    }

    /// Check if the map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the number of entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Create a builder for constructing an `OrdMap` from an unsorted collection of entries.
    ///
    /// The returned [`OrdMapStructor`] accumulates key-value pairs via
    /// [`insert`](OrdMapStructor::insert) and then sorts and deduplicates them
    /// when [`finish`](OrdMapStructor::finish) is called.  Duplicate keys are
    /// resolved by keeping the last inserted value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::OrdMap;
    ///
    /// let mut structor = OrdMap::structor();
    /// structor.insert("b", 2);
    /// structor.insert("a", 1);
    /// let map = structor.finish();
    ///
    /// assert_eq!(map.get(&"a"), Some(&1));
    /// assert_eq!(map.get(&"b"), Some(&2));
    /// ```
    #[inline]
    pub fn structor() -> OrdMapStructor<K, V> {
        OrdMapStructor::new()
    }
}

impl<K: Ord + Clone, V: Clone> OrdMap<K, V> {
    /// Insert a key-value pair.
    ///
    /// # Complexity
    /// O(log n)
    #[inline]
    pub fn insert(&self, key: K, value: V) -> Self {
        let (new_root, inserted) = insert_node(self.root.clone(), key, value);
        OrdMap {
            root: Some(new_root),
            len: if inserted { self.len + 1 } else { self.len },
        }
    }

    /// Remove a key.
    ///
    /// The key may be any borrowed form of the key type (e.g. `&str` for
    /// `String` keys), avoiding an owned-key allocation for the probe.
    ///
    /// # Complexity
    /// O(log n)
    #[inline]
    pub fn remove<Q>(&self, key: &Q) -> Self
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (new_root, removed) = remove_node(self.root.clone(), key);
        OrdMap {
            root: new_root,
            len: if removed {
                self.len.saturating_sub(1)
            } else {
                self.len
            },
        }
    }

    /// Get minimum key-value pair.
    #[inline]
    pub fn min(&self) -> Option<(&K, &V)> {
        min_node(&self.root)
    }

    /// Get maximum key-value pair.
    #[inline]
    pub fn max(&self) -> Option<(&K, &V)> {
        max_node(&self.root)
    }

    /// Union of two maps (requires `rayon` feature for parallel merge).
    ///
    /// When keys conflict, values from `other` take precedence.
    ///
    /// Currently delegates to the sequential merge-join — the previous "large
    /// input" branch duplicated the same sequential merge inline and no
    /// parallel implementation has paid off in benchmarks. The signature keeps
    /// the `Send + Sync` bounds so a parallel implementation stays non-breaking.
    #[cfg(feature = "rayon")]
    pub fn union_par(&self, other: &Self) -> Self
    where
        K: Send + Sync,
        V: Send + Sync,
    {
        self.union(other)
    }

    /// Sequential union (fallback for small maps).
    pub fn union(&self, other: &Self) -> Self {
        let mut result = Vec::with_capacity(self.len().saturating_add(other.len()));
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();

        let mut self_next = self_iter.next();
        let mut other_next = other_iter.next();

        loop {
            match (self_next, other_next) {
                (Some((k1, v1)), Some((k2, v2))) => match k1.cmp(k2) {
                    Ordering::Less => {
                        result.push((k1.clone(), v1.clone()));
                        self_next = self_iter.next();
                    }
                    Ordering::Equal => {
                        result.push((k2.clone(), v2.clone()));
                        self_next = self_iter.next();
                        other_next = other_iter.next();
                    }
                    Ordering::Greater => {
                        result.push((k2.clone(), v2.clone()));
                        other_next = other_iter.next();
                    }
                },
                (Some((k, v)), None) => {
                    result.push((k.clone(), v.clone()));
                    result.extend(self_iter.map(|(k, v)| (k.clone(), v.clone())));
                    break;
                }
                (None, Some((k, v))) => {
                    result.push((k.clone(), v.clone()));
                    result.extend(other_iter.map(|(k, v)| (k.clone(), v.clone())));
                    break;
                }
                (None, None) => break,
            }
        }

        ordmap_from_sorted_unique(result)
    }

    /// Intersection of two maps (requires `rayon` feature for parallel merge).
    ///
    /// Currently delegates to the sequential merge-join — the previous "large
    /// input" branch duplicated the same sequential merge inline and no
    /// parallel implementation has paid off in benchmarks. The signature keeps
    /// the `Send + Sync` bounds so a parallel implementation stays non-breaking.
    #[cfg(feature = "rayon")]
    pub fn intersection_par(&self, other: &Self) -> Self
    where
        K: Send + Sync,
        V: Send + Sync,
    {
        self.intersection(other)
    }

    /// Sequential intersection (fallback for small maps).
    pub fn intersection(&self, other: &Self) -> Self {
        let mut result = Vec::with_capacity(self.len().min(other.len()));
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();

        let mut self_next = self_iter.next();
        let mut other_next = other_iter.next();

        while let (Some((k1, v1)), Some((k2, _))) = (self_next, other_next) {
            match k1.cmp(k2) {
                Ordering::Less => {
                    self_next = self_iter.next();
                }
                Ordering::Equal => {
                    result.push((k1.clone(), v1.clone()));
                    self_next = self_iter.next();
                    other_next = other_iter.next();
                }
                Ordering::Greater => {
                    other_next = other_iter.next();
                }
            }
        }

        ordmap_from_sorted_unique(result)
    }

    /// Difference (keys in self but not in other) (requires `rayon` feature for parallel merge).
    ///
    /// Currently delegates to the sequential merge-join — the previous "large
    /// input" branch duplicated the same sequential merge inline and no
    /// parallel implementation has paid off in benchmarks. The signature keeps
    /// the `Send + Sync` bounds so a parallel implementation stays non-breaking.
    #[cfg(feature = "rayon")]
    pub fn difference_par(&self, other: &Self) -> Self
    where
        K: Send + Sync,
        V: Send + Sync,
    {
        self.difference(other)
    }

    /// Sequential difference (fallback for small maps).
    pub fn difference(&self, other: &Self) -> Self {
        let mut result = Vec::with_capacity(self.len());
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();

        let mut self_next = self_iter.next();
        let mut other_next = other_iter.next();

        loop {
            match (self_next, other_next) {
                (Some((k1, v1)), Some((k2, _))) => match k1.cmp(k2) {
                    Ordering::Less => {
                        result.push((k1.clone(), v1.clone()));
                        self_next = self_iter.next();
                    }
                    Ordering::Equal => {
                        self_next = self_iter.next();
                        other_next = other_iter.next();
                    }
                    Ordering::Greater => {
                        other_next = other_iter.next();
                    }
                },
                (Some((k, v)), None) => {
                    result.push((k.clone(), v.clone()));
                    result.extend(self_iter.map(|(k, v)| (k.clone(), v.clone())));
                    break;
                }
                (None, _) => break,
            }
        }

        ordmap_from_sorted_unique(result)
    }
}

impl<K: Ord, V> OrdMap<K, V> {
    /// Get a value by key.
    ///
    /// The key may be any borrowed form of the map's key type, so an
    /// `OrdMap<String, V>` can be queried with a `&str` without allocating an
    /// owned key — matching `std::collections::BTreeMap::get`.
    ///
    /// # Complexity
    /// O(log n)
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        get_node(&self.root, key)
    }

    /// Check if a key exists.
    ///
    /// Accepts any borrowed form of the key type (e.g. `&str` for `String`
    /// keys), avoiding an owned-key allocation per query.
    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }
}

impl<K, V> OrdMap<K, V> {
    /// Iterate over key-value pairs in order.
    #[inline]
    pub fn iter(&self) -> OrdMapIter<'_, K, V> {
        // Pre-size the stack to the AVL tree height (≤ 1.44 log2 n) to
        // avoid reallocation during in-order traversal.
        let stack_cap = if self.len > 1 {
            usize::BITS as usize - self.len.leading_zeros() as usize
        } else {
            0
        };
        OrdMapIter {
            stack: Vec::with_capacity(stack_cap),
            current: self.root.as_ref().map(Arc::as_ref),
        }
    }

    /// Iterate over keys in order.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }

    /// Iterate over values in order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }
}

// ============================================================================
// AVL Tree Operations
// ============================================================================

fn get_node<'a, K, V, Q>(node: &'a Option<Arc<Node<K, V>>>, key: &Q) -> Option<&'a V>
where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
{
    let n = node.as_ref()?;
    match key.cmp(n.key.borrow()) {
        Ordering::Less => get_node(&n.left, key),
        Ordering::Equal => Some(&n.value),
        Ordering::Greater => get_node(&n.right, key),
    }
}

fn insert_node<K: Ord + Clone, V: Clone>(
    node: Option<Arc<Node<K, V>>>,
    key: K,
    value: V,
) -> (Arc<Node<K, V>>, bool) {
    match node {
        None => (Arc::new(Node::new(key, value)), true),
        Some(n) => {
            let (new_node, inserted) = match key.cmp(&n.key) {
                Ordering::Less => {
                    let (new_left, inserted) = insert_node(n.left.clone(), key, value);
                    let mut new = Node {
                        key: n.key.clone(),
                        value: n.value.clone(),
                        left: Some(new_left),
                        right: n.right.clone(),
                        height: n.height,
                    };
                    update_height(&mut new);
                    (new, inserted)
                }
                Ordering::Equal => {
                    let new = Node {
                        key,
                        value,
                        left: n.left.clone(),
                        right: n.right.clone(),
                        height: n.height,
                    };
                    (new, false)
                }
                Ordering::Greater => {
                    let (new_right, inserted) = insert_node(n.right.clone(), key, value);
                    let mut new = Node {
                        key: n.key.clone(),
                        value: n.value.clone(),
                        left: n.left.clone(),
                        right: Some(new_right),
                        height: n.height,
                    };
                    update_height(&mut new);
                    (new, inserted)
                }
            };
            (balance(new_node), inserted)
        }
    }
}

fn remove_node<K, V, Q>(node: Option<Arc<Node<K, V>>>, key: &Q) -> (Option<Arc<Node<K, V>>>, bool)
where
    K: Clone + Borrow<Q>,
    V: Clone,
    Q: Ord + ?Sized,
{
    match node {
        None => (None, false),
        Some(n) => match key.cmp(n.key.borrow()) {
            Ordering::Less => {
                let (new_left, removed) = remove_node(n.left.clone(), key);
                let mut new = Node {
                    key: n.key.clone(),
                    value: n.value.clone(),
                    left: new_left,
                    right: n.right.clone(),
                    height: n.height,
                };
                update_height(&mut new);
                (Some(balance(new)), removed)
            }
            Ordering::Equal => {
                match (&n.left, &n.right) {
                    (None, None) => (None, true),
                    (Some(l), None) => (Some(l.clone()), true),
                    (None, Some(r)) => (Some(r.clone()), true),
                    (Some(_), Some(r)) => {
                        // Replace with successor (in-order successor from right subtree)
                        let right_subtree = Some(r.clone());
                        let (succ_key, succ_value) = min_node(&right_subtree).unwrap();
                        let succ_key = succ_key.clone();
                        let succ_value = succ_value.clone();
                        // Recurse by the successor's own key. `succ_key: K` and
                        // `K: Borrow<Q>`, so `succ_key.borrow(): &Q` keeps the
                        // descent type uniform (Borrow guarantees Ord consistency).
                        let (new_right, _) = remove_node(n.right.clone(), succ_key.borrow());
                        let mut new = Node {
                            key: succ_key,
                            value: succ_value,
                            left: n.left.clone(),
                            right: new_right,
                            height: n.height,
                        };
                        update_height(&mut new);
                        (Some(balance(new)), true)
                    }
                }
            }
            Ordering::Greater => {
                let (new_right, removed) = remove_node(n.right.clone(), key);
                let mut new = Node {
                    key: n.key.clone(),
                    value: n.value.clone(),
                    left: n.left.clone(),
                    right: new_right,
                    height: n.height,
                };
                update_height(&mut new);
                (Some(balance(new)), removed)
            }
        },
    }
}

fn min_node<K, V>(node: &Option<Arc<Node<K, V>>>) -> Option<(&K, &V)> {
    fn go<K, V>(n: &Node<K, V>) -> (&K, &V) {
        match &n.left {
            Some(left) => go(left),
            None => (&n.key, &n.value),
        }
    }
    node.as_ref().map(|n| go(n))
}

fn max_node<K, V>(node: &Option<Arc<Node<K, V>>>) -> Option<(&K, &V)> {
    fn go<K, V>(n: &Node<K, V>) -> (&K, &V) {
        match &n.right {
            Some(right) => go(right),
            None => (&n.key, &n.value),
        }
    }
    node.as_ref().map(|n| go(n))
}

fn balance<K: Clone, V: Clone>(mut node: Node<K, V>) -> Arc<Node<K, V>> {
    let bf = balance_factor(&node);

    if bf > 1 {
        // Right heavy
        if let Some(right) = node.right.take() {
            let right_node = Arc::try_unwrap(right).unwrap_or_else(|arc| (*arc).clone());
            if balance_factor(&right_node) < 0 {
                // Right-Left case
                node.right = Some(rotate_right(right_node));
            } else {
                node.right = Some(Arc::new(right_node));
            }
        }
        return rotate_left(node);
    }

    if bf < -1 {
        // Left heavy
        if let Some(left) = node.left.take() {
            let left_node = Arc::try_unwrap(left).unwrap_or_else(|arc| (*arc).clone());
            if balance_factor(&left_node) > 0 {
                // Left-Right case
                node.left = Some(rotate_left(left_node));
            } else {
                node.left = Some(Arc::new(left_node));
            }
        }
        return rotate_right(node);
    }

    Arc::new(node)
}

fn rotate_left<K: Clone, V: Clone>(mut node: Node<K, V>) -> Arc<Node<K, V>> {
    let right = node.right.take().expect("rotate_left: no right child");
    let right_left = right.left.clone();
    let right_right = right.right.clone();
    let left_height = height(&node.left);
    let right_left_height = height(&right_left);

    let new_left = Node {
        key: node.key,
        value: node.value,
        left: node.left,
        right: right_left,
        height: 1 + core::cmp::max(left_height, right_left_height),
    };

    let mut new_root = Node {
        key: right.key.clone(),
        value: right.value.clone(),
        left: Some(Arc::new(new_left)),
        right: right_right,
        height: 0,
    };
    update_height(&mut new_root);

    Arc::new(new_root)
}

fn rotate_right<K: Clone, V: Clone>(mut node: Node<K, V>) -> Arc<Node<K, V>> {
    let left = node.left.take().expect("rotate_right: no left child");
    let left_left = left.left.clone();
    let left_right = left.right.clone();
    let right_height = height(&node.right);
    let left_right_height = height(&left_right);

    let new_right = Node {
        key: node.key,
        value: node.value,
        left: left_right,
        right: node.right,
        height: 1 + core::cmp::max(left_right_height, right_height),
    };

    let mut new_root = Node {
        key: left.key.clone(),
        value: left.value.clone(),
        left: left_left,
        right: Some(Arc::new(new_right)),
        height: 0,
    };
    update_height(&mut new_root);

    Arc::new(new_root)
}

// ============================================================================
// Iterator
// ============================================================================

/// Iterator over map entries.
pub struct OrdMapIter<'a, K, V> {
    stack: Vec<&'a Node<K, V>>,
    current: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iterator for OrdMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(node) = self.current {
                self.stack.push(node);
                self.current = node.left.as_ref().map(Arc::as_ref);
            } else {
                let node = self.stack.pop()?;
                self.current = node.right.as_ref().map(Arc::as_ref);
                return Some((&node.key, &node.value));
            }
        }
    }
}

// ============================================================================
// FromIterator
// ============================================================================

impl<K: Ord, V> FromIterator<(K, V)> for OrdMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut s = OrdMap::structor();
        s.extend(iter);
        s.finish()
    }
}

impl<'a, K, V> IntoIterator for &'a OrdMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = OrdMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ============================================================================
// Serde
// ============================================================================
//
// S1 audit fix: the tree used to derive Serialize/Deserialize directly on
// `Node`, so an attacker-supplied payload could dictate `left`/`right`/
// `height` fields directly — unbounded recursion (stack-overflow DoS) and
// forged AVL invariants. Instead we serialize as an in-order entry sequence
// (depth O(log n), via the existing iterator) and deserialize by rebuilding
// through `insert`, so a forged tree shape is unrepresentable.

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<K: serde::Serialize, V: serde::Serialize> serde::Serialize for OrdMap<K, V> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(self.iter())
    }
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<'de, K, V> serde::Deserialize<'de> for OrdMap<K, V>
where
    // Mirrors `insert`'s real bounds (`impl<K: Ord + Clone, V: Clone> OrdMap<K, V>`).
    K: serde::Deserialize<'de> + Ord + Clone,
    V: serde::Deserialize<'de> + Clone,
{
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let entries = Vec::<(K, V)>::deserialize(d)?;
        Ok(entries
            .into_iter()
            .fold(OrdMap::new(), |m, (k, v)| m.insert(k, v)))
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
    fn test_basic_operations() {
        let map = OrdMap::new().insert("b", 2).insert("a", 1).insert("c", 3);

        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&"a"), Some(&1));
        assert_eq!(map.get(&"b"), Some(&2));
        assert_eq!(map.get(&"c"), Some(&3));
        assert_eq!(map.get(&"d"), None);
    }

    #[test]
    fn test_update() {
        let map = OrdMap::new().insert("a", 1);
        let map = map.insert("a", 10);

        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&"a"), Some(&10));
    }

    #[test]
    fn test_from_iter_duplicate_keys_last_wins() {
        let map: OrdMap<i32, i32> = [(1, 10), (2, 20), (1, 99)].into_iter().collect();

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&1), Some(&99));
        assert_eq!(map.get(&2), Some(&20));
    }

    #[test]
    fn test_structor_builds() {
        let mut s = OrdMap::structor();
        s.insert(2, "b");
        s.insert(1, "a");
        s.insert(3, "c");

        let map = s.finish();

        let keys: Vec<_> = map.keys().copied().collect();
        assert_eq!(keys, vec![1, 2, 3]);
    }

    #[test]
    fn test_structor_duplicate_keys_last_wins() {
        let mut s = OrdMap::structor();
        s.insert(1, "first");
        s.insert(2, "only");
        s.insert(1, "second");
        s.insert(1, "last");

        let map = s.finish();

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&1), Some(&"last"));
        assert_eq!(map.get(&2), Some(&"only"));
    }

    #[test]
    fn test_remove() {
        let map = OrdMap::new().insert("a", 1).insert("b", 2).insert("c", 3);

        let map = map.remove(&"b");
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&"b"), None);
    }

    #[test]
    fn test_iteration() {
        let map = OrdMap::new().insert(3, "c").insert(1, "a").insert(2, "b");

        let keys: Vec<_> = map.keys().copied().collect();
        assert_eq!(keys, vec![1, 2, 3]);
    }

    #[test]
    fn test_min_max() {
        let map = OrdMap::new().insert(3, "c").insert(1, "a").insert(2, "b");

        assert_eq!(map.min(), Some((&1, &"a")));
        assert_eq!(map.max(), Some((&3, &"c")));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn serde_roundtrip_map() {
        let m = (0..1000).fold(OrdMap::new(), |m, i| m.insert(i, i * 2));
        let json = serde_json::to_string(&m).unwrap();
        let back: OrdMap<i32, i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    /// S1 regression: attacker-shaped input can no longer dictate tree
    /// structure (depth/height/ordering) — only entries.
    #[test]
    fn serde_input_cannot_forge_structure() {
        let back: OrdMap<i32, i32> = serde_json::from_str("[[3,30],[1,10],[2,20]]").unwrap();
        assert_eq!(back.get(&1), Some(&10));
        assert_eq!(back.get(&2), Some(&20));
        assert_eq!(back.get(&3), Some(&30));
    }

    /// S1 regression: a large input must deserialize without overflowing the
    /// stack (rebuilt via `insert`, one AVL rotation at a time).
    #[test]
    fn serde_deep_input_no_overflow() {
        let json =
            serde_json::to_string(&(0..100_000u32).map(|i| (i, i)).collect::<Vec<_>>()).unwrap();
        let m: OrdMap<u32, u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(m.len(), 100_000);
    }
}
