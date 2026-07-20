//! Persistent Queue implementation.
//!
//! A fully persistent FIFO (First In, First Out) queue using the banker's
//! queue technique with two stacks for amortized O(1) operations.
//!
//! # Optimization Note
//!
//! This implementation stores the `front` vector in reverse order (stack)
//! to allow O(1) removal of the queue head (which is the stack top).
//! The `rear` vector is stored in normal order (newest elements at the end).

use core::iter::FromIterator;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A persistent (immutable) FIFO queue.
///
/// Uses the banker's queue technique: two lists (front and rear).
/// The `front` list stores elements in reverse order (head of queue is at end of vector).
/// The `rear` list stores elements in arrival order (tail of queue is at end of vector).
/// When `front` is empty, `rear` is reversed to become the new `front`.
///
/// # Time Complexity
///
/// | Operation | Time (amortized) |
/// |-----------|------------------|
/// | `enqueue` | O(1)             |
/// | `dequeue` | O(1)             |
/// | `peek`    | O(1)             |
/// | `len`     | O(1)             |
///
/// # Example
///
/// ```rust
/// use ordofp_core::pfds::Queue;
///
/// let q1 = Queue::new().enqueue(1).enqueue(2).enqueue(3);
/// let (first, q2) = q1.clone().dequeue().unwrap();
/// assert_eq!(first, 1);
/// assert_eq!(q2.peek(), Some(&2));
///
/// // q1 is still valid
/// assert_eq!(q1.peek(), Some(&1));
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct Queue<A> {
    /// Front of the queue (stored in reverse order, head is at end).
    front: Vec<A>,
    /// Rear of the queue (stored in normal order, new elements at end).
    rear: Vec<A>,
}

#[cfg(feature = "alloc")]
impl<A> Default for Queue<A> {
    fn default() -> Self {
        Queue::new()
    }
}

#[cfg(feature = "alloc")]
impl<A: PartialEq> PartialEq for Queue<A> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        // front is reversed, so iterate it in reverse to get queue order
        let self_iter = self.front.iter().rev().chain(self.rear.iter());
        let other_iter = other.front.iter().rev().chain(other.rear.iter());
        self_iter.eq(other_iter)
    }
}

#[cfg(feature = "alloc")]
impl<A: Eq> Eq for Queue<A> {}

#[cfg(feature = "alloc")]
impl<A> Queue<A> {
    /// Create a new empty queue.
    #[inline]
    pub fn new() -> Self {
        Queue {
            front: Vec::new(),
            rear: Vec::new(),
        }
    }

    /// Check if the queue is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.front.is_empty() && self.rear.is_empty()
    }

    /// Get the number of elements in the queue.
    #[inline]
    pub fn len(&self) -> usize {
        self.front.len() + self.rear.len()
    }

    /// Enqueue a value at the back of the queue.
    ///
    /// Returns a new queue with the value added.
    #[inline]
    pub fn enqueue(mut self, value: A) -> Self {
        self.rear.push(value);
        Queue {
            front: self.front,
            rear: self.rear,
        }
    }

    /// Dequeue a value from the front of the queue.
    ///
    /// Returns `Some((value, new_queue))` if non-empty, `None` otherwise.
    #[inline]
    pub fn dequeue(mut self) -> Option<(A, Self)>
    where
        A: Clone,
    {
        if self.front.is_empty() {
            if self.rear.is_empty() {
                return None;
            }
            // Move rear to front and reverse it to maintain invariant
            core::mem::swap(&mut self.front, &mut self.rear);
            self.front.reverse();
        }

        // Pop from end (which is head of queue)
        let value = self.front.pop()?;
        Some((
            value,
            Queue {
                front: self.front,
                rear: self.rear,
            },
        ))
    }

    /// Peek at the front value without removing it.
    #[inline]
    pub fn peek(&self) -> Option<&A> {
        if !self.front.is_empty() {
            // Head is at the end of front
            self.front.last()
        } else if !self.rear.is_empty() {
            // If front is empty, head is the first element of rear
            self.rear.first()
        } else {
            None
        }
    }

    /// Peek at the back value without removing it.
    #[inline]
    pub fn peek_back(&self) -> Option<&A> {
        if !self.rear.is_empty() {
            // Last enqueued is at the end of rear
            self.rear.last()
        } else if !self.front.is_empty() {
            // If rear is empty, back is the first element of front (since front is reversed)
            self.front.first()
        } else {
            None
        }
    }

    /// Map a function over the queue.
    #[inline]
    pub fn map<B, F>(&self, f: F) -> Queue<B>
    where
        A: Clone,
        F: Fn(&A) -> B,
    {
        Queue {
            // front is reversed, mapping keeps it reversed
            front: self.front.iter().map(&f).collect(),
            rear: self.rear.iter().map(&f).collect(),
        }
    }

    /// Fold the queue from front to back.
    #[inline]
    pub fn fold<B, F>(&self, init: B, f: F) -> B
    where
        A: Clone,
        F: Fn(B, &A) -> B,
    {
        // front is reversed, so we must fold it in reverse (rfold) to go Front->Back
        let acc = self.front.iter().rfold(init, &f);
        self.rear.iter().fold(acc, f)
    }

    /// Filter elements that satisfy the predicate.
    #[inline]
    pub fn filter<F>(&self, pred: F) -> Self
    where
        A: Clone,
        F: Fn(&A) -> bool,
    {
        // front is reversed. Filtering preserves order, so it remains reversed.
        let front: Vec<A> = self.front.iter().filter(|x| pred(x)).cloned().collect();
        let rear: Vec<A> = self.rear.iter().filter(|x| pred(x)).cloned().collect();
        Queue { front, rear }
    }

    /// Convert to a Vec (in FIFO order).
    #[inline]
    pub fn to_vec(&self) -> Vec<A>
    where
        A: Clone,
    {
        // front is reversed, so iterate reverse to get queue order
        let mut result: Vec<A> = Vec::with_capacity(self.len());
        result.extend(self.front.iter().rev().cloned());
        result.extend(self.rear.iter().cloned());
        result
    }

    /// Concatenate two queues.
    #[inline]
    pub fn concat(&self, other: &Self) -> Self
    where
        A: Clone,
    {
        let mut result = self.clone();
        // Iterate other in queue order
        // other.front is reversed, so rev() gives queue order
        for item in other.front.iter().rev() {
            result = result.enqueue(item.clone());
        }
        for item in &other.rear {
            result = result.enqueue(item.clone());
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> From<Vec<A>> for Queue<A> {
    fn from(vec: Vec<A>) -> Self {
        // Put everything in rear to avoid O(N) reverse.
        // First dequeue will handle reversal.
        Queue {
            front: Vec::new(),
            rear: vec,
        }
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> FromIterator<A> for Queue<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        Queue {
            front: Vec::new(),
            rear: iter.into_iter().collect(),
        }
    }
}

/// Iterator over a Queue.
#[cfg(feature = "alloc")]
pub struct QueueIter<A> {
    front: Vec<A>,
    rear: Vec<A>,
}

#[cfg(feature = "alloc")]
impl<A> Iterator for QueueIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        // front is reversed, so pop gives head
        if let Some(val) = self.front.pop() {
            Some(val)
        } else if !self.rear.is_empty() {
            // Move rear to front and reverse
            core::mem::swap(&mut self.front, &mut self.rear);
            self.front.reverse();
            self.front.pop()
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.front.len() + self.rear.len();
        (len, Some(len))
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> IntoIterator for Queue<A> {
    type Item = A;
    type IntoIter = QueueIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        QueueIter {
            front: self.front,
            rear: self.rear,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_new_is_empty() {
        let q: Queue<i32> = Queue::new();
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn test_enqueue_dequeue() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        assert_eq!(q.len(), 3);
        assert_eq!(q.peek(), Some(&1));

        let (first, q) = q
            .dequeue()
            .expect("queue with 3 elements should dequeue first element");
        assert_eq!(first, 1);
        assert_eq!(q.peek(), Some(&2));

        let (second, q) = q
            .dequeue()
            .expect("queue with 2 elements should dequeue second element");
        assert_eq!(second, 2);
        assert_eq!(q.peek(), Some(&3));

        let (third, q) = q
            .dequeue()
            .expect("queue with 1 element should dequeue third element");
        assert_eq!(third, 3);
        assert!(q.is_empty());
    }

    #[test]
    fn test_persistence() {
        let q1 = Queue::new().enqueue(1).enqueue(2);
        let q2 = q1.clone().enqueue(3);

        // q1 unchanged
        assert_eq!(q1.len(), 2);
        assert_eq!(q1.peek(), Some(&1));

        // q2 has new element
        assert_eq!(q2.len(), 3);
    }

    #[test]
    fn test_fifo_order() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        let items: Vec<_> = q.into_iter().collect();
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn test_map() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        let doubled = q.map(|x| x * 2);

        let items: Vec<_> = doubled.into_iter().collect();
        assert_eq!(items, vec![2, 4, 6]);
    }

    #[test]
    fn test_fold() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        let sum = q.fold(0, |acc, x| acc + x);
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_filter() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3).enqueue(4);
        let evens = q.filter(|x| x % 2 == 0);

        assert_eq!(evens.len(), 2);
        let items: Vec<_> = evens.into_iter().collect();
        assert_eq!(items, vec![2, 4]);
    }

    #[test]
    fn test_filter_reject_all_yields_empty_queue() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        let empty = q.filter(|_| false);

        assert!(
            empty.is_empty(),
            "filter rejecting all elements must be empty"
        );
        assert_eq!(
            empty.len(),
            0,
            "filter rejecting all elements must have length 0"
        );
        assert_eq!(
            empty.peek(),
            None,
            "peek on all-rejected filter result must be None"
        );
        assert!(
            empty.dequeue().is_none(),
            "dequeue on all-rejected filter result must return None"
        );
    }

    #[test]
    fn test_from_vec() {
        let v = vec![1, 2, 3];
        let q = Queue::from(v);

        assert_eq!(q.len(), 3);
        assert_eq!(q.peek(), Some(&1));
    }

    #[test]
    fn test_to_vec() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        assert_eq!(q.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_concat() {
        let q1 = Queue::new().enqueue(1).enqueue(2);
        let q2 = Queue::new().enqueue(3).enqueue(4);
        let combined = q1.concat(&q2);

        assert_eq!(combined.len(), 4);
        assert_eq!(combined.to_vec(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_concat_with_empty_queue() {
        let empty: Queue<i32> = Queue::new();
        let non_empty = Queue::new().enqueue(1).enqueue(2).enqueue(3);

        // empty ++ non_empty == non_empty
        let result = empty.concat(&non_empty);
        assert_eq!(result.to_vec(), vec![1, 2, 3]);

        // non_empty ++ empty == non_empty
        let result = non_empty.concat(&empty);
        assert_eq!(result.to_vec(), vec![1, 2, 3]);

        // empty ++ empty == empty
        let result = empty.concat(&empty);
        assert!(result.is_empty());
    }

    #[test]
    fn test_from_iter() {
        let q: Queue<i32> = vec![1, 2, 3].into_iter().collect();
        assert_eq!(q.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_peek_back() {
        let q = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        assert_eq!(q.peek(), Some(&1));
        assert_eq!(q.peek_back(), Some(&3));
    }

    #[test]
    fn test_peek_and_dequeue_on_empty_queue_return_none() {
        // An empty queue must return None for all access operations rather
        // than panicking, since there is no head, tail, or value to return.
        let q: Queue<i32> = Queue::new();
        assert_eq!(q.peek(), None, "peek on empty queue must be None");
        assert_eq!(q.peek_back(), None, "peek_back on empty queue must be None");
        assert!(
            q.dequeue().is_none(),
            "dequeue on empty queue must return None"
        );
    }
}
