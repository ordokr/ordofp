# OrdoFP Async Programming Guide

> *"Nunc fluens facit tempus, nunc stans facit aeternitatem."*
> — The flowing now makes time, the standing now makes eternity. (Boethius)

This guide provides a comprehensive introduction to asynchronous functional programming with OrdoFP 0.1.0. We'll explore how to compose async computations using monadic patterns, streams, and the new fiber-based concurrency system.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Core Async Types](#core-async-types)
3. [Fiber-Based Concurrency (v0.1.0)](#fiber-based-concurrency)
4. [Streams with Flumen](#streams-with-flumen)
5. [Async Type Classes](#async-type-classes)
6. [Async Monad Transformers](#async-monad-transformers)
7. [Async Macros](#async-macros)
8. [Runtime Integration](#runtime-integration)
9. [Effect System](#effect-system)
10. [Best Practices](#best-practices)
11. [Complete Example](#complete-example)

---

## Getting Started

### Feature Flags

OrdoFP's async support is opt-in via feature flags:

```toml
[dependencies]
# Core async support (no runtime dependency)
ordofp = { version = "0.1", features = ["async"] }

# With Tokio runtime
ordofp = { version = "0.1", features = ["tokio"] }

# With smol runtime (the legacy "async-std" feature is a back-compat alias for this)
ordofp = { version = "0.1", features = ["smol"] }
```

### Minimal Example

```rust
use ordofp::async_core::{Futurus, MonadAsync};

async fn example() {
    // Create a pure async value
    let computation = Futurus::purus(42);

    // Transform with fmap (Functor)
    let doubled = computation.fmap(|x| x * 2);

    // Chain with flat_map (Monad)
    let result = doubled
        .flat_map(|x| Futurus::purus(x + 1))
        .await;

    assert_eq!(result, 85); // (42 * 2) + 1
}
```

---

## Core Async Types

### Futurus — The Async Future Wrapper

`Futurus<T>` (Latin: "about to be") wraps async computations with monadic operations:

```rust
use ordofp::async_core::Futurus;

async fn futurus_examples() {
    // Create from a value (pure/return)
    let pure_value = Futurus::purus(42);

    // Create from an async block
    let async_value = Futurus::new(async {
        // Some async work...
        fetch_data().await
    });

    // Lazy creation with delay
    let delayed = Futurus::delay(|| async {
        expensive_computation().await
    });

    // Functor: map over the value
    let mapped = pure_value.fmap(|x| x * 2);

    // Monad: chain computations
    let chained = mapped.flat_map(|x| Futurus::purus(x + 1));

    // Applicative: combine two futures
    let a = Futurus::purus(10);
    let b = Futurus::purus(20);
    let sum = a.map2(b, |x, y| x + y);

    // Await the result
    let result = sum.await;
    assert_eq!(result, 30);
}
```

---

## Fiber-Based Concurrency

New in v0.1.0, `Fibra` provides lightweight, structured concurrency.

```rust
use ordofp::async_core::fibra::Fibra;

async fn fiber_example() {
    // Spawn lightweight fibers
    let f1 = Fibra::spawn(async { task1().await });
    let f2 = Fibra::spawn(async { task2().await });

    // Run in parallel and wait for both
    let (r1, r2) = Fibra::par(f1, f2).await;

    // Race two fibers (first to finish wins)
    let winner = Fibra::race(
        async { fast_task().await },
        async { slow_task().await }
    ).await;

    // Structured concurrency over a collection
    let results = Fibra::zip_par(vec![f1, f2, f3]).await;
}
```

---

## Streams with Flumen

`Flumen<T>` (Latin: "river, flowing stream") represents async sequences:

```rust
use ordofp::async_core::Flumen;

async fn flumen_examples() {
    // Create from an iterator
    let stream = Flumen::from_iter(vec![1, 2, 3, 4, 5]);

    // Functor: map over elements
    let doubled = stream.fmap(|x| x * 2);

    // Filter elements
    let even = doubled.filter(|x| x % 4 == 0);

    // FlatMap: expand each element
    let expanded = Flumen::from_iter(vec![1, 2])
        .flat_map(|x| Flumen::from_iter(vec![x, x * 10]));
    // Result: [1, 10, 2, 20]

    // Fold/reduce
    let sum = Flumen::from_iter(1..=5)
        .fold(0, |acc, x| acc + x)
        .await;
    assert_eq!(sum, 15);

    // Take and skip
    let middle = Flumen::from_iter(1..=10)
        .skip(2)
        .take(3)
        .collect_vec()
        .await;
    assert_eq!(middle, vec![3, 4, 5]);

    // Chain streams
    let s1 = Flumen::from_iter(vec![1, 2]);
    let s2 = Flumen::from_iter(vec![3, 4]);
    let combined = s1.chain(s2).collect_vec().await;
    assert_eq!(combined, vec![1, 2, 3, 4]);
}
```

### Stateful Stream Operations

**`scan` — running accumulation.** Produces intermediate accumulator values;
unlike `fold`, yields each state:

```rust
// Running sum
let running_sum = Flumen::from_iter(vec![1, 2, 3, 4])
    .scan(0, |acc, x| acc + x);
// Yields: [1, 3, 6, 10]

// Running product
let running_prod = Flumen::from_iter(vec![1, 2, 3, 4])
    .scan(1, |acc, x| acc * x);
// Yields: [1, 2, 6, 24]
```

**`scan_with` — stateful with early termination.** The state type can differ
from the output type; return `None` to terminate early:

```rust
// Take running sum until it exceeds 10
let bounded = Flumen::from_iter(vec![1, 2, 3, 4, 5, 6])
    .scan_with(0, |acc, x| {
        let new_acc = *acc + x;
        if new_acc > 10 {
            None  // Terminate
        } else {
            *acc = new_acc;
            Some(new_acc)
        }
    });
// Yields: [1, 3, 6, 10]
```

**`chunks` — windowed aggregation.** Collect items into fixed-size batches:

```rust
let windows = Flumen::from_iter(1..=10)
    .chunks(3);
// Yields: [[1, 2, 3], [4, 5, 6], [7, 8, 9], [10]]
```

---

## Async Type Classes

### FunctorAsync

Transform values inside async contexts:

```rust
use ordofp::async_core::FunctorAsync;

async fn functor_async_example() {
    // Option implements FunctorAsync
    let opt = Some(21);
    let doubled = FunctorAsync::fmap_async(opt, |x| async move { x * 2 }).await;
    assert_eq!(doubled, Some(42));

    // Result implements FunctorAsync
    let result: Result<i32, &str> = Ok(10);
    let mapped = FunctorAsync::fmap_async(result, |x| async move { x + 5 }).await;
    assert_eq!(mapped, Ok(15));

    // Vec implements FunctorAsync (processes first element with FnOnce)
    // For full Vec mapping, use FunctorAsyncMut
}
```

### MonadAsync

Chain async computations:

```rust
use ordofp::async_core::MonadAsync;

async fn monad_async_example() {
    // Chain Option computations
    let result = MonadAsync::flat_map_async(
        Some(5),
        |x| async move {
            if x > 0 { Some(x * 2) } else { None }
        }
    ).await;
    assert_eq!(result, Some(10));

    // Short-circuit on None
    let none_result = MonadAsync::flat_map_async(
        None::<i32>,
        |x| async move { Some(x * 2) }
    ).await;
    assert_eq!(none_result, None);
}
```

### ApplicatioAsync

Combine multiple async values:

```rust
use ordofp::async_core::ApplicatioAsync;

async fn applicative_async_example() {
    let a = Some(10);
    let b = Some(20);

    // Combine with an async function
    let sum = ApplicatioAsync::map2_async(a, b, |x, y| async move { x + y }).await;
    assert_eq!(sum, Some(30));

    // Short-circuits on None
    let partial = ApplicatioAsync::map2_async(Some(10), None::<i32>, |x, y| async move { x + y }).await;
    assert_eq!(partial, None);
}
```

### TraversableAsync

Transform collections with async operations:

```rust
use ordofp::async_core::TraversableAsync;

async fn traversable_async_example() {
    // Transform each element asynchronously
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled = numbers.traverse_async(|x| async move { x * 2 }).await;
    assert_eq!(doubled, vec![2, 4, 6, 8, 10]);

    // Useful for parallel-ish operations
    let urls = vec!["url1", "url2", "url3"];
    let responses = urls.traverse_async(|url| async move {
        fetch(url).await
    }).await;
}
```

---

## Async Monad Transformers

### LectorAsync — Environment/Configuration Reader

```rust
use ordofp::transformers::async_transforms::LectorAsync;

#[derive(Clone)]
struct AppConfig {
    api_url: String,
    timeout_ms: u64,
}

async fn lector_example() {
    // Ask for the entire environment
    let get_url = LectorAsync::<AppConfig, String>::ask()
        .fmap(|cfg| cfg.api_url.clone());

    // Ask for a specific field
    let get_timeout = LectorAsync::<AppConfig, u64>::asks(|cfg| cfg.timeout_ms);

    // Chain reader computations
    let fetch_with_config = LectorAsync::<AppConfig, String>::ask()
        .flat_map(|cfg| {
            let url = cfg.api_url.clone();
            LectorAsync::asks(move |cfg: &AppConfig| {
                format!("{}/api?timeout={}", url, cfg.timeout_ms)
            })
        });

    // Run with a config
    let config = AppConfig {
        api_url: "https://api.example.com".to_string(),
        timeout_ms: 5000,
    };

    let url = get_url.run(config.clone()).await;
    let full_url = fetch_with_config.run(config).await;
}
```

### StatusAsync — Stateful Computation

```rust
use ordofp::transformers::async_transforms::StatusAsync;

async fn status_example() {
    // Get current state
    let get_count = StatusAsync::<i32, i32>::get();

    // Set new state
    let set_ten = StatusAsync::<i32, ()>::put(10);

    // Modify state with a function
    let increment = StatusAsync::<i32, ()>::modify(|s| s + 1);
    let double = StatusAsync::<i32, ()>::modify(|s| s * 2);

    // Chain stateful operations
    let computation = StatusAsync::<i32, ()>::modify(|s| s * 2)
        .flat_map(|_| StatusAsync::<i32, ()>::modify(|s| s + 1))
        .flat_map(|_| StatusAsync::<i32, i32>::get());

    // Run with initial state
    let (final_state, value) = computation.run(10).await;
    assert_eq!(final_state, 21);  // (10 * 2) + 1
    assert_eq!(value, 21);

    // Just get final state
    let state = computation.clone().exec(10).await;

    // Just get final value
    let val = computation.eval(10).await;
}
```

### OptionTAsync — Optional Async Values

```rust
use ordofp::transformers::async_transforms::OptionTAsync;

async fn option_t_example() {
    // Create Some/None
    let some_val = OptionTAsync::some(42);
    let none_val = OptionTAsync::<i32>::none();

    // Transform the inner value
    let doubled = some_val.fmap(|x| x * 2);

    // Chain with short-circuit on None
    let result = OptionTAsync::some(10)
        .flat_map(|x| {
            if x > 5 {
                OptionTAsync::some(x * 2)
            } else {
                OptionTAsync::none()
            }
        });

    // Filter values
    let filtered = OptionTAsync::some(10).filter(|x| *x > 5);

    // Run to get Option<T>
    let opt = result.run().await;
    assert_eq!(opt, Some(20));
}
```

### EitherTAsync — Error Handling

```rust
use ordofp::transformers::async_transforms::EitherTAsync;

async fn either_t_example() {
    // Success case
    let ok_val = EitherTAsync::<String, i32>::right(42);

    // Error case
    let err_val = EitherTAsync::<String, i32>::left("error".to_string());

    // Transform success value
    let doubled = ok_val.fmap(|x| x * 2);

    // Transform error
    let mapped_err = EitherTAsync::<i32, String>::left(404)
        .map_err(|code| format!("Error: {}", code));

    // Chain with error short-circuit
    let validated = EitherTAsync::<String, i32>::right(10)
        .flat_map(|x| {
            if x > 0 {
                EitherTAsync::right(x * 2)
            } else {
                EitherTAsync::left("must be positive".to_string())
            }
        });

    // Handle errors
    let recovered = EitherTAsync::<String, i32>::left("oops".to_string())
        .handle_error(|_| EitherTAsync::right(0));

    // From Result
    let from_result: EitherTAsync<String, i32> = Ok(42).into();

    // Run to get Result<T, E>
    let result = validated.run().await;
    assert_eq!(result, Ok(20));
}
```

### ScriptorAsync — Logging/Accumulation

```rust
use ordofp::transformers::async_transforms::ScriptorAsync;

async fn scriptor_example() {
    // Write a log message (W = Vec<String>)
    let log_start = ScriptorAsync::<Vec<String>, ()>::tell(vec!["Starting...".to_string()]);

    // Pure value with no logs
    let pure_val = ScriptorAsync::<Vec<String>, i32>::purus(42);

    // Chain logging operations
    let computation = ScriptorAsync::<Vec<String>, i32>::tell(vec!["Step 1".to_string()])
        .then(ScriptorAsync::purus(42))
        .flat_map(|value| {
            ScriptorAsync::<Vec<String>, i32>::tell(vec![format!("Got: {}", value)])
                .then(ScriptorAsync::purus(value * 2))
        });

    // Run to get (logs, value)
    let (logs, result) = computation.run().await;
    assert_eq!(logs, vec!["Step 1", "Got: 42"]);
    assert_eq!(result, 84);

    // Just get logs
    let only_logs = computation.clone().exec().await;

    // Just get value
    let only_value = computation.eval().await;
}
```

---

## Async Macros

### mdo_async! — Do-Notation for Async

```rust
use ordofp::mdo_async;

async fn mdo_example() {
    async fn fetch_user(id: i32) -> String {
        format!("User {}", id)
    }

    async fn fetch_posts(user: &str) -> Vec<String> {
        vec![format!("{}'s post 1", user)]
    }

    let result = mdo_async! {
        let user_id = pure 42;
        let user = await fetch_user(user_id);
        let posts = await fetch_posts(&user);
        (user, posts)
    };

    // result = ("User 42", ["User 42's post 1"])
}
```

### pipe_async! — Left-to-Right Composition

```rust
use ordofp::pipe_async;

async fn pipe_example() {
    async fn add_one(x: i32) -> i32 { x + 1 }
    async fn double(x: i32) -> i32 { x * 2 }
    async fn to_string(x: i32) -> String { x.to_string() }

    // Pipe a value through async functions (left to right)
    let result = pipe_async!(10, add_one, double, to_string).await;
    // 10 -> 11 -> 22 -> "22"
    assert_eq!(result, "22");
}
```

### compose_async! — Right-to-Left Composition

```rust
use ordofp::compose_async;

async fn compose_example() {
    async fn add_one(x: i32) -> i32 { x + 1 }
    async fn double(x: i32) -> i32 { x * 2 }
    async fn subtract_three(x: i32) -> i32 { x - 3 }

    // Create a composed function (right to left: f(g(h(x))))
    let composed = compose_async!(add_one, double, subtract_three);

    let result = composed(10).await;
    // subtract_three(10) = 7
    // double(7) = 14
    // add_one(14) = 15
    assert_eq!(result, 15);
}
```

### chain_async! — Left-to-Right Function Chain

```rust
use ordofp::chain_async;

async fn chain_example() {
    async fn parse(s: &str) -> i32 { s.parse().unwrap() }
    async fn validate(x: i32) -> i32 { if x > 0 { x } else { 0 } }
    async fn process(x: i32) -> String { format!("Result: {}", x * 2) }

    // Create a chained function (left to right: h(g(f(x))))
    let pipeline = chain_async!(parse, validate, process);

    let result = pipeline("42").await;
    assert_eq!(result, "Result: 84");
}
```

---

## Runtime Integration

### Runtime Abstraction

Code can stay runtime-agnostic via the `RuntimeGenerare` spawning trait
(implementations: `TokioRuntime`, `SmolRuntime`; task handles are
`JoinManubrium`):

```rust
use ordofp_core::async_core::runtime::{RuntimeGenerare, TokioRuntime};

async fn run_tasks<R: RuntimeGenerare>(runtime: R) {
    let handle = runtime.spawn(async { computation() });
    let result = handle.await;
}
```

### Tokio

```rust
#[cfg(feature = "tokio")]
use ordofp::async_core::Futurus;

#[tokio::main]
async fn main() {
    let result = Futurus::purus(42)
        .fmap(|x| x * 2)
        .await;

    println!("Result: {}", result);
}

#[tokio::test]
async fn test_with_tokio() {
    use tokio::time::{sleep, Duration};

    async fn delayed_value(ms: u64, value: i32) -> i32 {
        sleep(Duration::from_millis(ms)).await;
        value
    }

    // Concurrent execution
    let (a, b) = tokio::join!(
        delayed_value(50, 10),
        delayed_value(50, 20)
    );

    assert_eq!(a + b, 30);
}
```

### smol

The `smol` feature integrates the smol runtime. (async-std is discontinued; the legacy `async-std` feature is kept only as a back-compat alias that selects the smol-backed runtime.)

```rust
#[cfg(feature = "smol")]
use ordofp::async_core::Futurus;

fn main() {
    smol::block_on(async {
        let result = Futurus::purus(42)
            .fmap(|x| x * 2)
            .await;

        println!("Result: {}", result);
    });
}

#[test]
fn test_with_smol() {
    use ordofp::async_core::Flumen;

    smol::block_on(async {
        let sum = Flumen::from_iter(1..=10)
            .filter(|x| x % 2 == 0)
            .fold(0, |acc, x| acc + x)
            .await;

        assert_eq!(sum, 30); // 2 + 4 + 6 + 8 + 10
    });
}
```

---

## Effect System

OrdoFP includes an effect system for tracking computational effects:

```rust
use ordofp::effects::{Effectus, EffectusHandler, IoEffectus, PurusEffectus};

// Mark effects on your computations
struct MyComputation<E: Effectus> {
    _effect: std::marker::PhantomData<E>,
}

// Pure computations (no side effects)
fn pure_computation() -> MyComputation<PurusEffectus> {
    MyComputation { _effect: std::marker::PhantomData }
}

// I/O computations
fn io_computation() -> MyComputation<IoEffectus> {
    MyComputation { _effect: std::marker::PhantomData }
}
```

---

## Best Practices

### 1. Prefer Explicit Type Annotations

```rust
// Good: Clear types help readability
let reader: LectorAsync<Config, String> = LectorAsync::asks(|c| c.url.clone());

// Less clear without annotation
let reader = LectorAsync::asks(|c: &Config| c.url.clone());
```

### 2. Use Transformers for Complex Workflows

```rust
// Instead of nested Options/Results
async fn complex_workflow() -> Option<Result<Data, Error>> {
    // ...messy nesting
}

// Use transformers
async fn clean_workflow() -> OptionTAsync<Result<Data, Error>> {
    OptionTAsync::some(initial_value)
        .flat_map(|v| process(v))
        .fmap(|v| transform(v))
}
```

### 3. Leverage Short-Circuit Semantics

```rust
// OptionTAsync short-circuits on None
let result = OptionTAsync::some(10)
    .flat_map(|x| validate(x))  // Returns None if invalid
    .flat_map(|x| process(x))   // Skipped if previous was None
    .flat_map(|x| save(x));     // Skipped if previous was None
```

### 4. Use mdo_async! for Readable Chains

```rust
// Instead of nested flat_maps
let result = a.flat_map(|x|
    b.flat_map(|y|
        c.fmap(|z| (x, y, z))
    )
);

// Use do-notation
let result = mdo_async! {
    let x = await a;
    let y = await b;
    let z = await c;
    (x, y, z)
};
```

---

## Complete Example

Here's a complete example combining multiple async patterns:

```rust
use ordofp::async_core::{Futurus, Flumen, TraversableAsync};
use ordofp::transformers::async_transforms::{
    LectorAsync, EitherTAsync, OptionTAsync
};
use ordofp::{mdo_async, pipe_async};

#[derive(Clone)]
struct AppConfig {
    api_base: String,
    max_retries: u32,
}

#[derive(Clone, Debug)]
struct User {
    id: i32,
    name: String,
}

// Simulated async operations
async fn fetch_user(id: i32) -> Result<User, String> {
    if id > 0 {
        Ok(User { id, name: format!("User{}", id) })
    } else {
        Err("Invalid user ID".to_string())
    }
}

async fn fetch_user_posts(user_id: i32) -> Vec<String> {
    vec![
        format!("Post 1 by user {}", user_id),
        format!("Post 2 by user {}", user_id),
    ]
}

#[tokio::main]
async fn main() {
    // Example 1: Using EitherTAsync for error handling
    let user_result = EitherTAsync::new(async { fetch_user(42).await })
        .fmap(|u| u.name.clone())
        .run()
        .await;

    println!("User: {:?}", user_result);

    // Example 2: Using LectorAsync for configuration
    let config = AppConfig {
        api_base: "https://api.example.com".to_string(),
        max_retries: 3,
    };

    let api_call = LectorAsync::<AppConfig, String>::asks(|cfg| cfg.api_base.clone())
        .fmap(|base| format!("{}/users", base));

    let url = api_call.run(config).await;
    println!("API URL: {}", url);

    // Example 3: Using TraversableAsync for batch operations
    let user_ids = vec![1, 2, 3];
    let users: Vec<Result<User, String>> = user_ids
        .traverse_async(|id| async move { fetch_user(id).await })
        .await;

    println!("Users: {:?}", users);

    // Example 4: Using Flumen for stream processing
    let post_counts = Flumen::from_iter(vec![1, 2, 3])
        .fmap(|id| fetch_user_posts(id))
        .fold(0, |acc, posts_future| async move {
            acc + posts_future.await.len()
        }.into())
        .await;

    // Example 5: Using mdo_async for readable composition
    let combined = mdo_async! {
        let user_id = pure 42;
        let user = await fetch_user(user_id);
        let posts = await match user {
            Ok(u) => fetch_user_posts(u.id),
            Err(_) => { async { vec![] }.await; vec![] }
        };
        (user, posts)
    };

    println!("Combined result: {:?}", combined);

    // Example 6: Using pipe_async for data transformation
    async fn to_upper(s: String) -> String { s.to_uppercase() }
    async fn add_prefix(s: String) -> String { format!("[USER] {}", s) }

    let formatted = pipe_async!(
        "alice".to_string(),
        to_upper,
        add_prefix
    ).await;

    println!("Formatted: {}", formatted);
}
```

---

## Further Reading

- [glossary.md](glossary.md) - Scholastic naming conventions
- [migration.md](migration.md) - lineage & version notes
- `cargo doc --open` - Full API reference (docs.rs hosting begins only if the crate is published)

---

*"Finis coronat opus."* — The end crowns the work.
