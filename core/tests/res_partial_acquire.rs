//! Leak guard: a composed resource whose later acquisition, or one of whose
//! releases, panics must still release every resource already acquired, in
//! reverse acquisition order.

#![cfg(all(feature = "async", feature = "linear"))]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

type Log = Arc<Mutex<Vec<&'static str>>>;

fn log() -> Log {
    Arc::new(Mutex::new(Vec::new()))
}

fn push(log: &Log, event: &'static str) {
    log.lock().unwrap().push(event);
}

fn events(log: &Log) -> Vec<&'static str> {
    log.lock().unwrap().clone()
}

mod async_core_res {
    use super::*;
    use ordofp_core::async_core::res::{Res, zip_all_res, zip_res};

    fn tracked(log: &Log, name: &'static str, release: &'static str) -> Res<&'static str> {
        let rel = Arc::clone(log);
        Res::make(move || name, move |_| push(&rel, release))
    }

    fn exploding() -> Res<&'static str> {
        Res::make(|| panic!("acquire failed"), |_| {})
    }

    #[test]
    fn zip_res_releases_first_when_second_acquire_panics() {
        let log = log();
        let zipped = zip_res(tracked(&log, "a", "release a"), exploding());
        assert!(catch_unwind(AssertUnwindSafe(|| zipped.uti(|_| ()))).is_err());
        assert_eq!(events(&log), ["release a"]);
    }

    #[test]
    fn zip_all_res_releases_acquired_prefix_in_reverse_when_later_acquire_panics() {
        let log = log();
        let all = zip_all_res(vec![
            tracked(&log, "a", "release a"),
            tracked(&log, "b", "release b"),
            exploding(),
            tracked(&log, "d", "release d"),
        ]);
        assert!(catch_unwind(AssertUnwindSafe(|| all.uti(|_| ()))).is_err());
        assert_eq!(events(&log), ["release b", "release a"]);
    }

    fn release_panics(log: &Log, name: &'static str, release: &'static str) -> Res<&'static str> {
        let rel = Arc::clone(log);
        Res::make(
            move || name,
            move |_| {
                push(&rel, release);
                panic!("release failed");
            },
        )
    }

    #[test]
    fn nested_zip_releases_inner_pair_in_reverse_when_outer_acquire_panics() {
        let log = log();
        let inner = zip_res(
            tracked(&log, "a", "release a"),
            tracked(&log, "b", "release b"),
        );
        let outer = zip_res(inner, exploding());
        assert!(catch_unwind(AssertUnwindSafe(|| outer.uti(|_| ()))).is_err());
        assert_eq!(events(&log), ["release b", "release a"]);
    }

    #[test]
    fn zip_res_releases_first_when_second_release_panics() {
        let log = log();
        let zipped = zip_res(
            tracked(&log, "a", "release a"),
            release_panics(&log, "b", "release b"),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| zipped.uti(|_| ()))).is_err());
        assert_eq!(events(&log), ["release b", "release a"]);
    }

    #[test]
    fn zip_all_res_releases_every_member_when_one_release_panics() {
        let log = log();
        let all = zip_all_res(vec![
            tracked(&log, "a", "release a"),
            release_panics(&log, "b", "release b"),
            tracked(&log, "c", "release c"),
        ]);
        assert!(catch_unwind(AssertUnwindSafe(|| all.uti(|_| ()))).is_err());
        assert_eq!(events(&log), ["release c", "release b", "release a"]);
    }

    #[test]
    fn zip_all_res_success_releases_in_reverse() {
        let log = log();
        let all = zip_all_res(vec![
            tracked(&log, "a", "release a"),
            tracked(&log, "b", "release b"),
            tracked(&log, "c", "release c"),
        ]);
        assert_eq!(all.uti(|items| items.concat()), "abc");
        assert_eq!(events(&log), ["release c", "release b", "release a"]);
    }

    #[test]
    fn zip_res_success_releases_in_reverse() {
        let log = log();
        let zipped = zip_res(
            tracked(&log, "a", "release a"),
            tracked(&log, "b", "release b"),
        );
        assert_eq!(zipped.uti(|(a, b)| format!("{a}{b}")), "ab");
        assert_eq!(events(&log), ["release b", "release a"]);
    }
}

mod linear_res {
    use super::*;
    use ordofp_core::linear::Res;

    fn tracked(log: &Log, name: &'static str, release: &'static str) -> Res<(), &'static str> {
        let rel = Arc::clone(log);
        Res::make(move || name, move |_| push(&rel, release))
    }

    #[test]
    fn zip_releases_first_when_second_acquire_panics() {
        let log = log();
        let exploding = Res::<(), &'static str>::make(|| panic!("acquire failed"), |_| {});
        let zipped = tracked(&log, "a", "release a").zip(exploding);
        assert!(catch_unwind(AssertUnwindSafe(|| zipped.use_res(|_| ()))).is_err());
        assert_eq!(events(&log), ["release a"]);
    }

    #[test]
    fn zip_releases_first_when_second_release_panics() {
        let log = log();
        let rel = Arc::clone(&log);
        let panicking = Res::<(), &'static str>::make(
            || "b",
            move |_| {
                push(&rel, "release b");
                panic!("release failed");
            },
        );
        let zipped = tracked(&log, "a", "release a").zip(panicking);
        assert!(catch_unwind(AssertUnwindSafe(|| zipped.use_res(|_| ()))).is_err());
        assert_eq!(events(&log), ["release b", "release a"]);
    }

    #[test]
    fn fmap_releases_through_inverse() {
        let log = log();
        let mapped = tracked(&log, "a", "release a").fmap(|s| s.len(), |_| "a");
        assert_eq!(mapped.use_res(|n| *n), 1);
        assert_eq!(events(&log), ["release a"]);
    }

    #[test]
    fn zip_success_releases_in_reverse() {
        let log = log();
        let zipped = tracked(&log, "a", "release a").zip(tracked(&log, "b", "release b"));
        assert_eq!(zipped.use_res(|(a, b)| format!("{a}{b}")), "ab");
        assert_eq!(events(&log), ["release b", "release a"]);
    }
}
