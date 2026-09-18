//! Vernacular API Examples
//!
//! Demonstrates using the English-language aliases instead of Latin names.
//!
//! # Running
//!
//! ```bash
//! cargo run --example 10_vernacular_api
//! ```

// Import using vernacular (English) names
use ordofp_core::vernacular::{
    // HList with English aliases
    Cons,
    // Data types
    Either,
    Monoid,
    Nil,
    Semigroup,
    Validated,

    iso,
    // Optics with English names
    lens,
    prism,
};

fn main() {
    #[derive(Clone, Debug)]
    struct Person {
        name: String,
        age: u32,
    }

    // Create a prism using English 'prism' function (vs 'divisio')
    #[derive(Clone, Debug, PartialEq)]
    enum Shape {
        Circle(f64),
        Rectangle(f64, f64),
    }

    println!("=== OrdoFP Vernacular API Examples ===\n");

    // Example 1: HList with English aliases
    println!("1. HList with Cons/Nil (vs Coniunctio/Nihil)");
    println!("---------------------------------------------");

    // Type-safe heterogeneous list
    let list: Cons<i32, Cons<bool, Cons<&str, Nil>>> = ordofp_core::hlist![42, true, "hello"];

    println!("   HList: {:?}", list.head);
    println!("   Tail head: {:?}", list.tail.head);

    // Example 2: Optics with English names
    println!("\n2. Optics (Lens, Prism, Iso)");
    println!("----------------------------");

    // Create a lens using English 'lens' function (vs 'aspectus')
    // Type inference handles the closure types
    let name_lens = lens(
        |p: &Person| p.name.clone(),
        |p: &Person, name: String| Person { name, age: p.age },
    );

    let alice = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    println!("   Original name: {}", name_lens.get(&alice));
    let bob = name_lens.set(&alice, "Bob".to_string());
    println!("   After set: {}", bob.name);

    // Type inference handles the closure types
    let circle_prism = prism(
        |s: &Shape| match s {
            Shape::Circle(r) => Some(*r),
            Shape::Rectangle(..) => None,
        },
        Shape::Circle,
    );

    let circle = Shape::Circle(5.0);
    println!("   Circle radius: {:?}", circle_prism.preview(&circle));

    let rect = Shape::Rectangle(10.0, 20.0);
    println!("   Is rectangle: {}", circle_prism.preview(&rect).is_none());

    // Create an iso using English 'iso' function (vs 'aequivalentia')
    // Type inference handles the closure types
    let tuple_iso = iso(
        |(a, b): &(i32, String)| (b.clone(), *a),
        |(b, a): &(String, i32)| (*a, b.clone()),
    );

    let pair = (42, "hello".to_string());
    let swapped = tuple_iso.forward(&pair);
    println!("   Swapped pair: {swapped:?}");

    // Example 3: Either (vs Aut)
    println!("\n3. Either (vs Aut)");
    println!("------------------");

    // Either is a sum type (Left | Right)
    // In OrdoFP, this is Aut (Sinister | Dexter)
    let success: Either<&str, i32> = Either::Dexter(42);
    let failure: Either<&str, i32> = Either::Sinister("error");

    println!("   Success: {success:?}");
    println!("   Failure: {failure:?}");

    // Pattern matching
    match success {
        Either::Sinister(err) => println!("   Error: {err}"),
        Either::Dexter(val) => println!("   Value: {val}"),
    }

    // Example 4: Functor/Monad (same names)
    println!("\n4. Functor and Monad");
    println!("--------------------");

    // Option implements Functor
    let opt = Some(42);
    // Note: fmap is available via the Functor trait
    println!("   Option Some(42): {opt:?}");

    // Result implements Functor
    let res: Result<i32, &str> = Ok(42);
    println!("   Result Ok(42): {res:?}");

    // Example 5: Semigroup and Monoid (Compositio and Unitas)
    println!("\n5. Semigroup and Monoid");
    println!("-----------------------");
    let combined = String::from("Hello, ").combine(&String::from("World!"));
    println!("   Semigroup combine: {combined}");
    let empty_str: String = Monoid::empty();
    println!("   Monoid empty: '{empty_str}'");

    // Example 6: Validated (Probatum)
    println!("\n6. Validated (Multi-Error Validation)");
    println!("-------------------------------------");
    let v1: Validated<&str, i32> = Validated::Valid(10);
    let v2: Validated<&str, i32> = Validated::Valid(20);
    let v_sum = Validated::lift2(|a, b| a + b, v1, v2);
    println!("   Validated sum: {v_sum:?}");

    // Example 7: Naming comparison table
    println!("\n7. Naming Comparison");
    println!("--------------------");
    println!("   | Latin (Scholastic)  | English (Vernacular) |");
    println!("   |---------------------|----------------------|");
    println!("   | Coniunctio          | Cons                 |");
    println!("   | Nihil               | Nil                  |");
    println!("   | Aspectus            | Lens                 |");
    println!("   | Divisio             | Prism                |");
    println!("   | Aequivalentia       | Iso                  |");
    println!("   | Aut                 | Either               |");
    println!("   | Compositio          | Semigroup            |");
    println!("   | Unitas              | Monoid               |");
    println!("   | Probatum            | Validated            |");
    println!("   | Linearis            | Linear               |");
    println!("   | Pigritia            | Lazy                 |");
    println!("   | Disiunctio          | Disiunctio (same)    |");
    println!("   | Absurdum            | Void                 |");
    println!("   | Vestigium           | Traced               |");

    println!("\n=== Vernacular Examples Complete ===");
}
