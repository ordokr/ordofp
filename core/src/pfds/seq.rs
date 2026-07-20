//! # Persistent Sequence
//!
//! A persistent sequence backed by a size-annotated, self-balancing binary
//! tree (an AVL tree keyed by position rather than by an ordered key),
//! supporting random access and operations at both ends.
//!
//! # Balancing
//!
//! The tree is height-balanced (AVL) after every insertion and removal, so
//! its depth is at most 1.44·log₂(n). `get`, `push_front`, `push_back`,
//! `pop_front`, `pop_back`, `insert_at`, and `update` are therefore O(log n)
//! worst-case regardless of insertion order — building via repeated
//! `push_back` no longer degrades to a spine. `Seq` is also built with an
//! iterative `Drop` and a balanced `FromIterator`, so neither collecting nor
//! dropping a large sequence can overflow the stack.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::iter::FromIterator;

// ============================================================================
// Seq
// ============================================================================

/// A persistent sequence with random access.
///
/// # Complexity
/// The tree self-balances (AVL) after every insertion and removal, so
/// `get`, `push_front`, `push_back`, `pop_front`, `pop_back`, and `update`
/// are O(log n) worst-case (see the module-level balancing note). `concat`
/// and `split_at` use a height-aware AVL join costing `O(|h_l` − `h_r`|) ≤
/// O(log n), and their results are themselves fully balanced AVL trees.
///
/// # Example
/// ```
/// use ordofp_core::pfds::Seq;
///
/// let seq = Seq::new()
///     .push_back(1)
///     .push_back(2)
///     .push_back(3);
///
/// assert_eq!(seq.len(), 3);
/// assert_eq!(seq.get(1), Some(&2));
/// ```
pub struct Seq<A> {
    root: Option<Arc<Node<A>>>,
}

struct Node<A> {
    value: A,
    size: usize,
    height: u8,
    left: Option<Arc<Node<A>>>,
    right: Option<Arc<Node<A>>>,
}

impl<A: Clone> Clone for Node<A> {
    fn clone(&self) -> Self {
        Node {
            value: self.value.clone(),
            size: self.size,
            height: self.height,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl<A> Node<A> {
    fn new(value: A) -> Self {
        Node {
            value,
            size: 1,
            height: 1,
            left: None,
            right: None,
        }
    }

    /// Build a branch node from a value and (already-built) children,
    /// deriving both the subtree `size` and the AVL `height` from the
    /// children. Every rebuild site funnels through here so both invariants
    /// stay consistent.
    fn with_children(value: A, left: Option<Arc<Node<A>>>, right: Option<Arc<Node<A>>>) -> Self {
        let size = 1 + node_size(&left) + node_size(&right);
        let height = 1 + core::cmp::max(node_height(&left), node_height(&right));
        Node {
            value,
            size,
            height,
            left,
            right,
        }
    }
}

fn node_size<A>(node: &Option<Arc<Node<A>>>) -> usize {
    node.as_ref().map_or(0, |n| n.size)
}

fn node_height<A>(node: &Option<Arc<Node<A>>>) -> u8 {
    node.as_ref().map_or(0, |n| n.height)
}

/// AVL balance factor: positive means right-heavy, negative means left-heavy.
///
/// Computed in `i16` so the subtraction cannot overflow even if a `u8` height
/// were somehow corrupted — a defensive measure so a malformed tree degrades
/// gracefully instead of aborting the process.
fn balance_factor<A>(node: &Node<A>) -> i16 {
    i16::from(node_height(&node.right)) - i16::from(node_height(&node.left))
}

impl<A> Clone for Seq<A> {
    fn clone(&self) -> Self {
        Seq {
            root: self.root.clone(),
        }
    }
}

impl<A> Drop for Seq<A> {
    fn drop(&mut self) {
        // Tear the tree down iteratively. The compiler-generated recursive
        // `Drop` would recurse to a depth equal to the tree height; even
        // though the tree is now AVL-balanced (height ≤ 1.44·log₂ n), an
        // explicit work-list keeps teardown provably stack-safe and matches
        // the iterative Drop used elsewhere in `pfds`.
        let mut work: Vec<Arc<Node<A>>> = Vec::new();
        work.extend(self.root.take());
        while let Some(node) = work.pop() {
            // Only uniquely-owned nodes are dismantled here; a node still
            // shared with another `Seq` is left for its last owner's Drop to
            // unlink, so structural sharing is respected.
            if let Ok(mut n) = Arc::try_unwrap(node) {
                work.extend(n.left.take());
                work.extend(n.right.take());
            }
        }
    }
}

impl<A: fmt::Debug> fmt::Debug for Seq<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<A: PartialEq> PartialEq for Seq<A> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<A: Eq> Eq for Seq<A> {}

impl<A> Default for Seq<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Seq<A> {
    /// Create an empty sequence.
    #[inline]
    pub fn new() -> Self {
        Seq { root: None }
    }

    /// Check if the sequence is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Get the number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        node_size(&self.root)
    }

    /// Get element at index.
    ///
    /// # Complexity
    /// O(log n) worst-case (self-balancing AVL; see module docs)
    #[inline]
    pub fn get(&self, index: usize) -> Option<&A> {
        get_at(&self.root, index)
    }

    /// Get first element.
    #[inline]
    pub fn first(&self) -> Option<&A> {
        self.get(0)
    }

    /// Get last element.
    #[inline]
    pub fn last(&self) -> Option<&A> {
        let len = self.len();
        if len == 0 { None } else { self.get(len - 1) }
    }
}

impl<A: Clone> Seq<A> {
    /// Push element to front.
    ///
    /// # Complexity
    /// O(log n) worst-case (self-balancing AVL; see module docs)
    #[inline]
    pub fn push_front(mut self, value: A) -> Self {
        // `take` moves the root out (leaving `self` empty) so the by-value
        // `self` can drop cheaply — a partial move is impossible now that
        // `Seq` implements `Drop`.
        Seq {
            root: Some(insert_at(self.root.take(), 0, value)),
        }
    }

    /// Push element to back.
    ///
    /// # Complexity
    /// O(log n) worst-case (self-balancing AVL; see module docs)
    #[inline]
    pub fn push_back(mut self, value: A) -> Self {
        let len = self.len();
        Seq {
            root: Some(insert_at(self.root.take(), len, value)),
        }
    }

    /// Pop from front.
    ///
    /// # Complexity
    /// O(log n) worst-case (self-balancing AVL; see module docs)
    #[inline]
    pub fn pop_front(mut self) -> Option<(A, Self)> {
        if self.is_empty() {
            return None;
        }
        let (value, new_root) = remove_at(self.root.take(), 0);
        Some((value, Seq { root: new_root }))
    }

    /// Pop from back.
    ///
    /// # Complexity
    /// O(log n) worst-case (self-balancing AVL; see module docs)
    #[inline]
    pub fn pop_back(mut self) -> Option<(Self, A)> {
        let len = self.len();
        if len == 0 {
            return None;
        }
        let (value, new_root) = remove_at(self.root.take(), len - 1);
        Some((Seq { root: new_root }, value))
    }

    /// Update element at index.
    ///
    /// # Complexity
    /// O(log n) worst-case (self-balancing AVL; see module docs)
    #[inline]
    pub fn update(&self, index: usize, value: A) -> Option<Self> {
        if index >= self.len() {
            return None;
        }
        Some(Seq {
            root: Some(update_at(&self.root, index, value)?),
        })
    }

    /// Split at index.
    ///
    /// # Complexity
    /// O(log n) — height-aware join; the result is a fully balanced AVL tree.
    #[inline]
    pub fn split_at(&self, index: usize) -> (Self, Self) {
        if index == 0 {
            return (Seq::new(), self.clone());
        }
        if index >= self.len() {
            return (self.clone(), Seq::new());
        }
        let (left, right) = split_tree(&self.root, index);
        (Seq { root: left }, Seq { root: right })
    }

    /// Concatenate two sequences.
    ///
    /// # Complexity
    /// O(log n) — height-aware join; the result is a fully balanced AVL tree.
    #[inline]
    pub fn concat(&self, other: &Self) -> Self {
        Seq {
            root: join_trees(&self.root, &other.root),
        }
    }

    /// Map a function over all elements.
    #[inline]
    pub fn map<B: Clone, F: Fn(&A) -> B>(&self, f: F) -> Seq<B> {
        self.iter().map(f).collect()
    }

    /// Filter elements.
    #[inline]
    pub fn filter<F: Fn(&A) -> bool>(&self, f: F) -> Self {
        self.iter().filter(|x| f(*x)).cloned().collect()
    }

    /// Reverse the sequence.
    #[inline]
    pub fn reverse(&self) -> Self {
        self.iter().rev().cloned().collect()
    }
}

impl<A> Seq<A> {
    /// Iterate over elements.
    #[inline]
    pub fn iter(&self) -> SeqIter<'_, A> {
        let len = self.len();
        // Pre-allocate log2(n)+1 slots for the traversal stack to avoid
        // reallocation during in-order traversal.
        let stack_cap = if len > 1 {
            usize::BITS as usize - len.leading_zeros() as usize
        } else {
            0
        };
        SeqIter {
            stack: Vec::with_capacity(stack_cap),
            current: self.root.as_ref().map(Arc::as_ref),
            back_stack: Vec::with_capacity(stack_cap),
            back_current: self.root.as_ref().map(Arc::as_ref),
            remaining: len,
        }
    }
}

// ============================================================================
// Tree Operations
// ============================================================================

fn get_at<A>(node: &Option<Arc<Node<A>>>, index: usize) -> Option<&A> {
    let n = node.as_ref()?;
    let left_size = node_size(&n.left);

    match index.cmp(&left_size) {
        core::cmp::Ordering::Less => get_at(&n.left, index),
        core::cmp::Ordering::Equal => Some(&n.value),
        core::cmp::Ordering::Greater => get_at(&n.right, index - left_size - 1),
    }
}

fn insert_at<A: Clone>(node: Option<Arc<Node<A>>>, index: usize, value: A) -> Arc<Node<A>> {
    match node {
        None => Arc::new(Node::new(value)),
        Some(n) => {
            let left_size = node_size(&n.left);

            let rebuilt = if index <= left_size {
                let new_left = insert_at(n.left.clone(), index, value);
                Node::with_children(n.value.clone(), Some(new_left), n.right.clone())
            } else {
                let new_right = insert_at(n.right.clone(), index - left_size - 1, value);
                Node::with_children(n.value.clone(), n.left.clone(), Some(new_right))
            };
            balance(rebuilt)
        }
    }
}

fn remove_at<A: Clone>(node: Option<Arc<Node<A>>>, index: usize) -> (A, Option<Arc<Node<A>>>) {
    let n = node.expect("remove_at called on empty node");
    let left_size = node_size(&n.left);

    match index.cmp(&left_size) {
        core::cmp::Ordering::Less => {
            let (value, new_left) = remove_at(n.left.clone(), index);
            let rebuilt = Node::with_children(n.value.clone(), new_left, n.right.clone());
            (value, Some(balance(rebuilt)))
        }
        core::cmp::Ordering::Equal => {
            let value = n.value.clone();
            // Splice this node out. With two children we replace it with its
            // in-order successor (the element at position 0 of the right
            // subtree) and remove that successor recursively — mirroring the
            // AVL removal in `ord_map.rs`, so the result stays balanced.
            let merged = match (&n.left, &n.right) {
                (None, None) => None,
                (Some(l), None) => Some(l.clone()),
                (None, Some(r)) => Some(r.clone()),
                (Some(_), Some(_)) => {
                    let (successor, new_right) = remove_at(n.right.clone(), 0);
                    let rebuilt = Node::with_children(successor, n.left.clone(), new_right);
                    Some(balance(rebuilt))
                }
            };
            (value, merged)
        }
        core::cmp::Ordering::Greater => {
            let (value, new_right) = remove_at(n.right.clone(), index - left_size - 1);
            let rebuilt = Node::with_children(n.value.clone(), n.left.clone(), new_right);
            (value, Some(balance(rebuilt)))
        }
    }
}

fn update_at<A: Clone>(
    node: &Option<Arc<Node<A>>>,
    index: usize,
    value: A,
) -> Option<Arc<Node<A>>> {
    let n = node.as_ref()?;
    let left_size = node_size(&n.left);

    let new_node = match index.cmp(&left_size) {
        core::cmp::Ordering::Less => {
            let new_left = update_at(&n.left, index, value);
            Node::with_children(n.value.clone(), new_left, n.right.clone())
        }
        core::cmp::Ordering::Equal => Node::with_children(value, n.left.clone(), n.right.clone()),
        core::cmp::Ordering::Greater => {
            let new_right = update_at(&n.right, index - left_size - 1, value);
            Node::with_children(n.value.clone(), n.left.clone(), new_right)
        }
    };

    Some(Arc::new(new_node))
}

/// A (possibly empty) shared subtree — the link type used throughout this
/// persistent balanced tree.
type Tree<A> = Option<Arc<Node<A>>>;

fn split_tree<A: Clone>(node: &Tree<A>, index: usize) -> (Tree<A>, Tree<A>) {
    match node {
        None => (None, None),
        Some(n) => {
            let left_size = node_size(&n.left);

            if index <= left_size {
                // This node's value (at position `left_size`) and its whole
                // right subtree belong to the right half. Split the left
                // subtree, then three-way-join the right remainder back onto
                // the pivot — both operands are already valid AVL trees, so
                // the join produces a fully balanced result.
                let (ll, lr) = split_tree(&n.left, index);
                let right = Some(join_nodes(lr, n.value.clone(), n.right.clone()));
                (ll, right)
            } else {
                // This node's value belongs to the left half.
                let (rl, rr) = split_tree(&n.right, index - left_size - 1);
                let left = Some(join_nodes(n.left.clone(), n.value.clone(), rl));
                (left, rr)
            }
        }
    }
}

/// Concatenate two (already-balanced) AVL trees.
///
/// When both operands are present, the left tree's maximum is lifted out as
/// the join pivot (leaving a still-balanced remainder) and a height-aware
/// three-way join splices the two trees together, so the result is a fully
/// balanced AVL tree in `O(|h_left` − `h_right`|).
fn join_trees<A: Clone>(
    left: &Option<Arc<Node<A>>>,
    right: &Option<Arc<Node<A>>>,
) -> Option<Arc<Node<A>>> {
    match (left, right) {
        (None, None) => None,
        (Some(l), None) => Some(l.clone()),
        (None, Some(r)) => Some(r.clone()),
        (Some(l), Some(r)) => {
            let (new_left, pivot) = extract_max(l);
            Some(join_nodes(new_left, pivot, Some(r.clone())))
        }
    }
}

/// Height-aware AVL join of two balanced subtrees around a `pivot` value that
/// sorts (positionally) between them.
///
/// If the two trees differ in height by at most 1, a single branch node
/// suffices. Otherwise we descend the *taller* tree's inner spine toward the
/// shorter tree until the attachment heights match to within 1, splice a join
/// node there, and rebalance on the way back up — exactly one rotation per
/// level may be needed, just like an AVL insertion. The unwound spine has
/// length `O(|h_left` − `h_right`|), so the whole join costs `O(|h_left` − `h_right`|)
/// and the result satisfies |balance factor| ≤ 1 at every node.
fn join_nodes<A: Clone>(left: Tree<A>, pivot: A, right: Tree<A>) -> Arc<Node<A>> {
    let hl = node_height(&left);
    let hr = node_height(&right);

    if hl > hr + 1 {
        // Left is the taller tree: descend its right spine.
        let l = left.expect("hl > hr + 1 implies the left subtree is present");
        let new_right = join_nodes(l.right.clone(), pivot, right);
        let rebuilt = Node::with_children(l.value.clone(), l.left.clone(), Some(new_right));
        balance(rebuilt)
    } else if hr > hl + 1 {
        // Right is the taller tree: descend its left spine.
        let r = right.expect("hr > hl + 1 implies the right subtree is present");
        let new_left = join_nodes(left, pivot, r.left.clone());
        let rebuilt = Node::with_children(r.value.clone(), Some(new_left), r.right.clone());
        balance(rebuilt)
    } else {
        // Heights already differ by at most 1: a single join node is balanced.
        Arc::new(Node::with_children(pivot, left, right))
    }
}

fn extract_max<A: Clone>(node: &Arc<Node<A>>) -> (Option<Arc<Node<A>>>, A) {
    match &node.right {
        None => (node.left.clone(), node.value.clone()),
        Some(right) => {
            let (new_right, max) = extract_max(right);
            let rebuilt = Node::with_children(node.value.clone(), node.left.clone(), new_right);
            (Some(balance(rebuilt)), max)
        }
    }
}

// ============================================================================
// AVL rebalancing
//
// Ported from `core/src/pfds/ord_map.rs`. The only
// adaptation is the descent key: `Seq` descends by index vs. left-subtree
// size instead of by `Ord` comparison, so the rotations here also recompute
// each rebuilt node's `size` (via `Node::with_children`) in addition to its
// AVL `height`.
// ============================================================================

/// Rebalance a single node whose children are already valid AVL subtrees,
/// performing at most one single or double rotation. Preserves in-order
/// position, so it is safe on the index-keyed `Seq` tree.
fn balance<A: Clone>(mut node: Node<A>) -> Arc<Node<A>> {
    let bf = balance_factor(&node);

    if bf > 1 {
        // Right heavy.
        if let Some(right) = node.right.take() {
            let right_node = Arc::try_unwrap(right).unwrap_or_else(|arc| (*arc).clone());
            if balance_factor(&right_node) < 0 {
                // Right-Left case.
                node.right = Some(rotate_right(right_node));
            } else {
                node.right = Some(Arc::new(right_node));
            }
        }
        return rotate_left(node);
    }

    if bf < -1 {
        // Left heavy.
        if let Some(left) = node.left.take() {
            let left_node = Arc::try_unwrap(left).unwrap_or_else(|arc| (*arc).clone());
            if balance_factor(&left_node) > 0 {
                // Left-Right case.
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
    // `with_children` recomputes both `size` and `height` for each new node.
    let new_left = Node::with_children(node.value, node.left, right.left.clone());
    Arc::new(Node::with_children(
        right.value.clone(),
        Some(Arc::new(new_left)),
        right.right.clone(),
    ))
}

fn rotate_right<A: Clone>(mut node: Node<A>) -> Arc<Node<A>> {
    let left = node.left.take().expect("rotate_right: no left child");
    let new_right = Node::with_children(node.value, left.right.clone(), node.right);
    Arc::new(Node::with_children(
        left.value.clone(),
        left.left.clone(),
        Some(Arc::new(new_right)),
    ))
}

// ============================================================================
// Iterator
// ============================================================================

/// Iterator over sequence elements.
///
/// The front cursor (`stack`/`current`) performs an in-order traversal; the
/// back cursor (`back_stack`/`back_current`) performs an independent reverse
/// in-order traversal. The shared `remaining` counter enforces the
/// `DoubleEndedIterator` meet-in-the-middle contract: the front yields the
/// first f elements and the back yields the last b elements, and both ends
/// stop once f + b reaches the sequence length, so mixed `next`/`next_back`
/// calls never duplicate or miss an element.
pub struct SeqIter<'a, A> {
    stack: Vec<&'a Node<A>>,
    current: Option<&'a Node<A>>,
    back_stack: Vec<&'a Node<A>>,
    back_current: Option<&'a Node<A>>,
    remaining: usize,
}

impl<'a, A> Iterator for SeqIter<'a, A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        loop {
            if let Some(node) = self.current {
                self.stack.push(node);
                self.current = node.left.as_ref().map(Arc::as_ref);
            } else {
                let node = self.stack.pop()?;
                self.current = node.right.as_ref().map(Arc::as_ref);
                self.remaining = self.remaining.saturating_sub(1);
                return Some(&node.value);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<A> ExactSizeIterator for SeqIter<'_, A> {}

impl<A> DoubleEndedIterator for SeqIter<'_, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        loop {
            if let Some(node) = self.back_current {
                self.back_stack.push(node);
                self.back_current = node.right.as_ref().map(Arc::as_ref);
            } else {
                let node = self.back_stack.pop()?;
                self.back_current = node.left.as_ref().map(Arc::as_ref);
                self.remaining = self.remaining.saturating_sub(1);
                return Some(&node.value);
            }
        }
    }
}

// ============================================================================
// FromIterator
// ============================================================================

impl<A> FromIterator<A> for Seq<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        // Build a perfectly balanced tree by recursive midpoint splitting:
        // depth is O(log n), so neither construction nor the later iterative
        // Drop can overflow the stack, and no per-element rebalancing is
        // needed (the shape is already AVL-valid).
        fn build<A, I: Iterator<Item = A>>(n: usize, it: &mut I) -> Option<Arc<Node<A>>> {
            if n == 0 {
                return None;
            }
            let left = build(n / 2, it);
            let elem = it
                .next()
                .expect("iterator shorter than its reported length");
            let right = build(n - n / 2 - 1, it);
            Some(Arc::new(Node::with_children(elem, left, right)))
        }

        let items: Vec<A> = iter.into_iter().collect();
        let n = items.len();
        let mut it = items.into_iter();
        Seq {
            root: build(n, &mut it),
        }
    }
}

impl<'a, A> IntoIterator for &'a Seq<A> {
    type Item = &'a A;
    type IntoIter = SeqIter<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ============================================================================
// Serde
// ============================================================================
//
// S1 audit fix: see ord_map.rs's serde section for the full rationale.
// Elements serialize as an in-order sequence and rebuild through Task 11's
// balanced `from_iter`, so a forged tree shape (size/height fields) is
// unrepresentable, and construction stays O(log n) deep.

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<A: serde::Serialize> serde::Serialize for Seq<A> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(self.iter())
    }
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<'de, A> serde::Deserialize<'de> for Seq<A>
where
    // `Seq`'s `FromIterator` (Task 11's balanced builder) needs no `Clone`
    // or `Ord` bound on `A` — it consumes the iterator directly.
    A: serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let items = Vec::<A>::deserialize(d)?;
        Ok(Seq::from_iter(items))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// Recursively verify the AVL invariants at every node and return the
    /// node's `(height, size)` so a parent can cross-check its own stored
    /// fields. Asserts, at each node:
    ///   1. |balance factor| ≤ 1 (the tree is height-balanced),
    ///   2. stored `height` == 1 + max(child heights),
    ///   3. stored `size`   == 1 + sum(child sizes).
    fn check_avl_invariant<A>(node: &Option<Arc<Node<A>>>) -> (u8, usize) {
        match node {
            None => (0, 0),
            Some(n) => {
                let (lh, ls) = check_avl_invariant(&n.left);
                let (rh, rs) = check_avl_invariant(&n.right);
                let bf = i16::from(rh) - i16::from(lh);
                assert!(bf.abs() <= 1, "AVL invariant broken: balance factor {bf}");
                let expected_height = 1 + core::cmp::max(lh, rh);
                assert_eq!(n.height, expected_height, "stored height out of sync");
                let expected_size = 1 + ls + rs;
                assert_eq!(n.size, expected_size, "stored size out of sync");
                (expected_height, expected_size)
            }
        }
    }

    #[test]
    fn test_basic_operations() {
        let seq = Seq::new().push_back(1).push_back(2).push_back(3);

        assert_eq!(seq.len(), 3);
        assert_eq!(seq.get(0), Some(&1));
        assert_eq!(seq.get(1), Some(&2));
        assert_eq!(seq.get(2), Some(&3));
        assert_eq!(seq.get(3), None);
    }

    #[test]
    fn test_push_front() {
        let seq = Seq::new().push_front(3).push_front(2).push_front(1);

        assert_eq!(seq.get(0), Some(&1));
        assert_eq!(seq.get(1), Some(&2));
        assert_eq!(seq.get(2), Some(&3));
    }

    #[test]
    fn test_pop() {
        let seq = Seq::from_iter([1, 2, 3]);

        let (val, seq) = seq
            .pop_front()
            .expect("seq has 3 elements; pop_front should return the first");
        assert_eq!(val, 1);
        assert_eq!(seq.len(), 2);

        let (seq, val) = seq
            .pop_back()
            .expect("seq has 2 elements; pop_back should return the last");
        assert_eq!(val, 3);
        assert_eq!(seq.len(), 1);
    }

    #[test]
    fn test_update() {
        let seq = Seq::from_iter([1, 2, 3]);
        let seq = seq
            .update(1, 20)
            .expect("index 1 is valid for a 3-element seq; update should succeed");

        assert_eq!(seq.get(1), Some(&20));
    }

    #[test]
    fn test_split_concat() {
        let seq = Seq::from_iter([1, 2, 3, 4, 5]);
        let (left, right) = seq.split_at(2);

        assert_eq!(left.len(), 2);
        assert_eq!(right.len(), 3);

        let rejoined = left.concat(&right);
        assert_eq!(rejoined.len(), 5);
    }

    #[test]
    fn test_iteration() {
        let seq = Seq::from_iter([1, 2, 3, 4, 5]);
        let collected: Vec<_> = seq.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_reverse_matches_manual() {
        let seq = Seq::from_iter([1, 2, 3, 4, 5]);
        let reversed: Vec<_> = seq.reverse().iter().copied().collect();
        assert_eq!(reversed, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_iter_rev_round_trip() {
        let items: Vec<i32> = (1..=33).collect();
        let seq = Seq::from_iter(items.iter().copied());
        let rev: Vec<_> = seq.iter().rev().copied().collect();
        let mut expected = items.clone();
        expected.reverse();
        assert_eq!(rev, expected);
    }

    #[test]
    fn test_interleaved_next_and_next_back() {
        let seq = Seq::from_iter([1, 2, 3, 4, 5, 6, 7]);
        let mut iter = seq.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next_back(), Some(&7));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next_back(), Some(&6));
        assert_eq!(iter.next_back(), Some(&5));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        // Front and back have met: both ends must stop, no duplicates.
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    /// H5 regression — reproduced pre-fix as a process abort at ~3k elements
    /// (right-spine `from_iter` plus recursive node `Drop`).
    #[test]
    fn collect_100k_no_overflow() {
        let s: Seq<u64> = (0..100_000).collect();
        assert_eq!(s.len(), 100_000);
        assert_eq!(s.get(99_999), Some(&99_999));
        drop(s);
    }

    #[test]
    fn push_back_loop_stays_logarithmic() {
        let mut s = Seq::new();
        for i in 0..100_000u64 {
            s = s.push_back(i);
        }
        assert_eq!(s.get(0), Some(&0));
        assert_eq!(s.get(99_999), Some(&99_999));
    }

    /// Front-insertion loop. `Seq` exposes no index-`insert_at`; inserting at
    /// index 0 is `push_front`, which drives the same left-descent + rebalance
    /// path the brief's `insert_at(0, _)` targeted.
    #[test]
    fn insert_at_front_loop_no_overflow() {
        let mut s = Seq::new();
        for i in 0..100_000u64 {
            s = s.push_front(i);
        }
        assert_eq!(s.len(), 100_000);
    }

    /// Model check vs `Vec`, exercising six mutators (`push_back`,
    /// `push_front`, `pop_front`, `pop_back`, `split_at`+`concat`, and
    /// `concat` with a fresh sequence) so the insert, removal, split, and join
    /// rebalancing paths are all covered on random operation sequences. The
    /// full AVL invariant and element-by-element equality against the `Vec`
    /// oracle are re-checked **after every op**, not just at the end.
    #[test]
    fn model_check_against_vec() {
        use quickcheck::quickcheck;
        fn prop(ops: Vec<(u8, u16)>) -> bool {
            let mut model: Vec<u16> = Vec::new();
            let mut seq: Seq<u16> = Seq::new();
            for (op, v) in ops {
                match op % 6 {
                    0 => {
                        model.push(v);
                        seq = seq.push_back(v);
                    }
                    1 => {
                        model.insert(0, v);
                        seq = seq.push_front(v);
                    }
                    2 => {
                        if !model.is_empty() {
                            model.remove(0);
                            seq = seq.pop_front().expect("non-empty seq must pop_front").1;
                        }
                    }
                    3 => {
                        if !model.is_empty() {
                            model.pop();
                            seq = seq.pop_back().expect("non-empty seq must pop_back").0;
                        }
                    }
                    4 => {
                        // Split at an in-range index, verify both halves are
                        // full AVL trees, then rejoin — a round-trip that must
                        // reproduce the original sequence.
                        let idx = (v as usize) % (model.len() + 1);
                        let (l, r) = seq.split_at(idx);
                        check_avl_invariant(&l.root);
                        check_avl_invariant(&r.root);
                        seq = l.concat(&r);
                    }
                    _ => {
                        // Concatenate a small fresh sequence on the right.
                        let extra: Vec<u16> = (0..(v % 5)).collect();
                        model.extend(extra.iter().copied());
                        seq = seq.concat(&extra.iter().copied().collect());
                    }
                }
                check_avl_invariant(&seq.root);
                if model.len() != seq.len()
                    || !model.iter().enumerate().all(|(i, x)| seq.get(i) == Some(x))
                {
                    return false;
                }
            }
            true
        }
        quickcheck(prop as fn(Vec<(u8, u16)>) -> bool);
    }

    /// 10k mixed push/pop operations, asserting the AVL invariant holds after
    /// every mutation.
    #[test]
    fn invariant_holds_under_mixed_push_pop() {
        let mut seq: Seq<u32> = Seq::new();
        let mut model: Vec<u32> = Vec::new();
        for i in 0..10_000u32 {
            match i % 5 {
                0 | 1 => {
                    seq = seq.push_back(i);
                    model.push(i);
                }
                2 => {
                    seq = seq.push_front(i);
                    model.insert(0, i);
                }
                3 => {
                    if !seq.is_empty() {
                        seq = seq.pop_front().expect("non-empty seq must pop_front").1;
                        model.remove(0);
                    }
                }
                _ => {
                    if !seq.is_empty() {
                        seq = seq.pop_back().expect("non-empty seq must pop_back").0;
                        model.pop();
                    }
                }
            }
            check_avl_invariant(&seq.root);
        }
        assert_eq!(seq.len(), model.len());
        assert!(model.iter().enumerate().all(|(i, x)| seq.get(i) == Some(x)));
    }

    /// `concat` of very-different-height operands must yield a fully balanced
    /// AVL tree, not a spine with unbounded skew.
    #[test]
    fn concat_unequal_heights_stays_balanced() {
        let big: Seq<u32> = (0..1000).collect();
        let small: Seq<u32> = (1000..1010).collect();

        let a = big.concat(&small);
        check_avl_invariant(&a.root);
        assert_eq!(a.len(), 1010);
        assert!((0..1010).all(|i| a.get(i as usize) == Some(&i)));

        let b = small.concat(&big);
        check_avl_invariant(&b.root);
        assert_eq!(b.len(), 1010);
    }

    /// Fold of 2000 single-element `concat`s — the exact spine-building case
    /// that overflowed the balance factor and panicked in the old join.
    #[test]
    fn concat_fold_of_singletons_stays_balanced() {
        let mut seq: Seq<u32> = Seq::new();
        for i in 0..2000u32 {
            let single: Seq<u32> = core::iter::once(i).collect();
            seq = seq.concat(&single);
        }
        check_avl_invariant(&seq.root);
        assert_eq!(seq.len(), 2000);
        assert!((0..2000).all(|i| seq.get(i as usize) == Some(&i)));
    }

    /// `split_at` at several indices of a 1000-element tree — both halves must
    /// be full AVL trees and rejoin to the original.
    #[test]
    fn split_at_various_indices_stays_balanced() {
        let seq: Seq<u32> = (0..1000).collect();
        for &idx in &[1usize, 2, 250, 499, 500, 501, 750, 998, 999] {
            let (left, right) = seq.split_at(idx);
            check_avl_invariant(&left.root);
            check_avl_invariant(&right.root);
            assert_eq!(left.len(), idx);
            assert_eq!(right.len(), 1000 - idx);
            assert!((0..idx).all(|i| left.get(i) == Some(&(i as u32))));
            assert!((0..1000 - idx).all(|i| right.get(i) == Some(&((idx + i) as u32))));
            let rejoined = left.concat(&right);
            check_avl_invariant(&rejoined.root);
            assert!((0..1000).all(|i| rejoined.get(i) == Some(&(i as u32))));
        }
    }

    #[test]
    fn test_empty_sequence_edge_cases() {
        let empty: Seq<i32> = Seq::new();

        // Empty sequence predicates
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        // Accessing elements on empty returns None
        assert_eq!(empty.first(), None);
        assert_eq!(empty.last(), None);
        assert_eq!(empty.get(0), None);

        // Popping from empty returns None
        assert!(empty.clone().pop_front().is_none());
        assert!(empty.clone().pop_back().is_none());

        // split_at(0) on empty: both halves are empty
        let (left, right) = empty.split_at(0);
        assert!(left.is_empty());
        assert!(right.is_empty());

        // split_at beyond length: (self, empty)
        let (left, right) = empty.split_at(10);
        assert!(left.is_empty());
        assert!(right.is_empty());

        // Concat of two empty sequences is empty
        assert!(empty.concat(&empty).is_empty());

        // Single-element sequence: first == last
        let one = Seq::new().push_back(42);
        assert_eq!(one.first(), Some(&42));
        assert_eq!(one.last(), Some(&42));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn serde_roundtrip_seq() {
        let s = (0..1000).fold(Seq::new(), super::Seq::push_back);
        let json = serde_json::to_string(&s).unwrap();
        let back: Seq<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    /// S1 regression: attacker-shaped input can no longer dictate tree
    /// structure (depth/height/size) — only elements, rebuilt via Task 11's
    /// balanced `from_iter`.
    #[test]
    fn serde_input_cannot_forge_structure() {
        let back: Seq<i32> = serde_json::from_str("[3,1,2]").unwrap();
        assert_eq!(back.get(0), Some(&3));
        assert_eq!(back.get(1), Some(&1));
        assert_eq!(back.get(2), Some(&2));
    }

    /// S1 regression: a large input must deserialize without overflowing the
    /// stack (rebuilt via the balanced `from_iter`, depth O(log n)).
    #[test]
    fn serde_deep_input_no_overflow() {
        let json = serde_json::to_string(&(0..100_000u32).collect::<Vec<_>>()).unwrap();
        let s: Seq<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(s.len(), 100_000);
        assert_eq!(s.get(99_999), Some(&99_999));
    }
}
