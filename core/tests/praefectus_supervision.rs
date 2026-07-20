#![cfg(feature = "tokio")]

use core::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use ordofp_core::async_core::fibra::FibraManubrium;
use ordofp_core::async_core::praefectus::{IntensitasRestart, Praefectus, StrategiaRestart};
use ordofp_core::async_core::runtime::{RuntimeGenerare, TokioRuntime};

/// M12 regression: a panicking child is restarted (within intensity) and the
/// factory runs again. Pre-fix: children are never restarted (the supervision
/// loop busy-spins `yield_now` and discards the strategy/intensity), so this
/// times out at "child must be restarted".
#[tokio::test]
async fn failing_child_is_restarted_then_stop_terminates() {
    let runs = Arc::new(AtomicU32::new(0));
    let runs2 = runs.clone();

    let mut sup = Praefectus::<TokioRuntime>::new(StrategiaRestart::UnusProUno).with_intensitas(
        IntensitasRestart {
            max_restarts: 5,
            within_seconds: 60,
        },
    );
    sup.add_child_fn("flaky", move || {
        let runs = runs2.clone();
        async move {
            // Panic on the first two runs, succeed on the third. A panicking
            // child under TokioRuntime becomes JoinError::Panic -> FibraExitus
            // Err, which the supervisor treats as a failure to restart.
            assert!(
                runs.fetch_add(1, Ordering::SeqCst) >= 2,
                "transient failure"
            );
        }
    });

    let stopper = sup.stop_handle();
    let fiber = sup.start();
    let id = fiber.id();
    let token = fiber.cancellation_token();
    let sup_handle = FibraManubrium::new(id, TokioRuntime::spawn(fiber), token);

    tokio::time::timeout(Duration::from_secs(10), async {
        while runs.load(Ordering::SeqCst) < 3 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("child must be restarted to its 3rd run");

    stopper.abrogare();
    let _ = tokio::time::timeout(Duration::from_secs(5), sup_handle.conjungere())
        .await
        .expect("stop must terminate the supervision loop");
}
