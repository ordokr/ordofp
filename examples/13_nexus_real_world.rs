//! Nexus Effect System - Real-World Integration Patterns
//!
//! Advanced examples showing how to use Nexus effects in realistic scenarios.
//!
//! # Running
//!
//! ```bash
//! cargo run --example 13_nexus_real_world --features "nexus,std"
//! ```

#[cfg(feature = "nexus")]
use ordofp_core::nexus::prelude::*;

#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::region::with_region;

#[cfg(feature = "nexus")]
use std::collections::HashMap;

// =============================================================================
// Pattern 1: Service Layer with Dependency Injection
// =============================================================================

#[cfg(feature = "nexus")]
mod service_layer {
    use super::ReaderComputation;

    /// Application dependencies injected via Reader effect
    #[derive(Clone)]
    pub struct ServiceContext {
        pub db_connection: String,
        pub cache_enabled: bool,
        pub max_retries: i32,
        pub timeout_ms: i32,
    }

    /// User repository using Reader for config access
    pub struct UserRepository;

    impl UserRepository {
        /// Returns a computation that looks up a user by `id`, consulting the cache when
        /// `ServiceContext::cache_enabled` is set and reading credentials from the ambient context.
        /// Succeeds for IDs in the range 1–99 and returns an error for any other value.
        pub fn find_by_id(id: i32) -> ReaderComputation<ServiceContext, Result<String, String>> {
            ReaderComputation::asks(move |ctx: &ServiceContext| {
                // Simulate database lookup using connection from context
                if ctx.cache_enabled {
                    println!("      [Cache] Checking cache for user {id}");
                }
                println!(
                    "      [DB:{}] Finding user {} (timeout: {}ms)",
                    ctx.db_connection, id, ctx.timeout_ms
                );

                if id > 0 && id < 100 {
                    Ok(format!("User_{id}"))
                } else {
                    Err(format!("User {id} not found"))
                }
            })
        }

        /// Persists a new user record and returns its assigned ID.
        ///
        /// Builds a [`ReaderComputation`] that writes a row to the database identified by
        /// `ServiceContext::db_connection`, logging the operation along with the configured
        /// retry limit. On success the computation resolves to `Ok(id)` where `id` is the
        /// newly assigned row identifier; on failure it resolves to `Err(message)`.
        ///
        /// In this demonstration the save always succeeds and returns the fixed ID `42`.
        pub fn save(name: &str) -> ReaderComputation<ServiceContext, Result<i32, String>> {
            let name = name.to_string();
            ReaderComputation::asks(move |ctx: &ServiceContext| {
                println!(
                    "      [DB:{}] Saving user {} (retries: {})",
                    ctx.db_connection, name, ctx.max_retries
                );
                Ok(42) // Return new ID
            })
        }
    }

    /// Order service composing multiple repository calls
    pub struct OrderService;

    impl OrderService {
        pub fn create_order(
            user_id: i32,
            items: Vec<String>,
        ) -> ReaderComputation<ServiceContext, Result<String, String>> {
            UserRepository::find_by_id(user_id).and_then(move |user_result| {
                ReaderComputation::asks(move |_ctx: &ServiceContext| match user_result {
                    Ok(user) => {
                        println!(
                            "      [Order] Creating order for {} with {} items",
                            user,
                            items.len()
                        );
                        Ok(format!("ORDER-{}-{}", user_id, items.len()))
                    }
                    Err(e) => Err(format!("Cannot create order: {e}")),
                })
            })
        }
    }

    pub fn run_example() {
        println!("=== Pattern 1: Service Layer with DI ===\n");

        let ctx = ServiceContext {
            db_connection: "postgres://localhost/orders".to_string(),
            cache_enabled: true,
            max_retries: 3,
            timeout_ms: 5000,
        };

        // Create an order
        let order_computation =
            OrderService::create_order(42, vec!["Widget".to_string(), "Gadget".to_string()]);

        println!("  Creating order:");
        match order_computation.run(&ctx) {
            Ok(order_id) => println!("    Success: {order_id}"),
            Err(e) => println!("    Error: {e}"),
        }

        // Save a user
        let save_computation = UserRepository::save("Bob");
        let _ = save_computation.run(&ctx);

        // Try with invalid user
        println!("\n  Creating order with invalid user:");
        let invalid_order = OrderService::create_order(999, vec!["Item".to_string()]);
        match invalid_order.run(&ctx) {
            Ok(order_id) => println!("    Success: {order_id}"),
            Err(e) => println!("    Error: {e}"),
        }

        println!();
    }
}

// =============================================================================
// Pattern 2: Pipeline Processing with Error Accumulation
// =============================================================================

#[cfg(feature = "nexus")]
mod pipeline_processing {
    use super::StatefulComputation;

    #[derive(Debug, Clone)]
    pub struct DataRecord {
        pub id: i32,
        pub value: String,
        pub score: i32,
    }

    #[derive(Debug)]
    pub struct ProcessingResult {
        pub processed: Vec<DataRecord>,
        pub errors: Vec<String>,
        pub stats: ProcessingStats,
    }

    #[derive(Debug, Default, Clone)]
    pub struct ProcessingStats {
        pub total: i32,
        pub valid: i32,
        pub invalid: i32,
        pub transformed: i32,
    }

    /// Process a batch of records using State for stats and Writer for errors
    pub fn process_batch(records: Vec<DataRecord>) -> (ProcessingResult, ProcessingStats) {
        let computation =
            StatefulComputation::<ProcessingStats, (Vec<DataRecord>, Vec<String>)>::new(
                |initial_stats: ProcessingStats| {
                    let mut stats = initial_stats;
                    let mut processed = Vec::new();
                    let mut errors = Vec::new();

                    for record in records {
                        stats.total += 1;

                        // Validate
                        if record.score < 0 {
                            stats.invalid += 1;
                            errors.push(format!("Record {}: negative score", record.id));
                            continue;
                        }

                        if record.value.is_empty() {
                            stats.invalid += 1;
                            errors.push(format!("Record {}: empty value", record.id));
                            continue;
                        }

                        stats.valid += 1;

                        // Transform
                        let transformed = DataRecord {
                            id: record.id,
                            value: record.value.to_uppercase(),
                            score: record.score * 2,
                        };
                        stats.transformed += 1;
                        processed.push(transformed);
                    }

                    ((processed, errors), stats)
                },
            );

        let ((processed, errors), stats) = computation.run(ProcessingStats::default());

        (
            ProcessingResult {
                processed,
                errors,
                stats: stats.clone(),
            },
            stats,
        )
    }

    pub fn run_example() {
        println!("=== Pattern 2: Pipeline Processing ===\n");

        let records = vec![
            DataRecord {
                id: 1,
                value: "apple".to_string(),
                score: 10,
            },
            DataRecord {
                id: 2,
                value: "banana".to_string(),
                score: 20,
            },
            DataRecord {
                id: 3,
                value: String::new(),
                score: 15,
            }, // Invalid: empty
            DataRecord {
                id: 4,
                value: "cherry".to_string(),
                score: -5,
            }, // Invalid: negative
            DataRecord {
                id: 5,
                value: "date".to_string(),
                score: 30,
            },
        ];

        println!("  Input records: {}", records.len());

        let (result, _stats) = process_batch(records);

        println!("\n  Processing results:");
        println!("    Stats: {:?}", result.stats);
        println!("    Processed: {:?}", result.processed);
        println!("    Errors: {:?}", result.errors);
        println!();
    }
}

// =============================================================================
// Pattern 3: AST Construction with Region Allocation
// =============================================================================

#[cfg(feature = "nexus")]
mod ast_construction {
    use super::{HashMap, with_region};

    /// Simple expression AST
    #[derive(Debug)]
    pub enum Expr<'a> {
        Num(i32),
        Var(&'a str),
        Add(&'a Expr<'a>, &'a Expr<'a>),
        Mul(&'a Expr<'a>, &'a Expr<'a>),
    }

    impl Expr<'_> {
        /// Evaluate the expression with a variable environment
        pub fn eval(&self, env: &HashMap<&str, i32>) -> i32 {
            match self {
                Expr::Num(n) => *n,
                Expr::Var(name) => *env.get(name).unwrap_or(&0),
                Expr::Add(a, b) => a.eval(env) + b.eval(env),
                Expr::Mul(a, b) => a.eval(env) * b.eval(env),
            }
        }
    }

    pub fn run_example() {
        println!("=== Pattern 3: AST with Region Allocation ===\n");

        // Build and evaluate AST using region allocation
        let result = with_region(|region| {
            // Build: (x + 10) * (y + 5)
            let x = region.alloc(Expr::Var(region.alloc_str("x")));
            let ten = region.alloc(Expr::Num(10));
            let y = region.alloc(Expr::Var(region.alloc_str("y")));
            let five = region.alloc(Expr::Num(5));

            let x_plus_10 = region.alloc(Expr::Add(x, ten));
            let y_plus_5 = region.alloc(Expr::Add(y, five));
            let expr = region.alloc(Expr::Mul(x_plus_10, y_plus_5));

            println!("  Built expression: (x + 10) * (y + 5)");
            println!("  Region stats: {} allocations", region.allocation_count());

            // Evaluate with different environments
            let mut env1 = HashMap::new();
            env1.insert("x", 3);
            env1.insert("y", 7);

            let mut env2 = HashMap::new();
            env2.insert("x", 10);
            env2.insert("y", 20);

            let r1 = expr.eval(&env1);
            let r2 = expr.eval(&env2);

            println!("  Evaluation:");
            println!("    x=3, y=7  → (3+10)*(7+5) = {r1}");
            println!("    x=10, y=20 → (10+10)*(20+5) = {r2}");

            (r1, r2)
        });
        // Region memory freed here

        println!("  Results (after region freed): {result:?}");
        println!();
    }
}

// =============================================================================
// Pattern 4: State Machine with Session Types
// =============================================================================

#[cfg(feature = "nexus")]
mod state_machine {
    use super::StatefulComputation;

    /// Order states
    ///
    /// (The "approved" state is named `Approbatus` to avoid colliding with
    /// the library's `Probatum` validation type.)
    #[derive(Debug, Clone, PartialEq)]
    pub enum OrderState {
        Created,
        Approbatus,
        Paid,
        Shipped,
        _Delivered,
        _Cancelled,
    }

    /// Order with state tracking
    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: String,
        pub state: OrderState,
        pub items: Vec<String>,
        pub total: i32,
        pub history: Vec<String>,
    }

    impl Order {
        pub fn new(id: &str, items: Vec<String>, total: i32) -> Self {
            Order {
                id: id.to_string(),
                state: OrderState::Created,
                items,
                total,
                history: vec!["Order created".to_string()],
            }
        }
    }

    /// State transitions using State effect
    pub fn validate_order() -> StatefulComputation<Order, Result<(), String>> {
        StatefulComputation::new(|order: Order| {
            let mut o = order;

            if o.state != OrderState::Created {
                return (Err("Can only validate created orders".to_string()), o);
            }

            if o.items.is_empty() {
                return (Err("Order has no items".to_string()), o);
            }

            o.state = OrderState::Approbatus;
            o.history.push("Order approved".to_string());
            (Ok(()), o)
        })
    }

    /// Processes payment for an order in the `Approbatus` (approved) state.
    ///
    /// Returns a [`StatefulComputation`] that transitions the [`Order`] from
    /// [`OrderState::Approbatus`] to [`OrderState::Paid`], recording the payment
    /// amount in the order history.  Fails with an error if the order is not
    /// currently in the `Approbatus` state.
    pub fn process_payment() -> StatefulComputation<Order, Result<(), String>> {
        StatefulComputation::new(|order: Order| {
            let mut o = order;

            if o.state != OrderState::Approbatus {
                return (Err("Can only pay for approved orders".to_string()), o);
            }

            o.state = OrderState::Paid;
            o.history.push(format!("Payment processed: ${}", o.total));
            (Ok(()), o)
        })
    }

    pub fn ship_order() -> StatefulComputation<Order, Result<(), String>> {
        StatefulComputation::new(|order: Order| {
            let mut o = order;

            if o.state != OrderState::Paid {
                return (Err("Can only ship paid orders".to_string()), o);
            }

            o.state = OrderState::Shipped;
            o.history.push("Order shipped".to_string());
            (Ok(()), o)
        })
    }

    pub fn run_example() {
        println!("=== Pattern 4: State Machine ===\n");

        let order = Order::new(
            "ORD-001",
            vec!["Widget".to_string(), "Gadget".to_string()],
            99,
        );
        println!("  Initial order: {} (State: {:?})", order.id, order.state);

        // Chain state transitions
        let workflow = validate_order()
            .and_then(|result| {
                StatefulComputation::new(move |o| match result {
                    Ok(()) => process_payment().run(o),
                    Err(e) => (Err(e), o),
                })
            })
            .and_then(|result| {
                StatefulComputation::new(move |o| match result {
                    Ok(()) => ship_order().run(o),
                    Err(e) => (Err(e), o),
                })
            });

        let (result, final_order) = workflow.run(order);

        println!("\n  Workflow result: {result:?}");
        println!("  Final state: {:?}", final_order.state);
        println!("  History:");
        for entry in &final_order.history {
            println!("    - {entry}");
        }
        println!();
    }
}

// =============================================================================
// Pattern 5: Retry Logic with Error Recovery
// =============================================================================

#[cfg(feature = "nexus")]
mod retry_pattern {
    use super::ErrorComputation;

    /// Simulate an operation that may fail
    fn flaky_operation(attempt: i32) -> ErrorComputation<String, String> {
        if attempt < 3 {
            ErrorComputation::err(format!("Attempt {attempt} failed: connection timeout"))
        } else {
            ErrorComputation::ok(format!("Success on attempt {attempt}"))
        }
    }

    /// Retry wrapper using Error effect
    pub fn with_retry<A: Clone + 'static>(
        operation: impl Fn(i32) -> ErrorComputation<String, A> + 'static,
        max_attempts: i32,
    ) -> ErrorComputation<String, A> {
        let mut last_error = String::new();

        for attempt in 1..=max_attempts {
            match operation(attempt).run() {
                Ok(result) => return ErrorComputation::ok(result),
                Err(e) => {
                    last_error = e;
                    println!("      Retry {attempt}/{max_attempts}: {last_error}");
                }
            }
        }

        ErrorComputation::err(format!(
            "All {max_attempts} attempts failed. Last error: {last_error}"
        ))
    }

    /// Fallback pattern
    pub fn with_fallback<A: Clone + 'static>(
        primary: ErrorComputation<String, A>,
        fallback: impl FnOnce() -> A + 'static,
    ) -> ErrorComputation<String, A> {
        match primary.run() {
            Ok(result) => ErrorComputation::ok(result),
            Err(_) => ErrorComputation::ok(fallback()),
        }
    }

    pub fn run_example() {
        println!("=== Pattern 5: Retry Logic ===\n");

        // Retry until success
        println!("  Operation with retry (max 5 attempts):");
        let result = with_retry(flaky_operation, 5);
        match result.run() {
            Ok(msg) => println!("    Final: {msg}"),
            Err(e) => println!("    Failed: {e}"),
        }

        // With fallback
        println!("\n  Operation with fallback:");
        let failing_op = ErrorComputation::<String, String>::err("Primary failed".to_string());
        let with_fb = with_fallback(failing_op, || "Fallback value".to_string());
        match with_fb.run() {
            Ok(msg) => println!("    Result: {msg}"),
            Err(e) => println!("    Error: {e}"),
        }

        println!();
    }
}

// =============================================================================
// Pattern 6: Incremental Computation Cache
// =============================================================================

#[cfg(feature = "nexus")]
mod incremental_cache {
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// Simple memoization cache using State effect
    pub struct MemoCache {
        cache: RefCell<HashMap<String, i32>>,
        hits: RefCell<i32>,
        misses: RefCell<i32>,
    }

    impl MemoCache {
        pub fn new() -> Self {
            MemoCache {
                cache: RefCell::new(HashMap::new()),
                hits: RefCell::new(0),
                misses: RefCell::new(0),
            }
        }

        pub fn get_or_compute<F: FnOnce() -> i32>(&self, key: &str, compute: F) -> i32 {
            if let Some(&value) = self.cache.borrow().get(key) {
                *self.hits.borrow_mut() += 1;
                return value;
            }

            *self.misses.borrow_mut() += 1;
            let value = compute();
            self.cache.borrow_mut().insert(key.to_string(), value);
            value
        }

        pub fn stats(&self) -> (i32, i32) {
            (*self.hits.borrow(), *self.misses.borrow())
        }
    }

    /// Expensive fibonacci computation
    fn fib(n: i32, cache: &MemoCache) -> i32 {
        if n <= 1 {
            return n;
        }

        cache.get_or_compute(&format!("fib_{n}"), || {
            fib(n - 1, cache) + fib(n - 2, cache)
        })
    }

    pub fn run_example() {
        println!("=== Pattern 6: Incremental/Memoized Computation ===\n");

        let cache = MemoCache::new();

        println!("  Computing Fibonacci sequence with memoization:");
        for n in &[10, 20, 25, 30] {
            let result = fib(*n, &cache);
            let (hits, misses) = cache.stats();
            println!("    fib({n}) = {result} (cache: {hits} hits, {misses} misses)");
        }

        // Recompute - should hit cache
        println!("\n  Recomputing (should use cache):");
        let result = fib(30, &cache);
        let (hits, misses) = cache.stats();
        println!("    fib(30) = {result} (cache: {hits} hits, {misses} misses)");

        println!();
    }
}

// =============================================================================
// Main
// =============================================================================

#[cfg(feature = "nexus")]
fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║      OrdoFP Nexus - Real-World Integration Patterns           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    service_layer::run_example();
    pipeline_processing::run_example();
    ast_construction::run_example();
    state_machine::run_example();
    retry_pattern::run_example();
    incremental_cache::run_example();

    println!("═══════════════════════════════════════════════════════════════");
    println!("All patterns demonstrated!");
    println!();
    println!("Summary of patterns:");
    println!("  1. Service Layer: Reader for dependency injection");
    println!("  2. Pipeline: State for stats, accumulate errors");
    println!("  3. AST Builder: Region for temporary allocations");
    println!("  4. State Machine: State for order workflow");
    println!("  5. Retry Logic: Error with recovery strategies");
    println!("  6. Memoization: Incremental computation caching");
}

#[cfg(not(feature = "nexus"))]
fn main() {
    println!("This example requires the 'nexus' feature.");
    println!("Run with: cargo run --example 13_nexus_real_world --features \"nexus,std\"");
}
