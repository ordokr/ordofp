//! smol Integration Tests for `OrdoFP` 2.0
//!
//! These tests verify that async transformers work correctly under the smol
//! runtime (the actively-maintained successor to the discontinued async-std).
//! Requires the `smol` feature to be enabled.
//!
//! Each test drives its future to completion with `smol::block_on`.

#![cfg(feature = "smol")]

use ordofp::async_core::{Flumen, Futurus, TraversableAsync};
use ordofp::transformers::async_transforms::{
    EitherTAsync, LectorAsync, OptionTAsync, ScriptorAsync, StatusAsync,
};

// ============================================================================
// Futurus Tests under smol
// ============================================================================

#[test]
fn test_futurus_purus() {
    smol::block_on(async {
        let fut = Futurus::purus(42);
        let result = fut.await;
        assert_eq!(result, 42);
    });
}

#[test]
fn test_futurus_fmap() {
    smol::block_on(async {
        let fut = Futurus::purus(21);
        let mapped = fut.fmap(|x| x * 2);
        let result = mapped.await;
        assert_eq!(result, 42);
    });
}

#[test]
fn test_futurus_flat_map() {
    smol::block_on(async {
        let fut = Futurus::purus(10);
        let chained = fut.flat_map(|x| Futurus::purus(x + 32));
        let result = chained.await;
        assert_eq!(result, 42);
    });
}

#[test]
fn test_futurus_map2() {
    smol::block_on(async {
        let fut1 = Futurus::purus(20);
        let fut2 = Futurus::purus(22);
        let combined = fut1.map2(fut2, |a, b| a + b);
        let result = combined.await;
        assert_eq!(result, 42);
    });
}

// ============================================================================
// Flumen (Stream) Tests under smol
// ============================================================================

#[test]
fn test_flumen_from_iter() {
    smol::block_on(async {
        let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5]);
        let result = stream.collect_vec().await;
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    });
}

#[test]
fn test_flumen_fmap() {
    smol::block_on(async {
        let stream = Flumen::from_iterator(vec![1, 2, 3]);
        let mapped = stream.fmap(|x| x * 2);
        let result = mapped.collect_vec().await;
        assert_eq!(result, vec![2, 4, 6]);
    });
}

#[test]
fn test_flumen_filter() {
    smol::block_on(async {
        let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5, 6]);
        let filtered = stream.filter(|x| x % 2 == 0);
        let result = filtered.collect_vec().await;
        assert_eq!(result, vec![2, 4, 6]);
    });
}

#[test]
fn test_flumen_flat_map() {
    smol::block_on(async {
        let stream = Flumen::from_iterator(vec![1, 2, 3]);
        let flat_mapped = stream.flat_map(|x| Flumen::from_iterator(vec![x, x * 10]));
        let result = flat_mapped.collect_vec().await;
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    });
}

#[test]
fn test_flumen_fold() {
    smol::block_on(async {
        let stream = Flumen::from_iterator(vec![1, 2, 3, 4, 5]);
        let sum = stream.fold(0, |acc, x| acc + x).await;
        assert_eq!(sum, 15);
    });
}

// ============================================================================
// LectorAsync Tests under smol
// ============================================================================

#[derive(Clone)]
struct Config {
    base_url: String,
    timeout_ms: u64,
}

#[test]
fn test_lector_async_ask() {
    smol::block_on(async {
        let config = Config {
            base_url: "https://api.example.com".to_string(),
            timeout_ms: 5000,
        };

        let reader = LectorAsync::<Config, String>::ask().fmap(|c| c.base_url.clone());

        let result = reader.run(config).await;
        assert_eq!(result, "https://api.example.com");
    });
}

#[test]
fn test_lector_async_asks() {
    smol::block_on(async {
        let config = Config {
            base_url: "https://api.example.com".to_string(),
            timeout_ms: 5000,
        };

        let reader = LectorAsync::<Config, u64>::asks(|c| c.timeout_ms);
        let result = reader.run(config).await;
        assert_eq!(result, 5000);
    });
}

// ============================================================================
// StatusAsync Tests under smol
// ============================================================================

#[test]
fn test_status_async_get_put() {
    smol::block_on(async {
        let computation =
            StatusAsync::<i32, ()>::get().flat_map(|s| StatusAsync::<i32, ()>::put(s + 10));

        let (final_state, ()) = computation.run(5).await;
        assert_eq!(final_state, 15);
    });
}

#[test]
fn test_status_async_modify() {
    smol::block_on(async {
        let computation = StatusAsync::<i32, ()>::modify(|s| s * 2)
            .flat_map(|()| StatusAsync::<i32, ()>::modify(|s| s + 1))
            .flat_map(|()| StatusAsync::<i32, i32>::get());

        let (final_state, value) = computation.run(10).await;
        assert_eq!(final_state, 21); // (10 * 2) + 1
        assert_eq!(value, 21);
    });
}

// ============================================================================
// OptionTAsync Tests under smol
// ============================================================================

#[test]
fn test_option_t_async_some() {
    smol::block_on(async {
        let opt = OptionTAsync::some(42);
        let result = opt.run().await;
        assert_eq!(result, Some(42));
    });
}

#[test]
fn test_option_t_async_none() {
    smol::block_on(async {
        let opt = OptionTAsync::<i32>::none();
        let result = opt.run().await;
        assert_eq!(result, None);
    });
}

#[test]
fn test_option_t_async_fmap() {
    smol::block_on(async {
        let opt = OptionTAsync::some(21);
        let mapped = opt.fmap(|x| x * 2);
        let result = mapped.run().await;
        assert_eq!(result, Some(42));
    });
}

#[test]
fn test_option_t_async_flat_map() {
    smol::block_on(async {
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
    });
}

// ============================================================================
// EitherTAsync Tests under smol
// ============================================================================

#[test]
fn test_either_t_async_right() {
    smol::block_on(async {
        let either = EitherTAsync::<String, i32>::right(42);
        let result = either.run().await;
        assert_eq!(result, Ok(42));
    });
}

#[test]
fn test_either_t_async_left() {
    smol::block_on(async {
        let either = EitherTAsync::<String, i32>::left("error".to_string());
        let result = either.run().await;
        assert_eq!(result, Err("error".to_string()));
    });
}

#[test]
fn test_either_t_async_fmap() {
    smol::block_on(async {
        let either = EitherTAsync::<String, i32>::right(21);
        let mapped = either.fmap(|x| x * 2);
        let result = mapped.run().await;
        assert_eq!(result, Ok(42));
    });
}

#[test]
fn test_either_t_async_flat_map() {
    smol::block_on(async {
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
    });
}

#[test]
fn test_either_t_async_handle_error() {
    smol::block_on(async {
        let either = EitherTAsync::<String, i32>::left("error".to_string());
        let handled = either.handle_error(|_| EitherTAsync::right(0));
        let result = handled.run().await;
        assert_eq!(result, Ok(0));
    });
}

// ============================================================================
// ScriptorAsync Tests under smol
// ============================================================================

#[test]
fn test_scriptor_async_tell() {
    smol::block_on(async {
        let writer = ScriptorAsync::<Vec<String>, ()>::tell(vec!["log1".to_string()]);
        let (logs, ()) = writer.run().await;
        assert_eq!(logs, vec!["log1".to_string()]);
    });
}

#[test]
fn test_scriptor_async_purus() {
    smol::block_on(async {
        let writer = ScriptorAsync::<Vec<String>, i32>::purus(42);
        let (logs, value): (Vec<String>, i32) = writer.run().await;
        assert!(logs.is_empty());
        assert_eq!(value, 42);
    });
}

#[test]
fn test_scriptor_async_flat_map() {
    smol::block_on(async {
        let writer = ScriptorAsync::<Vec<String>, i32>::tell(vec!["start".to_string()])
            .then(ScriptorAsync::purus(42))
            .flat_map(|v| {
                ScriptorAsync::<Vec<String>, i32>::tell(vec![format!("value: {}", v)])
                    .then(ScriptorAsync::purus(v))
            });

        let (logs, value) = writer.run().await;
        assert_eq!(logs, vec!["start".to_string(), "value: 42".to_string()]);
        assert_eq!(value, 42);
    });
}

// ============================================================================
// TraversableAsync Tests under smol
// ============================================================================

#[test]
fn test_traverse_vec_async() {
    smol::block_on(async {
        let vec = vec![1, 2, 3, 4, 5];
        let result = vec.traverse_async(|x| async move { x * 2 }).await;
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    });
}

#[test]
fn test_traverse_option_async() {
    smol::block_on(async {
        use ordofp::async_core::OptionTraverseAsync;

        let opt = Some(21);
        let result = opt.traverse_option_async(|x| async move { x * 2 }).await;
        assert_eq!(result, Some(42));

        let none: Option<i32> = None;
        let result2 = none.traverse_option_async(|x| async move { x * 2 }).await;
        assert_eq!(result2, None);
    });
}

#[test]
fn test_traverse_result_async() {
    smol::block_on(async {
        use ordofp::async_core::ResultTraverseAsync;

        let ok: Result<i32, String> = Ok(21);
        let result = ok.traverse_result_async(|x| async move { x * 2 }).await;
        assert_eq!(result, Ok(42));

        let err: Result<i32, String> = Err("error".to_string());
        let result2 = err.traverse_result_async(|x| async move { x * 2 }).await;
        assert_eq!(result2, Err("error".to_string()));
    });
}

// ============================================================================
// Concurrent Operations Tests (smol)
// ============================================================================

#[test]
fn test_concurrent_stream_processing() {
    smol::block_on(async {
        let stream = Flumen::from_iterator(0..10);
        let result = stream
            .fmap(|x| x * 2)
            .filter(|x| x % 4 == 0)
            .fold(0, |acc, x| acc + x)
            .await;

        // 0, 4, 8, 12, 16 = 40
        assert_eq!(result, 40);
    });
}

// ============================================================================
// Async Macro Tests under smol
// ============================================================================

use ordofp::{chain_async, compose_async, mdo_async, pipe_async};

#[test]
fn test_mdo_async_with_await() {
    smol::block_on(async {
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
    });
}

#[test]
fn test_pipe_async_smol() {
    smol::block_on(async {
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
    });
}

#[test]
fn test_compose_async_smol() {
    smol::block_on(async {
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
    });
}

#[test]
fn test_chain_async_smol() {
    smol::block_on(async {
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
    });
}
