//! Linear Types Module - Resource-safe programming with single-use guarantees
//!
//! > *"Semel et simul"*
//! > — Once and together. (Medieval Latin)
//!
//! This module provides linear type abstractions inspired by Idris 2's Quantitative
//! Type Theory (QTT) and Haskell's Linear Types extension. Linear types enforce
//! that values are used exactly once, enabling compile-time resource safety.
//!
//! # Overview
//!
//! Linear types provide:
//!
//! - [`Linearis`] - A wrapper enforcing single-use semantics
//! - [`Unrestricted`] - Explicitly non-linear (can be used multiple times)
//! - [`FunctorLinearis`] - Functor operations preserving linearity
//! - [`MonadLinearis`] - Monadic operations for linear computations
//! - [`Res`] - Resource management with guaranteed cleanup
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------:|
//! | Linear | Linearis | *linearis* = of a line, linear |
//! | Once | Semel | *semel* = once, a single time |
//! | Unrestricted | Liber | *liber* = free, unrestricted |
//! | Resource | Res | *res* = thing, matter, resource |
//! | Consume | Consumere | *consumere* = to use up |
//! | Duplicate | Duplicare | *duplicare* = to double |
//!
//! # Multiplicity System
//!
//! Following Idris 2's QTT, we distinguish three multiplicities:
//!
//! - **0 (Erased)**: Value exists only at compile time, erased at runtime
//! - **1 (Linear)**: Value must be used exactly once
//! - **ω (Unrestricted)**: Value can be used any number of times
//!
//! Rust's ownership system naturally enforces linearity through move semantics.
//! This module provides explicit types and traits to make linear programming
//! more ergonomic and self-documenting.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::linear::Linearis;
//!
//! // A resource that must be used exactly once
//! let file = Linearis::new(String::from("data.txt"));
//!
//! // Use the value linearly - this consumes it
//! let contents = file.consume_with(|name| format!("contents of {name}"));
//! assert_eq!(contents, "contents of data.txt");
//!
//! // file cannot be used again - it has been consumed!
//! ```
//!
//! # Feature Flags
//!
//! This module is available with the `linear` feature:
//!
//! ```toml
//! [dependencies]
//! ordofp = { version = "3.0", features = ["linear"] }
//! ```

mod combinators;
mod functor_linearis;
mod linearis;
mod monad_linearis;
mod res;
mod unrestricted;

pub use combinators::{
    CurriedLinear, consume_both, consume_either, linear_apply, linear_compose, linear_const,
    linear_curry, linear_discard, linear_dup, linear_first, linear_flip, linear_pair,
    linear_second, linear_seq_first, linear_seq_second, linear_swap, linear_uncurry,
};
pub use functor_linearis::{FunctorLinearis, FunctorLinearisTuple};
pub use linearis::{Linearis, LinearisExt};
pub use monad_linearis::{BindLinearis, MonadLinearis, join_linear, kleisli_compose_linear};
#[cfg(feature = "std")]
pub use res::bracket_safe;
pub use res::{Res, ResAsync, bracket};
pub use unrestricted::{Liber, Unrestricted};

/// Marker trait for types that can be used linearly.
///
/// Types implementing this trait guarantee that they have meaningful
/// single-use semantics. This is primarily a documentation trait,
/// as Rust's ownership system already enforces move semantics.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::UsusLinearis;
///
/// struct FileHandle { /* ... */ }
///
/// impl UsusLinearis for FileHandle {
///     fn use_once(self) {
///         // Close the file handle
///     }
/// }
/// ```
pub trait UsusLinearis: Sized {
    /// Use the value exactly once, consuming it.
    fn use_once(self);
}

/// Marker trait for types that explicitly support duplication.
///
/// This is the opposite of linear types - types implementing `Duplicabilis`
/// can be freely copied or cloned. This trait exists to make the distinction
/// between linear and non-linear types explicit in APIs.
///
/// # Note
///
/// This is automatically implemented for types that implement `Clone`.
pub trait Duplicabilis: Clone {
    /// Create a duplicate of this value.
    fn duplicare(&self) -> Self {
        self.clone()
    }
}

impl<T: Clone> Duplicabilis for T {}

/// Marker trait for types that can be safely discarded without use.
///
/// Some linear types require explicit consumption, while others can be
/// safely dropped. This trait marks types that can be discarded.
///
/// # Note
///
/// In Rust, all types can technically be dropped, but this trait
/// documents intentional discard capability.
pub trait Discardabilis: Sized {
    /// Explicitly discard this value without using it.
    fn discard(self) {
        drop(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicabilis_for_clone_types() {
        let x = 42i32;
        let y = x.duplicare();
        assert_eq!(x, y);
    }

    #[test]
    fn test_discardabilis() {
        struct Temp(i32);
        impl Discardabilis for Temp {}

        let t = Temp(42);
        assert_eq!(t.0, 42);
        t.discard(); // Should compile and run
    }
}
