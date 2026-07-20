//! # Async Functor Laws
//!
//! This module provides property-based laws for testing [`FunctorAsync`] implementations.
//!
//! ## Laws
//!
//! 1. **Identity**: `fa.fmap_async(id).await == fa`
//! 2. **Composition**: `fa.fmap_async(f).await.fmap_async(g).await == fa.fmap_async(|x| g(f(x))).await`
//!
//! ## Usage
//!
//! ```ignore
//! use ordofp_laws::async_functor_laws;
//!
//! // Test identity law for Option
//! assert!(async_functor_laws::option_identity_async(Some(42)).await);
//!
//! // Test composition law
//! assert!(async_functor_laws::option_composition_async(
//!     Some(5),
//!     |x| async move { x + 1 },
//!     |x| async move { x * 2 }
//! ).await);
//! ```

use crate::is_eq::IsEq;
use ordofp::async_core::FunctorAsync;
use std::future::Future;

/// The identity function.
pub fn id<T>(x: T) -> T {
    x
}

// ==================== Option Laws ====================

/// **Identity Law** for Option (async): `fa.fmap_async(|x| async { x }).await == fa`
pub async fn option_identity_async<A: Clone + Eq + Send + 'static>(fa: Option<A>) -> bool {
    let result = FunctorAsync::fmap_async(fa.clone(), |x| async move { x }).await;
    result == fa
}

/// **Composition Law** for Option (async):
/// `fa.fmap_async(f).await.fmap_async(g).await == fa.fmap_async(|x| async { g(f(x).await).await }).await`
pub async fn option_composition_async<A, B, C, F, G, Fut1, Fut2>(fa: Option<A>, f: F, g: G) -> bool
where
    A: Clone + Send + 'static,
    B: Clone + Send + 'static,
    C: Eq + Send + 'static,
    F: Fn(A) -> Fut1 + Clone + Send + Sync + 'static,
    G: Fn(B) -> Fut2 + Clone + Send + Sync + 'static,
    Fut1: Future<Output = B> + Send,
    Fut2: Future<Output = C> + Send,
{
    let f_clone = f.clone();
    let g_clone = g.clone();

    // LHS: fa.fmap_async(f).fmap_async(g)
    let intermediate: Option<B> = FunctorAsync::fmap_async(fa.clone(), f).await;
    let lhs: Option<C> = FunctorAsync::fmap_async(intermediate, g).await;

    // RHS: fa.fmap_async(|x| g(f(x)))
    let rhs: Option<C> = FunctorAsync::fmap_async(fa, move |x| {
        let f = f_clone.clone();
        let g = g_clone.clone();
        async move {
            let b = f(x).await;
            g(b).await
        }
    })
    .await;

    lhs == rhs
}

/// Returns an [`IsEq`] for the Option identity law (async).
pub async fn option_identity_eq_async<A: Clone + Send + 'static>(fa: Option<A>) -> IsEq<Option<A>> {
    let mapped = FunctorAsync::fmap_async(fa.clone(), |x| async move { x }).await;
    IsEq::equal_under_law(mapped, fa)
}

// ==================== Result Laws ====================

/// **Identity Law** for Result (async): `fa.fmap_async(|x| async { x }).await == fa`
pub async fn result_identity_async<A, E>(fa: Result<A, E>) -> bool
where
    A: Clone + Eq + Send + 'static,
    E: Clone + Eq + Send + 'static,
{
    let result = FunctorAsync::fmap_async(fa.clone(), |x| async move { x }).await;
    result == fa
}

/// **Composition Law** for Result (async):
pub async fn result_composition_async<A, B, C, E, F, G, Fut1, Fut2>(
    fa: Result<A, E>,
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
    Fut1: Future<Output = B> + Send,
    Fut2: Future<Output = C> + Send,
{
    let f_clone = f.clone();
    let g_clone = g.clone();

    // LHS: fa.fmap_async(f).fmap_async(g)
    let intermediate: Result<B, E> = FunctorAsync::fmap_async(fa.clone(), f).await;
    let lhs: Result<C, E> = FunctorAsync::fmap_async(intermediate, g).await;

    // RHS: fa.fmap_async(|x| g(f(x)))
    let rhs: Result<C, E> = FunctorAsync::fmap_async(fa, move |x| {
        let f = f_clone.clone();
        let g = g_clone.clone();
        async move {
            let b = f(x).await;
            g(b).await
        }
    })
    .await;

    lhs == rhs
}

/// Returns an [`IsEq`] for the Result identity law (async).
pub async fn result_identity_eq_async<A, E>(fa: Result<A, E>) -> IsEq<Result<A, E>>
where
    A: Clone + Send + 'static,
    E: Clone + Send + 'static,
{
    let mapped = FunctorAsync::fmap_async(fa.clone(), |x| async move { x }).await;
    IsEq::equal_under_law(mapped, fa)
}

// ==================== Vec Laws ====================
//
// Note: Vec's FunctorAsync implementation uses FnOnce and only processes
// the first element. For full Vec mapping, use FunctorAsyncMut with fmap_async_mut.
// These laws use FunctorAsyncMut for correct semantics.

use ordofp::async_core::FunctorAsyncMut;

/// **Identity Law** for Vec (async): `fa.fmap_async_mut(|x| async { x }).await == fa`
///
/// Uses `FunctorAsyncMut` to properly map over all elements.
pub async fn vec_identity_async<A: Clone + Eq + Send + 'static>(fa: Vec<A>) -> bool {
    let result = FunctorAsyncMut::fmap_async_mut(fa.clone(), |x| async move { x }).await;
    result == fa
}

/// **Composition Law** for Vec (async):
///
/// Uses `FunctorAsyncMut` to properly map over all elements.
/// Note: Due to `FnMut` closure semantics, this test verifies the law
/// for specific function instances rather than the general case.
pub async fn vec_composition_async<A, B, C, F, G, Fut1, Fut2>(
    fa: Vec<A>,
    mut f: F,
    mut g: G,
) -> bool
where
    A: Clone + Send + 'static,
    B: Clone + Send + 'static,
    C: Eq + Send + 'static,
    F: FnMut(A) -> Fut1 + Clone + Send + 'static,
    G: FnMut(B) -> Fut2 + Clone + Send + 'static,
    Fut1: Future<Output = B> + Send,
    Fut2: Future<Output = C> + Send,
{
    // LHS: fa.fmap_async_mut(f).fmap_async_mut(g)
    let intermediate: Vec<B> = FunctorAsyncMut::fmap_async_mut(fa.clone(), &mut f).await;
    let lhs: Vec<C> = FunctorAsyncMut::fmap_async_mut(intermediate, &mut g).await;

    // For the RHS, compute the expected result by applying composed functions
    let mut rhs = Vec::with_capacity(fa.len());
    for a in fa {
        let b = f(a).await;
        let c = g(b).await;
        rhs.push(c);
    }

    lhs == rhs
}

/// Returns an [`IsEq`] for the Vec identity law (async).
///
/// Uses `FunctorAsyncMut` to properly map over all elements.
pub async fn vec_identity_eq_async<A: Clone + Send + 'static>(fa: Vec<A>) -> IsEq<Vec<A>> {
    let mapped = FunctorAsyncMut::fmap_async_mut(fa.clone(), |x| async move { x }).await;
    IsEq::equal_under_law(mapped, fa)
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

        // SAFETY: `noop_raw_waker()` satisfies the `RawWaker` contract: the
        // vtable functions are all no-ops that never dereference the null data
        // pointer, and there is no heap allocation to drop, so the waker can
        // be safely constructed and dropped without violating any invariant.
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
    fn test_option_identity_async_some() {
        let result = block_on(option_identity_async(Some(42)));
        assert!(result);
    }

    #[test]
    fn test_option_identity_async_none() {
        let result = block_on(option_identity_async(None::<i32>));
        assert!(result);
    }

    #[test]
    fn test_option_composition_async_some() {
        let result = block_on(option_composition_async(
            Some(5),
            |x| async move { x + 1 },
            |x| async move { x * 2 },
        ));
        assert!(result);
    }

    #[test]
    fn test_option_composition_async_none() {
        let result = block_on(option_composition_async(
            None::<i32>,
            |x| async move { x + 1 },
            |x| async move { x * 2 },
        ));
        assert!(result);
    }

    #[test]
    fn test_option_composition_async_type_change() {
        let result = block_on(option_composition_async(
            Some(42),
            |x: i32| async move { x.to_string() },
            |s: String| async move { s.len() },
        ));
        assert!(result);
    }

    // ==================== Result Tests ====================

    #[test]
    fn test_result_identity_async_ok() {
        let result = block_on(result_identity_async(Ok::<i32, String>(42)));
        assert!(result);
    }

    #[test]
    fn test_result_identity_async_err() {
        let result = block_on(result_identity_async(Err::<i32, String>(
            "error".to_string(),
        )));
        assert!(result);
    }

    #[test]
    fn test_result_composition_async_ok() {
        let result = block_on(result_composition_async(
            Ok::<i32, String>(5),
            |x| async move { x + 1 },
            |x| async move { x * 2 },
        ));
        assert!(result);
    }

    #[test]
    fn test_result_composition_async_err() {
        let result = block_on(result_composition_async(
            Err::<i32, String>("error".to_string()),
            |x| async move { x + 1 },
            |x| async move { x * 2 },
        ));
        assert!(result);
    }

    // ==================== Vec Tests ====================

    #[test]
    fn test_vec_identity_async() {
        let result = block_on(vec_identity_async(vec![1, 2, 3]));
        assert!(result);
    }

    #[test]
    fn test_vec_identity_async_empty() {
        let result = block_on(vec_identity_async(Vec::<i32>::new()));
        assert!(result);
    }

    #[test]
    fn test_vec_composition_async() {
        let result = block_on(vec_composition_async(
            vec![1, 2, 3],
            |x| async move { x + 1 },
            |x| async move { x * 2 },
        ));
        assert!(result);
    }

    // ==================== IsEq Tests ====================

    #[test]
    fn test_option_identity_eq_async() {
        let eq = block_on(option_identity_eq_async(Some(42)));
        assert!(eq.holds());
    }

    #[test]
    fn test_result_identity_eq_async() {
        let eq = block_on(result_identity_eq_async(Ok::<i32, String>(42)));
        assert!(eq.holds());
    }

    #[test]
    fn test_vec_identity_eq_async() {
        let eq = block_on(vec_identity_eq_async(vec![1, 2, 3]));
        assert!(eq.holds());
    }
}
