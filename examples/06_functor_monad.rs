//! Example 06: Functors, Applicatives, and Monads
//!
//! This example demonstrates `OrdoFP`'s GAT-based type class implementations
//! for functional programming patterns.
//!
//! Run with: `cargo run --example 06_functor_monad`

use ordofp::prelude::*;
use ordofp::typeclasses::Apply;

fn main() {
    println!("--- Example 06: Functors, Applicatives, and Monads ---\n");

    // =========================================
    // Part 1: Functor - Mapping over containers
    // =========================================
    println!("=== Part 1: Functor (map) ===");

    // Option as Functor
    let opt_value = Some(5);
    let doubled = opt_value.map(|x| x * 2);
    println!("Option: Some(5).map(x * 2) = {doubled:?}");

    let none_value: Option<i32> = None;
    let doubled_none = none_value.map(|x| x * 2);
    println!("Option: None.map(x * 2) = {doubled_none:?}");

    // Result as Functor
    let ok_result: Result<i32, &str> = Ok(10);
    let transformed = ok_result.map(|x| format!("Value: {x}"));
    println!("Result: Ok(10).map(format) = {transformed:?}");

    let err_result: Result<i32, &str> = Err("error occurred");
    let transformed_err = err_result.map(|x| format!("Value: {x}"));
    println!("Result: Err.map(format) = {transformed_err:?}");

    // Vec as Functor
    let numbers = vec![1, 2, 3, 4, 5];
    let squared: Vec<i32> = numbers.map(|x| x * x);
    println!("Vec: [1,2,3,4,5].map(x * x) = {squared:?}");

    println!();

    // =========================================
    // Part 2: Applicative - Applying wrapped functions
    // =========================================
    println!("=== Part 2: Applicative (apply) ===");

    // Option: apply a wrapped function
    let add_ten = Some(|x: i32| x + 10);
    let value = Some(32);
    let result = value.apply(add_ten);
    println!("Option: Some(32).apply(Some(|x| x + 10)) = {result:?}");

    // If either is None, result is None
    let no_func: Option<fn(i32) -> i32> = None;
    let result_none = Some(42).apply(no_func);
    println!("Option: Some(42).apply(None) = {result_none:?}");

    // Vec: Cartesian product application
    let vals = vec![1, 2, 3];
    let funcs: Vec<fn(i32) -> i32> = vec![|x| x + 100, |x| x * 10];
    let applied: Vec<i32> = vals.apply(funcs);
    println!("Vec: [1,2,3].apply([+100, *10]) = {applied:?}");

    // Pure: lifting a value into the context
    let pure_opt: Option<i32> = <Option<i32>>::pure_target(42);
    println!("Option::pure_target(42) = {pure_opt:?}");

    println!();

    // =========================================
    // Part 3: Monad - Chaining operations
    // =========================================
    println!("=== Part 3: Monad (flat_map) ===");

    // Safe division that returns Option
    fn safe_divide(a: i32, b: i32) -> Option<i32> {
        if b == 0 { None } else { Some(a / b) }
    }

    // Chain operations that might fail
    let result = Some(100)
        .flat_map(|x| safe_divide(x, 5))
        .flat_map(|x| safe_divide(x, 2))
        .map(|x| x + 1);
    println!("100 / 5 / 2 + 1 = {result:?}");

    // Short-circuit on None
    let failed = Some(100)
        .flat_map(|x| safe_divide(x, 0)) // Division by zero!
        .flat_map(|x| safe_divide(x, 2))
        .map(|x| x + 1);
    println!("100 / 0 / 2 + 1 = {failed:?}");

    // Vec flat_map (like flat_map on iterators)
    let lists = vec![1, 2, 3];
    let expanded: Vec<i32> = lists.flat_map(|x| vec![x, x * 10, x * 100]);
    println!("Vec: [1,2,3].flat_map(|x| [x, x*10, x*100]) = {expanded:?}");

    println!();

    // =========================================
    // Part 4: Practical Example - Data Pipeline
    // =========================================
    println!("=== Part 4: Practical Data Pipeline ===");

    #[derive(Debug, Clone)]
    struct User {
        id: i32,
        name: String,
    }

    #[derive(Debug)]
    struct Profile {
        _user_id: i32,
        _bio: String,
    }

    // Simulated database lookups
    fn find_user(id: i32) -> Option<User> {
        match id {
            1 => Some(User {
                id: 1,
                name: "Alice".to_string(),
            }),
            2 => Some(User {
                id: 2,
                name: "Bob".to_string(),
            }),
            _ => None,
        }
    }

    fn find_profile(user: &User) -> Option<Profile> {
        if user.id == 1 {
            Some(Profile {
                _user_id: user.id,
                _bio: format!("{}'s awesome profile", user.name),
            })
        } else {
            None // Bob has no profile
        }
    }

    // Chain the lookups monadically
    let alice_profile = find_user(1).flat_map(|u| find_profile(&u));
    println!("Alice's profile: {alice_profile:?}");

    let bob_profile = find_user(2).flat_map(|u| find_profile(&u));
    println!("Bob's profile: {bob_profile:?}");

    let unknown_profile = find_user(999).flat_map(|u| find_profile(&u));
    println!("Unknown user's profile: {unknown_profile:?}");

    println!();

    // =========================================
    // Part 5: Monad Laws Demonstration
    // =========================================
    println!("=== Part 5: Monad Laws ===");

    let a = 5;
    let f = |x: i32| Some(x * 2);
    let g = |x: i32| Some(x + 1);

    // Left Identity: pure(a).flat_map(f) == f(a)
    let left_id_left = <Option<i32>>::pure_target(a).flat_map(f);
    let left_id_right = f(a);
    println!(
        "Left Identity: pure({}).flat_map(f) = {:?}, f({}) = {:?} => {}",
        a,
        left_id_left,
        a,
        left_id_right,
        left_id_left == left_id_right
    );

    // Right Identity: m.flat_map(pure) == m
    let m = Some(42);
    let right_id_left = m.flat_map(<Option<i32>>::pure_target);
    let right_id_right = m;
    println!(
        "Right Identity: m.flat_map(pure) = {:?}, m = {:?} => {}",
        right_id_left,
        right_id_right,
        right_id_left == right_id_right
    );

    // Associativity: m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
    let m2 = Some(5);
    let assoc_left = m2.flat_map(f).flat_map(g);
    let assoc_right = Some(5).flat_map(|x| f(x).flat_map(g));
    println!(
        "Associativity: m.flat_map(f).flat_map(g) = {:?}, m.flat_map(|x| f(x).flat_map(g)) = {:?} => {}",
        assoc_left,
        assoc_right,
        assoc_left == assoc_right
    );

    println!("\n=== All examples completed! ===");
}
