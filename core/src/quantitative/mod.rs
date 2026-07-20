//! Quantitative Types Module - Type-level multiplicity tracking
//!
//! > *"Quot usus, tot modi"*
//! > — As many uses, so many modes. (Neo-Latin)
//!
//! This module provides quantitative type abstractions inspired by Idris 2's
//! Quantitative Type Theory (QTT). QTT extends linear types with explicit
//! multiplicity annotations that track how many times a value is used.
//!
//! # Overview
//!
//! Quantitative Type Theory distinguishes three multiplicities:
//!
//! | Multiplicity | Name | Meaning |
//! |--------------|------|---------|
//! | 0 | Zero/Erased | Compile-time only, erased at runtime |
//! | 1 | One/Linear | Used exactly once |
//! | ω | Omega/Unrestricted | Used any number of times |
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------:|
//! | Multiplicity | Multiplicitas | *multiplicitas* = manifoldness |
//! | Zero | Nihil | *nihil* = nothing |
//! | One | Semel | *semel* = once |
//! | Omega | Omega | ω = unlimited |
//! | Linear | Linearis | *linearis* = of a line |
//! | Erased | Erasum | *erasum* = wiped out |
//! | Handle | Manus | *manus* = hand |
//! | Pair | Par | *par* = equal, pair |
//! | Function | Functio | *functio* = performance |
//! | Monad | Monas | *monas* = unit |
//!
//! # Type-Level Multiplicities
//!
//! This module encodes multiplicities at the type level using phantom types,
//! enabling compile-time verification of usage patterns.
//!
//! ```rust
//! use ordofp_core::quantitative::{Multiplicitas, Qtt, Semel, Omega};
//!
//! // A linear value (used exactly once)
//! let linear: Qtt<i32, Semel> = Qtt::new(42);
//! assert_eq!(linear.multiplicity(), Multiplicitas::Semel);
//!
//! // An unrestricted value (can be cloned)
//! let unrestricted: Qtt<i32, Omega> = Qtt::new(42);
//! assert_eq!(unrestricted.multiplicity(), Multiplicitas::Omega);
//! ```
//!
//! # Reference
//!
//! - [Idris 2: Quantitative Type Theory in Practice](https://arxiv.org/abs/2104.00480)
//! - [The Syntax and Semantics of Quantitative Type Theory](https://bentnib.org/quantitative-type-theory.pdf)

extern crate alloc;

mod functio;
mod manus;
mod monas;
mod multiplicitas;
mod par;
mod qtt;

pub use functio::{
    FunctioLinearis, FunctioSemel, linear_apply, linear_compose, linear_const, linear_curry,
    linear_flip, linear_uncurry,
};
pub use manus::{ManusGuard, ManusLinearis};
pub use monas::{
    MonasLinearis, QttMonad, bind_qtt, join_qtt, kleisli_qtt, lift2_qtt, map2_qtt, map3_qtt,
    purus_qtt, sequence_qtt, traverse_qtt,
};
pub use multiplicitas::{
    Multiplicitas, MultiplicitasSemiring, Nihil, Omega, Semel, Usage, is_subusage, mult_add,
    mult_mul,
};
pub use par::{AdditiveChoice, ParLinearis, TensorLinearis, WithLinearis};
pub use qtt::{Qtt, QttErasum, QttExt, QttLiber, QttLinearis};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplicitas_variants() {
        let zero = Multiplicitas::Nihil;
        let one = Multiplicitas::Semel;
        let omega = Multiplicitas::Omega;

        assert!(zero.is_nihil());
        assert!(one.is_semel());
        assert!(omega.is_omega());
    }

    #[test]
    fn test_multiplicitas_semiring() {
        // 0 * x = 0
        assert_eq!(
            mult_mul(Multiplicitas::Nihil, Multiplicitas::Omega),
            Multiplicitas::Nihil
        );

        // 1 * x = x
        assert_eq!(
            mult_mul(Multiplicitas::Semel, Multiplicitas::Omega),
            Multiplicitas::Omega
        );

        // ω + ω = ω
        assert_eq!(
            mult_add(Multiplicitas::Omega, Multiplicitas::Omega),
            Multiplicitas::Omega
        );
    }
}
