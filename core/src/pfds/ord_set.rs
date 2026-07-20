//! # Persistent Ordered Set
//!
//! A persistent ordered set using a balanced binary search tree.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::iter::FromIterator;

// ============================================================================
// OrdSet
// ============================================================================

/// A persistent ordered set with O(log n) operations.
///
/// # Example
/// ```
/// use ordofp_core::pfds::OrdSet;
///
/// let set = OrdSet::new()
///     .insert(3)
///     .insert(1)
///     .insert(2);
///
/// assert!(set.contains(&2));
/// assert_eq!(set.len(), 3);
/// ```
pub struct OrdSet<A> {
    root: Option<Arc<Node<A>>>,
    len: usize,
}

struct Node<A> {
    value: A,
    left: Option<Arc<Node<A>>>,
    right: Option<Arc<Node<A>>>,
    height: u8,
}

impl<A: Clone> Clone for Node<A> {
    fn clone(&self) -> Self {
        Node {
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            height: self.height,
        }
    }
}

impl<A> Node<A> {
    fn new(value: A) -> Self {
        Node {
            value,
            left: None,
            right: None,
            height: 1,
        }
    }
}

fn height<A>(node: &Option<Arc<Node<A>>>) -> u8 {
    node.as_ref().map_or(0, |n| n.height)
}

fn balance_factor<A>(node: &Node<A>) -> i8 {
    height(&node.right) as i8 - height(&node.left) as i8
}

fn update_height<A>(node: &mut Node<A>) {
    node.height = 1 + core::cmp::max(height(&node.left), height(&node.right));
}

fn build_balanced_from_sorted_iter<A, I>(iter: &mut I, len: usize) -> (Option<Arc<Node<A>>>, u8)
where
    I: Iterator<Item = A>,
{
    if len == 0 {
        return (None, 0);
    }

    let left_len = len / 2;
    let right_len = len - left_len - 1;

    let (left, hl) = build_balanced_from_sorted_iter(iter, left_len);
    let value = iter.next().expect("iterator length mismatch");
    let (right, hr) = build_balanced_from_sorted_iter(iter, right_len);

    let height = 1 + core::cmp::max(hl, hr);

    (
        Some(Arc::new(Node {
            value,
            left,
            right,
            height,
        })),
        height,
    )
}

fn ordset_from_sorted_unique<A>(items: Vec<A>) -> OrdSet<A> {
    let len = items.len();
    let mut iter = items.into_iter();
    let (root, _) = build_balanced_from_sorted_iter(&mut iter, len);
    OrdSet { root, len }
}

/// A builder for [`OrdSet`] that collects items in any order and constructs a
/// balanced persistent ordered set upon [`finish`](OrdSetStructor::finish).
///
/// Use this when you need to insert many items before querying; it sorts and
/// deduplicates exactly once, which is more efficient than inserting into the
/// tree one element at a time.
pub struct OrdSetStructor<A> {
    items: Vec<A>,
}

impl<A> Default for OrdSetStructor<A> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A> OrdSetStructor<A> {
    /// Creates an empty `OrdSetStructor` with no pre-allocated capacity.
    #[inline]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Creates an empty `OrdSetStructor` with at least the given capacity pre-allocated.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Stages a single element to be included in the final `OrdSet`.
    #[inline]
    pub fn push(&mut self, value: A) {
        self.items.push(value);
    }

    /// Stages all elements from an iterator to be included in the final `OrdSet`.
    #[inline]
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = A>,
    {
        self.items.extend(iter);
    }
}

impl<A: Ord> OrdSetStructor<A> {
    /// Sorts and deduplicates all staged elements, returning the final `OrdSet`.
    #[inline]
    pub fn finish(mut self) -> OrdSet<A> {
        self.items.sort();

        let mut unique: Vec<A> = Vec::with_capacity(self.items.len());
        for item in self.items {
            if let Some(last) = unique.last()
                && Ordering::Equal == last.cmp(&item)
            {
                continue;
            }
            unique.push(item);
        }

        ordset_from_sorted_unique(unique)
    }

    /// Finish building with parallel sort (requires `rayon` feature).
    ///
    /// Uses Rayon for parallel sorting when the entry count is large.
    #[cfg(feature = "rayon")]
    #[inline]
    pub fn finish_par(mut self) -> OrdSet<A>
    where
        A: Send,
    {
        use rayon::prelude::*;

        // Use parallel sort for large collections
        if self.items.len() > 1024 {
            self.items.par_sort();
        } else {
            self.items.sort();
        }

        let mut unique: Vec<A> = Vec::with_capacity(self.items.len());
        for item in self.items {
            if let Some(last) = unique.last()
                && Ordering::Equal == last.cmp(&item)
            {
                continue;
            }
            unique.push(item);
        }

        ordset_from_sorted_unique(unique)
    }
}

impl<A> Clone for OrdSet<A> {
    fn clone(&self) -> Self {
        OrdSet {
            root: self.root.clone(),
            len: self.len,
        }
    }
}

impl<A: fmt::Debug> fmt::Debug for OrdSet<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<A> Default for OrdSet<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: PartialEq> PartialEq for OrdSet<A> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<A: Eq> Eq for OrdSet<A> {}

impl<A> OrdSet<A> {
    /// Create an empty set.
    #[inline]
    pub fn new() -> Self {
        OrdSet { root: None, len: 0 }
    }

    /// Check if the set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a builder ([`OrdSetStructor`]) for incrementally constructing an `OrdSet`.
    #[inline]
    pub fn structor() -> OrdSetStructor<A> {
        OrdSetStructor::new()
    }
}

impl<A: Ord + Clone> OrdSet<A> {
    /// Insert an element.
    ///
    /// # Complexity
    /// O(log n)
    #[inline]
    pub fn insert(&self, value: A) -> Self {
        let (new_root, inserted) = insert_node(self.root.clone(), value);
        OrdSet {
            root: Some(new_root),
            len: if inserted { self.len + 1 } else { self.len },
        }
    }

    /// Remove an element.
    ///
    /// The probe may be any borrowed form of the element type (e.g. `&str` for
    /// `String`), avoiding an owned-value allocation for the probe.
    ///
    /// # Complexity
    /// O(log n)
    #[inline]
    pub fn remove<Q>(&self, value: &Q) -> Self
    where
        A: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (new_root, removed) = remove_node(self.root.clone(), value);
        OrdSet {
            root: new_root,
            len: if removed {
                self.len.saturating_sub(1)
            } else {
                self.len
            },
        }
    }

    /// Get minimum element.
    #[inline]
    pub fn min(&self) -> Option<&A> {
        min_node(&self.root)
    }

    /// Get maximum element.
    #[inline]
    pub fn max(&self) -> Option<&A> {
        max_node(&self.root)
    }

    /// Union of two sets (parallel version, requires `rayon` feature).
    ///
    /// Currently delegates to the sequential merge-join — parallelizing the
    /// in-order merge has not paid off in benchmarks. The signature keeps the
    /// `Send + Sync` bounds so a parallel implementation stays non-breaking.
    #[cfg(feature = "rayon")]
    pub fn union_par(&self, other: &Self) -> Self
    where
        A: Send + Sync,
    {
        self.union(other)
    }

    /// Union of two sets.
    ///
    /// This implementation merges the two in-order iterators and then bulk-builds
    /// a balanced tree, avoiding repeated `insert` calls.
    pub fn union(&self, other: &Self) -> Self {
        let mut a = self.iter();
        let mut b = other.iter();

        let mut next_a = a.next();
        let mut next_b = b.next();

        let mut out = Vec::with_capacity(self.len.saturating_add(other.len));

        loop {
            match (next_a, next_b) {
                (None, None) => break,
                (Some(x), None) => {
                    out.push(x.clone());
                    out.extend(a.cloned());
                    break;
                }
                (None, Some(y)) => {
                    out.push(y.clone());
                    out.extend(b.cloned());
                    break;
                }
                (Some(x), Some(y)) => match x.cmp(y) {
                    Ordering::Less => {
                        out.push(x.clone());
                        next_a = a.next();
                        next_b = Some(y);
                    }
                    Ordering::Equal => {
                        out.push(x.clone());
                        next_a = a.next();
                        next_b = b.next();
                    }
                    Ordering::Greater => {
                        out.push(y.clone());
                        next_a = Some(x);
                        next_b = b.next();
                    }
                },
            }
        }

        ordset_from_sorted_unique(out)
    }

    /// Intersection of two sets (parallel version, requires `rayon` feature).
    ///
    /// Currently delegates to the sequential merge-join — parallelizing the
    /// in-order merge has not paid off in benchmarks. The signature keeps the
    /// `Send + Sync` bounds so a parallel implementation stays non-breaking.
    #[cfg(feature = "rayon")]
    pub fn intersection_par(&self, other: &Self) -> Self
    where
        A: Send + Sync,
    {
        self.intersection(other)
    }

    /// Intersection of two sets.
    ///
    /// Merges the two in-order iterators (like a merge-join) and bulk-builds.
    pub fn intersection(&self, other: &Self) -> Self {
        let mut a = self.iter();
        let mut b = other.iter();

        let mut next_a = a.next();
        let mut next_b = b.next();

        let mut out = Vec::with_capacity(core::cmp::min(self.len, other.len));

        while let (Some(x), Some(y)) = (next_a, next_b) {
            match x.cmp(y) {
                Ordering::Less => next_a = a.next(),
                Ordering::Equal => {
                    out.push(x.clone());
                    next_a = a.next();
                    next_b = b.next();
                }
                Ordering::Greater => next_b = b.next(),
            }
        }

        ordset_from_sorted_unique(out)
    }

    /// Difference of two sets (parallel version, requires `rayon` feature).
    ///
    /// Currently delegates to the sequential merge-join — parallelizing the
    /// in-order merge has not paid off in benchmarks. The signature keeps the
    /// `Send + Sync` bounds so a parallel implementation stays non-breaking.
    #[cfg(feature = "rayon")]
    pub fn difference_par(&self, other: &Self) -> Self
    where
        A: Send + Sync,
    {
        self.difference(other)
    }

    /// Difference of two sets (self - other).
    ///
    /// Merges the two in-order iterators and bulk-builds.
    pub fn difference(&self, other: &Self) -> Self {
        let mut a = self.iter();
        let mut b = other.iter();

        let mut next_a = a.next();
        let mut next_b = b.next();

        let mut out = Vec::with_capacity(self.len);

        while let Some(x) = next_a {
            match next_b {
                None => {
                    out.push(x.clone());
                    out.extend(a.cloned());
                    break;
                }
                Some(y) => match x.cmp(y) {
                    Ordering::Less => {
                        out.push(x.clone());
                        next_a = a.next();
                        next_b = Some(y);
                    }
                    Ordering::Equal => {
                        next_a = a.next();
                        next_b = b.next();
                    }
                    Ordering::Greater => {
                        next_a = Some(x);
                        next_b = b.next();
                    }
                },
            }
        }

        ordset_from_sorted_unique(out)
    }
}

impl<A: Ord> OrdSet<A> {
    /// Check if an element exists.
    ///
    /// # Complexity
    /// O(log n)
    ///
    /// The probe may be any borrowed form of the element type, so an
    /// `OrdSet<String>` can be queried with a `&str` without allocating.
    #[inline]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        A: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        contains_node(&self.root, value)
    }
}

impl<A> OrdSet<A> {
    /// Iterate over elements in order.
    #[inline]
    pub fn iter(&self) -> OrdSetIter<'_, A> {
        // Pre-size the stack to the AVL tree height (≤ 1.44 log2 n) to
        // avoid reallocation during in-order traversal.
        let stack_cap = if self.len > 1 {
            usize::BITS as usize - self.len.leading_zeros() as usize
        } else {
            0
        };
        OrdSetIter {
            stack: Vec::with_capacity(stack_cap),
            current: self.root.as_ref().map(Arc::as_ref),
        }
    }
}

// ============================================================================
// AVL Tree Operations
// ============================================================================

fn contains_node<A, Q>(node: &Option<Arc<Node<A>>>, value: &Q) -> bool
where
    A: Borrow<Q>,
    Q: Ord + ?Sized,
{
    match node {
        None => false,
        Some(n) => match value.cmp(n.value.borrow()) {
            Ordering::Less => contains_node(&n.left, value),
            Ordering::Equal => true,
            Ordering::Greater => contains_node(&n.right, value),
        },
    }
}

fn insert_node<A: Ord + Clone>(node: Option<Arc<Node<A>>>, value: A) -> (Arc<Node<A>>, bool) {
    match node {
        None => (Arc::new(Node::new(value)), true),
        Some(n) => {
            let (new_node, inserted) = match value.cmp(&n.value) {
                Ordering::Less => {
                    let (new_left, inserted) = insert_node(n.left.clone(), value);
                    let mut new = Node {
                        value: n.value.clone(),
                        left: Some(new_left),
                        right: n.right.clone(),
                        height: n.height,
                    };
                    update_height(&mut new);
                    (new, inserted)
                }
                Ordering::Equal => {
                    return (n, false);
                }
                Ordering::Greater => {
                    let (new_right, inserted) = insert_node(n.right.clone(), value);
                    let mut new = Node {
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

fn remove_node<A, Q>(node: Option<Arc<Node<A>>>, value: &Q) -> (Option<Arc<Node<A>>>, bool)
where
    A: Clone + Borrow<Q>,
    Q: Ord + ?Sized,
{
    match node {
        None => (None, false),
        Some(n) => match value.cmp(n.value.borrow()) {
            Ordering::Less => {
                let (new_left, removed) = remove_node(n.left.clone(), value);
                let mut new = Node {
                    value: n.value.clone(),
                    left: new_left,
                    right: n.right.clone(),
                    height: n.height,
                };
                update_height(&mut new);
                (Some(balance(new)), removed)
            }
            Ordering::Equal => match (&n.left, &n.right) {
                (None, None) => (None, true),
                (Some(l), None) => (Some(l.clone()), true),
                (None, Some(r)) => (Some(r.clone()), true),
                (Some(_), Some(r)) => {
                    let succ = min_node(&Some(r.clone())).unwrap().clone();
                    // `succ: A`, `A: Borrow<Q>` → `succ.borrow(): &Q` keeps the
                    // descent type uniform (Borrow guarantees Ord consistency).
                    let (new_right, _) = remove_node(n.right.clone(), succ.borrow());
                    let mut new = Node {
                        value: succ,
                        left: n.left.clone(),
                        right: new_right,
                        height: n.height,
                    };
                    update_height(&mut new);
                    (Some(balance(new)), true)
                }
            },
            Ordering::Greater => {
                let (new_right, removed) = remove_node(n.right.clone(), value);
                let mut new = Node {
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

fn min_node<A>(node: &Option<Arc<Node<A>>>) -> Option<&A> {
    fn go<A>(n: &Node<A>) -> &A {
        match &n.left {
            Some(left) => go(left),
            None => &n.value,
        }
    }
    node.as_ref().map(|n| go(n))
}

fn max_node<A>(node: &Option<Arc<Node<A>>>) -> Option<&A> {
    fn go<A>(n: &Node<A>) -> &A {
        match &n.right {
            Some(right) => go(right),
            None => &n.value,
        }
    }
    node.as_ref().map(|n| go(n))
}

fn balance<A: Clone>(mut node: Node<A>) -> Arc<Node<A>> {
    let bf = balance_factor(&node);

    if bf > 1 {
        if let Some(right) = node.right.take() {
            let right_node = Arc::try_unwrap(right).unwrap_or_else(|arc| (*arc).clone());
            if balance_factor(&right_node) < 0 {
                node.right = Some(rotate_right(right_node));
            } else {
                node.right = Some(Arc::new(right_node));
            }
        }
        return rotate_left(node);
    }

    if bf < -1 {
        if let Some(left) = node.left.take() {
            let left_node = Arc::try_unwrap(left).unwrap_or_else(|arc| (*arc).clone());
            if balance_factor(&left_node) > 0 {
                node.left = Some(rotate_left(left_node));
            } else {
                node.left = Some(Arc::new(left_node));
            }
        }
        return rotate_right(node);
    }

    Arc::new(node)
}

fn rotate_left<A: Clone>(mut node: Node<A>) -> Arc<Node<A>> {
    let right = node.right.take().expect("rotate_left: no right child");
    let right_left = right.left.clone();
    let right_right = right.right.clone();
    let left_height = height(&node.left);
    let right_left_height = height(&right_left);

    let new_left = Node {
        value: node.value,
        left: node.left,
        right: right_left,
        height: 1 + core::cmp::max(left_height, right_left_height),
    };

    let mut new_root = Node {
        value: right.value.clone(),
        left: Some(Arc::new(new_left)),
        right: right_right,
        height: 0,
    };
    update_height(&mut new_root);

    Arc::new(new_root)
}

fn rotate_right<A: Clone>(mut node: Node<A>) -> Arc<Node<A>> {
    let left = node.left.take().expect("rotate_right: no left child");
    let left_left = left.left.clone();
    let left_right = left.right.clone();
    let right_height = height(&node.right);
    let left_right_height = height(&left_right);

    let new_right = Node {
        value: node.value,
        left: left_right,
        right: node.right,
        height: 1 + core::cmp::max(left_right_height, right_height),
    };

    let mut new_root = Node {
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

/// Iterator over set elements.
pub struct OrdSetIter<'a, A> {
    stack: Vec<&'a Node<A>>,
    current: Option<&'a Node<A>>,
}

impl<'a, A> Iterator for OrdSetIter<'a, A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(node) = self.current {
                self.stack.push(node);
                self.current = node.left.as_ref().map(Arc::as_ref);
            } else {
                let node = self.stack.pop()?;
                self.current = node.right.as_ref().map(Arc::as_ref);
                return Some(&node.value);
            }
        }
    }
}

// ============================================================================
// FromIterator
// ============================================================================

impl<A: Ord> FromIterator<A> for OrdSet<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let mut s = OrdSet::structor();
        s.extend(iter);
        s.finish()
    }
}

impl<'a, A> IntoIterator for &'a OrdSet<A> {
    type Item = &'a A;
    type IntoIter = OrdSetIter<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ============================================================================
// Serde
// ============================================================================
//
// S1 audit fix: see ord_map.rs's serde section for the full rationale. As
// there, elements serialize as an in-order sequence and rebuild through
// `insert`, so a forged tree shape is unrepresentable.

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<A: serde::Serialize> serde::Serialize for OrdSet<A> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(self.iter())
    }
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<'de, A> serde::Deserialize<'de> for OrdSet<A>
where
    // Mirrors `insert`'s real bounds (`impl<A: Ord + Clone> OrdSet<A>`).
    A: serde::Deserialize<'de> + Ord + Clone,
{
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let items = Vec::<A>::deserialize(d)?;
        Ok(items.into_iter().fold(OrdSet::new(), |s, v| s.insert(v)))
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
        let set = OrdSet::new().insert(3).insert(1).insert(2);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(!set.contains(&4));
    }

    #[test]
    fn test_duplicates() {
        let set = OrdSet::new().insert(1).insert(1).insert(1);

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_remove() {
        let set = OrdSet::new().insert(1).insert(2).insert(3);

        let set = set.remove(&2);
        assert_eq!(set.len(), 2);
        assert!(!set.contains(&2));
    }

    #[test]
    fn test_iteration() {
        let set = OrdSet::new().insert(3).insert(1).insert(2);

        let values: Vec<_> = set.iter().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_structor_builds() {
        let mut s = OrdSet::structor();
        s.push(3);
        s.push(1);
        s.push(2);
        s.push(2);

        let set = s.finish();

        let values: Vec<_> = set.iter().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_set_operations() {
        let a = OrdSet::from_iter([1, 2, 3]);
        let b = OrdSet::from_iter([2, 3, 4]);

        let union: Vec<_> = a.union(&b).iter().copied().collect();
        assert_eq!(union, vec![1, 2, 3, 4]);

        let intersection: Vec<_> = a.intersection(&b).iter().copied().collect();
        assert_eq!(intersection, vec![2, 3]);

        let diff: Vec<_> = a.difference(&b).iter().copied().collect();
        assert_eq!(diff, vec![1]);
    }

    #[test]
    fn test_min_max() {
        let set = OrdSet::from_iter([3, 1, 4, 1, 5, 9, 2, 6]);

        assert_eq!(set.min(), Some(&1));
        assert_eq!(set.max(), Some(&9));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn serde_roundtrip_set() {
        let s = (0..1000).fold(OrdSet::new(), |s, i| s.insert(i));
        let json = serde_json::to_string(&s).unwrap();
        let back: OrdSet<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    /// S1 regression: attacker-shaped input can no longer dictate tree
    /// structure (depth/height/ordering) — only elements.
    #[test]
    fn serde_input_cannot_forge_structure() {
        let back: OrdSet<i32> = serde_json::from_str("[3,1,2]").unwrap();
        assert!(back.contains(&1));
        assert!(back.contains(&2));
        assert!(back.contains(&3));
    }

    /// S1 regression: a large input must deserialize without overflowing the
    /// stack (rebuilt via `insert`, one AVL rotation at a time).
    #[test]
    fn serde_deep_input_no_overflow() {
        let json = serde_json::to_string(&(0..100_000u32).collect::<Vec<_>>()).unwrap();
        let s: OrdSet<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(s.len(), 100_000);
    }
}
