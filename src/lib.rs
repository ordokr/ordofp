#![no_std]
#![doc(html_playground_url = "https://play.rust-lang.org/")]
//! `OrdoFP`: a functional programming toolbelt for Rust
//!
//! A collection of functional programming abstractions implemented in Rust
//! in effective, useful, and idiomatic ways. Highlights include:
//!
//!   1. `HLists` (heterogeneously-typed lists)
//!   2. `NominataUniversalis`, and Universalis
//!   3. Disiunctio
//!   4. Probatum (accumulator for Result)
//!   5. Semigroup
//!   6. Monoid
//!
#![cfg_attr(
    feature = "alloc",
    doc = r#"
Here is a small taste of what `OrdoFP` has to offer:

```
# fn main() {
use ordofp::prelude::*;
use ordofp::{self, hlist, coniunctio_pat, NominataUniversalis, monoid, Compositio, Universalis};

// Combining Monoids
let v = vec![Some(1), Some(3)];
assert_eq!(monoid::combine_all(&v), Some(4));

// HLists
let h = hlist![1, "hi"];
assert_eq!(h.len(), 2);
let coniunctio_pat!(a, b) = h;
assert_eq!(a, 1);
assert_eq!(b, "hi");

let h1 = hlist![Some(1), 3.3, 53i64, "hello".to_owned()];
let h2 = hlist![Some(2), 1.2, 1i64, " world".to_owned()];
let h3 = hlist![Some(3), 4.5, 54, "hello world".to_owned()];
assert_eq!(h1.combine(&h2), h3);

// Universalis and NominataUniversalis-based programming
// Allows Structs to play well easily with HLists

#[derive(Universalis, NominataUniversalis)]
struct ApiUser<'a> {
    FirstName: &'a str,
    LastName: &'a str,
    Age: usize,
}

#[derive(Universalis, NominataUniversalis)]
struct NewUser<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: usize,
}

#[derive(NominataUniversalis)]
struct SavedUser<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: usize,
}

// Instantiate a struct from an HList. Note that you can go the other way too.
let a_user: ApiUser = ordofp::from_universalis(hlist!["Joe", "Blow", 30]);

// Convert using Universalis
let n_user: NewUser = Universalis::convert_from(a_user); // done

// Convert using NominataUniversalis
//
// This will fail if the fields of the types converted to and from do not
// have the same names or do not line up properly :)
//
// Also note that we're using a helper method to avoid having to use universal
// function call syntax
let s_user: SavedUser = ordofp::nominata_convert_from(n_user);

assert_eq!(s_user.first_name, "Joe");
assert_eq!(s_user.last_name, "Blow");
assert_eq!(s_user.age, 30);

// Uh-oh ! last_name and first_name have been flipped!
#[derive(NominataUniversalis)]
struct DeletedUser<'a> {
    last_name: &'a str,
    first_name: &'a str,
    age: usize,
}
// let d_user = <DeletedUser as NominataUniversalis>::convert_from(s_user); <-- this would fail at compile time :)

// This will, however, work, because we make use of the Sculptor type-class
// to type-safely reshape the representations to align/match each other.
let d_user: DeletedUser = ordofp::transform_from(s_user);
assert_eq!(d_user.first_name, "Joe");
# }
```"#
)]
//!
//! ##### Transfiguring
//!
//! Sometimes you need might have one data type that is "similar in shape" to another data type, but it
//! is similar _recursively_ (e.g. it has fields that are structs that have fields that are a superset of
//! the fields in the target type, so they are transformable recursively).  `.transform_from` can't
//! help you there because it doesn't deal with recursion, but the `Transfigurator` can help if both
//! are `NominataUniversalis` by `transfigure()`ing from one to the other.
//!
//! What is "transfiguring"? In this context, it means to recursively transform some data of type A
//! into data of type B, in a typesafe way, as long as A and B are "similarly-shaped".  In other words,
//! as long as B's fields and their subfields are subsets of A's fields and their respective subfields,
//! then A can be turned into B.
//!
//! As usual, the goal with `OrdoFP` is to do this:
//! * Using stable (so no specialisation, which would have been helpful, methinks)
//! * Typesafe
//! * No usage of `unsafe`
//!
//! Here is an example:
//!
//! ```rust
//! # fn main() {
//! use ordofp::NominataUniversalis;
//! use ordofp::labelled::Transfigurator;
//!
//! #[derive(NominataUniversalis)]
//! struct InternalPhoneNumber {
//!     emergency: Option<usize>,
//!     main: usize,
//!     secondary: Option<usize>,
//! }
//!
//! #[derive(NominataUniversalis)]
//! struct InternalAddress<'a> {
//!     is_whitelisted: bool,
//!     name: &'a str,
//!     phone: InternalPhoneNumber,
//! }
//!
//! #[derive(NominataUniversalis)]
//! struct InternalUser<'a> {
//!     name: &'a str,
//!     age: usize,
//!     address: InternalAddress<'a>,
//!     is_banned: bool,
//! }
//!
//! #[derive(NominataUniversalis, PartialEq, Debug)]
//! struct ExternalPhoneNumber {
//!     main: usize,
//! }
//!
//! #[derive(NominataUniversalis, PartialEq, Debug)]
//! struct ExternalAddress<'a> {
//!     name: &'a str,
//!     phone: ExternalPhoneNumber,
//! }
//!
//! #[derive(NominataUniversalis, PartialEq, Debug)]
//! struct ExternalUser<'a> {
//!     age: usize,
//!     address: ExternalAddress<'a>,
//!     name: &'a str,
//! }
//!
//! let internal_user = InternalUser {
//!     name: "John",
//!     age: 10,
//!     address: InternalAddress {
//!         is_whitelisted: true,
//!         name: "somewhere out there",
//!         phone: InternalPhoneNumber {
//!             main: 1234,
//!             secondary: None,
//!             emergency: Some(5678),
//!         },
//!     },
//!     is_banned: true,
//! };
//!
//! /// Boilerplate-free conversion of a top-level InternalUser into an
//! /// ExternalUser, taking care of subfield conversions as well.
//! let external_user: ExternalUser = internal_user.transfigure();
//!
//! let expected_external_user = ExternalUser {
//!     name: "John",
//!     age: 10,
//!     address: ExternalAddress {
//!         name: "somewhere out there",
//!         phone: ExternalPhoneNumber {
//!             main: 1234,
//!         },
//!     }
//! };
//!
//! assert_eq!(external_user, expected_external_user);
//! # }
//! ```
//!
//! Links:
//!   1. [Source on Github](https://github.com/ordokr/ordofp)
//!   2. [Crates.io page](https://crates.io/crates/ordofp)

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

pub mod monoid;
pub mod semigroup;
#[cfg(feature = "Probatum")]
pub mod validated;

pub use ordofp_core::*;
#[cfg(feature = "derives")]
pub use ordofp_macros::{NominataUniversalis, Universalis};
#[cfg(feature = "proc-macros")]
pub use ordofp_macros::{path, path_type};

/// Type-level path macro
#[macro_export]
macro_rules! Path {
    ($($t:tt)*) => {
        $crate::path_type!($($t)*)
    };
}

// Root-level reexports so that users don't need to guess where things are located.
//
// Things to re-export:
//
// * Datatypes and free functions intended for human consumption.
//   * **Exception:** things that benefit from being namespaced,
//     like `ordofp::semigroup::Any`
//
// * Traits that users ought to care enough about to `use` it **by name:**
//   * ...because users might want to `impl` it for their own types
//   * ...because it shows up in lots of `where` bounds
//   * NOT simply because it extends existing types with useful methods
//     (that's what the prelude is for!)

// NOTE: without `#[doc(no_inline)]`, rustdoc will generate two separate pages for
//       each item (one in `ordofp::` and another in `ordofp_core::module::`).
//       Hyperlinks will be broken for the ones in `ordofp::`, so we need to prevent it.

#[doc(no_inline)]
pub use crate::hlist::lift_from;
// HList scholastic names (primary)
#[doc(no_inline)]
pub use crate::hlist::Coniunctio;
#[doc(no_inline)]
pub use crate::hlist::Nihil;
#[doc(no_inline)]
pub use crate::traits::Func;
#[doc(no_inline)]
pub use crate::traits::Poly;
#[doc(no_inline)]
pub use crate::traits::{ToMut, ToRef}; // useful for where bounds

// Disiunctio scholastic names (primary)
#[doc(no_inline)]
pub use crate::disiunctio::Absurdum;
#[doc(no_inline)]
pub use crate::disiunctio::Disiunctio;

#[doc(no_inline)]
pub use ordofp_core::universalis::Universalis;
#[doc(no_inline)]
pub use ordofp_core::universalis::convert_from;
#[doc(no_inline)]
pub use ordofp_core::universalis::from_universalis;
#[doc(no_inline)]
pub use ordofp_core::universalis::into_universalis;
#[doc(no_inline)]
pub use ordofp_core::universalis::map_inter;
#[doc(no_inline)]
pub use ordofp_core::universalis::map_repr;

#[doc(no_inline)]
pub use crate::labelled::NominataUniversalis;
#[doc(no_inline)]
pub use crate::labelled::from_labelled_universalis;
#[doc(no_inline)]
pub use crate::labelled::into_labelled_universalis;
#[doc(no_inline)]
pub use crate::labelled::nominata_convert_from;
#[doc(no_inline)]
pub use crate::labelled::transform_from;

// Compositio scholastic name (primary)
#[doc(no_inline)]
pub use crate::semigroup::Compositio;

// Unitas scholastic name (primary)
#[doc(no_inline)]
pub use crate::monoid::Unitas;

#[doc(no_inline)]
#[cfg(feature = "Probatum")]
pub use crate::validated::IntoProbatum;
#[doc(no_inline)]
#[cfg(feature = "Probatum")]
pub use crate::validated::Probatum;

// GAT typeclasses (Applicatio is the GAT-based implementation)
#[doc(no_inline)]
pub use crate::typeclasses::Applicatio;
#[doc(no_inline)]
pub use crate::typeclasses::Apply;
#[doc(no_inline)]
pub use crate::typeclasses::Functor;
#[doc(no_inline)]
pub use crate::typeclasses::Monad;

// Re-export Zipper for convenient access
#[doc(no_inline)]
#[cfg(feature = "alloc")]
pub use crate::zipper::Zipper;

pub mod prelude {
    //! Traits that need to be imported for the complete `ordofp` experience.
    //!
    //! The intent here is that `use ordofp::prelude::*` is enough to provide
    //! access to any missing methods advertised in ordofp's documentation.

    #[doc(no_inline)]
    pub use crate::hlist::HList; // for LEN
    #[doc(no_inline)]
    pub use crate::hlist::LiftFrom;
    #[doc(no_inline)]
    pub use crate::hlist::LiftInto;

    #[doc(no_inline)]
    #[cfg(feature = "Probatum")]
    pub use crate::validated::IntoProbatum;

    // GAT typeclasses
    #[doc(no_inline)]
    pub use crate::typeclasses::Applicatio;
    #[doc(no_inline)]
    pub use crate::typeclasses::Apply;
    #[doc(no_inline)]
    pub use crate::typeclasses::Functor;
    #[doc(no_inline)]
    pub use crate::typeclasses::Monad;
}
