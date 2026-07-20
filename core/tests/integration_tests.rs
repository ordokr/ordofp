//! Integration Tests for `OrdoFP`
//!
//! These tests verify that all major features work together correctly.

use ordofp_core::prelude::*;

// =============================================================================
// Phase 1: Core Type Classes
// =============================================================================

mod phase1_type_classes {
    use super::*;

    #[test]
    fn test_functor_option() {
        let opt = Some(42);
        let mapped = opt.map(|x| x * 2);
        assert_eq!(mapped, Some(84));
    }

    #[test]
    fn test_functor_result() {
        let res: Result<i32, &str> = Ok(42);
        let mapped = res.map(|x| x * 2);
        assert_eq!(mapped, Ok(84));
    }

    #[test]
    fn test_hlist_basic() {
        let list = hlist![1, "hello", true];
        assert_eq!(list.head, 1);
        assert_eq!(list.tail.head, "hello");
        assert!(list.tail.tail.head);
    }

    #[test]
    fn test_hlist_type_alias() {
        let list: HList![i32, &str, bool] = hlist![42, "world", false];
        assert_eq!(list.head, 42);
    }
}

// =============================================================================
// Phase 2: Optics
// =============================================================================

mod phase2_optics {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    #[test]
    fn test_lens_get_set() {
        let name_lens = lens(
            |p: &Person| p.name.clone(),
            |p: &Person, name: String| Person { name, age: p.age },
        );

        let alice = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        assert_eq!(name_lens.get(&alice), "Alice");

        let bob = name_lens.set(&alice, "Bob".to_string());
        assert_eq!(bob.name, "Bob");
        assert_eq!(bob.age, 30);
    }

    #[test]
    fn test_lens_modify() {
        let age_lens = lens(
            |p: &Person| p.age,
            |p: &Person, age: u32| Person {
                name: p.name.clone(),
                age,
            },
        );

        let person = Person {
            name: "Charlie".to_string(),
            age: 25,
        };

        let older = age_lens.modify(&person, |a| a + 1);
        assert_eq!(older.age, 26);
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Shape {
        Circle(f64),
        Rectangle(f64, f64),
    }

    #[test]
    fn test_prism_preview_review() {
        let circle_prism = prism(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        let circle = Shape::Circle(5.0);
        let rect = Shape::Rectangle(3.0, 4.0);

        assert_eq!(circle_prism.preview(&circle), Some(5.0));
        assert_eq!(circle_prism.preview(&rect), None);
        assert_eq!(circle_prism.review(10.0), Shape::Circle(10.0));
    }

    #[test]
    fn test_iso_forward_backward() {
        let swap_iso = iso(
            |(a, b): &(i32, String)| (b.clone(), *a),
            |(b, a): &(String, i32)| (*a, b.clone()),
        );

        let pair = (42, "hello".to_string());
        let swapped = swap_iso.forward(&pair);
        assert_eq!(swapped, ("hello".to_string(), 42));

        let back = swap_iso.backward(&swapped);
        assert_eq!(back, pair);
    }
}

// =============================================================================
// Phase 3: Disiunctio
// =============================================================================

mod phase3_disiunctio {
    use super::*;

    #[test]
    fn test_disiunctio_injection() {
        type MySum = Disiunctio!(i32, String, bool);

        let int_val: MySum = DisiunctioInjector::inject(42i32);
        let str_val: MySum = DisiunctioInjector::inject("hello".to_string());
        let bool_val: MySum = DisiunctioInjector::inject(true);

        // Verify the values are in the right variant
        match int_val {
            Disiunctio::Sinister(n) => assert_eq!(n, 42),
            _ => panic!("Expected Sinister"),
        }

        match str_val {
            Disiunctio::Dexter(Disiunctio::Sinister(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected nested Sinister"),
        }

        match bool_val {
            Disiunctio::Dexter(Disiunctio::Dexter(Disiunctio::Sinister(b))) => assert!(b),
            _ => panic!("Expected deeply nested Sinister"),
        }
    }
}

// =============================================================================
// Phase 4: Data Types
// =============================================================================

mod phase4_data_types {
    use super::*;

    #[test]
    fn test_either_sinister_dexter() {
        let left: Aut<&str, i32> = Aut::Sinister("error");
        let right: Aut<&str, i32> = Aut::Dexter(42);

        assert!(left.is_sinister());
        assert!(right.is_dexter());

        // Using English aliases
        let left2: Either<&str, i32> = Either::Sinister("error");
        let right2: Either<&str, i32> = Either::Dexter(42);

        assert!(left2.is_sinister());
        assert!(right2.is_dexter());
    }

    #[test]
    fn test_either_map() {
        let right: Aut<&str, i32> = Aut::Dexter(21);
        let mapped = right.map(|x| x * 2);

        match mapped {
            Aut::Dexter(n) => assert_eq!(n, 42),
            _ => panic!("Expected Dexter"),
        }
    }
}

// =============================================================================
// Phase 6: Easy API
// =============================================================================

#[cfg(feature = "alloc")]
mod phase6_easy_api {
    use super::*;

    #[test]
    fn test_state_operations() {
        let result = run_with_state(0i32, |counter| {
            *counter += 10;
            *counter *= 2;
            *counter
        });
        assert_eq!(result, 20);
    }

    #[test]
    fn test_state_monad() {
        let computation = get::<i32>()
            .and_then(|x| modify(|s: i32| s + 10).then(state_pure(x)))
            .map(|x| x * 2);

        let (result, final_state) = computation.run(5);
        assert_eq!(result, 10); // 5 * 2
        assert_eq!(final_state, 15); // 5 + 10
    }

    #[test]
    fn test_reader_operations() {
        #[derive(Clone)]
        struct Config {
            multiplier: i32,
        }

        let config = Config { multiplier: 3 };

        let result = run_with_config(&config, |cfg| 14 * cfg.multiplier);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_reader_monad() {
        #[derive(Clone)]
        struct Env {
            base: i32,
        }

        let env = Env { base: 10 };

        let reader = ask::<Env>()
            .map(|e| e.base)
            .and_then(|x| asks(move |e: &Env| x + e.base));

        assert_eq!(reader.run(&env), 20);
    }

    #[test]
    fn test_retry_success() {
        let mut attempts = 0;
        let result: Result<i32, &str> = retry(3, || {
            attempts += 1;
            if attempts >= 2 {
                Ok(42)
            } else {
                Err("not ready")
            }
        });

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 2);
    }

    #[test]
    fn test_fallback() {
        let result: Result<i32, &str> = fallback(|| Err("primary failed"), || Ok(42));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_io_operations() {
        let computation = io(|| 21).map(|x| x * 2);
        assert_eq!(computation.run(), 42);
    }

    #[test]
    fn test_io_chaining() {
        let chained = io(|| 10).and_then(|x| io(move || x + 20)).map(|x| x + 12);

        assert_eq!(chained.run(), 42);
    }

    #[test]
    fn test_combinators() {
        let result = chain(|| 1, |x| x + 10, |x| x * 2);
        assert_eq!(result, 22);

        let (a, b) = both(|| 20, || 22);
        assert_eq!(a + b, 42);

        let cond_true = when(true, || 42, || 0);
        let cond_false = when(false, || 42, || 0);
        assert_eq!(cond_true, 42);
        assert_eq!(cond_false, 0);
    }

    #[test]
    fn test_result_extensions() {
        let ok: Result<i32, &str> = Ok(42);

        assert!(ok.is_ok_and(|x| x > 0));
        assert!(!ok.is_ok_and(|x| x < 0));

        let swapped = ok.swap();
        assert_eq!(swapped, Err(42));
    }

    #[test]
    fn test_partition_results() {
        let results: Vec<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(2), Err("b"), Ok(3)];
        let (successes, errors) = partition_results_vec(results);

        assert_eq!(successes, vec![1, 2, 3]);
        assert_eq!(errors, vec!["a", "b"]);
    }
}

// =============================================================================
// Phase 7: Performance Features
// =============================================================================

#[cfg(feature = "alloc")]
mod phase7_performance {
    use super::*;

    #[test]
    fn test_arena_allocation() {
        let result = with_arena(|arena| {
            let x = arena.alloc(42);
            let y = arena.alloc(100);
            *x + *y
        });
        assert_eq!(result, 142);
    }

    #[test]
    fn test_arena_slice() {
        with_arena(|arena| {
            let slice = arena.alloc_slice(&[1, 2, 3, 4, 5]);
            assert_eq!(slice.iter().sum::<i32>(), 15);
        });
    }

    #[test]
    fn test_arena_string() {
        with_arena(|arena| {
            let s = arena.alloc_str("hello world");
            assert_eq!(s.len(), 11);
        });
    }

    #[test]
    fn test_specialization_hints() {
        // These should compile and not affect correctness
        assert!(likely(true));
        assert!(!unlikely(false));

        let result = cold_path(|| 42);
        assert_eq!(result, 42);

        let result = hot_path(|| 100);
        assert_eq!(result, 100);

        let x = black_box(42);
        assert_eq!(x, 42);
    }
}

// =============================================================================
// Cross-Phase Integration
// =============================================================================

#[cfg(feature = "alloc")]
mod cross_phase_integration {
    use super::*;

    #[test]
    fn test_hlist_with_state() {
        // Use HList with state management
        let list = hlist![1i32, 2i32, 3i32];

        let sum = run_with_state(0i32, |acc| {
            *acc += list.head;
            *acc += list.tail.head;
            *acc += list.tail.tail.head;
            *acc
        });

        assert_eq!(sum, 6);
    }

    #[test]
    fn test_optics_with_state() {
        #[derive(Clone, Debug)]
        struct Counter {
            value: i32,
        }

        let value_lens = lens(
            |c: &Counter| c.value,
            |_c: &Counter, v: i32| Counter { value: v },
        );

        let counter = Counter { value: 0 };

        // Use lens with state-like updates
        let updated = value_lens.modify(&counter, |v| v + 10);
        let updated = value_lens.modify(&updated, |v| v * 2);

        assert_eq!(value_lens.get(&updated), 20);
    }

    #[test]
    fn test_either_with_reader() {
        #[derive(Clone)]
        struct Config {
            threshold: i32,
        }

        let config = Config { threshold: 50 };

        let result = run_with_config(&config, |cfg| {
            let value = 42;
            if value > cfg.threshold {
                Aut::<&str, i32>::Dexter(value)
            } else {
                Aut::Sinister("below threshold")
            }
        });

        assert!(result.is_sinister());
    }

    #[test]
    fn test_arena_with_hlist() {
        with_arena(|arena| {
            // Allocate HList components in arena
            let a = arena.alloc(42i32);
            let b = arena.alloc("hello");
            let c = arena.alloc(true);

            // Create HList referencing arena values
            let list = hlist![*a, *b, *c];
            assert_eq!(list.head, 42);
        });
    }

    #[test]
    fn test_io_with_result_extensions() {
        let computation = io(|| {
            let result: Result<i32, &str> = Ok(42);
            result.map(|x| x * 2)
        });

        let result = computation.run();
        assert_eq!(result, Ok(84));
    }
}
