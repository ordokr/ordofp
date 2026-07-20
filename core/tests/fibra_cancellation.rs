#![cfg(feature = "tokio")]

use core::time::Duration;
use ordofp_core::async_core::fibra::{Fibra, FibraError, FibraManubrium, certamen, par};
use ordofp_core::async_core::runtime::{RuntimeGenerare, TokioRuntime};

/// H2 regression: cancelling a parked fiber must wake it.
/// Pre-fix this times out (deadlock).
#[tokio::test]
async fn cancel_wakes_parked_fiber() {
    let fibra: Fibra<()> = Fibra::new(core::future::pending());
    let id = fibra.id();
    let token = fibra.cancellation_token();
    let handle = FibraManubrium::new(id, TokioRuntime::spawn(fibra), token);
    // Give the spawned fiber a chance to run and park on the inner `Pending`
    // before we cancel it — otherwise the cancel could land before the
    // fiber's first poll, which trivially observes the flag without
    // exercising the wake path this test targets.
    TokioRuntime::yield_now().await;
    let out = tokio::time::timeout(Duration::from_secs(5), handle.abrogare_et_conjungere())
        .await
        .expect("cancellation must complete, not deadlock");
    assert!(matches!(out, Err(FibraError::Abrogatus)));
}

/// H4 regression: first-to-complete wins even when the first argument never
/// completes. Pre-fix: times out.
#[tokio::test]
async fn certamen_pending_vs_ready_resolves() {
    let never: Fibra<i32> = Fibra::new(core::future::pending());
    let ready: Fibra<i32> = Fibra::new(async { 42 });
    let out = tokio::time::timeout(Duration::from_secs(5), certamen(never, ready))
        .await
        .expect("race must resolve");
    assert_eq!(out.unwrap(), 42);
}

/// par must interleave: A waits on a message B sends. Sequential execution
/// (A to completion, then B) deadlocks. Pre-fix: times out.
#[tokio::test]
async fn par_interleaves_cross_dependent_fibers() {
    let (tx, rx) = tokio::sync::oneshot::channel::<i32>();
    let a: Fibra<i32> = Fibra::new(async move { rx.await.unwrap() });
    let b: Fibra<i32> = Fibra::new(async move {
        tx.send(7).unwrap();
        1
    });
    let out = tokio::time::timeout(Duration::from_secs(5), par(a, b))
        .await
        .expect("par must interleave");
    assert_eq!(out.unwrap(), (7, 1));
}
