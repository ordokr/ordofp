//! Composed updates preserve structure and accept single-use owned callbacks.
#![cfg(feature = "Probatum")]

use ordofp_core::{optics::Aspectus, pfds::Seq, validated::Probatum};
use std::cell::Cell;

#[test]
fn composed_updates_only_set_after_success() {
    let setters = Cell::new(0);
    let outer = Aspectus::new(
        |s: &((i32, i32), i32)| s.0,
        |s: &((i32, i32), i32), a| {
            setters.set(setters.get() + 1);
            (a, s.1)
        },
    );
    let inner = Aspectus::new(
        |s: &(i32, i32)| s.0,
        |s: &(i32, i32), a| {
            setters.set(setters.get() + 1);
            (a, s.1)
        },
    );
    let lens = outer.compose(&inner);
    let source = ((1, 2), 3);
    assert_eq!(lens.modify_option(&source, |_| None), None);
    assert_eq!(lens.modify_result(&source, |_| Err("bad")), Err("bad"));
    assert_eq!(
        lens.modify_validated(&source, |_| Probatum::invalid_many(["a", "b"])),
        Probatum::invalid_many(["a", "b"])
    );
    assert_eq!(setters.get(), 0);
    assert_eq!(source, ((1, 2), 3));
    assert_eq!(
        lens.modify_option(&source, |x| Some(x + 1)),
        Some(((2, 2), 3))
    );
    assert_eq!(
        lens.modify_result::<(), _>(&source, |x| Ok(x + 2)),
        Ok(((3, 2), 3))
    );
    assert_eq!(
        lens.modify_validated::<(), _>(&source, |x| Probatum::Valid(x + 3)),
        Probatum::Valid(((4, 2), 3))
    );
    assert_eq!(setters.get(), 6);
    assert_eq!(source, ((1, 2), 3));
}

#[derive(Debug, PartialEq)]
struct Owned(i32);

#[test]
fn validation_callbacks_can_consume_captures() {
    let captured = Owned(10);
    let v = Probatum::<(), _>::Valid(Owned(1)).map(|_| captured);
    assert_eq!(v, Probatum::Valid(Owned(10)));
    let captured = Owned(20);
    assert_eq!(
        v.map2(Probatum::Valid(Owned(2)), |_, _| captured),
        Probatum::Valid(Owned(20))
    );
    let captured = Owned(30);
    assert_eq!(
        Probatum::<(), _>::Valid(Owned(1)).map3(
            Probatum::Valid(Owned(2)),
            Probatum::Valid(Owned(3)),
            |_, _, _| captured
        ),
        Probatum::Valid(Owned(30))
    );
    let called = Cell::new(false);
    let captured = Owned(40);
    let invalid = Probatum::<_, Owned>::invalid_many(["bad"]);
    assert_eq!(
        invalid.map(|_| {
            called.set(true);
            captured
        }),
        Probatum::invalid_many(["bad"])
    );
    assert!(!called.get());
}

#[test]
fn seq_map_needs_no_clone_on_either_side() {
    let input: Seq<_> = [Owned(1), Owned(2)].into_iter().collect();
    let output = input.map(|x| Owned(x.0 + 1));
    assert_eq!(output.iter().map(|x| x.0).collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(input.iter().map(|x| x.0).collect::<Vec<_>>(), vec![1, 2]);
}
