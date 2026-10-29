//! Regression and coherence checks for public Apply sequencing.

use ordofp_core::typeclasses::Apply;
#[cfg(any(feature = "alloc", feature = "Probatum"))]
use ordofp_core::typeclasses::Functor;
#[cfg(feature = "Probatum")]
use ordofp_core::validated::Probatum;

#[test]
fn option_observes_both_operands() {
    assert_eq!(Some(1).ap_left(None::<i32>), None);
    assert_eq!(None::<i32>.ap_right(Some(2)), None);
}

#[test]
fn result_observes_both_operands_and_prefers_left_error() {
    let ok: Result<i32, &str> = Ok(1);
    let left: Result<i32, &str> = Err("left");
    let right: Result<i32, &str> = Err("right");
    assert_eq!(ok.ap_left(right), right);
    assert_eq!(left.ap_right(ok), left);
    assert_eq!(left.ap_left(right), left);
    assert_eq!(left.ap_right(right), left);
}

#[cfg(feature = "alloc")]
#[test]
fn vec_cartesian_order_and_empty_operands() {
    assert_eq!(vec![1, 2].ap_left(vec![3, 4, 5]), vec![1, 1, 1, 2, 2, 2]);
    assert_eq!(vec![1, 2].ap_right(vec![3, 4, 5]), vec![3, 4, 5, 3, 4, 5]);
    assert!(vec![1].ap_left(Vec::<i32>::new()).is_empty());
    assert!(Vec::<i32>::new().ap_right(vec![1]).is_empty());
    assert!(Vec::<i32>::new().ap_left(vec![1]).is_empty());
    assert!(vec![1].ap_right(Vec::<i32>::new()).is_empty());
}

#[cfg(feature = "Probatum")]
#[test]
fn validation_accumulates_in_left_to_right_order() {
    let left = Probatum::<_, i32>::invalid_many(["l1", "l2"]);
    let right = Probatum::<_, i32>::invalid_many(["r1", "r2"]);
    let expected = Probatum::invalid_many(["l1", "l2", "r1", "r2"]);
    assert_eq!(left.clone().ap_left(right.clone()), expected);
    assert_eq!(left.clone().ap_right(right.clone()), expected);
    assert_eq!(Probatum::Valid(1).ap_left(right.clone()), right);
    assert_eq!(left.clone().ap_right(Probatum::Valid(2)), left);
}

#[test]
fn derived_operations_agree_with_apply() {
    for left in [None, Some(1)] {
        for right in [None, Some(2)] {
            assert_eq!(left.ap_left(right), right.apply(left.map(|a| move |_| a)));
            assert_eq!(left.ap_right(right), right.apply(left.map(|_| |b| b)));
        }
    }
    #[cfg(feature = "alloc")]
    for left in [vec![], vec![1], vec![1, 2]] {
        for right in [vec![], vec![3], vec![3, 4]] {
            assert_eq!(
                left.clone().ap_left(right.clone()),
                right
                    .clone()
                    .apply(Functor::map(left.clone(), |a| move |_| a))
            );
            assert_eq!(
                left.clone().ap_right(right.clone()),
                right.clone().apply(Functor::map(left.clone(), |_| |b| b))
            );
        }
    }
}

#[cfg(feature = "Probatum")]
#[test]
fn non_clone_values_and_errors_follow_existing_contracts() {
    #[derive(Debug, PartialEq)]
    struct Owned(i32);
    assert_eq!(Some(Owned(1)).ap_right(Some(2)), Some(2));
    assert_eq!(Some(1).ap_left(Some(Owned(2))), Some(1));
    let left: Result<i32, Owned> = Err(Owned(3));
    assert_eq!(left.ap_left(Ok(Owned(2))), Err(Owned(3)));
    assert_eq!(
        Probatum::<&str, _>::Valid(Owned(1)).ap_right(Probatum::Valid(2)),
        Probatum::Valid(2)
    );
    assert_eq!(
        Probatum::<Owned, _>::Valid(1).ap_left(Probatum::Valid(Owned(2))),
        Probatum::Valid(1)
    );
}

#[cfg(feature = "Probatum")]
#[test]
fn sequencing_coherence_properties() {
    fn property(xs: Vec<i8>, ys: Vec<i8>, fail_left: bool, fail_right: bool) -> bool {
        let xs: Vec<_> = xs.into_iter().take(16).collect();
        let ys: Vec<_> = ys.into_iter().take(16).collect();
        let vec_ok = xs.clone().ap_left(ys.clone())
            == ys.clone().apply(Functor::map(xs.clone(), |a| move |_| a))
            && xs.clone().ap_right(ys.clone()) == ys.apply(Functor::map(xs, |_| |b| b));
        let l: Result<i8, i8> = if fail_left { Err(1) } else { Ok(3) };
        let r = if fail_right { Err(2) } else { Ok(4) };
        let result_ok = l.ap_left(r) == r.apply(l.map(|a| move |_| a))
            && l.ap_right(r) == r.apply(l.map(|_| |b| b));
        let l = Probatum::from(l);
        let r = Probatum::from(r);
        vec_ok
            && result_ok
            && l.clone().ap_left(r.clone()) == r.clone().apply(l.clone().map(|a| move |_| a))
            && l.clone().ap_right(r.clone()) == r.apply(l.map(|_| |b| b))
    }
    quickcheck::quickcheck(property as fn(Vec<i8>, Vec<i8>, bool, bool) -> bool);
}
