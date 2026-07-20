//! Characterization tests for `pfds::Stack` and `pfds::{OrdMap, OrdSet}`,
//! written ahead of two refactors: recursive `len`/`get`/`update`/`fold` →
//! iterative, and `union_par`/`difference_par` inline duplicates →
//! delegation to the sequential ops.
//!
//! Pinned current behavior, including quirks:
//! - `Stack::get(0)` is the **top**; `fold` is a top-first left fold;
//!   `to_vec` is top-first; `concat` stacks `self`'s elements above `other`'s.
//! - `Stack::update` reports out-of-bounds — including on an Empty stack —
//!   as `StackError::IndexOutOfBounds` (the `Empty` variant is never produced).
//! - `OrdMap::union` is **right-biased** on duplicate keys (`other` wins),
//!   while `OrdMap::intersection` is **left-biased** (`self`'s value wins).
//! - All `*_par` set/map operations agree with their sequential versions on
//!   both sides of the internal 1024-length branch.

#![cfg(feature = "alloc")]

use ordofp_core::pfds::{OrdMap, OrdSet, Stack, StackError};

fn stack_of(values: &[i32]) -> Stack<i32> {
    // Pushes left to right, so the LAST element of `values` ends up on top.
    values.iter().fold(Stack::new(), |s, &v| s.push(v))
}

#[test]
fn stack_model_equivalence_at_safe_depth() {
    let depth = 1000;
    let s = (0..depth).fold(Stack::new(), ordofp_core::pfds::Stack::push);
    let model: Vec<i32> = (0..depth).rev().collect();

    assert_eq!(s.len(), depth as usize);
    assert_eq!(s.to_vec(), model);

    // get(i) is the i-th element from the top.
    assert_eq!(s.get(0), Some(&(depth - 1)));
    assert_eq!(s.get(500), Some(&(depth - 1 - 500)));
    assert_eq!(s.get((depth - 1) as usize), Some(&0));
    assert_eq!(s.get(depth as usize), None);

    // fold visits top-first, agreeing with to_vec order.
    let folded = s.fold(Vec::new(), |mut acc, &x| {
        acc.push(x);
        acc
    });
    assert_eq!(folded, model);
}

#[test]
fn stack_fold_is_top_first_left_fold() {
    let s = stack_of(&[1, 2, 3]); // top = 3
    let order = s.fold(String::new(), |acc, x| format!("{acc}{x}"));
    assert_eq!(order, "321");
}

#[test]
fn stack_update_pins() {
    let s = stack_of(&[1, 2, 3]); // to_vec = [3, 2, 1]

    assert_eq!(s.update(0, 9).unwrap().to_vec(), vec![9, 2, 1]);
    assert_eq!(s.update(2, 9).unwrap().to_vec(), vec![3, 2, 9]);
    assert_eq!(s.update(3, 9), Err(StackError::IndexOutOfBounds));
    assert_eq!(s.update(usize::MAX, 9), Err(StackError::IndexOutOfBounds));

    // Updating an Empty stack is IndexOutOfBounds.
    assert_eq!(
        Stack::<i32>::new().update(0, 9),
        Err(StackError::IndexOutOfBounds)
    );

    // Persistence: the original is untouched.
    assert_eq!(s.to_vec(), vec![3, 2, 1]);
}

#[test]
fn stack_combinator_pins() {
    let s = stack_of(&[1, 2, 3]); // to_vec = [3, 2, 1]

    assert_eq!(s.reverse().to_vec(), vec![1, 2, 3]);
    assert_eq!(s.map(|x| x * 2).to_vec(), vec![6, 4, 2]);
    assert_eq!(s.filter(|x| x % 2 == 1).to_vec(), vec![3, 1]);

    // concat keeps self's elements above other's.
    let t = stack_of(&[4, 5]); // to_vec = [5, 4]
    assert_eq!(s.concat(&t).to_vec(), vec![3, 2, 1, 5, 4]);

    let empty = Stack::<i32>::new();
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert_eq!(empty.peek(), None);
    assert_eq!(empty.clone().pop(), None);
    assert_eq!(empty.get(0), None);
    assert_eq!(empty.to_vec(), Vec::<i32>::new());
    assert!(empty.reverse().is_empty());
    assert!(empty.map(|x| x + 1).is_empty());
    assert!(empty.filter(|_| true).is_empty());
    assert_eq!(empty.fold(7, |acc, x| acc + x), 7);
}

#[test]
fn stack_ops_handle_deep_stacks_without_overflow() {
    // Regression guard: len/get/update/fold were formerly recursive
    // and overflowed the thread stack at this depth.
    let depth: i32 = 100_000;
    let s = (0..depth).fold(Stack::new(), ordofp_core::pfds::Stack::push);

    assert_eq!(s.len(), depth as usize);
    assert_eq!(s.get((depth - 1) as usize), Some(&0));
    assert_eq!(
        s.fold(0i64, |acc, &x| acc + i64::from(x)),
        (0..i64::from(depth)).sum::<i64>()
    );

    let updated = s.update((depth - 1) as usize, -1).unwrap();
    assert_eq!(updated.get((depth - 1) as usize), Some(&-1));
    assert_eq!(updated.get(0), Some(&(depth - 1)));

    // Stack's Drop is iterative (Task 9), so these deep stacks can drop
    // normally at the end of the test without overflowing the thread stack.
}

// ---------------------------------------------------------------------------
// OrdMap / OrdSet
// ---------------------------------------------------------------------------

fn map_of(pairs: &[(i32, i32)]) -> OrdMap<i32, i32> {
    pairs
        .iter()
        .fold(OrdMap::new(), |m, &(k, v)| m.insert(k, v))
}

fn set_of(values: impl IntoIterator<Item = i32>) -> OrdSet<i32> {
    values.into_iter().fold(OrdSet::new(), |s, v| s.insert(v))
}

fn map_pairs(m: &OrdMap<i32, i32>) -> Vec<(i32, i32)> {
    m.iter().map(|(k, v)| (*k, *v)).collect()
}

fn set_items(s: &OrdSet<i32>) -> Vec<i32> {
    s.iter().copied().collect()
}

#[test]
fn ordmap_union_is_right_biased_on_duplicate_keys() {
    let a = map_of(&[(1, 10), (2, 20)]);
    let b = map_of(&[(2, 99), (3, 30)]);

    // On the duplicate key 2, OTHER's value (99) wins.
    assert_eq!(map_pairs(&a.union(&b)), vec![(1, 10), (2, 99), (3, 30)]);
    // Flipped, self is `b`, other is `a`: a's value (20) wins.
    assert_eq!(map_pairs(&b.union(&a)), vec![(1, 10), (2, 20), (3, 30)]);

    assert_eq!(map_pairs(&a.union(&a)), map_pairs(&a));
    assert_eq!(map_pairs(&a.union(&OrdMap::new())), map_pairs(&a));
    assert_eq!(map_pairs(&OrdMap::new().union(&a)), map_pairs(&a));
}

#[test]
fn ordmap_intersection_is_left_biased_and_difference_keeps_self_only() {
    let a = map_of(&[(1, 10), (2, 20), (4, 40)]);
    let b = map_of(&[(2, 99), (3, 30), (4, 44)]);

    // Intersection keeps SELF's values.
    assert_eq!(map_pairs(&a.intersection(&b)), vec![(2, 20), (4, 40)]);
    assert_eq!(map_pairs(&b.intersection(&a)), vec![(2, 99), (4, 44)]);

    // Difference: keys in self but not in other.
    assert_eq!(map_pairs(&a.difference(&b)), vec![(1, 10)]);
    assert_eq!(map_pairs(&b.difference(&a)), vec![(3, 30)]);
    assert_eq!(map_pairs(&a.difference(&a)), Vec::new());
}

#[test]
fn ordset_union_intersection_difference_pins() {
    let a = set_of([1, 2, 4]);
    let b = set_of([2, 3, 4, 5]);

    assert_eq!(set_items(&a.union(&b)), vec![1, 2, 3, 4, 5]);
    assert_eq!(set_items(&a.intersection(&b)), vec![2, 4]);
    assert_eq!(set_items(&a.difference(&b)), vec![1]);
    assert_eq!(set_items(&b.difference(&a)), vec![3, 5]);

    let empty = OrdSet::new();
    assert_eq!(set_items(&a.union(&empty)), vec![1, 2, 4]);
    assert_eq!(set_items(&empty.union(&a)), vec![1, 2, 4]);
    assert_eq!(set_items(&a.intersection(&empty)), Vec::new());
}

/// Inputs sized to exercise both sides of the internal `len() > 1024`
/// branch inside the `*_par` operations.
#[cfg(feature = "rayon")]
mod par_equivalence {
    use super::*;

    fn large_map_a() -> OrdMap<i32, i32> {
        map_of(&(0..1500).map(|i| (i, i * 2)).collect::<Vec<_>>())
    }

    fn large_map_b() -> OrdMap<i32, i32> {
        map_of(&(750..2250).map(|i| (i, i * 3)).collect::<Vec<_>>())
    }

    #[test]
    fn ordmap_par_ops_agree_with_sequential_small() {
        let a = map_of(&[(1, 10), (2, 20), (4, 40)]);
        let b = map_of(&[(2, 99), (3, 30)]);

        assert_eq!(map_pairs(&a.union_par(&b)), map_pairs(&a.union(&b)));
        assert_eq!(
            map_pairs(&a.intersection_par(&b)),
            map_pairs(&a.intersection(&b))
        );
        assert_eq!(
            map_pairs(&a.difference_par(&b)),
            map_pairs(&a.difference(&b))
        );
    }

    #[test]
    fn ordmap_par_ops_agree_with_sequential_large() {
        let a = large_map_a();
        let b = large_map_b();

        assert_eq!(map_pairs(&a.union_par(&b)), map_pairs(&a.union(&b)));
        assert_eq!(
            map_pairs(&a.intersection_par(&b)),
            map_pairs(&a.intersection(&b))
        );
        assert_eq!(
            map_pairs(&a.difference_par(&b)),
            map_pairs(&a.difference(&b))
        );

        // Sanity on the merged content itself: overlap [750, 1500) takes b's
        // values (union is right-biased).
        let u = a.union_par(&b);
        assert_eq!(u.get(&0), Some(&0));
        assert_eq!(u.get(&800), Some(&2400));
        assert_eq!(u.get(&2249), Some(&6747));
        assert_eq!(u.len(), 2250);
    }

    #[test]
    fn ordset_par_ops_agree_with_sequential() {
        let small_a = set_of([1, 2, 4]);
        let small_b = set_of([2, 3, 5]);
        assert_eq!(
            set_items(&small_a.union_par(&small_b)),
            set_items(&small_a.union(&small_b))
        );
        assert_eq!(
            set_items(&small_a.intersection_par(&small_b)),
            set_items(&small_a.intersection(&small_b))
        );
        assert_eq!(
            set_items(&small_a.difference_par(&small_b)),
            set_items(&small_a.difference(&small_b))
        );

        let big_a = set_of(0..1500);
        let big_b = set_of(750..2250);
        assert_eq!(
            set_items(&big_a.union_par(&big_b)),
            set_items(&big_a.union(&big_b))
        );
        assert_eq!(
            set_items(&big_a.intersection_par(&big_b)),
            set_items(&big_a.intersection(&big_b))
        );
        assert_eq!(
            set_items(&big_a.difference_par(&big_b)),
            set_items(&big_a.difference(&big_b))
        );
        assert_eq!(big_a.union_par(&big_b).len(), 2250);
    }
}
