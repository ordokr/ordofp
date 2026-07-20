//! # Persistent Functional Data Structures (PFDS)
//!
//! This module provides persistent (immutable) data structures inspired by
//! Okasaki's "Purely Functional Data Structures" and the rust-fp library.
//!
//! ## Overview
//!
//! Persistent data structures preserve previous versions when modified,
//! enabling safe sharing and eliminating the need for explicit copying.
//! All operations return new structures, leaving the originals intact.
//!
//! ## Available Structures
//!
//! ### Simple Structures
//! - [`Stack`] - A persistent LIFO stack with O(1) push/pop
//! - [`Queue`] - A persistent FIFO queue with amortized O(1) operations
//! - [`Deque`] - A double-ended queue with O(1) operations at both ends
//!
//! ### Tree-Based Structures (AVL trees)
//! - [`Seq`] - Efficient sequence with O(log n) random access and O(log n) ends
//! - [`OrdMap`] - Ordered map with O(log n) insert/lookup/remove
//! - [`OrdSet`] - Ordered set with O(log n) operations and set algebra
//!
//! ## Example
//!
//! ```rust
//! use ordofp_core::pfds::{Stack, Queue, Seq, OrdMap, OrdSet};
//!
//! // Stack: LIFO operations
//! let s1 = Stack::new().push(1).push(2).push(3);
//! let (top, s2) = s1.clone().pop().unwrap();
//! assert_eq!(top, 3);
//! assert_eq!(s1.peek(), Some(&3)); // s1 unchanged!
//!
//! // Queue: FIFO operations
//! let q1 = Queue::new().enqueue(1).enqueue(2).enqueue(3);
//! let (first, q2) = q1.clone().dequeue().unwrap();
//! assert_eq!(first, 1);
//!
//! // Seq: Random access
//! let seq = Seq::new().push_back(1).push_back(2).push_back(3);
//! assert_eq!(seq.get(1), Some(&2));
//!
//! // OrdMap: Key-value storage
//! let map = OrdMap::new().insert("a", 1).insert("b", 2);
//! assert_eq!(map.get(&"a"), Some(&1));
//!
//! // OrdSet: Unique sorted elements
//! let set = OrdSet::new().insert(3).insert(1).insert(2);
//! assert_eq!(set.min(), Some(&1));
//! ```
//!
//! ## Persistence
//!
//! All structures use structural sharing via `Arc`, so "copying" is cheap:
//!
//! ```rust
//! use ordofp_core::pfds::Stack;
//!
//! let s1 = Stack::new().push(1).push(2).push(3);
//! let s2 = s1.clone(); // Cheap - shares structure
//! let s3 = s2.push(4); // s2's data is shared with s1
//!
//! assert_eq!(s1.len(), 3);
//! assert_eq!(s3.len(), 4);
//! ```

mod deque;
mod ord_map;
mod ord_set;
mod queue;
mod seq;
mod stack;

#[cfg(test)]
mod model_check_tests;

pub use deque::Deque;
pub use ord_map::{OrdMap, OrdMapStructor};
pub use ord_set::{OrdSet, OrdSetStructor};
pub use queue::Queue;
pub use seq::Seq;
pub use stack::{Stack, StackError};
