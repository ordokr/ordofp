//! Module for holding the Compositio (Semigroup) typeclass definition and typeclass instances
//!
//! You can, for example, combine tuples.
#![cfg_attr(
    feature = "alloc",
    doc = r#"
# Examples

```
# fn main() {
use ordofp::Compositio;
use ordofp::hlist;

let t1 = (1, 2.5f32, String::from("hi"), Some(3));
let t2 = (1, 2.5f32, String::from(" world"), None);

let expected = (2, 5.0f32, String::from("hi world"), Some(3));

assert_eq!(t1.combine(&t2), expected);

// ultimately, the Tuple-based Compositio implementations are only available for a maximum of
// 26 elements. If you need more, use HList, which has no such limit.

let h1 = hlist![1, 3.3, 53i64];
let h2 = hlist![2, 1.2, 1i64];
let h3 = hlist![3, 4.5, 54];
assert_eq!(h1.combine(&h2), h3)
# }
```"#
)]

// Re-export everything from core's typeclasses module
// Primary scholastic names
pub use ordofp_core::typeclasses::{Compositio, combine_all_option, combine_n};

// Re-export wrapper types from core's wrappers module
// Primary scholastic names
pub use ordofp_core::wrappers::{Aliquid, Max, Min, Multiplicatio, Omnis};
