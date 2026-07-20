//! Optics for `OrdoFP`
//!
//! > *\"Aspectus est actus intellectus quo rem intuetur.\"*
//! > — An aspect is the act of the intellect by which it beholds a thing. (Scholastic)
//!
//! Composable accessors for nested immutable data: lenses, prisms, isos,
//! traversals, and profunctor-style optics.
//!
//! # `OrdoFP` 3.0 Optics Enhancement (Optica Nova)
//!
//! This module now includes advanced optics features:
//!
//! - **Profunctor Optics** - Van Laarhoven style optics for principled composition
//! - **Affine Traversals** - Optics that focus on 0 or 1 elements
//! - **At Optics** - Indexed container access with `Ad` trait
//!
//! # Scholastic Names
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Lens | Aspectus | *aspectus* = sight, appearance |
//! | Prism | Divisio | *divisio* = division |
//! | Iso | Aequivalentia | *aequivalentia* = equivalence |
//! | Traversal | Iteratio | *iteratio* = repetition |
//! | Affine | Affinis | *affinis* = neighboring, related |
//! | At | Ad | *ad* = at, to |
//! | Profunctor | Profunctor | *pro* + *functor* |
//! | Strong | Fortis | *fortis* = strong |
//! | Choice | Electio | *electio* = choice |
//!
//! # Overview
//!
//! Optics provide a unified interface for:
//! - **Aspectus** (Lens): Focus on a single field within a product type (struct)
//! - **Divisio** (Prism): Focus on a single variant within a sum type (enum)
//! - **Aequivalentia** (Iso): Bidirectional lossless transformation between two types
//! - **`IteratioAffinis`** (Affine): Focus on 0 or 1 elements (optional access)
//! - **Ad** (At): Indexed container access
//!
//! # Aspectus (Lens) Example
//!
//! ```rust
//! use ordofp_core::optics::{Aspectus, aspectus};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! // Create an aspectus focusing on the `name` field
//! let name_aspectus = aspectus(
//!     |p: &Person| p.name.clone(),
//!     |p: &Person, name: String| Person { name, age: p.age },
//! );
//!
//! let person = Person { name: "Alice".to_string(), age: 30 };
//!
//! // Get the name
//! assert_eq!(name_aspectus.get(&person), "Alice");
//!
//! // Set a new name (returns a new Person)
//! let updated = name_aspectus.set(&person, "Bob".to_string());
//! assert_eq!(updated.name, "Bob");
//!
//! // Modify the name using a function
//! let uppercased = name_aspectus.modify(&person, |s| s.to_uppercase());
//! assert_eq!(uppercased.name, "ALICE");
//! ```
//!
//! # Divisio (Prism) Example
//!
//! ```rust
//! use ordofp_core::optics::{Divisio, divisio};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum Shape {
//!     Circle(f64),
//!     Rectangle(f64, f64),
//! }
//!
//! // Create a divisio focusing on the Circle variant
//! let circle_divisio = divisio(
//!     |s: &Shape| match s {
//!         Shape::Circle(r) => Some(*r),
//!         _ => None,
//!     },
//!     |r: f64| Shape::Circle(r),
//! );
//!
//! let circle = Shape::Circle(5.0);
//! let rect = Shape::Rectangle(3.0, 4.0);
//!
//! // Preview (get if the variant matches)
//! assert_eq!(circle_divisio.preview(&circle), Some(5.0));
//! assert_eq!(circle_divisio.preview(&rect), None);
//!
//! // Review (construct the variant)
//! assert_eq!(circle_divisio.review(10.0), Shape::Circle(10.0));
//! ```
//!
//! # Aequivalentia (Iso) Example
//!
//! ```rust
//! use ordofp_core::optics::{Aequivalentia, aequivalentia};
//!
//! // Isomorphism between (A, B) and (B, A)
//! let swap_aeq = aequivalentia(
//!     |pair: &(i32, String)| (pair.1.clone(), pair.0),
//!     |pair: &(String, i32)| (pair.1, pair.0.clone()),
//! );
//!
//! let pair = (42, "hello".to_string());
//! let swapped = swap_aeq.forward(&pair);
//! assert_eq!(swapped, ("hello".to_string(), 42));
//!
//! let back = swap_aeq.backward(&swapped);
//! assert_eq!(back, pair);
//! ```
//!
//! # Composition
//!
//! Optics can be composed to access deeply nested data:
//!
//! ```rust
//! use ordofp_core::optics::{Aspectus, aspectus};
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct Address {
//!     street: String,
//!     city: String,
//! }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct Person {
//!     name: String,
//!     address: Address,
//! }
//!
//! let address_aspectus = aspectus(
//!     |p: &Person| p.address.clone(),
//!     |p: &Person, addr: Address| Person { name: p.name.clone(), address: addr },
//! );
//!
//! let street_aspectus = aspectus(
//!     |a: &Address| a.street.clone(),
//!     |a: &Address, street: String| Address { street, city: a.city.clone() },
//! );
//!
//! // Compose aspectus to focus on person.address.street
//! let person_street = address_aspectus.compose(&street_aspectus);
//!
//! let person = Person {
//!     name: "Alice".to_string(),
//!     address: Address {
//!         street: "123 Main St".to_string(),
//!         city: "Springfield".to_string(),
//!     },
//! };
//!
//! assert_eq!(person_street.get(&person), "123 Main St");
//!
//! let updated = person_street.set(&person, "456 Oak Ave".to_string());
//! assert_eq!(updated.address.street, "456 Oak Ave");
//! ```
//!
//! # Performance: read-only access on hot paths
//!
//! [`Aspectus::get`] returns the focus **by value**, so the getter must clone it;
//! worse, [`ComposedAspectus::get`] clones the *entire intermediate* structure on
//! every read (measured ~144 ns / ~7 allocations for a `String` field two levels
//! deep). When you only need to **read** (not `set`/`modify`), use the borrowing
//! [`AspectusRef`] and [`AspectusRef::compose`] instead: reads return a `&` and
//! compose with **zero clones**, matching hand-written field access (~0.27 ns,
//! ~520× faster). Reach for `Aspectus` only when you need the owning `set`/`modify`.
//!
//! ```rust
//! use ordofp_core::optics::AspectusRef;
//!
//! struct Address { street: String }
//! struct Person { address: Address }
//!
//! let address = AspectusRef::new(|p: &Person| &p.address);
//! let street = AspectusRef::new(|a: &Address| &a.street);
//! let person_street = address.compose(&street);
//!
//! let p = Person { address: Address { street: "123 Main St".to_string() } };
//! assert_eq!(person_street.get(&p), "123 Main St"); // no clone
//! ```
//!
//! # Aspectus Laws
//!
//! A well-behaved aspectus must satisfy three laws:
//!
//! 1. **`GetSet`**: `aspectus.set(s, aspectus.get(s)) == s`
//!    Setting what you got doesn't change anything.
//!
//! 2. **`SetGet`**: `aspectus.get(aspectus.set(s, a)) == a`
//!    Getting what you set returns what you set.
//!
//! 3. **`SetSet`**: `aspectus.set(aspectus.set(s, a), b) == aspectus.set(s, b)`
//!    Setting twice is the same as setting once with the final value.
//!
//! # Divisio Laws
//!
//! A well-behaved divisio must satisfy two laws:
//!
//! 1. **`PreviewReview`**: `divisio.preview(divisio.review(a)) == Some(a)`
//!    Previewing a reviewed value returns that value.
//!
//! 2. **`ReviewPreview`**: If `divisio.preview(s) == Some(a)`, then `divisio.review(a) == s`
//!    Reviewing a previewed value reconstructs the original (when the preview succeeds).

pub mod affine;
pub mod at;
mod iso;
mod lens;
mod prism;
pub mod profunctor;
/// Iteratio (traversal) optics: focus on zero-or-more elements of a
/// structure at once, for bulk get/modify.
pub mod traversal;

// Primary exports (scholastic names)
pub use iso::{
    Aequivalentia, AequivalentiaRef, ComposedAequivalentia, PermutatioAequivalentia, aequivalentia,
    identitas, permutatio,
};
pub use lens::{Aspectus, AspectusRef, ComposedAspectus, ComposedAspectusRef, aspectus};
pub use prism::{ComposedDivisio, Divisio, DivisioRef, divisio};
pub use traversal::Iteratio;

// Profunctor optics exports
pub use profunctor::{
    AequivalentiaProfunctor, AequivalentiaSimplexProf, AspectusProfunctor, AspectusSimplexProf,
    ComposedOptic, DivisioProfunctor, DivisioSimplexProf, Electio, Fortis, FunctioProf,
    OpticumProfunctor, Profunctor, aequivalentia_profunctor, aspectus_profunctor,
    aspectus_profunctor_poly, divisio_profunctor, divisio_profunctor_poly,
};

// Affine traversal exports
pub use affine::{
    ComposedIteratioAffinis, IteratioAffinis, IteratioAffinisPolymorphica, iteratio_affinis,
    iteratio_option,
};
#[cfg(feature = "alloc")]
pub use affine::{iteratio_at_index, iteratio_at_key};

// At (indexed access) exports
pub use at::{Ad, AdExt, AdInserere, AdRemovere, AspectusAd, Ix, aspectus_ad};

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::string::{String, ToString};

    // Test structures
    #[derive(Clone, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Address {
        street: String,
        city: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct PersonWithAddress {
        person: Person,
        address: Address,
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Shape {
        Circle(f64),
        Rectangle(f64, f64),
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Result2<T, E> {
        Ok(T),
        Err(E),
    }

    // ==================== Lens Tests ====================

    #[test]
    fn test_lens_get() {
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name: String| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        assert_eq!(name_lens.get(&person), "Alice");
    }

    #[test]
    fn test_lens_set() {
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name: String| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let updated = name_lens.set(&person, "Bob".to_string());
        assert_eq!(updated.name, "Bob");
        assert_eq!(updated.age, 30);
    }

    #[test]
    fn test_lens_modify() {
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name: String| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let modified = name_lens.modify(&person, |s| s.to_uppercase());
        assert_eq!(modified.name, "ALICE");
    }

    #[test]
    fn test_lens_composition() {
        let address_lens = aspectus(
            |p: &PersonWithAddress| p.address.clone(),
            |p: &PersonWithAddress, addr: Address| PersonWithAddress {
                person: p.person.clone(),
                address: addr,
            },
        );

        let street_lens = aspectus(
            |a: &Address| a.street.clone(),
            |a: &Address, street: String| Address {
                street,
                city: a.city.clone(),
            },
        );

        let composed = address_lens.compose(&street_lens);

        let person_with_addr = PersonWithAddress {
            person: Person {
                name: "Alice".to_string(),
                age: 30,
            },
            address: Address {
                street: "123 Main St".to_string(),
                city: "Springfield".to_string(),
            },
        };

        assert_eq!(composed.get(&person_with_addr), "123 Main St");

        let updated = composed.set(&person_with_addr, "456 Oak Ave".to_string());
        assert_eq!(updated.address.street, "456 Oak Ave");
    }

    #[test]
    fn test_lens_law_get_set() {
        // GetSet: aspectus.set(s, aspectus.get(s)) == s
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name: String| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let result = name_lens.set(&person, name_lens.get(&person));
        assert_eq!(result, person);
    }

    #[test]
    fn test_lens_law_set_get() {
        // SetGet: aspectus.get(aspectus.set(s, a)) == a
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name: String| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let new_name = "Bob".to_string();
        let result = name_lens.get(&name_lens.set(&person, new_name.clone()));
        assert_eq!(result, new_name);
    }

    #[test]
    fn test_lens_law_set_set() {
        // SetSet: aspectus.set(aspectus.set(s, a), b) == aspectus.set(s, b)
        let name_lens = aspectus(
            |p: &Person| p.name.clone(),
            |p: &Person, name: String| Person { name, age: p.age },
        );

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        let a = "Bob".to_string();
        let b = "Charlie".to_string();

        let result1 = name_lens.set(&name_lens.set(&person, a), b.clone());
        let result2 = name_lens.set(&person, b);
        assert_eq!(result1, result2);
    }

    // ==================== Prism Tests ====================

    #[test]
    fn test_prism_preview_some() {
        let circle_prism = divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        let circle = Shape::Circle(5.0);
        assert_eq!(circle_prism.preview(&circle), Some(5.0));
    }

    #[test]
    fn test_prism_preview_none() {
        let circle_prism = divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        let rect = Shape::Rectangle(3.0, 4.0);
        assert_eq!(circle_prism.preview(&rect), None);
    }

    #[test]
    fn test_prism_review() {
        let circle_prism = divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        assert_eq!(circle_prism.review(10.0), Shape::Circle(10.0));
    }

    #[test]
    fn test_prism_modify_some() {
        let circle_prism = divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        let circle = Shape::Circle(5.0);
        let doubled = circle_prism.modify(&circle, |r| r * 2.0);
        assert_eq!(doubled, Some(Shape::Circle(10.0)));
    }

    #[test]
    fn test_prism_modify_none() {
        let circle_prism = divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        let rect = Shape::Rectangle(3.0, 4.0);
        let result = circle_prism.modify(&rect, |r| r * 2.0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_prism_composition() {
        // Divisio for Result2::Ok
        let ok_prism = divisio(
            |r: &Result2<Shape, String>| match r {
                Result2::Ok(s) => Some(s.clone()),
                _ => None,
            },
            Result2::Ok,
        );

        // Divisio for Shape::Circle
        let circle_prism = divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        let composed = ok_prism.compose(&circle_prism);

        let ok_circle: Result2<Shape, String> = Result2::Ok(Shape::Circle(5.0));
        let ok_rect: Result2<Shape, String> = Result2::Ok(Shape::Rectangle(3.0, 4.0));
        let err: Result2<Shape, String> = Result2::Err("error".to_string());

        assert_eq!(composed.preview(&ok_circle), Some(5.0));
        assert_eq!(composed.preview(&ok_rect), None);
        assert_eq!(composed.preview(&err), None);
    }

    #[test]
    fn test_prism_law_preview_review() {
        // PreviewReview: divisio.preview(divisio.review(a)) == Some(a)
        let circle_prism = divisio(
            |s: &Shape| match s {
                Shape::Circle(r) => Some(*r),
                _ => None,
            },
            Shape::Circle,
        );

        let radius = 5.0;
        let result = circle_prism.preview(&circle_prism.review(radius));
        assert_eq!(result, Some(radius));
    }

    // ==================== Iso Tests ====================

    #[test]
    fn test_iso_forward() {
        let swap_iso = aequivalentia(
            |(a, b): &(i32, String)| (b.clone(), *a),
            |(b, a): &(String, i32)| (*a, b.clone()),
        );

        let pair = (42, "hello".to_string());
        let swapped = swap_iso.forward(&pair);
        assert_eq!(swapped, ("hello".to_string(), 42));
    }

    #[test]
    fn test_iso_backward() {
        let swap_iso = aequivalentia(
            |(a, b): &(i32, String)| (b.clone(), *a),
            |(b, a): &(String, i32)| (*a, b.clone()),
        );

        let pair = ("hello".to_string(), 42);
        let back = swap_iso.backward(&pair);
        assert_eq!(back, (42, "hello".to_string()));
    }

    #[test]
    fn test_iso_roundtrip_forward_backward() {
        let swap_iso = aequivalentia(
            |(a, b): &(i32, String)| (b.clone(), *a),
            |(b, a): &(String, i32)| (*a, b.clone()),
        );

        let original = (42, "hello".to_string());
        let roundtrip = swap_iso.backward(&swap_iso.forward(&original));
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn test_iso_roundtrip_backward_forward() {
        let swap_iso = aequivalentia(
            |(a, b): &(i32, String)| (b.clone(), *a),
            |(b, a): &(String, i32)| (*a, b.clone()),
        );

        let original = ("hello".to_string(), 42);
        let roundtrip = swap_iso.forward(&swap_iso.backward(&original));
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn test_iso_composition() {
        // Aequivalentia from (A, B) to (B, A)
        let swap1 = aequivalentia(
            |(a, b): &(i32, String)| (b.clone(), *a),
            |(b, a): &(String, i32)| (*a, b.clone()),
        );

        // Aequivalentia from (B, A) to ((B, A), ())
        let wrap = aequivalentia(
            |pair: &(String, i32)| (pair.clone(), ()),
            |(pair, ()): &((String, i32), ())| pair.clone(),
        );

        let composed = swap1.compose(&wrap);

        let original = (42, "hello".to_string());
        let result = composed.forward(&original);
        assert_eq!(result, (("hello".to_string(), 42), ()));
    }

    #[test]
    fn test_iso_reverse() {
        let swap_iso = aequivalentia(
            |(a, b): &(i32, String)| (b.clone(), *a),
            |(b, a): &(String, i32)| (*a, b.clone()),
        );

        let reversed = swap_iso.reverse();

        let pair = ("hello".to_string(), 42);
        let result = reversed.forward(&pair);
        assert_eq!(result, (42, "hello".to_string()));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_lens_to_iso() {
        // When an aspectus is bijective, it can be viewed as an aequivalentia
        // For example, a newtype wrapper
        #[derive(Clone, Debug, PartialEq)]
        struct Age(u32);

        let age_iso = aequivalentia(|a: &Age| a.0, |n: &u32| Age(*n));

        let age = Age(30);
        assert_eq!(age_iso.forward(&age), 30);
        assert_eq!(age_iso.backward(&30), Age(30));
    }

    #[test]
    fn test_complex_nested_structure() {
        #[derive(Clone, Debug, PartialEq)]
        struct Config {
            settings: Settings,
        }

        #[derive(Clone, Debug, PartialEq)]
        struct Settings {
            theme: Theme,
        }

        #[derive(Clone, Debug, PartialEq)]
        enum Theme {
            #[allow(dead_code)] // fixture variant; only Dark/Custom are exercised
            Light,
            Dark,
            Custom(String),
        }

        // Aspectus to settings
        let settings_lens = aspectus(
            |c: &Config| c.settings.clone(),
            |_c: &Config, s: Settings| Config { settings: s },
        );

        // Aspectus to theme
        let theme_lens = aspectus(
            |s: &Settings| s.theme.clone(),
            |_s: &Settings, t: Theme| Settings { theme: t },
        );

        // Compose to get config.settings.theme
        let config_theme = settings_lens.compose(&theme_lens);

        // Divisio for Custom theme
        let custom_prism = divisio(
            |t: &Theme| match t {
                Theme::Custom(s) => Some(s.clone()),
                _ => None,
            },
            Theme::Custom,
        );

        let config = Config {
            settings: Settings {
                theme: Theme::Custom("Ocean".to_string()),
            },
        };

        // Get the theme
        let theme = config_theme.get(&config);
        assert_eq!(theme, Theme::Custom("Ocean".to_string()));

        // Check if it's a custom theme
        let custom = custom_prism.preview(&theme);
        assert_eq!(custom, Some("Ocean".to_string()));

        // Update to a new theme
        let new_config = config_theme.set(&config, Theme::Dark);
        assert_eq!(new_config.settings.theme, Theme::Dark);
    }
}
