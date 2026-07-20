//! Persistent Stack implementation.
//!
//! A fully persistent LIFO (Last In, First Out) stack using a linked list
//! with reference counting for structural sharing.

use core::iter::FromIterator;

#[cfg(feature = "alloc")]
use alloc::{rc::Rc, vec::Vec};

/// Error type for Stack operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StackError {
    /// Index out of bounds.
    IndexOutOfBounds,
}

/// A persistent (immutable) stack.
///
/// All operations preserve the original stack, returning new versions.
/// Uses structural sharing via `Rc` for efficiency.
///
/// # Time Complexity
///
/// | Operation | Time |
/// |-----------|------|
/// | `push`    | O(1) |
/// | `pop`     | O(1) |
/// | `peek`    | O(1) |
/// | `len`     | O(n) |
/// | `get`     | O(n) |
///
/// # Example
///
/// ```rust
/// use ordofp_core::pfds::Stack;
///
/// let s1 = Stack::new().push(1).push(2).push(3);
/// assert_eq!(s1.peek(), Some(&3));
///
/// let (top, s2) = s1.clone().pop().unwrap();
/// assert_eq!(top, 3);
/// assert_eq!(s2.peek(), Some(&2));
///
/// // s1 is still valid and unchanged
/// assert_eq!(s1.peek(), Some(&3));
/// ```
#[cfg(feature = "alloc")]
pub struct Stack<A> {
    head: Link<A>,
}

#[cfg(feature = "alloc")]
type Link<A> = Option<Rc<Node<A>>>;

#[cfg(feature = "alloc")]
struct Node<A> {
    elem: A,
    next: Link<A>,
}

#[cfg(feature = "alloc")]
impl<A> Default for Stack<A> {
    #[inline]
    fn default() -> Self {
        Stack { head: None }
    }
}

// Manual: O(1), no `A: Clone` bound (derive would add one).
#[cfg(feature = "alloc")]
impl<A> Clone for Stack<A> {
    #[inline]
    fn clone(&self) -> Self {
        Stack {
            head: self.head.clone(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<A: core::fmt::Debug> core::fmt::Debug for Stack<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(feature = "alloc")]
impl<A> Drop for Stack<A> {
    fn drop(&mut self) {
        // Iteratively unlink uniquely-owned nodes. Default drop recursion
        // depth equals stack length — a ~30k-element stack aborts the process.
        let mut cur = self.head.take();
        while let Some(node) = cur {
            match Rc::try_unwrap(node) {
                Ok(mut n) => cur = n.next.take(),
                // Shared tail: another Stack still owns it; that owner's
                // Drop unlinks it iteratively in turn.
                Err(_) => break,
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<A: PartialEq> PartialEq for Stack<A> {
    fn eq(&self, other: &Self) -> bool {
        let (mut a, mut b) = (&self.head, &other.head);
        loop {
            match (a, b) {
                (None, None) => return true,
                (Some(x), Some(y)) => {
                    // Shared spine: equal by construction.
                    if Rc::ptr_eq(x, y) {
                        return true;
                    }
                    if x.elem != y.elem {
                        return false;
                    }
                    a = &x.next;
                    b = &y.next;
                }
                _ => return false,
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<A: Eq> Eq for Stack<A> {}

#[cfg(feature = "alloc")]
impl<A> Stack<A> {
    /// Create a new empty stack.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Stack;
    ///
    /// let s: Stack<i32> = Stack::new();
    /// assert!(s.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Stack { head: None }
    }

    /// Check if the stack is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Push a value onto the stack.
    ///
    /// Returns a new stack with the value at the top.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Stack;
    ///
    /// let s = Stack::new().push(1).push(2);
    /// assert_eq!(s.peek(), Some(&2));
    /// ```
    #[inline]
    pub fn push(mut self, value: A) -> Self {
        // `take`, never destructure: Stack has Drop (E0509).
        let next = self.head.take();
        Stack {
            head: Some(Rc::new(Node { elem: value, next })),
        }
    }

    /// Pop the top value from the stack.
    ///
    /// Returns `Some((value, new_stack))` if non-empty, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Stack;
    ///
    /// let s = Stack::new().push(1).push(2);
    /// let (top, rest) = s.pop().unwrap();
    /// assert_eq!(top, 2);
    /// assert_eq!(rest.peek(), Some(&1));
    /// ```
    #[inline]
    pub fn pop(mut self) -> Option<(A, Self)>
    where
        A: Clone,
    {
        let node = self.head.take()?;
        match Rc::try_unwrap(node) {
            Ok(n) => Some((n.elem, Stack { head: n.next })),
            // Shared: clone the top element, share the tail.
            Err(rc) => Some((
                rc.elem.clone(),
                Stack {
                    head: rc.next.clone(),
                },
            )),
        }
    }

    /// Peek at the top value without removing it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Stack;
    ///
    /// let s = Stack::new().push(1).push(2);
    /// assert_eq!(s.peek(), Some(&2));
    /// assert_eq!(s.peek(), Some(&2)); // Still there
    /// ```
    #[inline]
    pub fn peek(&self) -> Option<&A> {
        self.head.as_deref().map(|n| &n.elem)
    }

    /// Get the tail of the stack (everything except the top).
    ///
    /// Returns the tail stack, if any (O(1) clone via structural sharing).
    #[inline]
    pub fn tail(&self) -> Option<Self> {
        self.head.as_ref().map(|node| Stack {
            head: node.next.clone(),
        })
    }

    /// Get the length of the stack.
    ///
    /// Note: This is O(n) as it traverses the entire stack.
    pub fn len(&self) -> usize {
        // Use iterative approach to avoid recursion limit
        let mut count = 0;
        let mut current = self.head.as_deref();
        while let Some(node) = current {
            count += 1;
            current = node.next.as_deref();
        }
        count
    }

    /// Get the element at the specified index.
    ///
    /// Index 0 is the top of the stack.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&A> {
        // Use iterative approach to avoid recursion limit
        let mut current = self.head.as_deref();
        let mut remaining = index;
        while let Some(node) = current {
            if remaining == 0 {
                return Some(&node.elem);
            }
            remaining -= 1;
            current = node.next.as_deref();
        }
        None
    }

    /// Update the element at the specified index.
    ///
    /// Returns a new stack with the updated value. The nodes before `index`
    /// are copied; the tail after it is shared with the original stack.
    ///
    /// # Errors
    ///
    /// Returns `StackError::IndexOutOfBounds` if `index >= self.len()`.
    pub fn update(&self, index: usize, value: A) -> Result<Self, StackError>
    where
        A: Clone,
    {
        // Use iterative approach to avoid recursion limit: walk to `index`
        // collecting the prefix, replace the element, share the tail.
        // (No with_capacity(index): a wildly out-of-bounds index must
        // return Err, not attempt a huge allocation.)
        let mut prefix: Vec<&A> = Vec::new();
        let mut current = self.head.as_deref();
        loop {
            match current {
                None => return Err(StackError::IndexOutOfBounds),
                Some(node) => {
                    if prefix.len() == index {
                        let updated = Stack {
                            head: Some(Rc::new(Node {
                                elem: value,
                                next: node.next.clone(),
                            })),
                        };
                        return Ok(prefix
                            .into_iter()
                            .rev()
                            .fold(updated, |acc, x| acc.push(x.clone())));
                    }
                    prefix.push(&node.elem);
                    current = node.next.as_deref();
                }
            }
        }
    }

    /// Reverse the stack.
    ///
    /// Returns a new stack with elements in reverse order.
    pub fn reverse(&self) -> Self
    where
        A: Clone,
    {
        let mut result = Stack::new();
        let mut current = self.head.as_deref();
        while let Some(node) = current {
            result = result.push(node.elem.clone());
            current = node.next.as_deref();
        }
        result
    }

    /// Map a function over the stack.
    ///
    /// Returns a new stack with the function applied to each element.
    #[inline]
    pub fn map<B, F>(&self, f: F) -> Stack<B>
    where
        A: Clone,
        F: Fn(&A) -> B,
    {
        // Use iterative approach to avoid recursion limit
        let mut mapped: Vec<B> = Vec::with_capacity(self.len());
        let mut current = self.head.as_deref();
        while let Some(node) = current {
            mapped.push(f(&node.elem));
            current = node.next.as_deref();
        }
        // Build result in reverse
        mapped.into_iter().rev().fold(Stack::new(), Stack::push)
    }

    /// Fold the stack from left to right.
    #[inline]
    pub fn fold<B, F>(&self, init: B, f: F) -> B
    where
        F: Fn(B, &A) -> B,
    {
        // Use iterative approach to avoid recursion limit
        let mut acc = init;
        let mut current = self.head.as_deref();
        while let Some(node) = current {
            acc = f(acc, &node.elem);
            current = node.next.as_deref();
        }
        acc
    }

    /// Filter elements that satisfy the predicate.
    #[inline]
    pub fn filter<F>(&self, pred: F) -> Self
    where
        A: Clone,
        F: Fn(&A) -> bool,
    {
        // Use iterative approach to avoid recursion limit
        let mut filtered: Vec<&A> = Vec::with_capacity(self.len());
        let mut current = self.head.as_deref();
        while let Some(node) = current {
            if pred(&node.elem) {
                filtered.push(&node.elem);
            }
            current = node.next.as_deref();
        }
        // Build result in reverse
        filtered
            .into_iter()
            .rev()
            .fold(Stack::new(), |acc, x| acc.push(x.clone()))
    }

    /// Concatenate two stacks.
    ///
    /// Elements of `self` come before elements of `other`.
    #[inline]
    pub fn concat(&self, other: &Self) -> Self
    where
        A: Clone,
    {
        // Collect self's elements, then build on top of other
        let mut elements: Vec<&A> = Vec::with_capacity(self.len());
        let mut current = self.head.as_deref();
        while let Some(node) = current {
            elements.push(&node.elem);
            current = node.next.as_deref();
        }
        // Build result by pushing in reverse order onto other
        elements
            .into_iter()
            .rev()
            .fold(other.clone(), |acc, x| acc.push(x.clone()))
    }

    /// Convert to a Vec.
    #[inline]
    pub fn to_vec(&self) -> Vec<A>
    where
        A: Clone,
    {
        let mut result = Vec::with_capacity(self.len());
        let mut current = self.head.as_deref();
        while let Some(node) = current {
            result.push(node.elem.clone());
            current = node.next.as_deref();
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> From<Vec<A>> for Stack<A> {
    fn from(vec: Vec<A>) -> Self {
        vec.into_iter().rev().fold(Stack::new(), Stack::push)
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> FromIterator<A> for Stack<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        // Collect to vec first, then reverse to maintain order
        let vec: Vec<A> = iter.into_iter().collect();
        vec.into_iter().rev().fold(Stack::new(), Stack::push)
    }
}

/// Iterator over a Stack.
#[cfg(feature = "alloc")]
pub struct StackIter<'a, A> {
    current: Option<&'a Node<A>>,
}

#[cfg(feature = "alloc")]
impl<'a, A> Iterator for StackIter<'a, A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current.take()?;
        self.current = node.next.as_deref();
        Some(&node.elem)
    }
}

#[cfg(feature = "alloc")]
impl<'a, A> IntoIterator for &'a Stack<A> {
    type Item = &'a A;
    type IntoIter = StackIter<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        StackIter {
            current: self.head.as_deref(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<A> Stack<A> {
    /// Returns an iterator over references to elements.
    #[inline]
    pub fn iter(&self) -> StackIter<'_, A> {
        StackIter {
            current: self.head.as_deref(),
        }
    }
}

#[cfg(feature = "serde")]
impl<A: serde::Serialize> serde::Serialize for Stack<A> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Top-first element sequence; iterative, unlike the old derived
        // node-nesting which recursed per element.
        s.collect_seq(self.iter())
    }
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<'de, A: serde::Deserialize<'de>> serde::Deserialize<'de> for Stack<A> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let items = Vec::<A>::deserialize(d)?;
        // Rebuild through push so invariants hold by construction.
        Ok(items.into_iter().rev().fold(Stack::new(), Stack::push))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_new_is_empty() {
        let s: Stack<i32> = Stack::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_push_pop() {
        let s = Stack::new().push(1).push(2).push(3);
        assert_eq!(s.len(), 3);
        assert_eq!(s.peek(), Some(&3));

        let (top, rest) = s
            .pop()
            .expect("stack with 3 elements should pop successfully");
        assert_eq!(top, 3);
        assert_eq!(rest.peek(), Some(&2));
    }

    #[test]
    fn test_persistence() {
        let s1 = Stack::new().push(1).push(2);
        let s2 = s1.clone().push(3);

        // s1 unchanged
        assert_eq!(s1.peek(), Some(&2));
        assert_eq!(s1.len(), 2);

        // s2 has new element
        assert_eq!(s2.peek(), Some(&3));
        assert_eq!(s2.len(), 3);
    }

    #[test]
    fn test_get() {
        let s = Stack::new().push(1).push(2).push(3);
        assert_eq!(s.get(0), Some(&3));
        assert_eq!(s.get(1), Some(&2));
        assert_eq!(s.get(2), Some(&1));
        assert_eq!(s.get(3), None);
    }

    #[test]
    fn test_update() {
        let s1 = Stack::new().push(1).push(2).push(3);
        let s2 = s1
            .update(1, 5)
            .expect("index 1 is valid for a 3-element stack");

        assert_eq!(s1.get(1), Some(&2)); // unchanged
        assert_eq!(s2.get(1), Some(&5)); // updated
    }

    #[test]
    fn test_reverse() {
        let s = Stack::new().push(1).push(2).push(3);
        let r = s.reverse();

        assert_eq!(r.peek(), Some(&1));
        let (_, r) = r.pop().expect("reversed stack has at least one element");
        assert_eq!(r.peek(), Some(&2));
    }

    #[test]
    fn test_map() {
        let s = Stack::new().push(1).push(2).push(3);
        let doubled = s.map(|x| x * 2);

        assert_eq!(doubled.peek(), Some(&6));
        assert_eq!(doubled.get(1), Some(&4));
        assert_eq!(doubled.get(2), Some(&2));
    }

    #[test]
    fn test_fold() {
        let s = Stack::new().push(1).push(2).push(3);
        let sum = s.fold(0, |acc, x| acc + x);
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_filter() {
        let s = Stack::new().push(1).push(2).push(3).push(4);
        let evens = s.filter(|x| x % 2 == 0);

        assert_eq!(evens.len(), 2);
        assert_eq!(evens.peek(), Some(&4));
    }

    #[test]
    fn test_concat() {
        let s1 = Stack::new().push(1).push(2);
        let s2 = Stack::new().push(3).push(4);
        let combined = s1.concat(&s2);

        assert_eq!(combined.len(), 4);
        assert_eq!(combined.to_vec(), vec![2, 1, 4, 3]);
    }

    #[test]
    fn test_from_vec() {
        let v = vec![1, 2, 3];
        let s = Stack::from(v);

        assert_eq!(s.peek(), Some(&1));
        assert_eq!(s.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_iter() {
        let s = Stack::new().push(1).push(2).push(3);
        let items: Vec<_> = s.into_iter().copied().collect();
        assert_eq!(items, vec![3, 2, 1]);
    }

    #[test]
    fn test_from_iter() {
        let s: Stack<i32> = vec![1, 2, 3].into_iter().collect();
        assert_eq!(s.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_update_empty_stack_returns_error() {
        let s: Stack<i32> = Stack::new();
        assert_eq!(s.update(0, 42), Err(StackError::IndexOutOfBounds));
    }

    #[test]
    fn test_update_out_of_bounds_returns_error() {
        let s = Stack::new().push(1).push(2);
        // Stack has indices 0 and 1; index 2 is out of bounds
        assert_eq!(s.update(2, 99), Err(StackError::IndexOutOfBounds));
    }

    /// `peek`, `pop`, and `tail` on an empty stack must all return `None`,
    /// and `filter` with a predicate that rejects every element must yield
    /// an empty stack.
    #[test]
    fn test_empty_stack_observer_edge_cases() {
        let empty: Stack<i32> = Stack::new();

        assert_eq!(empty.peek(), None, "peek on empty stack must return None");
        assert!(
            empty.clone().pop().is_none(),
            "pop on empty stack must return None"
        );
        assert!(
            empty.tail().is_none(),
            "tail on empty stack must return None"
        );

        let s = Stack::new().push(1).push(2).push(3);
        let none_pass = s.filter(|_| false);
        assert!(
            none_pass.is_empty(),
            "filter rejecting all elements must yield an empty stack"
        );
    }

    /// H6 regression (reproduced pre-fix as a process abort at ~30k).
    #[test]
    fn drop_deep_stack_no_overflow() {
        let mut s = Stack::new();
        for i in 0..100_000u32 {
            s = s.push(i);
        }
        drop(s);
    }

    /// H6 regression: eq on two independently built stacks (no shared Rc
    /// spine, so the ptr_eq shortcut cannot save us).
    #[test]
    fn eq_deep_stacks_no_overflow() {
        let build = || (0..100_000u32).fold(Stack::new(), super::Stack::push);
        assert_eq!(build(), build());
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn serde_roundtrip_is_sequence_shaped() {
        let s = Stack::new().push(1).push(2).push(3);
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "[3,2,1]"); // top-first sequence, not nested nodes
        let back: Stack<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn serde_deep_input_no_overflow() {
        let json = serde_json::to_string(&(0..100_000u32).collect::<Vec<_>>()).unwrap();
        let s: Stack<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(s.len(), 100_000);
    }
}
