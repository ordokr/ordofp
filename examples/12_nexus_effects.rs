//! Nexus Effect System Examples
//!
//! Comprehensive examples demonstrating `OrdoFP`'s Nexus effect system.
//!
//! # Running
//!
//! ```bash
//! cargo run --example 12_nexus_effects --features "nexus,std"
//! ```
//!
//! # What is Nexus?
//!
//! Nexus is `OrdoFP`'s type-level effect system that provides:
//! - Zero-cost abstractions for common effect patterns
//! - Type-safe effect tracking via const-Universalis bitmasks
//! - Composable effect handlers
//! - Advanced effects: Region memory, Checkpointing, Sessions, Probabilistic

#[cfg(feature = "nexus")]
use ordofp_core::nexus::prelude::*;

#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::region::{RegionVec, with_region};

#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::checkpoint::{
    CheckpointContext, ResumableComputation, StepResult,
};

// =============================================================================
// Example 1: State Effect - Counter with History
// =============================================================================

#[cfg(feature = "nexus")]
fn example_state_counter() {
    println!("=== Example 1: State Effect ===\n");

    // Simple counter that tracks its history
    let computation = StatefulComputation::<(i32, Vec<i32>), i32>::new(|(count, history)| {
        let mut c = count;
        let mut h = history;

        // Perform several operations
        c += 1;
        h.push(c);
        c *= 2;
        h.push(c);
        c += 10;
        h.push(c);

        (c, (c, h))
    });

    let (result, (_final_count, history)) = computation.run((0, Vec::new()));

    println!("  Counter computation:");
    println!("    Initial: 0");
    println!("    +1 → *2 → +10");
    println!("    Final result: {result}");
    println!("    History: {history:?}");
    println!();
}

// =============================================================================
// Example 2: Reader Effect - Configuration
// =============================================================================

#[cfg(feature = "nexus")]
fn example_reader_config() {
    println!("=== Example 2: Reader Effect ===\n");

    #[derive(Clone)]
    struct AppConfig {
        database_url: String,
        max_connections: i32,
        timeout_ms: i32,
    }

    let config = AppConfig {
        database_url: "postgres://localhost/mydb".to_string(),
        max_connections: 10,
        timeout_ms: 5000,
    };

    // Build a connection string using config
    let build_connection = ReaderComputation::<AppConfig, String>::asks(|c: &AppConfig| {
        format!(
            "{}?pool_size={}&timeout={}",
            c.database_url, c.max_connections, c.timeout_ms
        )
    });

    // Calculate total timeout with retries
    let _total_timeout = ReaderComputation::<AppConfig, i32>::asks(|c: &AppConfig| {
        c.timeout_ms * 3 // 3 retries
    });

    // Combine computations
    let combined = build_connection.and_then(|conn_str| {
        ReaderComputation::asks(move |c: &AppConfig| {
            format!("{} (max timeout: {}ms)", conn_str, c.timeout_ms * 3)
        })
    });

    let result = combined.run(&config);
    println!("  Config-based computation:");
    println!("    Connection: {result}");
    println!();
}

// =============================================================================
// Example 3: Error Effect - Validation Pipeline
// =============================================================================

#[cfg(feature = "nexus")]
fn example_error_validation() {
    println!("=== Example 3: Error Effect ===\n");

    #[derive(Debug, Clone)]
    struct User {
        name: String,
        email: String,
        age: i32,
    }

    fn validate_name(name: &str) -> ErrorComputation<String, String> {
        if name.len() >= 2 {
            ErrorComputation::ok(name.to_string())
        } else {
            ErrorComputation::err("Name must be at least 2 characters".to_string())
        }
    }

    fn validate_email(email: &str) -> ErrorComputation<String, String> {
        if email.contains('@') {
            ErrorComputation::ok(email.to_string())
        } else {
            ErrorComputation::err("Email must contain @".to_string())
        }
    }

    fn validate_age(age: i32) -> ErrorComputation<String, i32> {
        if (18..=120).contains(&age) {
            ErrorComputation::ok(age)
        } else {
            ErrorComputation::err("Age must be between 18 and 120".to_string())
        }
    }

    // Valid user
    let valid_result = validate_name("Alice").and_then(|name| {
        validate_email("alice@example.com").and_then(move |email| {
            validate_age(25).map(move |age| User {
                name: name.clone(),
                email,
                age,
            })
        })
    });

    println!("  Validation pipeline:");
    match valid_result.run() {
        Ok(user) => println!(
            "    Valid user: {} ({}), age {}",
            user.name, user.email, user.age
        ),
        Err(e) => println!("    Validation error: {e}"),
    }

    // Invalid user
    let invalid_result = validate_name("A").and_then(|name| {
        validate_email("invalid")
            .and_then(move |email| validate_age(15).map(move |age| User { name, email, age }))
    });

    match invalid_result.run() {
        Ok(user) => println!("    Valid user: {user:?}"),
        Err(e) => println!("    Validation error: {e}"),
    }
    println!();
}

// =============================================================================
// Example 4: Writer Effect - Logging
// =============================================================================

#[cfg(feature = "nexus")]
fn example_writer_logging() {
    println!("=== Example 4: Writer Effect ===\n");

    fn process_item(item: i32) -> WriterComputation<Vec<String>, i32> {
        WriterComputation::tell(vec![format!("Processing item: {}", item)]).and_then(move |()| {
            let result = item * 2;
            WriterComputation::tell(vec![format!("Result: {}", result)]).map(move |()| result)
        })
    }

    let computation = process_item(5).and_then(|r1| {
        process_item(r1).and_then(move |r2| {
            WriterComputation::tell(vec![format!("Final: {}", r1 + r2)]).map(move |()| r1 + r2)
        })
    });

    let (result, log) = computation.run();

    println!("  Logged computation:");
    println!("    Result: {result}");
    println!("    Log:");
    for entry in log {
        println!("      - {entry}");
    }
    println!();
}

// =============================================================================
// Example 5: IO Effect - Side Effects
// =============================================================================

#[cfg(feature = "nexus")]
fn example_io_effects() {
    println!("=== Example 5: IO Effect ===\n");

    // IO computation that simulates reading and processing
    let read_and_process = IoComputation::new(|| {
        // Simulate reading a value
        42
    })
    .map(|x| {
        // Process it
        x * 2
    })
    .and_then(|x| {
        IoComputation::new(move || {
            // Simulate writing
            println!("    [IO] Writing result: {x}");
            x
        })
    });

    println!("  IO computation:");
    let result = read_and_process.run();
    println!("    Final result: {result}");
    println!();
}

// =============================================================================
// Example 6: Region Effect - Scoped Memory
// =============================================================================

#[cfg(feature = "nexus")]
fn example_region_memory() {
    println!("=== Example 6: Region Effect ===\n");

    // Allocate temporary data in a region
    let result = with_region(|region| {
        // Allocate some values
        let x = region.alloc(42);
        let y = region.alloc(58);

        // Allocate a string
        let msg = region.alloc_str("Hello from region!");

        // Allocate a slice
        let numbers = region.alloc_slice(&[1, 2, 3, 4, 5]);

        // Use a region vector
        let mut vec = RegionVec::<i32>::with_capacity(region, 10);
        vec.push(10);
        vec.push(20);
        vec.push(30);

        println!("    Region allocations:");
        println!("      x = {x}, y = {y}");
        println!("      msg = \"{msg}\"");
        println!("      numbers = {numbers:?}");
        println!("      vec sum = {}", vec.iter().sum::<i32>());
        println!(
            "      Stats: {} allocations, {} bytes",
            region.allocation_count(),
            region.bytes_allocated()
        );

        // Return computed value (region memory freed after this)
        *x + *y + vec.iter().sum::<i32>()
    });

    println!("    Result (after region freed): {result}");
    println!();
}

// =============================================================================
// Example 7: Checkpoint Effect - Resumable Computation
// =============================================================================

#[cfg(feature = "nexus")]
fn example_checkpoint_resumable() {
    println!("=== Example 7: Checkpoint Effect ===\n");

    // Create a resumable computation with (current, target) state
    let step = |state: &(i32, i32)| {
        let (current, target) = *state;
        if current >= target {
            StepResult::Done(current)
        } else if current % 5 == 0 && current > 0 {
            // Checkpoint every 5 steps
            StepResult::Checkpoint((current + 1, target))
        } else {
            StepResult::Continue((current + 1, target))
        }
    };

    let computation = ResumableComputation::new((0i32, 15i32), step, "counter");
    let mut ctx = CheckpointContext::new();

    println!("  Resumable computation:");
    println!("    Running computation from 0 to 15...");

    let result = computation.run(&mut ctx);

    println!("    Final result: {result}");
    println!(
        "    Checkpoints created: {}",
        ctx.stats().checkpoints_created
    );
    println!();
}

// =============================================================================
// Example 8: Probabilistic Effect - Placeholder
// =============================================================================

// Note: Full probabilistic programming example requires the ordofp_bayes crate.
// See ordofp_bayes/examples/simple_inference.rs for probabilistic programming.

// =============================================================================
// Example 9: Combined Effects - Transaction Simulation
// =============================================================================

#[cfg(feature = "nexus")]
fn example_combined_transaction() {
    println!("=== Example 9: Combined Effects ===\n");

    #[derive(Clone)]
    struct Account {
        id: String,
        balance: i32,
    }

    // Simulate a transaction with state + error handling
    let transaction = StatefulComputation::<Account, Result<String, String>>::new(|account| {
        let mut acc = account;

        // Check balance
        if acc.balance < 100 {
            return (Err("Insufficient funds".to_string()), acc);
        }

        // Withdraw
        acc.balance -= 100;

        // Simulate processing fee
        acc.balance -= 5;

        // Deposit bonus
        acc.balance += 10;

        let msg = format!("Transaction complete. New balance: {}", acc.balance);
        (Ok(msg), acc)
    });

    let initial_account = Account {
        id: "ACC001".to_string(),
        balance: 500,
    };

    println!("  Transaction simulation:");
    println!("    Initial balance: {}", initial_account.balance);

    let (result, final_account) = transaction.run(initial_account);

    match result {
        Ok(msg) => println!("    Success: {} (Account: {})", msg, final_account.id),
        Err(e) => println!("    Error: {e}"),
    }
    println!("    Final balance: {}", final_account.balance);
    println!();
}

// =============================================================================
// Example 10: Real-World Pattern - Parser with Region Allocation
// =============================================================================

#[cfg(feature = "nexus")]
fn example_parser_pattern() {
    println!("=== Example 10: Parser Pattern ===\n");

    #[derive(Debug)]
    enum Token<'a> {
        Identifier(&'a str),
        Number(i32),
        Operator(&'a str),
    }

    // Parse a simple expression using region allocation
    let tokens = with_region(|region| {
        let input = "x + 42 * y";

        // Allocate tokens in the region
        let mut tokens = RegionVec::<Token>::with_capacity(region, 10);

        for part in input.split_whitespace() {
            let token = if let Ok(n) = part.parse::<i32>() {
                Token::Number(n)
            } else if part == "+" || part == "*" || part == "-" || part == "/" {
                Token::Operator(region.alloc_str(part))
            } else {
                Token::Identifier(region.alloc_str(part))
            };
            tokens.push(token);
        }

        println!("    Input: \"{input}\"");
        println!("    Tokens:");
        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::Identifier(s) => println!("      {i}: Identifier({s})"),
                Token::Number(n) => println!("      {i}: Number({n})"),
                Token::Operator(o) => println!("      {i}: Operator({o})"),
            }
        }

        // Return token count (tokens freed after scope)
        tokens.len()
    });

    println!("    Parsed {tokens} tokens (region memory freed)");
    println!();
}

// =============================================================================
// Main
// =============================================================================

#[cfg(feature = "nexus")]
fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║         OrdoFP Nexus Effect System - Examples                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    example_state_counter();
    example_reader_config();
    example_error_validation();
    example_writer_logging();
    example_io_effects();
    example_region_memory();
    example_checkpoint_resumable();
    example_combined_transaction();
    example_parser_pattern();

    println!("═══════════════════════════════════════════════════════════════");
    println!("All examples completed!");
    println!();
    println!("Key takeaways:");
    println!("  • State: Thread mutable state through computations");
    println!("  • Reader: Access read-only configuration");
    println!("  • Error: Handle failures with short-circuit semantics");
    println!("  • Writer: Accumulate logs/output");
    println!("  • IO: Encapsulate side effects");
    println!("  • Region: Scoped memory management");
    println!("  • Checkpoint: Suspendable/resumable computations");
    println!("  • Probabilistic: Sampling and Bayesian inference");
    println!();
    println!("For more details, see: core/src/nexus/effects/");
}

#[cfg(not(feature = "nexus"))]
fn main() {
    println!("This example requires the 'nexus' feature.");
    println!("Run with: cargo run --example 12_nexus_effects --features \"nexus,std\"");
}
