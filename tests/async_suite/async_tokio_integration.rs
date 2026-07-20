//! Tokio Integration Tests for `OrdoFP` 2.0
//!
//! These tests verify that async transformers work correctly under the Tokio runtime.
//! Requires the `tokio` feature to be enabled.

#![cfg(feature = "tokio")]

use ordofp::async_core::{Flumen, Futurus, TraversableAsync};
use ordofp::transformers::async_transforms::{
    EitherTAsync, LectorAsync, OptionTAsync, ScriptorAsync, StatusAsync,
};

// ============================================================================
// Futurus Tests under Tokio
// ============================================================================

#[tokio::test]
async fn test_futurus_purus() {
    let fut = Futurus::purus(42);
    let result = fut.await;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_futurus_fmap() {
    let fut = Futurus::purus(21);
    let mapped = fut.fmap(|x| x * 2);
    let result = mapped.await;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_futurus_flat_map() {
    let fut = Futurus::purus(10);
    let chained = fut.flat_map(|x| Futurus::purus(x + 32));
    let result = chained.await;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_futurus_map2() {
    let fut1 = Futurus::purus(20);
    let fut2 = Futurus::purus(22);
    // Futurus has its own map2 method (not the ApplicatioAsync trait)
    let combined = fut1.map2(fut2, |a, b| a + b);
    let result = combined.await;
    assert_eq!(result, 42);
}

// ============================================================================
// Flumen (Stream) Tests under Tokio
// ============================================================================

#[tokio::test]
async fn test_flumen_from_iter() {
    let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5]);
    let result = stream.collect_vec().await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_flumen_fmap() {
    let stream = Flumen::from_iterator(vec![1, 2, 3]);
    let mapped = stream.fmap(|x| x * 2);
    let result = mapped.collect_vec().await;
    assert_eq!(result, vec![2, 4, 6]);
}

#[tokio::test]
async fn test_flumen_filter() {
    let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5, 6]);
    let filtered = stream.filter(|x| x % 2 == 0);
    let result = filtered.collect_vec().await;
    assert_eq!(result, vec![2, 4, 6]);
}

#[tokio::test]
async fn test_flumen_flat_map() {
    let stream = Flumen::from_iterator(vec![1, 2, 3]);
    let flat_mapped = stream.flat_map(|x| Flumen::from_iterator(vec![x, x * 10]));
    let result = flat_mapped.collect_vec().await;
    assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
}

#[tokio::test]
async fn test_flumen_fold() {
    let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5]);
    let sum = stream.fold(0, |acc, x| acc + x).await;
    assert_eq!(sum, 15);
}

#[tokio::test]
async fn test_flumen_take_skip() {
    let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5]);
    let result = stream.skip(2).take(2).collect_vec().await;
    assert_eq!(result, vec![3, 4]);
}

#[tokio::test]
async fn test_flumen_chain() {
    let s1 = Flumen::from_iterator(vec![1, 2]);
    let s2 = Flumen::from_iterator(vec![3, 4]);
    let chained = s1.chain(s2);
    let result = chained.collect_vec().await;
    assert_eq!(result, vec![1, 2, 3, 4]);
}

// ============================================================================
// LectorAsync Tests under Tokio
// ============================================================================

#[derive(Clone)]
struct Config {
    base_url: String,
    timeout_ms: u64,
}

#[tokio::test]
async fn test_lector_async_ask() {
    let config = Config {
        base_url: "https://api.example.com".to_string(),
        timeout_ms: 5000,
    };

    let reader = LectorAsync::<Config, String>::ask().fmap(|c| c.base_url.clone());

    let result = reader.run(config).await;
    assert_eq!(result, "https://api.example.com");
}

#[tokio::test]
async fn test_lector_async_asks() {
    let config = Config {
        base_url: "https://api.example.com".to_string(),
        timeout_ms: 5000,
    };

    let reader = LectorAsync::<Config, u64>::asks(|c| c.timeout_ms);
    let result = reader.run(config).await;
    assert_eq!(result, 5000);
}

#[tokio::test]
async fn test_lector_async_flat_map() {
    let config = Config {
        base_url: "https://api.example.com".to_string(),
        timeout_ms: 5000,
    };

    let reader = LectorAsync::<Config, String>::ask().flat_map(|c| {
        let url = c.base_url.clone();
        // LectorAsync::asks takes Fn(&E) -> B - specify type explicitly
        LectorAsync::<Config, String>::asks(move |c2: &Config| {
            format!("{}/timeout/{}", url, c2.timeout_ms)
        })
    });

    let result = reader.run(config).await;
    assert_eq!(result, "https://api.example.com/timeout/5000");
}

// ============================================================================
// StatusAsync Tests under Tokio
// ============================================================================

#[tokio::test]
async fn test_status_async_get_put() {
    let computation =
        StatusAsync::<i32, ()>::get().flat_map(|s| StatusAsync::<i32, ()>::put(s + 10));

    let (final_state, ()) = computation.run(5).await;
    assert_eq!(final_state, 15);
}

#[tokio::test]
async fn test_status_async_modify() {
    let computation = StatusAsync::<i32, ()>::modify(|s| s * 2)
        .flat_map(|()| StatusAsync::<i32, ()>::modify(|s| s + 1))
        .flat_map(|()| StatusAsync::<i32, i32>::get());

    let (final_state, value) = computation.run(10).await;
    assert_eq!(final_state, 21); // (10 * 2) + 1
    assert_eq!(value, 21);
}

#[tokio::test]
async fn test_status_async_exec_eval() {
    // Simple test: run purus and verify state is unchanged, value is returned
    let computation = StatusAsync::<i32, String>::purus("result".to_string());

    let (state1, value1) = computation.clone().run(0).await;
    assert_eq!(state1, 0); // State unchanged
    assert_eq!(value1, "result");

    // Test with state modification
    let modify_comp = StatusAsync::<i32, ()>::modify(|state| state + 1);
    let final_state = modify_comp.exec(0).await;
    assert_eq!(final_state, 1);
}

// ============================================================================
// OptionTAsync Tests under Tokio
// ============================================================================

#[tokio::test]
async fn test_option_t_async_some() {
    let opt = OptionTAsync::some(42);
    let result = opt.run().await;
    assert_eq!(result, Some(42));
}

#[tokio::test]
async fn test_option_t_async_none() {
    let opt = OptionTAsync::<i32>::none();
    let result = opt.run().await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_option_t_async_fmap() {
    let opt = OptionTAsync::some(21);
    let mapped = opt.fmap(|x| x * 2);
    let result = mapped.run().await;
    assert_eq!(result, Some(42));
}

#[tokio::test]
async fn test_option_t_async_flat_map() {
    let opt = OptionTAsync::some(10);
    let chained = opt.flat_map(|x| {
        if x > 5 {
            OptionTAsync::some(x * 2)
        } else {
            OptionTAsync::none()
        }
    });
    let result = chained.run().await;
    assert_eq!(result, Some(20));
}

#[tokio::test]
async fn test_option_t_async_short_circuit() {
    let opt = OptionTAsync::<i32>::none();
    let chained = opt.flat_map(|x| OptionTAsync::some(x * 2));
    let result = chained.run().await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_option_t_async_filter() {
    let opt1 = OptionTAsync::some(10);
    let filtered1 = opt1.filter(|x| *x > 5);
    assert_eq!(filtered1.run().await, Some(10));

    let opt2 = OptionTAsync::some(3);
    let filtered2 = opt2.filter(|x| *x > 5);
    assert_eq!(filtered2.run().await, None);
}

// ============================================================================
// EitherTAsync Tests under Tokio
// ============================================================================

#[tokio::test]
async fn test_either_t_async_right() {
    let either = EitherTAsync::<String, i32>::right(42);
    let result = either.run().await;
    assert_eq!(result, Ok(42));
}

#[tokio::test]
async fn test_either_t_async_left() {
    let either = EitherTAsync::<String, i32>::left("error".to_string());
    let result = either.run().await;
    assert_eq!(result, Err("error".to_string()));
}

#[tokio::test]
async fn test_either_t_async_fmap() {
    let either = EitherTAsync::<String, i32>::right(21);
    let mapped = either.fmap(|x| x * 2);
    let result = mapped.run().await;
    assert_eq!(result, Ok(42));
}

#[tokio::test]
async fn test_either_t_async_flat_map() {
    let either = EitherTAsync::<String, i32>::right(10);
    let chained = either.flat_map(|x| {
        if x > 5 {
            EitherTAsync::right(x * 2)
        } else {
            EitherTAsync::left("too small".to_string())
        }
    });
    let result = chained.run().await;
    assert_eq!(result, Ok(20));
}

#[tokio::test]
async fn test_either_t_async_map_err() {
    let either = EitherTAsync::<i32, String>::left(404);
    let mapped = either.map_err(|code| format!("Error code: {code}"));
    let result = mapped.run().await;
    assert_eq!(result, Err("Error code: 404".to_string()));
}

#[tokio::test]
async fn test_either_t_async_handle_error() {
    let either = EitherTAsync::<String, i32>::left("error".to_string());
    let handled = either.handle_error(|_| EitherTAsync::right(0));
    let result = handled.run().await;
    assert_eq!(result, Ok(0));
}

// ============================================================================
// ScriptorAsync Tests under Tokio
// ============================================================================

#[tokio::test]
async fn test_scriptor_async_tell() {
    // ScriptorAsync<W, A> where W is the log type - so Vec<String> is the W
    let writer = ScriptorAsync::<Vec<String>, ()>::tell(vec!["log1".to_string()]);
    let (logs, ()) = writer.run().await;
    assert_eq!(logs, vec!["log1".to_string()]);
}

#[tokio::test]
async fn test_scriptor_async_purus() {
    let writer = ScriptorAsync::<Vec<String>, i32>::purus(42);
    let (logs, value): (Vec<String>, i32) = writer.run().await;
    assert!(logs.is_empty());
    assert_eq!(value, 42);
}

#[tokio::test]
async fn test_scriptor_async_flat_map() {
    // flat_map requires Vec<T> as the W type for the flat_map impl
    let writer = ScriptorAsync::<Vec<String>, i32>::tell(vec!["start".to_string()])
        .then(ScriptorAsync::purus(42))
        .flat_map(|v| {
            ScriptorAsync::<Vec<String>, i32>::tell(vec![format!("value: {}", v)])
                .then(ScriptorAsync::purus(v))
        });

    let (logs, value) = writer.run().await;
    assert_eq!(logs, vec!["start".to_string(), "value: 42".to_string()]);
    assert_eq!(value, 42);
}

// ============================================================================
// TraversableAsync Tests under Tokio
// ============================================================================

#[tokio::test]
async fn test_traverse_vec_async() {
    let vec = vec![1, 2, 3, 4, 5];
    let result = vec.traverse_async(|x| async move { x * 2 }).await;
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
}

#[tokio::test]
async fn test_traverse_option_async() {
    use ordofp::async_core::OptionTraverseAsync;

    let opt = Some(21);
    let result = opt.traverse_option_async(|x| async move { x * 2 }).await;
    assert_eq!(result, Some(42));

    let none: Option<i32> = None;
    let result2 = none.traverse_option_async(|x| async move { x * 2 }).await;
    assert_eq!(result2, None);
}

#[tokio::test]
async fn test_traverse_result_async() {
    use ordofp::async_core::ResultTraverseAsync;

    let ok: Result<i32, String> = Ok(21);
    let result = ok.traverse_result_async(|x| async move { x * 2 }).await;
    assert_eq!(result, Ok(42));

    let err: Result<i32, String> = Err("error".to_string());
    let result2 = err.traverse_result_async(|x| async move { x * 2 }).await;
    assert_eq!(result2, Err("error".to_string()));
}

// ============================================================================
// Concurrent Operations Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_futures() {
    use tokio::time::{Duration, sleep};

    async fn delayed_value(ms: u64, value: i32) -> i32 {
        sleep(Duration::from_millis(ms)).await;
        value
    }

    let start = std::time::Instant::now();

    // Run two futures concurrently
    let (a, b) = tokio::join!(delayed_value(50, 10), delayed_value(50, 20));

    let elapsed = start.elapsed();

    assert_eq!(a + b, 30);
    // Should complete in ~50ms (concurrent), not ~100ms (sequential)
    assert!(elapsed.as_millis() < 100);
}

#[tokio::test]
async fn test_concurrent_stream_processing() {
    let stream = Flumen::from_iterator(0..10);
    let result = stream
        .fmap(|x| x * 2)
        .filter(|x| x % 4 == 0)
        .fold(0, |acc, x| acc + x)
        .await;

    // 0, 4, 8, 12, 16 = 40
    assert_eq!(result, 40);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_either_t_async_error_chain() {
    async fn validate_positive(x: i32) -> Result<i32, String> {
        if x > 0 {
            Ok(x)
        } else {
            Err("must be positive".to_string())
        }
    }

    async fn validate_even(x: i32) -> Result<i32, String> {
        if x % 2 == 0 {
            Ok(x)
        } else {
            Err("must be even".to_string())
        }
    }

    // Valid case - use From<Result<A, E>> to convert
    let result1: EitherTAsync<String, i32> = validate_positive(4).await.into();
    let result1 = result1
        .flat_map(|x| EitherTAsync::new(async move { validate_even(x).await }))
        .run()
        .await;
    assert_eq!(result1, Ok(4));

    // Fails first validation
    let result2: EitherTAsync<String, i32> = validate_positive(-1).await.into();
    let result2 = result2
        .flat_map(|x| EitherTAsync::new(async move { validate_even(x).await }))
        .run()
        .await;
    assert_eq!(result2, Err("must be positive".to_string()));

    // Fails second validation
    let result3: EitherTAsync<String, i32> = validate_positive(3).await.into();
    let result3 = result3
        .flat_map(|x| EitherTAsync::new(async move { validate_even(x).await }))
        .run()
        .await;
    assert_eq!(result3, Err("must be even".to_string()));
}

// ============================================================================
// Async Macro Tests under Tokio
// ============================================================================

use ordofp::{chain_async, compose_async, mdo_async, pipe_async};

#[tokio::test]
async fn test_mdo_async_with_await() {
    async fn fetch_value(x: i32) -> i32 {
        x * 2
    }

    let result = mdo_async! {
        let a = pure 10;
        let b = await fetch_value(a);
        let c = await fetch_value(b);
        c
    };

    // 10 -> 20 -> 40
    assert_eq!(result, 40);
}

#[tokio::test]
async fn test_pipe_async_tokio() {
    async fn add_one(x: i32) -> i32 {
        x + 1
    }
    async fn double(x: i32) -> i32 {
        x * 2
    }
    async fn subtract_three(x: i32) -> i32 {
        x - 3
    }

    let result = pipe_async!(10, add_one, double, subtract_three).await;
    // 10 -> 11 -> 22 -> 19
    assert_eq!(result, 19);
}

#[tokio::test]
async fn test_compose_async_tokio() {
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
    let result = composed(10).await;
    // 10 - 3 = 7, 7 * 2 = 14, 14 + 1 = 15
    assert_eq!(result, 15);
}

#[tokio::test]
async fn test_chain_async_tokio() {
    async fn add_one(x: i32) -> i32 {
        x + 1
    }
    async fn double(x: i32) -> i32 {
        x * 2
    }
    async fn subtract_three(x: i32) -> i32 {
        x - 3
    }

    // chain_async!(f, g, h)(x) = h(g(f(x))) - left to right
    let chained = chain_async!(add_one, double, subtract_three);
    let result = chained(10).await;
    // 10 + 1 = 11, 11 * 2 = 22, 22 - 3 = 19
    assert_eq!(result, 19);
}
