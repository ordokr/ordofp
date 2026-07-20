//! # Async Monad Laws
//!
//! This module provides property-based laws for testing [`MonadAsync`] implementations.
//!
//! ## Laws
//!
//! 1. **Left Identity**: `pure(a).flat_map_async(f).await == f(a).await`
//! 2. **Right Identity**: `m.flat_map_async(|x| async { pure(x) }).await == m`
//! 3. **Associativity**: `m.flat_map_async(f).await.flat_map_async(g).await == m.flat_map_async(|x| f(x).await.flat_map_async(g)).await`
//!
//! ## Usage
//!
//! ```ignore
//! use ordofp_laws::async_monad_laws;
//!
//! // Test left identity
//! assert!(async_monad_laws::option_left_identity_async(5, |x| async move { Some(x * 2) }).await);
//!
//! // Test right identity
//! assert!(async_monad_laws::option_right_identity_async(Some(42)).await);
//!
//! // Test associativity
//! assert!(async_monad_laws::option_associativity_async(
//!     Some(5),
//!     |x| async move { Some(x + 1) },
//!     |x| async move { Some(x * 2) }
//! ).await);
//! ```

use crate::is_eq::IsEq;
use ordofp::async_core::MonadAsync;
use std::future::Future;

// ==================== Option Laws ====================

/// **Left Identity Law** for Option (async): `pure(a).flat_map_async(f).await == f(a).await`
pub async fn option_left_identity_async<A, B, F, Fut>(a: A, f: F) -> bool
where
    A: Clone + Send + 'static,
    B: Eq + Send + 'static,
    F: Fn(A) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Option<B>> + Send,
{
    let f_clone = f.clone();
    let lhs: Option<B> = MonadAsync::flat_map_async(Some(a.clone()), f).await;
    let rhs: Option<B> = f_clone(a).await;
    lhs == rhs
}

/// **Right Identity Law** for Option (async): `m.flat_map_async(|x| async { Some(x) }).await == m`
pub async fn option_right_identity_async<A>(ma: Option<A>) -> bool
where
    A: Clone + Eq + Send + 'static,
{
    let lhs: Option<A> = MonadAsync::flat_map_async(ma.clone(), |x| async move { Some(x) }).await;
    lhs == ma
}

/// **Associativity Law** for Option (async):
/// `m.flat_map_async(f).await.flat_map_async(g).await == m.flat_map_async(|x| f(x).await.flat_map_async(g)).await`
pub async fn option_associativity_async<A, B, C, F, G, Fut1, Fut2>(
    ma: Option<A>,
    f: F,
    g: G,
) -> bool
where
    A: Clone + Send + 'static,
    B: Clone + Send + 'static,
    C: Eq + Send + 'static,
    F: Fn(A) -> Fut1 + Clone + Send + Sync + 'static,
    G: Fn(B) -> Fut2 + Clone + Send + Sync + 'static,
    Fut1: Future<Output = Option<B>> + Send,
    Fut2: Future<Output = Option<C>> + Send,
{
    let f_clone = f.clone();
    let g_clone = g.clone();

    // LHS: m.flat_map_async(f).flat_map_async(g)
    let intermediate: Option<B> = MonadAsync::flat_map_async(ma.clone(), f).await;
    let lhs: Option<C> = MonadAsync::flat_map_async(intermediate, g).await;

    // RHS: m.flat_map_async(|x| f(x).flat_map_async(g))
    let rhs: Option<C> = MonadAsync::flat_map_async(ma, move |x| {
        let f = f_clone.clone();
        let g = g_clone.clone();
        async move {
            let mb = f(x).await;
            MonadAsync::flat_map_async(mb, g).await
        }
    })
    .await;

    lhs == rhs
}

/// Returns an [`IsEq`] for the Option left identity law (async).
pub async fn option_left_identity_eq_async<A, B, F, Fut>(a: A, f: F) -> IsEq<Option<B>>
where
    A: Clone + Send + 'static,
    B: Send + 'static,
    F: Fn(A) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Option<B>> + Send,
{
    let f_clone = f.clone();
    let lhs = MonadAsync::flat_map_async(Some(a.clone()), f).await;
    let rhs = f_clone(a).await;
    IsEq::equal_under_law(lhs, rhs)
}

/// Returns an [`IsEq`] for the Option right identity law (async).
pub async fn option_right_identity_eq_async<A>(ma: Option<A>) -> IsEq<Option<A>>
where
    A: Clone + Send + 'static,
{
    let lhs = MonadAsync::flat_map_async(ma.clone(), |x| async move { Some(x) }).await;
    IsEq::equal_under_law(lhs, ma)
}

// ==================== Result Laws ====================

/// **Left Identity Law** for Result (async): `pure(a).flat_map_async(f).await == f(a).await`
pub async fn result_left_identity_async<A, B, E, F, Fut>(a: A, f: F) -> bool
where
    A: Clone + Send + 'static,
    B: Eq + Send + 'static,
    E: Clone + Eq + Send + 'static,
    F: Fn(A) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<B, E>> + Send,
{
    let f_clone = f.clone();
    let lhs: Result<B, E> = MonadAsync::flat_map_async(Ok(a.clone()), f).await;
    let rhs: Result<B, E> = f_clone(a).await;
    lhs == rhs
}

/// **Right Identity Law** for Result (async): `m.flat_map_async(|x| async { Ok(x) }).await == m`
pub async fn result_right_identity_async<A, E>(ma: Result<A, E>) -> bool
where
    A: Clone + Eq + Send + 'static,
    E: Clone + Eq + Send + 'static,
{
    let lhs: Result<A, E> = MonadAsync::flat_map_async(ma.clone(), |x| async move { Ok(x) }).await;
    lhs == ma
}

/// **Associativity Law** for Result (async):
pub async fn result_associativity_async<A, B, C, E, F, G, Fut1, Fut2>(
    ma: Result<A, E>,
    f: F,
    g: G,
) -> bool
where
    A: Clone + Send + 'static,
    B: Clone + Send + 'static,
    C: Eq + Send + 'static,
    E: Clone + Eq + Send + 'static,
    F: Fn(A) -> Fut1 + Clone + Send + Sync + 'static,
    G: Fn(B) -> Fut2 + Clone + Send + Sync + 'static,
    Fut1: Future<Output = Result<B, E>> + Send,
    Fut2: Future<Output = Result<C, E>> + Send,
{
    let f_clone = f.clone();
    let g_clone = g.clone();

    // LHS: m.flat_map_async(f).flat_map_async(g)
    let intermediate: Result<B, E> = MonadAsync::flat_map_async(ma.clone(), f).await;
    let lhs: Result<C, E> = MonadAsync::flat_map_async(intermediate, g).await;

    // RHS: m.flat_map_async(|x| f(x).flat_map_async(g))
    let rhs: Result<C, E> = MonadAsync::flat_map_async(ma, move |x| {
        let f = f_clone.clone();
        let g = g_clone.clone();
        async move {
            let mb = f(x).await;
            MonadAsync::flat_map_async(mb, g).await
        }
    })
    .await;

    lhs == rhs
}

/// Returns an [`IsEq`] for the Result left identity law (async).
pub async fn result_left_identity_eq_async<A, B, E, F, Fut>(a: A, f: F) -> IsEq<Result<B, E>>
where
    A: Clone + Send + 'static,
    B: Send + 'static,
    E: Clone + Send + 'static,
    F: Fn(A) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<B, E>> + Send,
{
    let f_clone = f.clone();
    let lhs = MonadAsync::flat_map_async(Ok(a.clone()), f).await;
    let rhs = f_clone(a).await;
    IsEq::equal_under_law(lhs, rhs)
}

// ==================== Vec Laws ====================
//
// Note: Vec's MonadAsync implementation uses FnOnce. For full Vec monad operations,
// use MonadAsyncMut with flat_map_async_mut. These laws use MonadAsyncMut.

use ordofp::async_core::MonadAsyncMut;

/// **Left Identity Law** for Vec (async): `pure(a).flat_map_async_mut(f).await == f(a).await`
///
/// Uses `MonadAsyncMut` to properly `flat_map` over all elements.
pub async fn vec_left_identity_async<A, B, F, Fut>(a: A, mut f: F) -> bool
where
    A: Clone + Send + 'static,
    B: Eq + Send + 'static,
    F: FnMut(A) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Vec<B>> + Send,
{
    let mut f_clone = f.clone();
    let lhs: Vec<B> = MonadAsyncMut::flat_map_async_mut(vec![a.clone()], &mut f).await;
    let rhs: Vec<B> = f_clone(a).await;
    lhs == rhs
}

/// **Right Identity Law** for Vec (async): `m.flat_map_async_mut(|x| async { vec![x] }).await == m`
///
/// Uses `MonadAsyncMut` to properly `flat_map` over all elements.
pub async fn vec_right_identity_async<A>(ma: Vec<A>) -> bool
where
    A: Clone + Eq + Send + 'static,
{
    let lhs: Vec<A> =
        MonadAsyncMut::flat_map_async_mut(ma.clone(), |x| async move { vec![x] }).await;
    lhs == ma
}

/// **Associativity Law** for Vec (async):
///
/// Uses `MonadAsyncMut` to properly `flat_map` over all elements.
/// Note: Due to `FnMut` closure semantics, this test verifies the law
/// for specific function instances rather than the general case.
pub async fn vec_associativity_async<A, B, C, F, G, Fut1, Fut2>(
    ma: Vec<A>,
    mut f: F,
    mut g: G,
) -> bool
where
    A: Clone + Send + 'static,
    B: Clone + Send + 'static,
    C: Eq + Send + 'static,
    F: FnMut(A) -> Fut1 + Clone + Send + 'static,
    G: FnMut(B) -> Fut2 + Clone + Send + 'static,
    Fut1: Future<Output = Vec<B>> + Send,
    Fut2: Future<Output = Vec<C>> + Send,
{
    // LHS: m.flat_map_async_mut(f).flat_map_async_mut(g)
    let intermediate: Vec<B> = MonadAsyncMut::flat_map_async_mut(ma.clone(), &mut f).await;
    let lhs: Vec<C> = MonadAsyncMut::flat_map_async_mut(intermediate, &mut g).await;

    // For the RHS, we need to reconstruct the equivalent computation.
    // Since the async closures share mutable state, we compute the expected
    // result by manually applying the functions in the associative manner.
    let mut rhs = Vec::new();
    for a in ma {
        let mb = f(a).await;
        for b in mb {
            let mc = g(b).await;
            rhs.extend(mc);
        }
    }

    lhs == rhs
}

/// Map-FlatMap coherence (async): `m.fmap_async(f) == m.flat_map_async(|x| async { pure(f(x)) })`
pub async fn option_map_flatmap_coherence_async<A, B, F, Fut>(ma: Option<A>, f: F) -> bool
where
    A: Clone + Send + 'static,
    B: Eq + Send + 'static,
    F: Fn(A) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = B> + Send,
{
    use ordofp::async_core::FunctorAsync;

    let f_clone = f.clone();
    let lhs: Option<B> = FunctorAsync::fmap_async(ma.clone(), f).await;
    let rhs: Option<B> = MonadAsync::flat_map_async(ma, move |x| {
        let f = f_clone.clone();
        async move { Some(f(x).await) }
    })
    .await;
    lhs == rhs
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple executor for testing
    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        fn noop_raw_waker() -> RawWaker {
            fn noop(_: *const ()) {}
            fn clone_waker(_: *const ()) -> RawWaker {
                noop_raw_waker()
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, noop, noop, noop);
            RawWaker::new(std::ptr::null(), &VTABLE)
        }

        // SAFETY: `noop_raw_waker()` constructs a `RawWaker` whose vtable functions
        // (`clone_waker`, and three `noop` aliases for wake/wake_by_ref/drop) all
        // satisfy the `RawWakerVTable` contract:
        //   - `clone_waker` returns a fresh, self-consistent `RawWaker` with the same
        //     static vtable, so cloned wakers are also valid.
        //   - The null data pointer is never dereferenced by any vtable function; all
        //     four functions either ignore the pointer or construct a new `RawWaker`
        //     from the same null value and static vtable.
        //   - The vtable is `'static`, satisfying the lifetime requirement.
        // Together these invariants make the `Waker` sound to create and use.
        let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
        let mut cx = Context::from_waker(&waker);
        let mut fut = std::pin::pin!(fut);

        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(result) => return result,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    // ==================== Option Tests ====================

    #[test]
    fn test_option_left_identity_async() {
        let result = block_on(option_left_identity_async(
            5,
            |x| async move { Some(x * 2) },
        ));
        assert!(result);
    }

    #[test]
    fn test_option_left_identity_async_with_none_result() {
        let result = block_on(option_left_identity_async(5, |x| async move {
            if x > 10 { Some(x) } else { None }
        }));
        assert!(result);
    }

    #[test]
    fn test_option_right_identity_async_some() {
        let result = block_on(option_right_identity_async(Some(42)));
        assert!(result);
    }

    #[test]
    fn test_option_right_identity_async_none() {
        let result = block_on(option_right_identity_async(None::<i32>));
        assert!(result);
    }

    #[test]
    fn test_option_associativity_async_some() {
        let result = block_on(option_associativity_async(
            Some(5),
            |x| async move { Some(x + 1) },
            |x| async move { Some(x * 2) },
        ));
        assert!(result);
    }

    #[test]
    fn test_option_associativity_async_none() {
        let result = block_on(option_associativity_async(
            None::<i32>,
            |x| async move { Some(x + 1) },
            |x| async move { Some(x * 2) },
        ));
        assert!(result);
    }

    #[test]
    fn test_option_associativity_async_with_short_circuit() {
        // f returns None for x > 5
        let result = block_on(option_associativity_async(
            Some(10),
            |x| async move { if x > 5 { None } else { Some(x + 1) } },
            |x| async move { Some(x * 2) },
        ));
        assert!(result);
    }

    #[test]
    fn test_option_map_flatmap_coherence_async() {
        let result = block_on(option_map_flatmap_coherence_async(
            Some(5),
            |x| async move { x * 3 },
        ));
        assert!(result);
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_left_identity_async() {
        let result = block_on(result_left_identity_async::<_, _, String, _, _>(
            5,
            |x| async move { Ok(x * 2) },
        ));
        assert!(result);
    }

    #[test]
    fn test_result_right_identity_async_ok() {
        let result = block_on(result_right_identity_async(Ok::<i32, String>(42)));
        assert!(result);
    }

    #[test]
    fn test_result_right_identity_async_err() {
        let result = block_on(result_right_identity_async(Err::<i32, String>(
            "error".to_string(),
        )));
        assert!(result);
    }

    #[test]
    fn test_result_associativity_async_ok() {
        let result = block_on(result_associativity_async(
            Ok::<i32, String>(5),
            |x| async move { Ok(x + 1) },
            |x| async move { Ok(x * 2) },
        ));
        assert!(result);
    }

    #[test]
    fn test_result_associativity_async_err() {
        let result = block_on(result_associativity_async(
            Err::<i32, String>("error".to_string()),
            |x| async move { Ok(x + 1) },
            |x| async move { Ok(x * 2) },
        ));
        assert!(result);
    }

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_left_identity_async() {
        let result = block_on(vec_left_identity_async(
            5,
            |x| async move { vec![x, x * 2] },
        ));
        assert!(result);
    }

    #[test]
    fn test_vec_right_identity_async() {
        let result = block_on(vec_right_identity_async(vec![1, 2, 3]));
        assert!(result);
    }

    #[test]
    fn test_vec_right_identity_async_empty() {
        let result = block_on(vec_right_identity_async(Vec::<i32>::new()));
        assert!(result);
    }

    #[test]
    fn test_vec_associativity_async() {
        let result = block_on(vec_associativity_async(
            vec![1, 2],
            |x| async move { vec![x, x * 10] },
            |x| async move { vec![x + 1] },
        ));
        assert!(result);
    }

    // ==================== IsEq Tests ====================

    #[test]
    fn test_option_left_identity_eq_async() {
        let eq = block_on(option_left_identity_eq_async(
            5,
            |x| async move { Some(x * 2) },
        ));
        assert!(eq.holds());
    }

    #[test]
    fn test_option_right_identity_eq_async() {
        let eq = block_on(option_right_identity_eq_async(Some(42)));
        assert!(eq.holds());
    }

    #[test]
    fn test_result_left_identity_eq_async() {
        let eq = block_on(result_left_identity_eq_async::<_, _, String, _, _>(
            5,
            |x| async move { Ok(x * 2) },
        ));
        assert!(eq.holds());
    }
}
