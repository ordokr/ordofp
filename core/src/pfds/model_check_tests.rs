//! Randomized model checks for the persistent ordered structures.
//!
//! Each structure is exercised with thousands of deterministic, seeded mixed
//! operations (including a deletion/pop-heavy second phase and a full drain) and
//! checked against the matching `std::collections` oracle after operations
//! (length, membership/lookup, ordered iteration, and front/back as
//! applicable). This flushes out latent rebalance/ordering bugs of the kind the
//! B-tree deletion fix addressed.
#![cfg(test)]

use super::{Deque, OrdMap, OrdSet, Seq};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::vec::Vec;

/// Deterministic xorshift64* RNG — no external dependency, fully reproducible.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u32) -> u32 {
        (self.next_u64() % u64::from(n)) as u32
    }
}

const KEY_SPACE: i32 = 256;
const SEEDS: u64 = 6;
const STEPS: u32 = 3000;

// =============================================================================
// OrdMap vs std::collections::BTreeMap
// =============================================================================

fn check_ordmap(ours: &OrdMap<i32, i32>, oracle: &BTreeMap<i32, i32>, seed: u64, step: u32) {
    assert_eq!(
        ours.len(),
        oracle.len(),
        "seed={seed} step={step} OrdMap len"
    );
    assert_eq!(
        ours.is_empty(),
        oracle.is_empty(),
        "seed={seed} step={step} OrdMap is_empty"
    );
    for p in 0..KEY_SPACE {
        assert_eq!(
            ours.get(&p),
            oracle.get(&p),
            "seed={seed} step={step} OrdMap get({p})"
        );
        assert_eq!(
            ours.contains_key(&p),
            oracle.contains_key(&p),
            "seed={seed} step={step} OrdMap contains_key({p})"
        );
    }
    let ours_iter: Vec<(i32, i32)> = ours.iter().map(|(k, v)| (*k, *v)).collect();
    let oracle_iter: Vec<(i32, i32)> = oracle.iter().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(
        ours_iter, oracle_iter,
        "seed={seed} step={step} OrdMap ordered iteration"
    );
    assert_eq!(
        ours.min().map(|(k, v)| (*k, *v)),
        oracle.iter().next().map(|(k, v)| (*k, *v)),
        "seed={seed} step={step} OrdMap min"
    );
    assert_eq!(
        ours.max().map(|(k, v)| (*k, *v)),
        oracle.iter().next_back().map(|(k, v)| (*k, *v)),
        "seed={seed} step={step} OrdMap max"
    );
}

#[test]
fn ordmap_model_check_vs_std() {
    for seed in 1..=SEEDS {
        let mut rng = Rng::new(seed);
        let mut ours: OrdMap<i32, i32> = OrdMap::new();
        let mut oracle: BTreeMap<i32, i32> = BTreeMap::new();
        for step in 0..STEPS {
            let k = rng.below(KEY_SPACE as u32) as i32;
            let remove = if step > STEPS / 2 {
                rng.below(3) != 0
            } else {
                rng.below(2) == 0
            };
            if remove {
                ours = ours.remove(&k);
                oracle.remove(&k);
            } else {
                let v = rng.next_u64() as i32;
                ours = ours.insert(k, v);
                oracle.insert(k, v);
            }
            if step % 300 == 0 {
                check_ordmap(&ours, &oracle, seed, step);
            }
        }
        check_ordmap(&ours, &oracle, seed, u32::MAX);
        // Full drain.
        let keys: Vec<i32> = oracle.keys().copied().collect();
        for k in keys {
            ours = ours.remove(&k);
            oracle.remove(&k);
        }
        assert!(
            ours.is_empty(),
            "seed={seed} OrdMap not empty after drain (len={})",
            ours.len()
        );
        check_ordmap(&ours, &oracle, seed, 0);
    }
}

// =============================================================================
// OrdSet vs std::collections::BTreeSet
// =============================================================================

fn check_ordset(ours: &OrdSet<i32>, oracle: &BTreeSet<i32>, seed: u64, step: u32) {
    assert_eq!(
        ours.len(),
        oracle.len(),
        "seed={seed} step={step} OrdSet len"
    );
    assert_eq!(
        ours.is_empty(),
        oracle.is_empty(),
        "seed={seed} step={step} OrdSet is_empty"
    );
    for p in 0..KEY_SPACE {
        assert_eq!(
            ours.contains(&p),
            oracle.contains(&p),
            "seed={seed} step={step} OrdSet contains({p})"
        );
    }
    let ours_iter: Vec<i32> = ours.iter().copied().collect();
    let oracle_iter: Vec<i32> = oracle.iter().copied().collect();
    assert_eq!(
        ours_iter, oracle_iter,
        "seed={seed} step={step} OrdSet ordered iteration"
    );
    assert_eq!(
        ours.min().copied(),
        oracle.iter().next().copied(),
        "seed={seed} OrdSet min"
    );
    assert_eq!(
        ours.max().copied(),
        oracle.iter().next_back().copied(),
        "seed={seed} OrdSet max"
    );
}

#[test]
fn ordset_model_check_vs_std() {
    for seed in 1..=SEEDS {
        let mut rng = Rng::new(seed ^ 0xABCD);
        let mut ours: OrdSet<i32> = OrdSet::new();
        let mut oracle: BTreeSet<i32> = BTreeSet::new();
        for step in 0..STEPS {
            let x = rng.below(KEY_SPACE as u32) as i32;
            let remove = if step > STEPS / 2 {
                rng.below(3) != 0
            } else {
                rng.below(2) == 0
            };
            if remove {
                ours = ours.remove(&x);
                oracle.remove(&x);
            } else {
                ours = ours.insert(x);
                oracle.insert(x);
            }
            if step % 300 == 0 {
                check_ordset(&ours, &oracle, seed, step);
            }
        }
        check_ordset(&ours, &oracle, seed, u32::MAX);
        let items: Vec<i32> = oracle.iter().copied().collect();
        for x in items {
            ours = ours.remove(&x);
            oracle.remove(&x);
        }
        assert!(
            ours.is_empty(),
            "seed={seed} OrdSet not empty after drain (len={})",
            ours.len()
        );
        check_ordset(&ours, &oracle, seed, 0);
    }
}

// =============================================================================
// Seq vs std::collections::VecDeque  (sequence with front/back + indexing)
// =============================================================================

fn check_seq(ours: &Seq<i32>, oracle: &VecDeque<i32>, seed: u64, step: u32) {
    assert_eq!(ours.len(), oracle.len(), "seed={seed} step={step} Seq len");
    assert_eq!(
        ours.is_empty(),
        oracle.is_empty(),
        "seed={seed} step={step} Seq is_empty"
    );
    assert_eq!(
        ours.first(),
        oracle.front(),
        "seed={seed} step={step} Seq first"
    );
    assert_eq!(
        ours.last(),
        oracle.back(),
        "seed={seed} step={step} Seq last"
    );
    let ours_iter: Vec<i32> = ours.iter().copied().collect();
    let oracle_iter: Vec<i32> = oracle.iter().copied().collect();
    assert_eq!(
        ours_iter, oracle_iter,
        "seed={seed} step={step} Seq front..back order"
    );
    for i in 0..oracle.len() {
        assert_eq!(
            ours.get(i),
            oracle.get(i),
            "seed={seed} step={step} Seq get({i})"
        );
    }
}

#[test]
fn seq_model_check_vs_std() {
    for seed in 1..=SEEDS {
        let mut rng = Rng::new(seed ^ 0x5151);
        let mut ours: Seq<i32> = Seq::new();
        let mut oracle: VecDeque<i32> = VecDeque::new();
        for step in 0..STEPS {
            let pop_heavy = step > STEPS / 2;
            let push = if pop_heavy {
                rng.below(4) == 0
            } else {
                rng.below(2) == 0
            };
            if push {
                let v = rng.next_u64() as i32;
                if rng.below(2) == 0 {
                    ours = ours.push_front(v);
                    oracle.push_front(v);
                } else {
                    ours = ours.push_back(v);
                    oracle.push_back(v);
                }
            } else if rng.below(2) == 0 {
                // pop_front: Seq returns (A, Self); consumes self even on None.
                if let Some((v, rest)) = ours.pop_front() {
                    ours = rest;
                    assert_eq!(
                        Some(v),
                        oracle.pop_front(),
                        "seed={seed} step={step} Seq pop_front"
                    );
                } else {
                    ours = Seq::new();
                    assert_eq!(
                        None,
                        oracle.pop_front(),
                        "seed={seed} step={step} Seq pop_front empty"
                    );
                }
            } else {
                // pop_back: Seq returns (Self, A).
                if let Some((rest, v)) = ours.pop_back() {
                    ours = rest;
                    assert_eq!(
                        Some(v),
                        oracle.pop_back(),
                        "seed={seed} step={step} Seq pop_back"
                    );
                } else {
                    ours = Seq::new();
                    assert_eq!(
                        None,
                        oracle.pop_back(),
                        "seed={seed} step={step} Seq pop_back empty"
                    );
                }
            }
            if step % 300 == 0 {
                check_seq(&ours, &oracle, seed, step);
            }
        }
        check_seq(&ours, &oracle, seed, u32::MAX);
        // Full drain alternating ends.
        let mut from_front = true;
        loop {
            if oracle.is_empty() {
                assert!(
                    ours.is_empty(),
                    "seed={seed} Seq not empty when oracle drained"
                );
                break;
            }
            if from_front {
                let (v, rest) = ours.pop_front().expect("Seq non-empty pop_front");
                ours = rest;
                assert_eq!(Some(v), oracle.pop_front(), "seed={seed} Seq drain front");
            } else {
                let (rest, v) = ours.pop_back().expect("Seq non-empty pop_back");
                ours = rest;
                assert_eq!(Some(v), oracle.pop_back(), "seed={seed} Seq drain back");
            }
            from_front = !from_front;
        }
    }
}

// =============================================================================
// Deque vs std::collections::VecDeque
// =============================================================================

fn check_deque(ours: &Deque<i32>, oracle: &VecDeque<i32>, seed: u64, step: u32) {
    assert_eq!(
        ours.len(),
        oracle.len(),
        "seed={seed} step={step} Deque len"
    );
    assert_eq!(
        ours.is_empty(),
        oracle.is_empty(),
        "seed={seed} step={step} Deque is_empty"
    );
    assert_eq!(
        ours.peek_front(),
        oracle.front(),
        "seed={seed} step={step} Deque peek_front"
    );
    assert_eq!(
        ours.peek_back(),
        oracle.back(),
        "seed={seed} step={step} Deque peek_back"
    );
    let ours_vec: Vec<i32> = ours.to_vec();
    let oracle_vec: Vec<i32> = oracle.iter().copied().collect();
    assert_eq!(
        ours_vec, oracle_vec,
        "seed={seed} step={step} Deque front..back order"
    );
}

#[test]
fn deque_model_check_vs_std() {
    for seed in 1..=SEEDS {
        let mut rng = Rng::new(seed ^ 0xDECA);
        let mut ours: Deque<i32> = Deque::new();
        let mut oracle: VecDeque<i32> = VecDeque::new();
        for step in 0..STEPS {
            let pop_heavy = step > STEPS / 2;
            let push = if pop_heavy {
                rng.below(4) == 0
            } else {
                rng.below(2) == 0
            };
            if push {
                let v = rng.next_u64() as i32;
                if rng.below(2) == 0 {
                    ours = ours.push_front(v);
                    oracle.push_front(v);
                } else {
                    ours = ours.push_back(v);
                    oracle.push_back(v);
                }
            } else if rng.below(2) == 0 {
                // Deque pop_front returns (A, Self); consumes self even on None.
                if let Some((v, rest)) = ours.pop_front() {
                    ours = rest;
                    assert_eq!(
                        Some(v),
                        oracle.pop_front(),
                        "seed={seed} step={step} Deque pop_front"
                    );
                } else {
                    ours = Deque::new();
                    assert_eq!(
                        None,
                        oracle.pop_front(),
                        "seed={seed} step={step} Deque pop_front empty"
                    );
                }
            } else {
                // Deque pop_back returns (A, Self).
                if let Some((v, rest)) = ours.pop_back() {
                    ours = rest;
                    assert_eq!(
                        Some(v),
                        oracle.pop_back(),
                        "seed={seed} step={step} Deque pop_back"
                    );
                } else {
                    ours = Deque::new();
                    assert_eq!(
                        None,
                        oracle.pop_back(),
                        "seed={seed} step={step} Deque pop_back empty"
                    );
                }
            }
            if step % 300 == 0 {
                check_deque(&ours, &oracle, seed, step);
            }
        }
        check_deque(&ours, &oracle, seed, u32::MAX);
        // Full drain alternating ends.
        let mut from_front = true;
        loop {
            if oracle.is_empty() {
                assert!(
                    ours.is_empty(),
                    "seed={seed} Deque not empty when oracle drained"
                );
                break;
            }
            if from_front {
                let (v, rest) = ours.pop_front().expect("Deque non-empty pop_front");
                ours = rest;
                assert_eq!(Some(v), oracle.pop_front(), "seed={seed} Deque drain front");
            } else {
                let (v, rest) = ours.pop_back().expect("Deque non-empty pop_back");
                ours = rest;
                assert_eq!(Some(v), oracle.pop_back(), "seed={seed} Deque drain back");
            }
            from_front = !from_front;
        }
    }
}
