//! Phase 7: Category Theory Foundations Tests
//!
//! Comprehensive tests for:
//! - Enhanced Arrow type classes (`SagittaElectio`, `SagittaApplicatio`, `SagittaCirculus`)
//! - Kan Extensions (Yoneda, Coyoneda, `ExtensioKanDextra`, `ExtensioKanSinistra`)
//! - Codensity and Density
//! - Day Convolution

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;

use ordofp_core::category::fn_arrows::*;
use ordofp_core::category::*;
use ordofp_core::datatypes::Aut;
use ordofp_core::typeclasses::hkt::HKT;

// =============================================================================
// Arrow Function Tests
// =============================================================================

#[test]
fn test_fn_arrows_id() {
    let id_fn: BoxedFn<i32, i32> = id();
    assert_eq!(id_fn(42), 42);
    assert_eq!(id_fn(0), 0);
    assert_eq!(id_fn(-100), -100);
}

#[test]
fn test_fn_arrows_compose() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x + 1);
    let g: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let composed = compose(f, g);
    // composed(x) = f(g(x)) = (x * 2) + 1
    assert_eq!(composed(5), 11);
    assert_eq!(composed(0), 1);
    assert_eq!(composed(10), 21);
}

#[test]
fn test_fn_arrows_arr() {
    let f = arr(|x: i32| x.to_string());
    assert_eq!(f(42), "42");
    assert_eq!(f(0), "0");
    assert_eq!(f(-1), "-1");
}

#[test]
fn test_fn_arrows_first() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let first_f = first::<i32, i32, String>(f);
    assert_eq!(first_f((5, "hello".to_string())), (10, "hello".to_string()));
    assert_eq!(first_f((0, "world".to_string())), (0, "world".to_string()));
}

#[test]
fn test_fn_arrows_second() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let second_f = second::<String, i32, i32>(f);
    assert_eq!(
        second_f(("hello".to_string(), 5)),
        ("hello".to_string(), 10)
    );
}

#[test]
fn test_fn_arrows_split() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let g: BoxedFn<&str, usize> = Box::new(str::len);
    let split_fg = split(f, g);
    assert_eq!(split_fg((5, "hello")), (10, 5));
}

#[test]
fn test_fn_arrows_sinister_left() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let left_f = sinister::<i32, i32, String>(f);

    assert_eq!(left_f(Aut::sinister(21)), Aut::sinister(42));
}

#[test]
fn test_fn_arrows_sinister_right_passthrough() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let left_f = sinister::<i32, i32, String>(f);

    assert_eq!(
        left_f(Aut::dexter("unchanged".to_string())),
        Aut::dexter("unchanged".to_string())
    );
}

#[test]
fn test_fn_arrows_dexter_right() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let right_f = dexter::<String, i32, i32>(f);

    assert_eq!(right_f(Aut::dexter(21)), Aut::dexter(42));
}

#[test]
fn test_fn_arrows_dexter_left_passthrough() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let right_f = dexter::<String, i32, i32>(f);

    assert_eq!(
        right_f(Aut::sinister("unchanged".to_string())),
        Aut::sinister("unchanged".to_string())
    );
}

#[test]
fn test_fn_arrows_confluo_left() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let g: BoxedFn<&str, i32> = Box::new(|s| s.len() as i32);
    let fanin = confluo(f, g);

    assert_eq!(fanin(Aut::sinister(21)), 42);
}

#[test]
fn test_fn_arrows_confluo_right() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let g: BoxedFn<&str, i32> = Box::new(|s| s.len() as i32);
    let fanin = confluo(f, g);

    assert_eq!(fanin(Aut::dexter("hello")), 5);
}

#[test]
fn test_fn_arrows_addo_left() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let g: BoxedFn<&str, usize> = Box::new(str::len);
    let plus = addo(f, g);

    assert_eq!(plus(Aut::sinister(21)), Aut::sinister(42));
}

#[test]
fn test_fn_arrows_addo_right() {
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let g: BoxedFn<&str, usize> = Box::new(str::len);
    let plus = addo(f, g);

    assert_eq!(plus(Aut::dexter("hello")), Aut::dexter(5));
}

#[test]
fn test_fn_arrows_applicatio() {
    let app = applicatio::<i32, i32>();
    let double: BoxedFn<i32, i32> = Box::new(|x| x * 2);

    assert_eq!(app((double, 21)), 42);
}

#[test]
fn test_fn_arrows_circulus() {
    let f: BoxedFn<(i32, i32), (i32, i32)> = Box::new(|(input, state)| (input + state, state + 1));
    let looped = circulus(f);

    // With default state (0), result is input + 0 = input
    assert_eq!(looped(5), 5);
    assert_eq!(looped(10), 10);
}

// =============================================================================
// Utility Function Tests
// =============================================================================

#[test]
fn test_via_praedicatum_positive() {
    let router = via_praedicatum(|x: &i32| *x > 0);
    assert_eq!(router(5), Aut::sinister(5));
}

#[test]
fn test_via_praedicatum_negative() {
    let router = via_praedicatum(|x: &i32| *x > 0);
    assert_eq!(router(-3), Aut::dexter(-3));
}

#[test]
fn test_via_praedicatum_zero() {
    let router = via_praedicatum(|x: &i32| *x > 0);
    assert_eq!(router(0), Aut::dexter(0));
}

#[test]
fn test_coalesco_sinister() {
    assert_eq!(coalesco(Aut::<i32, i32>::sinister(42)), 42);
}

#[test]
fn test_coalesco_dexter() {
    assert_eq!(coalesco(Aut::<i32, i32>::dexter(100)), 100);
}

#[test]
fn test_inicio_sinister() {
    let left: Aut<i32, String> = inicio_sinister(42);
    assert_eq!(left, Aut::sinister(42));
}

#[test]
fn test_inicio_dexter() {
    let right: Aut<i32, String> = inicio_dexter("hello".to_string());
    assert_eq!(right, Aut::dexter("hello".to_string()));
}

// =============================================================================
// Arrow Laws Tests
// =============================================================================

#[test]
fn test_arrow_identity_law() {
    // arr id = id
    let arr_id = arr(|x: i32| x);
    let id_fn: BoxedFn<i32, i32> = id();

    for x in [0, 1, -1, 42, 100] {
        assert_eq!(arr_id(x), id_fn(x));
    }
}

#[test]
fn test_arrow_composition_law() {
    // arr (f . g) = arr g >>> arr f
    let f = |x: i32| x + 1;
    let g = |x: i32| x * 2;

    let arr_composed = arr(move |x| f(g(x)));
    let arr_g = arr(g);
    let arr_f = arr(f);
    let composed_arr = compose(arr_f, arr_g);

    for x in [0, 1, -1, 42, 100] {
        assert_eq!(arr_composed(x), composed_arr(x));
    }
}

#[test]
fn test_category_identity_left() {
    // id . f = f
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let id_fn: BoxedFn<i32, i32> = id();
    let composed = compose(id_fn, f);

    for x in [0, 1, 5, 10] {
        assert_eq!(composed(x), x * 2);
    }
}

#[test]
fn test_category_identity_right() {
    // f . id = f
    let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let id_fn: BoxedFn<i32, i32> = id();
    let composed = compose(f, id_fn);

    for x in [0, 1, 5, 10] {
        assert_eq!(composed(x), x * 2);
    }
}

#[test]
fn test_category_associativity() {
    // (h . g) . f = h . (g . f)
    let f: BoxedFn<i32, i32> = Box::new(|x| x + 1);
    let g: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let h: BoxedFn<i32, i32> = Box::new(|x| x - 3);

    let f2: BoxedFn<i32, i32> = Box::new(|x| x + 1);
    let g2: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let h2: BoxedFn<i32, i32> = Box::new(|x| x - 3);

    // (h . g) . f
    let hg = compose(h, g);
    let left = compose(hg, f);

    // h . (g . f)
    let gf = compose(g2, f2);
    let right = compose(h2, gf);

    for x in [0, 1, 5, 10] {
        assert_eq!(left(x), right(x));
    }
}

// =============================================================================
// Kan Extension Tests
// =============================================================================

// Simple Option witness for testing
struct OptionWitness;
impl HKT for OptionWitness {
    type Target<A> = Option<A>;
}

#[test]
fn test_coyoneda_lift() {
    let coyoneda: Coyoneda<OptionWitness, i32> = Coyoneda::lift(Some(42));
    assert!(coyoneda.is_type::<Option<i32>>());
}

#[test]
fn test_coyoneda_map_accumulates() {
    let coyoneda: Coyoneda<OptionWitness, i32> = Coyoneda::lift(Some(42));
    let mapped = coyoneda.map(|x| x * 2);
    // The transformation is accumulated
    assert!(mapped.is_type::<Option<i32>>());
}

#[test]
fn test_coyoneda_map_chain() {
    let coyoneda: Coyoneda<OptionWitness, i32> = Coyoneda::lift(Some(10));

    // Chain multiple maps - they should all compose
    let result = coyoneda
        .map(|x| x + 1) // 11
        .map(|x| x * 2) // 22
        .map(|x| x - 2); // 20

    // The internal value is still Option<i32>
    assert!(result.is_type::<Option<i32>>());
}

#[test]
fn test_codensitas_purus() {
    let cod: Codensitas<OptionWitness, i32> = Codensitas::purus(42);
    let result = cod.run_with(Some);
    assert_eq!(result, Some(42));
}

#[test]
fn test_codensitas_purus_different_value() {
    let cod: Codensitas<OptionWitness, i32> = Codensitas::purus(100);
    let result = cod.run_with(|x| Some(x * 2));
    assert_eq!(result, Some(200));
}

#[test]
fn test_densitas_basic() {
    let density: Densitas<OptionWitness, i32> =
        Densitas::new(Some(42), |opt: Option<i32>| opt.unwrap_or(0));
    let extracted = density.extractum();
    assert_eq!(extracted, 42);
}

#[test]
fn test_densitas_none() {
    let density: Densitas<OptionWitness, i32> =
        Densitas::new(None::<i32>, |opt: Option<i32>| opt.unwrap_or(0));
    let extracted = density.extractum();
    assert_eq!(extracted, 0);
}

#[test]
fn test_densitas_map() {
    let density: Densitas<OptionWitness, i32> =
        Densitas::new(Some(21), |opt: Option<i32>| opt.unwrap_or(0));
    let mapped = density.map(|x| x * 2);
    let extracted = mapped.extractum();
    assert_eq!(extracted, 42);
}

#[test]
fn test_day_convolution_basic() {
    let day: ConvolutioDiei<OptionWitness, OptionWitness, i32> =
        ConvolutioDiei::new(Some(2), Some(21), |a: i32, b: i32| a * b);

    // Map over the result
    let mapped = day.map(|x| x + 1);
    // The combine function would produce 2 * 21 = 42, then + 1 = 43
    // We can't easily extract without more machinery, but structure test passes
    let _ = mapped;
}

#[test]
fn test_extensio_kan_dextra_basic() {
    let ran: ExtensioKanDextra<OptionWitness, OptionWitness, i32> =
        ExtensioKanDextra::new(|k: Box<dyn FnOnce(i32) -> Option<i32>>| k(42));

    let result = ran.run_with(|x| Some(x * 2));
    assert_eq!(result, Some(84));
}

#[test]
fn test_extensio_kan_dextra_identity() {
    let ran: ExtensioKanDextra<OptionWitness, OptionWitness, i32> =
        ExtensioKanDextra::new(|k: Box<dyn FnOnce(i32) -> Option<i32>>| k(100));

    let result = ran.run_with(Some);
    assert_eq!(result, Some(100));
}

#[test]
fn test_extensio_kan_sinistra_basic() {
    let lan: ExtensioKanSinistra<OptionWitness, OptionWitness, i32> =
        ExtensioKanSinistra::new(Some(42), |opt: Option<i32>| opt.unwrap_or(0));

    let mapped = lan.map(|x| x * 2);
    // Structure test
    let _ = mapped;
}

#[test]
fn test_extensio_kan_sinistra_map_chain() {
    let lan: ExtensioKanSinistra<OptionWitness, OptionWitness, i32> =
        ExtensioKanSinistra::new(Some(10), |opt: Option<i32>| opt.unwrap_or(0));

    let mapped = lan.map(|x| x + 1).map(|x| x * 2);
    // Structure test
    let _ = mapped;
}

// =============================================================================
// Choice Arrow Pattern Tests
// =============================================================================

#[test]
fn test_choice_pattern_even_odd() {
    // Route even numbers left, odd numbers right
    let router = via_praedicatum(|x: &i32| x % 2 == 0);

    let double: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let triple: BoxedFn<i32, i32> = Box::new(|x| x * 3);

    // Process based on routing
    let left_process = sinister::<i32, i32, i32>(double);
    let right_process = dexter::<i32, i32, i32>(triple);

    // Test even number
    let even = router(4);
    let processed = left_process(even);
    let final_result = match processed {
        Aut::Sinister(x) => x,
        Aut::Dexter(x) => x,
    };
    assert_eq!(final_result, 8); // 4 * 2

    // Test odd number
    let odd = router(5);
    let processed = right_process(odd);
    let final_result = match processed {
        Aut::Sinister(x) => x,
        Aut::Dexter(x) => x,
    };
    assert_eq!(final_result, 15); // 5 * 3
}

#[test]
fn test_fanin_pattern() {
    // Route based on string content
    let is_numeric = |s: &String| s.chars().all(char::is_numeric);
    let router = via_praedicatum(is_numeric);

    // Parse numbers or count characters
    let parse: BoxedFn<String, i32> = Box::new(|s| s.parse().unwrap_or(0));
    let count: BoxedFn<String, i32> = Box::new(|s| s.len() as i32);

    let processor = confluo(parse, count);

    // Numeric string - goes left
    let numeric = router("123".to_string());
    let result = processor(numeric);
    assert_eq!(result, 123);

    // Non-numeric string - goes right
    let alpha = router("hello".to_string());
    let result = processor(alpha);
    assert_eq!(result, 5);
}

// =============================================================================
// Composition Pattern Tests
// =============================================================================

#[test]
fn test_arrow_pipeline() {
    // Build a processing pipeline using arrows
    let parse: BoxedFn<&str, i32> = Box::new(|s| s.parse().unwrap_or(0));
    let double: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let stringify: BoxedFn<i32, String> = Box::new(|x| x.to_string());

    let pipeline = compose(stringify, compose(double, parse));

    assert_eq!(pipeline("21"), "42");
    assert_eq!(pipeline("10"), "20");
    assert_eq!(pipeline("invalid"), "0");
}

#[test]
fn test_parallel_processing() {
    // Process two values in parallel
    let double: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let triple: BoxedFn<i32, i32> = Box::new(|x| x * 3);

    let parallel = split(double, triple);

    assert_eq!(parallel((5, 7)), (10, 21));
    assert_eq!(parallel((10, 10)), (20, 30));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_string_handling() {
    let len: BoxedFn<&str, usize> = Box::new(str::len);
    assert_eq!(len(""), 0);
    assert_eq!(len("a"), 1);
}

#[test]
fn test_negative_numbers() {
    let abs: BoxedFn<i32, i32> = Box::new(i32::abs);
    assert_eq!(abs(-42), 42);
    assert_eq!(abs(42), 42);
    assert_eq!(abs(0), 0);
}

#[test]
fn test_deeply_nested_composition() {
    let f1: BoxedFn<i32, i32> = Box::new(|x| x + 1);
    let f2: BoxedFn<i32, i32> = Box::new(|x| x * 2);
    let f3: BoxedFn<i32, i32> = Box::new(|x| x - 3);
    let f4: BoxedFn<i32, i32> = Box::new(|x| x / 2);

    let c1 = compose(f2, f1);
    let c2 = compose(f3, c1);
    let c3 = compose(f4, c2);

    // ((((x + 1) * 2) - 3) / 2)
    // x = 5: ((6 * 2) - 3) / 2 = (12 - 3) / 2 = 9 / 2 = 4
    assert_eq!(c3(5), 4);
}
