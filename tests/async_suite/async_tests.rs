//! Async type class tests for `OrdoFP` 2.0
//!
//! These tests verify the async typeclass implementations.
//! Uses a minimal executor for running async tests.

#![cfg(feature = "async")]

use ordofp::async_core::Futurus;
use ordofp::async_core::{ApplicatioAsync, FunctorAsync, MonadAsync};
use ordofp::async_core::{ApplicatioAsyncMut, FunctorAsyncMut, MonadAsyncMut};

// ============================================================================
// Simple executor for testing
// ============================================================================

/// Regression suite for the formerly first-element-only Vec async impls:
/// `fmap_async` / `flat_map_async` must process EVERY
/// element (functor identity law), and `map2_async` pairs elements zip-wise
/// up to the shorter length.
mod vec_async_laws {
    use super::*;

    #[test]
    fn vec_fmap_async_maps_all_elements_identity_law() {
        let mapped = block_on(vec![1, 2, 3].fmap_async(|x| async move { x }));
        assert_eq!(mapped, vec![1, 2, 3]);

        let doubled = block_on(vec![1, 2, 3].fmap_async(|x| async move { x * 2 }));
        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn vec_flat_map_async_processes_all_elements() {
        let result = block_on(vec![1, 2, 3].flat_map_async(|x| async move { vec![x, x * 10] }));
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn vec_map2_async_zips_to_shorter_length() {
        let result = block_on(vec![1, 2, 3].map2_async(vec![10, 20], |a, b| async move { a + b }));
        assert_eq!(result, vec![11, 22]);

        let empty =
            block_on(Vec::<i32>::new().map2_async(vec![1], |a, b: i32| async move { a + b }));
        assert_eq!(empty, Vec::<i32>::new());
    }
}

/// A minimal executor that blocks on a future.
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    // Create a no-op waker
    fn noop_raw_waker() -> RawWaker {
        fn noop(_: *const ()) {}
        fn clone_waker(_: *const ()) -> RawWaker {
            noop_raw_waker()
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, noop, noop, noop);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }

    // SAFETY: `noop_raw_waker()` constructs a `RawWaker` whose vtable contains
    // valid, non-null function pointers: `clone_waker` returns another correctly
    // formed `RawWaker`, and the other three slots (wake, wake_by_ref, drop) are
    // no-ops that accept any data pointer including null. The null data pointer is
    // consistent across all vtable functions, satisfying the invariant that the
    // data pointer semantics are defined entirely by the vtable. All requirements
    // of `Waker::from_raw` are therefore met.
    let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => {
                // For simple cases without real async ops, this shouldn't happen
                // In a real executor, we'd park the thread here
                std::thread::yield_now();
            }
        }
    }
}

// ============================================================================
// FunctorAsync tests for Option
// ============================================================================

#[test]
fn test_option_fmap_async_some() {
    let result = block_on(async {
        let opt = Some(5);
        opt.fmap_async(|x| async move { x * 2 }).await
    });
    assert_eq!(result, Some(10));
}

#[test]
fn test_option_fmap_async_none() {
    let result = block_on(async {
        let opt: Option<i32> = None;
        opt.fmap_async(|x| async move { x * 2 }).await
    });
    assert_eq!(result, None);
}

#[test]
fn test_option_map_const_async() {
    let result = block_on(async {
        let opt = Some(5);
        opt.map_const_async(async { "constant" }).await
    });
    assert_eq!(result, Some("constant"));
}

#[test]
fn test_option_void_async() {
    let result = block_on(async {
        let opt = Some(42);
        opt.void_async().await
    });
    assert_eq!(result, Some(()));
}

// ============================================================================
// FunctorAsync tests for Result
// ============================================================================

#[test]
fn test_result_fmap_async_ok() {
    let result = block_on(async {
        let res: Result<i32, &str> = Ok(5);
        res.fmap_async(|x| async move { x * 2 }).await
    });
    assert_eq!(result, Ok(10));
}

#[test]
fn test_result_fmap_async_err() {
    let result = block_on(async {
        let res: Result<i32, &str> = Err("error");
        res.fmap_async(|x| async move { x * 2 }).await
    });
    assert_eq!(result, Err("error"));
}

// ============================================================================
// FunctorAsync tests for Vec
// ============================================================================

#[test]
fn test_vec_fmap_async_mut() {
    let result = block_on(async {
        let vec = vec![1, 2, 3];
        vec.fmap_async_mut(|x| async move { x * 2 }).await
    });
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn test_vec_fmap_async_empty() {
    let result = block_on(async {
        let vec: Vec<i32> = vec![];
        vec.fmap_async_mut(|x| async move { x * 2 }).await
    });
    assert_eq!(result, Vec::<i32>::new());
}

// ============================================================================
// ApplicatioAsync tests for Option
// ============================================================================

#[test]
fn test_option_pure_async() {
    let result = block_on(async { Option::<i32>::pure_async(42).await });
    assert_eq!(result, Some(42));
}

#[test]
fn test_option_map2_async() {
    let result = block_on(async {
        let a = Some(5);
        let b = Some(10);
        a.map2_async(b, |x, y| async move { x + y }).await
    });
    assert_eq!(result, Some(15));
}

#[test]
fn test_option_map2_async_first_none() {
    let result = block_on(async {
        let a: Option<i32> = None;
        let b = Some(10);
        a.map2_async(b, |x, y| async move { x + y }).await
    });
    assert_eq!(result, None);
}

#[test]
fn test_option_map2_async_second_none() {
    let result = block_on(async {
        let a = Some(5);
        let b: Option<i32> = None;
        a.map2_async(b, |x, y| async move { x + y }).await
    });
    assert_eq!(result, None);
}

// ============================================================================
// ApplicatioAsync tests for Result
// ============================================================================

#[test]
fn test_result_pure_async() {
    let result = block_on(async { Result::<i32, ()>::pure_async(42).await });
    assert_eq!(result, Ok(42));
}

#[test]
fn test_result_map2_async() {
    let result = block_on(async {
        let a: Result<i32, &str> = Ok(5);
        let b: Result<i32, &str> = Ok(10);
        a.map2_async(b, |x, y| async move { x + y }).await
    });
    assert_eq!(result, Ok(15));
}

#[test]
fn test_result_map2_async_first_err() {
    let result = block_on(async {
        let a: Result<i32, &str> = Err("first error");
        let b: Result<i32, &str> = Ok(10);
        a.map2_async(b, |x, y| async move { x + y }).await
    });
    assert_eq!(result, Err("first error"));
}

// ============================================================================
// ApplicatioAsync tests for Vec
// ============================================================================

#[test]
fn test_vec_pure_async() {
    let result = block_on(async { Vec::<i32>::pure_async(42).await });
    assert_eq!(result, vec![42]);
}

#[test]
fn test_vec_map2_async_mut() {
    // Vec Applicative uses cartesian product semantics
    let result = block_on(async {
        let a = vec![1, 2];
        let b = vec![10, 20];
        a.map2_async_mut(b, |x, y| async move { x + y }).await
    });
    // Cartesian product: [(1,10), (1,20), (2,10), (2,20)] -> [11, 21, 12, 22]
    assert_eq!(result, vec![11, 21, 12, 22]);
}

// ============================================================================
// MonadAsync tests for Option
// ============================================================================

#[test]
fn test_option_flat_map_async_some_to_some() {
    let result = block_on(async {
        let opt = Some(5);
        opt.flat_map_async(|x| async move { Some(x * 2) }).await
    });
    assert_eq!(result, Some(10));
}

#[test]
fn test_option_flat_map_async_some_to_none() {
    let result = block_on(async {
        let opt = Some(5);
        opt.flat_map_async(|_| async move { None::<i32> }).await
    });
    assert_eq!(result, None);
}

#[test]
fn test_option_flat_map_async_none() {
    let result = block_on(async {
        let opt: Option<i32> = None;
        opt.flat_map_async(|x| async move { Some(x * 2) }).await
    });
    assert_eq!(result, None);
}

#[test]
fn test_option_bind_async() {
    let result = block_on(async {
        let opt = Some(5);
        opt.bind_async(|x| async move { Some(x + 1) }).await
    });
    assert_eq!(result, Some(6));
}

#[test]
fn test_option_and_then_async() {
    let result = block_on(async {
        let opt = Some(5);
        opt.and_then_async(|x| async move { Some(x * 3) }).await
    });
    assert_eq!(result, Some(15));
}

// ============================================================================
// MonadAsync tests for Result
// ============================================================================

#[test]
fn test_result_flat_map_async_ok() {
    let result = block_on(async {
        let res: Result<i32, &str> = Ok(5);
        res.flat_map_async(|x| async move { Ok::<i32, &str>(x * 2) })
            .await
    });
    assert_eq!(result, Ok(10));
}

#[test]
fn test_result_flat_map_async_ok_to_err() {
    let result = block_on(async {
        let res: Result<i32, &str> = Ok(5);
        res.flat_map_async(|_| async move { Err::<i32, &str>("inner error") })
            .await
    });
    assert_eq!(result, Err("inner error"));
}

#[test]
fn test_result_flat_map_async_err() {
    let result = block_on(async {
        let res: Result<i32, &str> = Err("outer error");
        res.flat_map_async(|x| async move { Ok::<i32, &str>(x * 2) })
            .await
    });
    assert_eq!(result, Err("outer error"));
}

// ============================================================================
// MonadAsync tests for Vec
// ============================================================================

#[test]
fn test_vec_flat_map_async_mut() {
    let result = block_on(async {
        let vec = vec![1, 2, 3];
        vec.flat_map_async_mut(|x| async move { vec![x, x * 10] })
            .await
    });
    assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
}

#[test]
fn test_vec_flat_map_async_mut_empty() {
    let result = block_on(async {
        let vec: Vec<i32> = vec![];
        vec.flat_map_async_mut(|x| async move { vec![x, x * 10] })
            .await
    });
    assert_eq!(result, Vec::<i32>::new());
}

// ============================================================================
// Futurus tests
// ============================================================================

#[test]
fn test_futurus_purus() {
    let result = block_on(Futurus::purus(42));
    assert_eq!(result, 42);
}

#[test]
fn test_futurus_fmap() {
    let result = block_on(Futurus::purus(5).fmap(|x| x * 2));
    assert_eq!(result, 10);
}

#[test]
fn test_futurus_flat_map() {
    let result = block_on(Futurus::purus(5).flat_map(|x| Futurus::purus(x * 2)));
    assert_eq!(result, 10);
}

#[test]
fn test_futurus_chain() {
    let result = block_on(
        Futurus::purus(5)
            .fmap(|x| x * 2)
            .flat_map(|x| Futurus::purus(x + 1))
            .fmap(|x| x * 3),
    );
    assert_eq!(result, 33); // ((5 * 2) + 1) * 3 = 33
}

#[test]
fn test_futurus_map2() {
    let result = block_on(Futurus::purus(5).map2(Futurus::purus(10), |a, b| a + b));
    assert_eq!(result, 15);
}

#[test]
fn test_futurus_then() {
    let result = block_on(Futurus::purus(5).then(Futurus::purus(42)));
    assert_eq!(result, 42);
}

#[test]
fn test_futurus_skip() {
    let result = block_on(Futurus::purus(5).skip(Futurus::purus(42)));
    assert_eq!(result, 5);
}

#[test]
fn test_futurus_void() {
    assert_eq!(block_on(Futurus::purus(42).void()), ());
}

#[test]
fn test_futurus_flatten() {
    let result = block_on(Futurus::purus(Futurus::purus(42)).flatten());
    assert_eq!(result, 42);
}

#[test]
fn test_futurus_from() {
    let result = block_on(Futurus::from(42));
    assert_eq!(result, 42);
}

#[test]
fn test_futurus_default() {
    let result: i32 = block_on(Futurus::<i32>::default());
    assert_eq!(result, 0);
}

// ============================================================================
// Async Law Tests
// ============================================================================

#[test]
fn test_option_functor_async_identity_law() {
    // fa.fmap_async(|x| async { x }).await == fa
    let result = block_on(async {
        let opt = Some(42);
        opt.fmap_async(|x| async move { x }).await
    });
    assert_eq!(result, Some(42));
}

#[test]
fn test_option_functor_async_composition_law() {
    // fa.fmap_async(f).await.fmap_async(g).await == fa.fmap_async(|x| async { g(f(x).await).await }).await
    let f = |x: i32| async move { x + 1 };
    let g = |x: i32| async move { x * 2 };

    let opt = Some(5);

    // Left side: apply f then g
    let left = block_on(async {
        let step1 = opt.fmap_async(f).await;
        step1.fmap_async(g).await
    });

    // Right side: compose f and g
    let right = block_on(async {
        let opt = Some(5);
        opt.fmap_async(|x| async move { g(f(x).await).await }).await
    });

    assert_eq!(left, right);
    assert_eq!(left, Some(12)); // (5 + 1) * 2 = 12
}

#[test]
fn test_option_monad_async_left_identity_law() {
    // pure(a).flat_map_async(f) == f(a)
    let a = 5;
    let f = |x: i32| async move { Some(x * 2) };

    let left = block_on(async { Some(a).flat_map_async(f).await });

    let right = block_on(async { f(a).await });

    assert_eq!(left, right);
    assert_eq!(left, Some(10));
}

#[test]
fn test_option_monad_async_right_identity_law() {
    // m.flat_map_async(|x| async { pure(x) }) == m
    let m = Some(42);

    let left = block_on(async {
        let m = Some(42);
        m.flat_map_async(|x| async move { Some(x) }).await
    });

    assert_eq!(left, m);
}

#[test]
fn test_option_monad_async_associativity_law() {
    // m.flat_map_async(f).flat_map_async(g) == m.flat_map_async(|x| f(x).flat_map_async(g))
    let f = |x: i32| async move { Some(x + 1) };
    let g = |x: i32| async move { Some(x * 2) };

    let left = block_on(async {
        let step1 = Some(5).flat_map_async(f).await;
        step1.flat_map_async(g).await
    });

    let right = block_on(async {
        Some(5)
            .flat_map_async(|x| async move { f(x).await.flat_map_async(g).await })
            .await
    });

    assert_eq!(left, right);
    assert_eq!(left, Some(12)); // (5 + 1) * 2 = 12
}

#[test]
fn test_futurus_monad_left_identity_law() {
    // pure(a).flat_map(f) == f(a)
    let a = 5;
    let f = |x: i32| Futurus::purus(x * 2);

    let left = block_on(Futurus::purus(a).flat_map(f));
    let right = block_on(f(a));

    assert_eq!(left, right);
    assert_eq!(left, 10);
}

#[test]
fn test_futurus_monad_right_identity_law() {
    // m.flat_map(pure) == m
    let m = Futurus::purus(42);

    let left = block_on(m.flat_map(Futurus::purus));
    let right = block_on(Futurus::purus(42));

    assert_eq!(left, right);
}

#[test]
fn test_futurus_monad_associativity_law() {
    // m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
    let f = |x: i32| Futurus::purus(x + 1);
    let g = |x: i32| Futurus::purus(x * 2);

    let left = block_on(Futurus::purus(5).flat_map(f).flat_map(g));

    let right = block_on(Futurus::purus(5).flat_map(move |x| f(x).flat_map(g)));

    assert_eq!(left, right);
    assert_eq!(left, 12); // (5 + 1) * 2 = 12
}

// ============================================================================
// Async Transformer Tests
// ============================================================================

use ordofp::transformers::async_transforms::{
    EitherTAsync, LectorAsync, OptionTAsync, ScriptorAsync, StatusAsync,
};

// ----------------------------------------------------------------------------
// OptionTAsync tests
// ----------------------------------------------------------------------------

#[test]
fn test_option_t_async_some() {
    let result = block_on(async { OptionTAsync::some(42).run().await });
    assert_eq!(result, Some(42));
}

#[test]
fn test_option_t_async_none() {
    let result = block_on(async { OptionTAsync::<i32>::none().run().await });
    assert_eq!(result, None);
}

#[test]
fn test_option_t_async_fmap() {
    let result = block_on(async { OptionTAsync::some(21).fmap(|x| x * 2).run().await });
    assert_eq!(result, Some(42));
}

#[test]
fn test_option_t_async_flat_map() {
    let result = block_on(async {
        OptionTAsync::some(5)
            .flat_map(|x| {
                if x > 0 {
                    OptionTAsync::some(x * 2)
                } else {
                    OptionTAsync::none()
                }
            })
            .run()
            .await
    });
    assert_eq!(result, Some(10));
}

#[test]
fn test_option_t_async_flat_map_to_none() {
    let result = block_on(async {
        OptionTAsync::some(-5)
            .flat_map(|x| {
                if x > 0 {
                    OptionTAsync::some(x * 2)
                } else {
                    OptionTAsync::none()
                }
            })
            .run()
            .await
    });
    assert_eq!(result, None);
}

#[test]
fn test_option_t_async_filter() {
    let result = block_on(async { OptionTAsync::some(10).filter(|x| *x > 5).run().await });
    assert_eq!(result, Some(10));

    let result2 = block_on(async { OptionTAsync::some(3).filter(|x| *x > 5).run().await });
    assert_eq!(result2, None);
}

#[test]
fn test_option_t_async_or_else() {
    let result = block_on(async {
        OptionTAsync::<i32>::none()
            .or_else(|| OptionTAsync::some(42))
            .run()
            .await
    });
    assert_eq!(result, Some(42));
}

#[test]
fn test_option_t_async_unwrap_or() {
    let result = block_on(async { OptionTAsync::<i32>::none().unwrap_or(42).await });
    assert_eq!(result, 42);
}

// ----------------------------------------------------------------------------
// EitherTAsync tests
// ----------------------------------------------------------------------------

#[test]
fn test_either_t_async_right() {
    let result = block_on(async { EitherTAsync::<String, i32>::right(42).run().await });
    assert_eq!(result, Ok(42));
}

#[test]
fn test_either_t_async_left() {
    let result = block_on(async {
        EitherTAsync::<String, i32>::left("error".to_string())
            .run()
            .await
    });
    assert_eq!(result, Err("error".to_string()));
}

#[test]
fn test_either_t_async_fmap() {
    let result = block_on(async {
        EitherTAsync::<String, i32>::right(21)
            .fmap(|x| x * 2)
            .run()
            .await
    });
    assert_eq!(result, Ok(42));
}

#[test]
fn test_either_t_async_map_err() {
    let result = block_on(async {
        EitherTAsync::<i32, String>::left(404)
            .map_err(|code| format!("Error {code}"))
            .run()
            .await
    });
    assert_eq!(result, Err("Error 404".to_string()));
}

#[test]
fn test_either_t_async_flat_map() {
    let result = block_on(async {
        EitherTAsync::<String, i32>::right(5)
            .flat_map(|x| {
                if x > 0 {
                    EitherTAsync::right(x * 2)
                } else {
                    EitherTAsync::left("negative".to_string())
                }
            })
            .run()
            .await
    });
    assert_eq!(result, Ok(10));
}

#[test]
fn test_either_t_async_flat_map_to_err() {
    let result = block_on(async {
        EitherTAsync::<String, i32>::right(-5)
            .flat_map(|x| {
                if x > 0 {
                    EitherTAsync::right(x * 2)
                } else {
                    EitherTAsync::left("negative".to_string())
                }
            })
            .run()
            .await
    });
    assert_eq!(result, Err("negative".to_string()));
}

#[test]
fn test_either_t_async_handle_error() {
    let result = block_on(async {
        EitherTAsync::<String, i32>::left("error".to_string())
            .handle_error(|_| EitherTAsync::right(0))
            .run()
            .await
    });
    assert_eq!(result, Ok(0));
}

#[test]
fn test_either_t_async_ensure() {
    let result = block_on(async {
        EitherTAsync::<String, i32>::right(5)
            .ensure(|x| *x > 0, || "must be positive".to_string())
            .run()
            .await
    });
    assert_eq!(result, Ok(5));

    let result2 = block_on(async {
        EitherTAsync::<String, i32>::right(-5)
            .ensure(|x| *x > 0, || "must be positive".to_string())
            .run()
            .await
    });
    assert_eq!(result2, Err("must be positive".to_string()));
}

#[test]
fn test_either_t_async_to_option() {
    let result = block_on(async {
        EitherTAsync::<String, i32>::right(42)
            .to_option()
            .run()
            .await
    });
    assert_eq!(result, Some(42));

    let result2 = block_on(async {
        EitherTAsync::<String, i32>::left("error".to_string())
            .to_option()
            .run()
            .await
    });
    assert_eq!(result2, None);
}

// ----------------------------------------------------------------------------
// ScriptorAsync tests
// ----------------------------------------------------------------------------

#[test]
fn test_scriptor_async_purus() {
    let result = block_on(async { ScriptorAsync::<Vec<String>, i32>::purus(42).run().await });
    assert_eq!(result, (vec![], 42));
}

#[test]
fn test_scriptor_async_tell() {
    let result = block_on(async {
        ScriptorAsync::<Vec<String>, ()>::tell(vec!["log".to_string()])
            .run()
            .await
    });
    assert_eq!(result, (vec!["log".to_string()], ()));
}

#[test]
fn test_scriptor_async_fmap() {
    let result = block_on(async {
        ScriptorAsync::<Vec<String>, i32>::purus(21)
            .fmap(|x| x * 2)
            .run()
            .await
    });
    assert_eq!(result, (vec![], 42));
}

#[test]
fn test_scriptor_async_flat_map() {
    let result = block_on(async {
        ScriptorAsync::<Vec<String>, ()>::tell(vec!["first".to_string()])
            .then(ScriptorAsync::purus(5))
            .flat_map(|x| {
                ScriptorAsync::<Vec<String>, ()>::tell(vec!["second".to_string()])
                    .then(ScriptorAsync::purus(x * 2))
            })
            .run()
            .await
    });
    assert_eq!(
        result,
        (vec!["first".to_string(), "second".to_string()], 10)
    );
}

#[test]
fn test_scriptor_async_exec() {
    let result = block_on(async {
        ScriptorAsync::<Vec<String>, ()>::tell(vec!["log".to_string()])
            .then(ScriptorAsync::purus(42))
            .exec()
            .await
    });
    assert_eq!(result, vec!["log".to_string()]);
}

#[test]
fn test_scriptor_async_eval() {
    let result = block_on(async {
        ScriptorAsync::<Vec<String>, ()>::tell(vec!["log".to_string()])
            .then(ScriptorAsync::purus(42))
            .eval()
            .await
    });
    assert_eq!(result, 42);
}

// ----------------------------------------------------------------------------
// LectorAsync tests
// ----------------------------------------------------------------------------

#[test]
fn test_lector_async_purus() {
    let result = block_on(async {
        LectorAsync::<String, i32>::purus(42)
            .run("ignored".to_string())
            .await
    });
    assert_eq!(result, 42);
}

#[test]
fn test_lector_async_ask() {
    let result = block_on(async { LectorAsync::<i32, i32>::ask().run(42).await });
    assert_eq!(result, 42);
}

#[test]
fn test_lector_async_asks() {
    let result = block_on(async { LectorAsync::<i32, i32>::asks(|x| x * 2).run(21).await });
    assert_eq!(result, 42);
}

#[test]
fn test_lector_async_fmap() {
    let result = block_on(async {
        LectorAsync::new(|x: i32| async move { x })
            .fmap(|x| x * 2)
            .run(21)
            .await
    });
    assert_eq!(result, 42);
}

#[test]
fn test_lector_async_flat_map() {
    let result = block_on(async {
        LectorAsync::new(|x: i32| async move { x })
            .flat_map(|val| LectorAsync::new(move |env: i32| async move { val + env }))
            .run(10)
            .await
    });
    // First reader returns 10 (env), then flat_map uses 10 + 10 = 20
    assert_eq!(result, 20);
}

#[test]
fn test_lector_async_local() {
    let result = block_on(async {
        // A reader that expects i32
        let reader: LectorAsync<i32, i32> = LectorAsync::new(|x: i32| async move { x * 2 });
        // Adapt it to accept String
        let adapted = reader.local(|s: String| s.len() as i32);
        adapted.run("hello".to_string()).await
    });
    // "hello".len() = 5, 5 * 2 = 10
    assert_eq!(result, 10);
}

// ----------------------------------------------------------------------------
// StatusAsync tests
// ----------------------------------------------------------------------------

#[test]
fn test_status_async_purus() {
    let result = block_on(async {
        StatusAsync::<String, i32>::purus(42)
            .run("initial".to_string())
            .await
    });
    assert_eq!(result, ("initial".to_string(), 42));
}

#[test]
fn test_status_async_get() {
    let result = block_on(async { StatusAsync::<i32, i32>::get().run(42).await });
    assert_eq!(result, (42, 42));
}

#[test]
fn test_status_async_put() {
    let result = block_on(async { StatusAsync::<i32, ()>::put(100).run(0).await });
    assert_eq!(result, (100, ()));
}

#[test]
fn test_status_async_modify() {
    let result = block_on(async { StatusAsync::<i32, ()>::modify(|s| s + 1).run(0).await });
    assert_eq!(result, (1, ()));
}

#[test]
fn test_status_async_gets() {
    let result = block_on(async {
        StatusAsync::<i32, String>::gets(|s| format!("State is {s}"))
            .run(42)
            .await
    });
    assert_eq!(result, (42, "State is 42".to_string()));
}

#[test]
fn test_status_async_fmap() {
    let result = block_on(async { StatusAsync::<i32, i32>::get().fmap(|x| x * 2).run(21).await });
    assert_eq!(result, (21, 42));
}

#[test]
fn test_status_async_flat_map() {
    let result = block_on(async {
        StatusAsync::<i32, ()>::modify(|s| s + 1)
            .flat_map(|()| StatusAsync::<i32, ()>::modify(|s| s * 2))
            .flat_map(|()| StatusAsync::<i32, i32>::get())
            .run(5)
            .await
    });
    // 5 + 1 = 6, 6 * 2 = 12
    assert_eq!(result, (12, 12));
}

#[test]
fn test_status_async_exec() {
    let result = block_on(async { StatusAsync::<i32, ()>::modify(|s| s + 10).exec(0).await });
    assert_eq!(result, 10);
}

#[test]
fn test_status_async_eval() {
    let result = block_on(async { StatusAsync::<i32, i32>::get().eval(42).await });
    assert_eq!(result, 42);
}

// ============================================================================
// Flumen (Stream) Tests
// ============================================================================

use ordofp::async_core::Flumen;

#[test]
fn test_flumen_from_iter() {
    let result = block_on(async { Flumen::from_iterator(vec![1, 2, 3]).collect_vec().await });
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_flumen_once() {
    let result = block_on(async { Flumen::once(42).collect_vec().await });
    assert_eq!(result, vec![42]);
}

#[test]
fn test_flumen_empty() {
    let result = block_on(async { Flumen::<i32>::empty().collect_vec().await });
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_flumen_purus() {
    let result = block_on(async { Flumen::purus("hello").collect_vec().await });
    assert_eq!(result, vec!["hello"]);
}

#[test]
fn test_flumen_fmap() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3])
            .fmap(|x| x * 2)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn test_flumen_filter() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5, 6])
            .filter(|x| x % 2 == 0)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn test_flumen_filter_map() {
    let result = block_on(async {
        Flumen::from_iterator(vec!["1", "two", "3"])
            .filter_map(|s| s.parse::<i32>().ok())
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 3]);
}

#[test]
fn test_flumen_take() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .take(3)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_flumen_skip() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .skip(2)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![3, 4, 5]);
}

#[test]
fn test_flumen_take_while() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .take_while(|x| *x < 4)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_flumen_skip_while() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .skip_while(|x| *x < 3)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![3, 4, 5]);
}

#[test]
fn test_flumen_chain() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2])
            .chain(Flumen::from_iterator(vec![3, 4]))
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn test_flumen_zip() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3])
            .zip(Flumen::from_iterator(vec!["a", "b", "c"]))
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
}

#[test]
fn test_flumen_enumerate() {
    let result = block_on(async {
        Flumen::from_iterator(vec!["a", "b", "c"])
            .enumerate()
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![(0, "a"), (1, "b"), (2, "c")]);
}

#[test]
fn test_flumen_fold() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .fold(0, |acc, x| acc + x)
            .await
    });
    assert_eq!(result, 15);
}

#[test]
fn test_flumen_reduce() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .reduce(|a, b| a + b)
            .await
    });
    assert_eq!(result, Some(15));
}

#[test]
fn test_flumen_reduce_empty() {
    let result = block_on(async { Flumen::<i32>::empty().reduce(|a, b| a + b).await });
    assert_eq!(result, None);
}

#[test]
fn test_flumen_first() {
    let result = block_on(async { Flumen::from_iterator(vec![1, 2, 3]).first().await });
    assert_eq!(result, Some(1));
}

#[test]
fn test_flumen_first_empty() {
    let result = block_on(async { Flumen::<i32>::empty().first().await });
    assert_eq!(result, None);
}

#[test]
fn test_flumen_last() {
    let result = block_on(async { Flumen::from_iterator(vec![1, 2, 3]).last().await });
    assert_eq!(result, Some(3));
}

#[test]
fn test_flumen_count() {
    let result = block_on(async { Flumen::from_iterator(vec![1, 2, 3, 4, 5]).count().await });
    assert_eq!(result, 5);
}

#[test]
fn test_flumen_any() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .any(|x| *x == 3)
            .await
    });
    assert!(result);

    let result2 = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .any(|x| *x == 10)
            .await
    });
    assert!(!result2);
}

#[test]
fn test_flumen_all() {
    let result = block_on(async {
        Flumen::from_iterator(vec![2, 4, 6, 8])
            .all(|x| *x % 2 == 0)
            .await
    });
    assert!(result);

    let result2 = block_on(async {
        Flumen::from_iterator(vec![2, 4, 5, 8])
            .all(|x| *x % 2 == 0)
            .await
    });
    assert!(!result2);
}

#[test]
fn test_flumen_find() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5])
            .find(|x| x > &3)
            .await
    });
    assert_eq!(result, Some(4));
}

#[test]
fn test_flumen_flat_map() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2])
            .flat_map(|x| Flumen::from_iterator(vec![x, x * 10]))
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 10, 2, 20]);
}

// ============================================================================
// Flumen scan/chunks Tests (P1.1 Lazy Streams)
// ============================================================================

#[test]
fn test_flumen_scan_running_sum() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4])
            .scan(0, |acc, x| acc + x)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 3, 6, 10]);
}

#[test]
fn test_flumen_scan_running_product() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4])
            .scan(1, |acc, x| acc * x)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 2, 6, 24]);
}

#[test]
fn test_flumen_scan_empty() {
    let result = block_on(async {
        Flumen::<i32>::empty()
            .scan(0, |acc, x| acc + x)
            .collect_vec()
            .await
    });
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_flumen_scan_with_bounded() {
    // Take running sum until it exceeds 10
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3, 4, 5, 6])
            .scan_with(0, |acc, x| {
                let new_acc = *acc + x;
                if new_acc > 10 {
                    None
                } else {
                    *acc = new_acc;
                    Some(new_acc)
                }
            })
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 3, 6, 10]);
}

#[test]
fn test_flumen_scan_with_transform() {
    // Accumulate strings but output lengths
    let result = block_on(async {
        Flumen::from_iterator(vec!["a", "bb", "ccc"])
            .scan_with(String::new(), |acc, x| {
                acc.push_str(x);
                Some(acc.len())
            })
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![1, 3, 6]);
}

#[test]
fn test_flumen_chunks_even_division() {
    let result = block_on(async { Flumen::from_iterator(1..=6).chunks(3).collect_vec().await });
    assert_eq!(result, vec![vec![1, 2, 3], vec![4, 5, 6]]);
}

#[test]
fn test_flumen_chunks_uneven_division() {
    let result = block_on(async { Flumen::from_iterator(1..=10).chunks(3).collect_vec().await });
    assert_eq!(
        result,
        vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9], vec![10]]
    );
}

#[test]
fn test_flumen_chunks_size_one() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3])
            .chunks(1)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn test_flumen_chunks_larger_than_stream() {
    let result = block_on(async {
        Flumen::from_iterator(vec![1, 2, 3])
            .chunks(10)
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![vec![1, 2, 3]]);
}

#[test]
fn test_flumen_chunks_empty() {
    let result = block_on(async { Flumen::<i32>::empty().chunks(3).collect_vec().await });
    assert_eq!(result, Vec::<Vec<i32>>::new());
}

#[test]
fn test_flumen_scan_chain_composition() {
    // Test composing scan with other stream operations
    let result = block_on(async {
        Flumen::from_iterator(1..=10)
            .filter(|x| x % 2 == 0) // [2, 4, 6, 8, 10]
            .scan(0, |acc, x| acc + x) // [2, 6, 12, 20, 30]
            .fmap(|x| x * 2) // [4, 12, 24, 40, 60]
            .take(3) // [4, 12, 24]
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![4, 12, 24]);
}

#[test]
fn test_flumen_chunks_chain_composition() {
    // Test composing chunks with other stream operations
    let result = block_on(async {
        Flumen::from_iterator(1..=12)
            .chunks(4) // [[1,2,3,4], [5,6,7,8], [9,10,11,12]]
            .fmap(|chunk| chunk.iter().sum::<i32>()) // [10, 26, 42]
            .collect_vec()
            .await
    });
    assert_eq!(result, vec![10, 26, 42]);
}

// ============================================================================
// TraversableAsync Tests
// ============================================================================

use ordofp::async_core::{OptionTraverseAsync, ResultTraverseAsync, TraversableAsync};
use ordofp::async_core::{map_option_async, map_result_async, traverse_vec_async};

#[test]
fn test_traverse_vec_async() {
    let result = block_on(async { vec![1, 2, 3].traverse_async(|x| async move { x * 2 }).await });
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn test_traverse_vec_async_empty() {
    let result = block_on(async {
        Vec::<i32>::new()
            .traverse_async(|x| async move { x * 2 })
            .await
    });
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_traverse_option_async_some() {
    let result = block_on(async {
        Some(5)
            .traverse_option_async(|x| async move { x * 2 })
            .await
    });
    assert_eq!(result, Some(10));
}

#[test]
fn test_traverse_option_async_none() {
    let result = block_on(async {
        Option::<i32>::None
            .traverse_option_async(|x| async move { x * 2 })
            .await
    });
    assert_eq!(result, None);
}

#[test]
fn test_traverse_result_async_ok() {
    let result = block_on(async {
        Ok::<i32, String>(5)
            .traverse_result_async(|x| async move { x * 2 })
            .await
    });
    assert_eq!(result, Ok(10));
}

#[test]
fn test_traverse_result_async_err() {
    let result = block_on(async {
        Err::<i32, String>("error".to_string())
            .traverse_result_async(|x| async move { x * 2 })
            .await
    });
    assert_eq!(result, Err("error".to_string()));
}

#[test]
fn test_traverse_array_async() {
    let result = block_on(async { [1, 2, 3].traverse_async(|x| async move { x * 2 }).await });
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn test_traverse_vec_async_helper_function() {
    let result = block_on(async {
        traverse_vec_async(vec![1, 2, 3], |x| async move { format!("{x}") }).await
    });
    assert_eq!(
        result,
        vec!["1".to_string(), "2".to_string(), "3".to_string()]
    );
}

#[test]
fn test_map_option_async_helper() {
    let result = block_on(async { map_option_async(Some(5), |x| async move { x * 2 }).await });
    assert_eq!(result, Some(10));
}

#[test]
fn test_map_result_async_helper() {
    let result: Result<i32, String> =
        block_on(async { map_result_async(Ok(5), |x| async move { x * 2 }).await });
    assert_eq!(result, Ok(10));
}

// Test that TraversableAsync to Vec implementation works correctly for Option
#[test]
fn test_option_traverse_async_to_vec_some() {
    let result = block_on(async { Some(42).traverse_async(|x| async move { x }).await });
    assert_eq!(result, vec![42]);
}

#[test]
fn test_option_traverse_async_to_vec_none() {
    let result = block_on(async {
        Option::<i32>::None
            .traverse_async(|x| async move { x })
            .await
    });
    assert_eq!(result, Vec::<i32>::new());
}

// Test that TraversableAsync to Vec implementation works correctly for Result
#[test]
fn test_result_traverse_async_to_vec_ok() {
    let result = block_on(async {
        Ok::<i32, String>(42)
            .traverse_async(|x| async move { x })
            .await
    });
    assert_eq!(result, vec![42]);
}

#[test]
fn test_result_traverse_async_to_vec_err() {
    let result = block_on(async {
        Err::<i32, String>("error".to_string())
            .traverse_async(|x| async move { x })
            .await
    });
    assert_eq!(result, Vec::<i32>::new());
}

// ============================================================================
// Async Macro Tests
// ============================================================================

use ordofp::{chain_async, compose_async, mdo_async, pipe_async};

// ----------------------------------------------------------------------------
// mdo_async! tests
// ----------------------------------------------------------------------------

#[test]
fn test_mdo_async_option_success() {
    let result = mdo_async! {
        let x = bind Some(10);
        let y = bind Some(5);
        Some(x + y)
    };
    assert_eq!(result, Some(15));
}

#[test]
fn test_mdo_async_option_short_circuit() {
    let result = mdo_async! {
        let x = bind Some(10);
        let y = bind None::<i32>;
        Some(x + y)
    };
    assert_eq!(result, None);
}

#[test]
fn test_mdo_async_option_with_pure() {
    let result = mdo_async! {
        let x = bind Some(10);
        let doubled = pure x * 2;
        let y = bind Some(5);
        Some(doubled + y)
    };
    assert_eq!(result, Some(25));
}

#[test]
fn test_mdo_async_result_success() {
    fn parse(s: &str) -> Result<i32, &'static str> {
        s.parse().map_err(|_| "parse error")
    }

    let result: Result<i32, &str> = mdo_async! {
        let x = bind parse("10");
        let y = bind parse("5");
        Ok(x + y)
    };
    assert_eq!(result, Ok(15));
}

#[test]
fn test_mdo_async_result_failure() {
    fn parse(s: &str) -> Result<i32, &'static str> {
        s.parse().map_err(|_| "parse error")
    }

    let result: Result<i32, &str> = mdo_async! {
        let x = bind parse("10");
        let y = bind parse("not_a_number");
        Ok(x + y)
    };
    assert_eq!(result, Err("parse error"));
}

#[test]
fn test_mdo_async_with_await() {
    let result = block_on(async {
        mdo_async! {
            let x = pure 10;
            let y = await async { 5 };
            x + y
        }
    });
    assert_eq!(result, 15);
}

#[test]
fn test_mdo_async_mixed_operations() {
    let result = block_on(async {
        mdo_async! {
            let x = pure 10;
            let doubled = await async { x * 2 };
            let y = pure doubled + 5;
            y
        }
    });
    assert_eq!(result, 25);
}

#[test]
fn test_mdo_async_chained_awaits() {
    async fn fetch_value(x: i32) -> i32 {
        x + 1
    }

    let result = block_on(async {
        mdo_async! {
            let a = await fetch_value(10);
            let b = await fetch_value(a);
            let c = await fetch_value(b);
            c
        }
    });
    // 10 -> 11 -> 12 -> 13
    assert_eq!(result, 13);
}

// ----------------------------------------------------------------------------
// pipe_async! tests
// ----------------------------------------------------------------------------

#[test]
fn test_pipe_async_single_value() {
    let result = block_on(async { pipe_async!(42).await });
    assert_eq!(result, 42);
}

#[test]
fn test_pipe_async_one_function() {
    async fn double(x: i32) -> i32 {
        x * 2
    }

    let result = block_on(async { pipe_async!(21, double).await });
    assert_eq!(result, 42);
}

#[test]
fn test_pipe_async_multiple_functions() {
    async fn add_one(x: i32) -> i32 {
        x + 1
    }
    async fn double(x: i32) -> i32 {
        x * 2
    }
    async fn subtract_three(x: i32) -> i32 {
        x - 3
    }

    let result = block_on(async { pipe_async!(10, add_one, double, subtract_three).await });
    // 10 -> 11 -> 22 -> 19
    assert_eq!(result, 19);
}

#[test]
fn test_pipe_async_type_transformation() {
    async fn to_string(x: i32) -> String {
        x.to_string()
    }
    async fn append_exclaim(s: String) -> String {
        format!("{s}!")
    }
    async fn get_length(s: String) -> usize {
        s.len()
    }

    let result = block_on(async { pipe_async!(42, to_string, append_exclaim, get_length).await });
    // "42" -> "42!" -> 3
    assert_eq!(result, 3);
}

// ----------------------------------------------------------------------------
// compose_async! tests
// ----------------------------------------------------------------------------

#[test]
fn test_compose_async_single_function() {
    async fn double(x: i32) -> i32 {
        x * 2
    }

    let composed = compose_async!(double);
    let result = block_on(async { composed(21).await });
    assert_eq!(result, 42);
}

#[test]
fn test_compose_async_multiple_functions() {
    async fn add_one(x: i32) -> i32 {
        x + 1
    }
    async fn double(x: i32) -> i32 {
        x * 2
    }
    async fn subtract_three(x: i32) -> i32 {
        x - 3
    }

    // compose_async!(f, g, h)(x) = f(g(h(x)))
    let composed = compose_async!(add_one, double, subtract_three);
    let result = block_on(async { composed(10).await });
    // 10 - 3 = 7, 7 * 2 = 14, 14 + 1 = 15
    assert_eq!(result, 15);
}

// ----------------------------------------------------------------------------
// chain_async! tests
// ----------------------------------------------------------------------------

#[test]
fn test_chain_async_single_function() {
    async fn double(x: i32) -> i32 {
        x * 2
    }

    let chained = chain_async!(double);
    let result = block_on(async { chained(21).await });
    assert_eq!(result, 42);
}

#[test]
fn test_chain_async_multiple_functions() {
    async fn add_one(x: i32) -> i32 {
        x + 1
    }
    async fn double(x: i32) -> i32 {
        x * 2
    }
    async fn subtract_three(x: i32) -> i32 {
        x - 3
    }

    // chain_async!(f, g, h)(x) = h(g(f(x))) - left to right order
    let chained = chain_async!(add_one, double, subtract_three);
    let result = block_on(async { chained(10).await });
    // 10 + 1 = 11, 11 * 2 = 22, 22 - 3 = 19
    assert_eq!(result, 19);
}

#[test]
fn test_chain_async_type_transformation() {
    async fn parse(s: &str) -> i32 {
        s.parse().expect("s should be a valid integer string")
    }
    async fn double(x: i32) -> i32 {
        x * 2
    }
    async fn to_string(x: i32) -> String {
        x.to_string()
    }

    let process = chain_async!(parse, double, to_string);
    let result = block_on(async { process("21").await });
    assert_eq!(result, "42");
}
