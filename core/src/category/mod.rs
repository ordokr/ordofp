//! Category Theory Foundations
//!
//! > *"Categoriae sunt praedicamenta universalia."*
//! > — Categories are universal predicaments. (Aristotle, adapted)
//!
//! This module provides advanced category theory constructs including:
//!
//! - **Genus**: Universal HKT abstraction (type constructor witnesses)
//! - **Enhanced Arrows**: `ArrowChoice`, `ArrowApply`, `ArrowLoop`
//! - **Kan Extensions**: Yoneda, Coyoneda, Left/Right Kan Extensions
//!
//! # Module Structure
//!
//! - [`genus`] - Universal HKT abstraction (Genus, `FunctorGenus`, `MonadGenus`)
//! - [`sagitta`] - Enhanced Arrow type classes (`SagittaElectio`, `SagittaApplicatio`, `SagittaCirculus`)
//! - [`kan`] - Kan extensions and Yoneda embeddings
//!
//! # Latin Nomenclature
//!
//! | Concept | Latin Name | Etymology |
//! |---------|------------|-----------|
//! | Genus (HKT) | Genus | *genus* = kind, type, class |
//! | Arrow | Sagitta | *sagitta* = arrow |
//! | `ArrowChoice` | `SagittaElectio` | *electio* = choice |
//! | `ArrowApply` | `SagittaApplicatio` | *applicatio* = application |
//! | `ArrowLoop` | `SagittaCirculus` | *circulus* = circle, loop |
//! | Right Kan Extension | `ExtensioKanDextra` | *dexter* = right |
//! | Left Kan Extension | `ExtensioKanSinistra` | *sinister* = left |
//! | Yoneda | Yoneda | Named after Nobuo Yoneda |
//! | Coyoneda | Coyoneda | Co-Yoneda (dual) |
//!
//! # MSRV
//!
//! This module uses GATs (stable) on Edition 2024; the crate itself requires the pinned nightly (see rust-toolchain.toml).

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod genus;
pub mod kan;
pub mod sagitta;

pub use genus::*;
pub use kan::*;
pub use sagitta::*;
