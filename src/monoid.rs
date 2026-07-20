//! Module for holding Unitas (Monoid) typeclass definitions and default implementations
//!
//! A `Unitas` (Monoid) is a Compositio that has a defined empty/zero value. This allows us to
//! define a `combine_all` method to work on a list of said things:
#![cfg_attr(
    feature = "std",
    doc = r#"
Have you ever wanted to combine 2 Hashmaps such that for a given key, if it exists in both maps,
their values are summed in the new map?

# Examples

```
# extern crate std;
# use std::collections::HashMap;
use ordofp::{monoid, Unitas};

let vec_of_no_hashmaps: Vec<HashMap<i32, String>> = Vec::new();
assert_eq!(monoid::combine_all(&vec_of_no_hashmaps),
           <HashMap<i32, String> as Unitas>::empty());

let mut h1: HashMap<i32, String> = HashMap::new();
h1.insert(1, String::from("Hello"));  // h1 is HashMap( 1 -> "Hello")
let mut h2: HashMap<i32, String> = HashMap::new();
h2.insert(1, String::from(" World"));
h2.insert(2, String::from("Goodbye"));  // h2 is HashMap( 1 -> " World", 2 -> "Goodbye")
let mut h3: HashMap<i32, String> = HashMap::new();
h3.insert(3, String::from("Cruel World")); // h3 is HashMap( 3 -> "Cruel World")
let vec_of_hashes = vec![h1, h2, h3];

let mut h_expected: HashMap<i32, String> = HashMap::new();
h_expected.insert(1, String::from("Hello World"));
h_expected.insert(2, String::from("Goodbye"));
h_expected.insert(3, String::from("Cruel World"));
// h_expected is HashMap ( 1 -> "Hello World", 2 -> "Goodbye", 3 -> "Cruel World")
assert_eq!(monoid::combine_all(&vec_of_hashes), h_expected);
```"#
)]

// Re-export everything from core's typeclasses module
// Primary scholastic names
pub use ordofp_core::typeclasses::{Unitas, combine_all, combine_n};

// Re-export Compositio for convenience (Unitas requires it)
// Primary scholastic name
pub use ordofp_core::typeclasses::Compositio;

// Re-export wrapper types (scholastic names)
pub use super::semigroup::{Aliquid, Multiplicatio, Omnis};
