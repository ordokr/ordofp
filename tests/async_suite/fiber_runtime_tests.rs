//! Tests for `OrdoFP` 4.0 Phase 3: Advanced Fiber Runtime
//!
//! Tests ZIO-style effects, concurrent primitives, and the work-stealing scheduler.

#![cfg(feature = "async")]

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use ordofp_core::async_core::{
    Ambitus,
    Causa,
    Exitus,
    IndiciumExecutionis,
    OrdinariusConfig,
    PolitiaCircularis,
    PolitiaFortuita,
    PolitiaFurti,
    // Scheduler types
    Prioritas,
    Statisticae,
    // ZIO-style effects
    Zio,
    fail,
    from_option,

    from_result,
    succeed,
};

#[cfg(feature = "std")]
use ordofp_core::async_core::{
    CaudaBackpressure, Dilatum, MVarSync, OrdoGlobalis, OrdoLocalis, Referentia, Semaphorum,
};

// =============================================================================
// CAUSA TESTS
// =============================================================================

#[test]
fn test_causa_defectus() {
    let causa: Causa<&str> = Causa::defectus("error");
    assert!(causa.is_defectus());
    assert!(!causa.is_mors());
    assert!(!causa.is_interruptio());
    assert_eq!(causa.defect(), Some(&"error"));
}

#[test]
fn test_causa_mors() {
    let causa: Causa<&str> = Causa::mors("panic message");
    assert!(causa.is_mors());
    assert!(!causa.is_defectus());
}

#[test]
fn test_causa_defects() {
    let c1: Causa<&str> = Causa::defectus("error1");
    let c2: Causa<&str> = Causa::defectus("error2");
    let combined = c1.both(c2);

    let defects = combined.defects();
    assert_eq!(defects.len(), 2);
    assert!(defects.contains(&&"error1"));
    assert!(defects.contains(&&"error2"));
}

#[test]
fn test_causa_then() {
    let c1: Causa<&str> = Causa::defectus("first");
    let c2: Causa<&str> = Causa::defectus("second");
    let chained = c1.then(c2);

    assert_eq!(chained.defects().len(), 2);
}

#[test]
fn test_causa_map_error() {
    let causa: Causa<i32> = Causa::defectus(42);
    let mapped = causa.map_error(|x| x.to_string());

    assert!(mapped.is_defectus());
    assert_eq!(mapped.defect(), Some(&String::from("42")));
}

// =============================================================================
// EXITUS TESTS
// =============================================================================

#[test]
fn test_exitus_successus() {
    let exit: Exitus<&str, i32> = Exitus::successus(42);
    assert!(exit.is_successus());
    assert!(!exit.is_defectio());
    assert_eq!(exit.successus_value(), Some(&42));
}

#[test]
fn test_exitus_defectio() {
    let exit: Exitus<&str, i32> = Exitus::defectio("error");
    assert!(exit.is_defectio());
    assert!(!exit.is_successus());
    assert!(exit.causa().is_some());
}

#[test]
fn test_exitus_map() {
    let exit: Exitus<&str, i32> = Exitus::successus(21);
    let mapped = exit.map(|x| x * 2);
    assert_eq!(mapped.successus_value(), Some(&42));

    let failed: Exitus<&str, i32> = Exitus::defectio("error");
    let failed_mapped = failed.map(|x| x * 2);
    assert!(failed_mapped.is_defectio());
}

#[test]
fn test_exitus_flat_map() {
    let exit: Exitus<&str, i32> = Exitus::successus(20);
    let result = exit.flat_map(|x| Exitus::successus(x + 22));
    assert_eq!(result.successus_value(), Some(&42));

    let failed: Exitus<&str, i32> = Exitus::defectio("error");
    let failed_result = failed.flat_map(|x| Exitus::successus(x + 22));
    assert!(failed_result.is_defectio());
}

#[test]
fn test_exitus_fold() {
    let success: Exitus<&str, i32> = Exitus::successus(42);
    let s_result = success.fold(|x| format!("ok: {x}"), |_| String::from("err"));
    assert_eq!(s_result, "ok: 42");

    let failure: Exitus<&str, i32> = Exitus::defectio("error");
    let f_result = failure.fold(|x| format!("ok: {x}"), |_| String::from("err"));
    assert_eq!(f_result, "err");
}

#[test]
fn test_exitus_from_result() {
    let ok: Exitus<&str, i32> = Exitus::from_result(Ok(42));
    assert!(ok.is_successus());

    let err: Exitus<&str, i32> = Exitus::from_result(Err("error"));
    assert!(err.is_defectio());
}

#[test]
fn test_exitus_to_result() {
    let success: Exitus<&str, i32> = Exitus::successus(42);
    assert!(success.to_result().is_ok());

    let failure: Exitus<&str, i32> = Exitus::defectio("error");
    assert!(failure.to_result().is_err());
}

// =============================================================================
// AMBITUS TESTS
// =============================================================================

#[test]
fn test_ambitus_new() {
    let env = Ambitus::new(42);
    assert_eq!(*env.get(), 42);
}

#[test]
fn test_ambitus_into_inner() {
    let env = Ambitus::new(42);
    assert_eq!(env.into_inner(), 42);
}

#[test]
fn test_ambitus_map() {
    let env = Ambitus::new(21);
    let mapped = env.map(|x| x * 2);
    assert_eq!(*mapped.get(), 42);
}

// =============================================================================
// PRIORITAS AND INDICIUM TESTS
// =============================================================================

#[test]
fn test_prioritas_ordering() {
    assert!(Prioritas::Critica > Prioritas::Alta);
    assert!(Prioritas::Alta > Prioritas::Normalis);
    assert!(Prioritas::Normalis > Prioritas::Infima);
}

#[test]
fn test_prioritas_default() {
    assert_eq!(Prioritas::default(), Prioritas::Normalis);
}

#[test]
fn test_indicium_default() {
    assert_eq!(
        IndiciumExecutionis::default(),
        IndiciumExecutionis::Computatio
    );
}

// =============================================================================
// ORDINARIUS CONFIG TESTS
// =============================================================================

#[test]
fn test_ordinarius_config_default() {
    let config = OrdinariusConfig::default();
    assert_eq!(config.num_workers, 4);
    assert!(config.work_stealing);
}

#[test]
fn test_ordinarius_config_with_workers() {
    let config = OrdinariusConfig::with_workers(8);
    assert_eq!(config.num_workers, 8);
}

// =============================================================================
// STATISTICAE TESTS
// =============================================================================

#[test]
fn test_statisticae_new() {
    let stats = Statisticae::new();
    assert_eq!(
        stats
            .tasks_scheduled
            .load(core::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_statisticae_record_scheduled() {
    let stats = Statisticae::new();
    stats.record_scheduled();
    stats.record_scheduled();
    assert_eq!(
        stats
            .tasks_scheduled
            .load(core::sync::atomic::Ordering::Relaxed),
        2
    );
}

#[test]
fn test_statisticae_record_executed() {
    let stats = Statisticae::new();
    stats.record_executed();
    assert_eq!(
        stats
            .tasks_executed
            .load(core::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_statisticae_steal_success_rate() {
    let stats = Statisticae::new();
    stats.record_steal_attempt(true, 5);
    stats.record_steal_attempt(true, 3);
    stats.record_steal_attempt(false, 0);
    stats.record_steal_attempt(false, 0);

    assert!((stats.steal_success_rate() - 0.5).abs() < 0.001);
}

// =============================================================================
// POLITIA FURTI TESTS
// =============================================================================

#[test]
fn test_politia_fortuita_select_victim() {
    let policy = PolitiaFortuita::new();

    // Should not select self
    for _ in 0..10 {
        let victim = policy.select_victim(0, 4);
        assert!(victim.is_some());
        assert_ne!(
            victim.expect("victim selection should return Some when there are valid targets"),
            0
        );
    }
}

#[test]
fn test_politia_fortuita_single_worker() {
    let policy = PolitiaFortuita::new();
    let victim = policy.select_victim(0, 1);
    assert!(victim.is_none());
}

#[test]
fn test_politia_circularis() {
    let policy = PolitiaCircularis::new();

    let v1 = policy.select_victim(0, 4);
    let v2 = policy.select_victim(0, 4);

    assert!(v1.is_some());
    assert!(v2.is_some());
}

// =============================================================================
// CONCURRENT PRIMITIVES TESTS (std feature only)
// =============================================================================

#[cfg(feature = "std")]
mod std_tests {
    use super::*;

    #[test]
    fn test_referentia_new() {
        let r = Referentia::new(42);
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_referentia_set() {
        let r = Referentia::new(0);
        let old = r.set(42);
        assert_eq!(old, 0);
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_referentia_update() {
        let r = Referentia::new(21);
        r.update(|x| x * 2);
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_referentia_modify() {
        let r = Referentia::new(21);
        let result = r.modify(|x| (x * 2, "done"));
        assert_eq!(result, "done");
        assert_eq!(r.get(), 42);
    }

    #[test]
    fn test_semaphorum_new() {
        let sem = Semaphorum::new(3);
        assert_eq!(sem.available(), 3);
    }

    #[test]
    fn test_semaphorum_try_acquire() {
        let sem = Semaphorum::new(1);
        assert!(sem.try_acquire());
        assert!(!sem.try_acquire());
        sem.release();
        assert!(sem.try_acquire());
    }

    #[test]
    fn test_semaphorum_release_n() {
        let sem = Semaphorum::new(0);
        sem.release_n(5);
        assert_eq!(sem.available(), 5);
    }

    #[test]
    fn test_mvar_sync_empty() {
        let mvar: MVarSync<i32> = MVarSync::new_empty();
        assert!(mvar.is_empty());
    }

    #[test]
    fn test_mvar_sync_new() {
        let mvar = MVarSync::new(42);
        assert!(!mvar.is_empty());
    }

    #[test]
    fn test_mvar_sync_try_put_take() {
        let mvar: MVarSync<i32> = MVarSync::new_empty();
        assert!(mvar.try_put(42));
        assert!(!mvar.try_put(43)); // Full
        assert_eq!(mvar.try_take(), Some(42));
        assert!(mvar.is_empty());
    }

    #[test]
    fn test_cauda_new() {
        let queue: CaudaBackpressure<i32> = CaudaBackpressure::new(10);
        assert_eq!(queue.capacity(), 10);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_cauda_try_offer_take() {
        let queue = CaudaBackpressure::new(2);
        assert!(queue.try_offer(1));
        assert!(queue.try_offer(2));
        assert!(!queue.try_offer(3)); // Full

        assert_eq!(queue.try_take(), Some(1));
        assert_eq!(queue.try_take(), Some(2));
        assert_eq!(queue.try_take(), None);
    }

    #[test]
    fn test_cauda_drain() {
        let queue = CaudaBackpressure::new(10);
        queue.try_offer(1);
        queue.try_offer(2);
        queue.try_offer(3);

        let items = queue.drain();
        assert_eq!(items, vec![1, 2, 3]);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_dilatum_complete() {
        let deferred = Dilatum::<i32>::new();
        assert!(!deferred.is_completed());

        assert!(deferred.complete(42));
        assert!(deferred.is_completed());

        assert!(!deferred.complete(43)); // Already completed
        assert_eq!(deferred.try_get(), Some(42));
    }

    #[test]
    fn test_ordo_localis_empty() {
        let queue = OrdoLocalis::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_ordo_globalis_empty() {
        let queue = OrdoGlobalis::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(!queue.is_shutdown());
    }

    #[test]
    fn test_ordo_globalis_shutdown() {
        let queue = OrdoGlobalis::new();
        queue.shutdown();
        assert!(queue.is_shutdown());
    }
}

// =============================================================================
// ZIO FREE FUNCTION TESTS
// =============================================================================

#[test]
fn test_succeed_function() {
    // Just test that it compiles - runtime testing would need async
    let _: Zio<(), (), i32> = succeed(42);
}

#[test]
fn test_fail_function() {
    let _: Zio<(), &str, i32> = fail("error");
}

#[test]
fn test_from_result_ok() {
    let result: Result<i32, &str> = Ok(42);
    let _: Zio<(), &str, i32> = from_result(result);
}

#[test]
fn test_from_result_err() {
    let result: Result<i32, &str> = Err("error");
    let _: Zio<(), &str, i32> = from_result(result);
}

#[test]
fn test_from_option_some() {
    let opt: Option<i32> = Some(42);
    let _: Zio<(), (), i32> = from_option(opt);
}

#[test]
fn test_from_option_none() {
    let opt: Option<i32> = None;
    let _: Zio<(), (), i32> = from_option(opt);
}
