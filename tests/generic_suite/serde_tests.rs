//! Serde round-trip tests for all `OrdoFP` types.
//!
//! This module validates that all types serialize and deserialize correctly,
//! ensuring data integrity through JSON round-trips.

#![cfg(all(feature = "serde", feature = "alloc"))]

use ordofp_core::nonempty::NonEmpty;
use ordofp_core::pfds::{Deque, OrdMap, OrdSet, Queue, Seq, Stack};
use ordofp_core::zipper::Zipper;

// ============================================================================
// PFDS Round-Trip Tests
// ============================================================================

mod pfds_serde {
    use super::*;

    #[test]
    fn test_stack_roundtrip() {
        let stack = Stack::new().push(1).push(2).push(3);
        let json = serde_json::to_string(&stack).expect("serialize Stack");
        let deserialized: Stack<i32> = serde_json::from_str(&json).expect("deserialize Stack");

        assert_eq!(stack.len(), deserialized.len());
        assert_eq!(stack.peek(), deserialized.peek());
        assert_eq!(stack.to_vec(), deserialized.to_vec());
    }

    #[test]
    fn test_stack_empty_roundtrip() {
        let stack: Stack<i32> = Stack::new();
        let json = serde_json::to_string(&stack).expect("serialize empty Stack");
        let deserialized: Stack<i32> =
            serde_json::from_str(&json).expect("deserialize empty Stack");

        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_queue_roundtrip() {
        let queue = Queue::new().enqueue(1).enqueue(2).enqueue(3);
        let json = serde_json::to_string(&queue).expect("serialize Queue");
        let deserialized: Queue<i32> = serde_json::from_str(&json).expect("deserialize Queue");

        assert_eq!(queue.len(), deserialized.len());
        assert_eq!(queue.to_vec(), deserialized.to_vec());
    }

    #[test]
    fn test_queue_empty_roundtrip() {
        let queue: Queue<i32> = Queue::new();
        let json = serde_json::to_string(&queue).expect("serialize empty Queue");
        let deserialized: Queue<i32> =
            serde_json::from_str(&json).expect("deserialize empty Queue");

        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_deque_roundtrip() {
        let deque = Deque::new().push_back(1).push_back(2).push_front(0);
        let json = serde_json::to_string(&deque).expect("serialize Deque");
        let deserialized: Deque<i32> = serde_json::from_str(&json).expect("deserialize Deque");

        assert_eq!(deque.len(), deserialized.len());
        assert_eq!(deque.to_vec(), deserialized.to_vec());
    }

    #[test]
    fn test_deque_empty_roundtrip() {
        let deque: Deque<i32> = Deque::new();
        let json = serde_json::to_string(&deque).expect("serialize empty Deque");
        let deserialized: Deque<i32> =
            serde_json::from_str(&json).expect("deserialize empty Deque");

        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_seq_roundtrip() {
        let seq = Seq::new().push_back(1).push_back(2).push_back(3);
        let json = serde_json::to_string(&seq).expect("serialize Seq");
        let deserialized: Seq<i32> = serde_json::from_str(&json).expect("deserialize Seq");

        assert_eq!(seq.len(), deserialized.len());
        for i in 0..seq.len() {
            assert_eq!(seq.get(i), deserialized.get(i));
        }
    }

    #[test]
    fn test_seq_empty_roundtrip() {
        let seq: Seq<i32> = Seq::new();
        let json = serde_json::to_string(&seq).expect("serialize empty Seq");
        let deserialized: Seq<i32> = serde_json::from_str(&json).expect("deserialize empty Seq");

        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_ordmap_roundtrip() {
        let map = OrdMap::new().insert("a", 1).insert("b", 2).insert("c", 3);
        let json = serde_json::to_string(&map).expect("serialize OrdMap");
        let deserialized: OrdMap<&str, i32> =
            serde_json::from_str(&json).expect("deserialize OrdMap");

        assert_eq!(map.len(), deserialized.len());
        assert_eq!(map.get(&"a"), deserialized.get(&"a"));
        assert_eq!(map.get(&"b"), deserialized.get(&"b"));
        assert_eq!(map.get(&"c"), deserialized.get(&"c"));
    }

    #[test]
    fn test_ordmap_empty_roundtrip() {
        let map: OrdMap<String, i32> = OrdMap::new();
        let json = serde_json::to_string(&map).expect("serialize empty OrdMap");
        let deserialized: OrdMap<String, i32> =
            serde_json::from_str(&json).expect("deserialize empty OrdMap");

        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_ordset_roundtrip() {
        let set = OrdSet::new().insert(3).insert(1).insert(2);
        let json = serde_json::to_string(&set).expect("serialize OrdSet");
        let deserialized: OrdSet<i32> = serde_json::from_str(&json).expect("deserialize OrdSet");

        assert_eq!(set.len(), deserialized.len());
        assert!(deserialized.contains(&1));
        assert!(deserialized.contains(&2));
        assert!(deserialized.contains(&3));
    }

    #[test]
    fn test_ordset_empty_roundtrip() {
        let set: OrdSet<i32> = OrdSet::new();
        let json = serde_json::to_string(&set).expect("serialize empty OrdSet");
        let deserialized: OrdSet<i32> =
            serde_json::from_str(&json).expect("deserialize empty OrdSet");

        assert!(deserialized.is_empty());
    }
}

// ============================================================================
// Zipper Round-Trip Tests
// ============================================================================

mod zipper_serde {
    use super::*;

    #[test]
    fn test_zipper_roundtrip() {
        let zipper = Zipper::new(2, vec![1], vec![3, 4]);
        let json = serde_json::to_string(&zipper).expect("serialize Zipper");
        let deserialized: Zipper<i32> = serde_json::from_str(&json).expect("deserialize Zipper");

        assert_eq!(zipper.focus(), deserialized.focus());
        assert_eq!(zipper.to_vec(), deserialized.to_vec());
    }

    #[test]
    fn test_zipper_singleton_roundtrip() {
        let zipper = Zipper::singleton(42);
        let json = serde_json::to_string(&zipper).expect("serialize singleton Zipper");
        let deserialized: Zipper<i32> =
            serde_json::from_str(&json).expect("deserialize singleton Zipper");

        assert_eq!(zipper.focus(), deserialized.focus());
        assert_eq!(zipper.len(), deserialized.len());
    }

    #[test]
    fn test_zipper_from_vec_roundtrip() {
        let zipper = Zipper::from_vec(vec![1, 2, 3, 4, 5]).expect("non-empty vec yields a Zipper");
        let json = serde_json::to_string(&zipper).expect("serialize Zipper from vec");
        let deserialized: Zipper<i32> =
            serde_json::from_str(&json).expect("deserialize Zipper from vec");

        assert_eq!(zipper.focus(), deserialized.focus());
        assert_eq!(zipper.to_vec(), deserialized.to_vec());
    }

    #[test]
    fn test_zipper_after_navigation_roundtrip() {
        let zipper = Zipper::from_vec(vec![1, 2, 3, 4, 5])
            .expect("non-empty vec yields a Zipper")
            .focus_next()
            .expect("zipper has elements after index 0")
            .focus_next()
            .expect("zipper has elements after index 1");

        let json = serde_json::to_string(&zipper).expect("serialize navigated Zipper");
        let deserialized: Zipper<i32> =
            serde_json::from_str(&json).expect("deserialize navigated Zipper");

        assert_eq!(zipper.focus(), deserialized.focus());
        assert_eq!(*zipper.focus(), 3); // Verify focus is correct
    }
}

// ============================================================================
// NonEmpty Round-Trip Tests
// ============================================================================

mod nonempty_serde {
    use super::*;

    #[test]
    fn test_nonempty_roundtrip() {
        let ne = NonEmpty::new(1, vec![2, 3, 4]);
        let json = serde_json::to_string(&ne).expect("serialize NonEmpty");
        let deserialized: NonEmpty<i32> =
            serde_json::from_str(&json).expect("deserialize NonEmpty");

        assert_eq!(ne.head(), deserialized.head());
        assert_eq!(ne.len(), deserialized.len());
        assert_eq!(ne.to_vec(), deserialized.to_vec());
    }

    #[test]
    fn test_nonempty_singleton_roundtrip() {
        let ne = NonEmpty::singleton(42);
        let json = serde_json::to_string(&ne).expect("serialize singleton NonEmpty");
        let deserialized: NonEmpty<i32> =
            serde_json::from_str(&json).expect("deserialize singleton NonEmpty");

        assert_eq!(ne.head(), deserialized.head());
        assert_eq!(ne.len(), 1);
    }

    #[test]
    fn test_nonempty_string_roundtrip() {
        let ne = NonEmpty::new("hello".to_string(), vec!["world".to_string()]);
        let json = serde_json::to_string(&ne).expect("serialize NonEmpty<String>");
        let deserialized: NonEmpty<String> =
            serde_json::from_str(&json).expect("deserialize NonEmpty<String>");

        assert_eq!(ne.head(), deserialized.head());
        assert_eq!(ne.to_vec(), deserialized.to_vec());
    }
}

// ============================================================================
// Probatum Round-Trip Tests (requires Probatum feature)
// ============================================================================

#[cfg(feature = "Probatum")]
mod probatum_serde {
    use ordofp::Probatum;
    use ordofp::{HList, hlist};

    #[test]
    fn test_probatum_ok_roundtrip() {
        let probatum: Probatum<String, HList!(i32, String)> =
            Probatum::valid(hlist![42, "hello".to_string()]);

        let json = serde_json::to_string(&probatum).expect("serialize Probatum::Valid");
        let deserialized: Probatum<String, HList!(i32, String)> =
            serde_json::from_str(&json).expect("deserialize Probatum::Valid");

        assert!(deserialized.is_valid());
        assert_eq!(probatum, deserialized);
    }

    #[test]
    fn test_probatum_err_roundtrip() {
        let probatum: Probatum<String, HList!(i32)> =
            Probatum::invalid_many(["error1".to_string(), "error2".to_string()]);

        let json = serde_json::to_string(&probatum).expect("serialize Probatum::Invalid");
        let deserialized: Probatum<String, HList!(i32)> =
            serde_json::from_str(&json).expect("deserialize Probatum::Invalid");

        assert!(deserialized.is_invalid());
        assert_eq!(probatum, deserialized);
    }
}

// ============================================================================
// Complex Nested Type Round-Trip Tests
// ============================================================================

mod complex_serde {
    use super::*;

    #[test]
    fn test_nested_stack_of_queues() {
        let inner1 = Queue::new().enqueue(1).enqueue(2);
        let inner2 = Queue::new().enqueue(3).enqueue(4);
        let outer = Stack::new().push(inner1).push(inner2);

        let json = serde_json::to_string(&outer).expect("serialize nested Stack<Queue>");
        let deserialized: Stack<Queue<i32>> =
            serde_json::from_str(&json).expect("deserialize nested Stack<Queue>");

        assert_eq!(outer.len(), deserialized.len());
    }

    #[test]
    fn test_ordmap_of_nonempty() {
        let ne1 = NonEmpty::new(1, vec![2, 3]);
        let ne2 = NonEmpty::new(4, vec![5, 6]);
        let map = OrdMap::new()
            .insert("first".to_string(), ne1)
            .insert("second".to_string(), ne2);

        let json = serde_json::to_string(&map).expect("serialize OrdMap<String, NonEmpty>");
        let deserialized: OrdMap<String, NonEmpty<i32>> =
            serde_json::from_str(&json).expect("deserialize OrdMap<String, NonEmpty>");

        assert_eq!(map.len(), deserialized.len());
        assert!(deserialized.get(&"first".to_string()).is_some());
    }

    #[test]
    fn test_zipper_of_strings() {
        let zipper = Zipper::new(
            "focus".to_string(),
            vec!["left1".to_string(), "left2".to_string()],
            vec!["right1".to_string(), "right2".to_string()],
        );

        let json = serde_json::to_string(&zipper).expect("serialize Zipper<String>");
        let deserialized: Zipper<String> =
            serde_json::from_str(&json).expect("deserialize Zipper<String>");

        assert_eq!(zipper.focus(), deserialized.focus());
        assert_eq!(zipper.to_vec(), deserialized.to_vec());
    }
}

// ============================================================================
// Edge Cases and Stress Tests
// ============================================================================

mod edge_cases_serde {
    use super::*;

    #[test]
    fn test_stack_medium_roundtrip() {
        // Note: Stack uses recursive Arc structure, so JSON recursion limit
        // restricts max size. Use ~50 elements for reliable JSON round-trips.
        let mut stack = Stack::new();
        for i in 0..50 {
            stack = stack.push(i);
        }

        let json = serde_json::to_string(&stack).expect("serialize medium Stack");
        let deserialized: Stack<i32> =
            serde_json::from_str(&json).expect("deserialize medium Stack");

        assert_eq!(stack.len(), deserialized.len());
        assert_eq!(stack.peek(), deserialized.peek());
        assert_eq!(stack.to_vec(), deserialized.to_vec());
    }

    #[test]
    fn test_seq_medium_roundtrip() {
        // Note: Seq uses tree structure, more depth-efficient than Stack.
        // ~100 elements work reliably with JSON.
        let mut seq = Seq::new();
        for i in 0..100 {
            seq = seq.push_back(i);
        }

        let json = serde_json::to_string(&seq).expect("serialize medium Seq");
        let deserialized: Seq<i32> = serde_json::from_str(&json).expect("deserialize medium Seq");

        assert_eq!(seq.len(), deserialized.len());
        // Spot check some values
        assert_eq!(seq.get(0), deserialized.get(0));
        assert_eq!(seq.get(50), deserialized.get(50));
        assert_eq!(seq.get(99), deserialized.get(99));
    }

    #[test]
    fn test_ordmap_large_roundtrip() {
        let mut map = OrdMap::new();
        for i in 0..100 {
            map = map.insert(format!("key_{i}"), i);
        }

        let json = serde_json::to_string(&map).expect("serialize large OrdMap");
        let deserialized: OrdMap<String, i32> =
            serde_json::from_str(&json).expect("deserialize large OrdMap");

        assert_eq!(map.len(), deserialized.len());
        assert_eq!(
            map.get(&"key_0".to_string()),
            deserialized.get(&"key_0".to_string())
        );
        assert_eq!(
            map.get(&"key_50".to_string()),
            deserialized.get(&"key_50".to_string())
        );
    }

    #[test]
    fn test_special_characters_roundtrip() {
        let ne = NonEmpty::new(
            "hello\nworld\t\"quoted\"".to_string(),
            vec!["emoji: 🦀".to_string(), "unicode: 日本語".to_string()],
        );

        let json = serde_json::to_string(&ne).expect("serialize special chars");
        let deserialized: NonEmpty<String> =
            serde_json::from_str(&json).expect("deserialize special chars");

        assert_eq!(ne.head(), deserialized.head());
        assert_eq!(ne.to_vec(), deserialized.to_vec());
    }
}
