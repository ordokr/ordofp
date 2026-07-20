//! `OrdoFP` Prelude
//!
//! > *"Initium sapientiae est definitio nominum."*
//! > — The beginning of wisdom is the definition of terms. (Socrates)
//!
//! This prelude provides convenient access to the most commonly used types
//! and traits in `OrdoFP`. Import it with:
//!
//! ```rust
//! use ordofp_core::prelude::*;
//! ```
//!
//! # What's Included
//!
//! ## Type Classes (GATs)
//! - [`Functor`] - Types that can be mapped over
//! - [`Applicative`] - Functors with pure and apply
//! - [`Monad`] - Applicatives with bind (`and_then`)
//! - [`Foldable`] - Types that can be folded
//! - [`Traversable`] - Foldables that can be traversed with an applicative effect
//!
//! ## `HList` (Heterogeneous Lists)
//! - [`trait@HList`] - Type-level heterogeneous list
//! - [`Coniunctio`] / [`Cons`] - `HList` cons cell
//! - [`Nihil`] / [`Nil`] - `HList` terminator
//! - [`hlist!`] - Macro for creating `HLists`
//!
//! ## Disiunctio (Sum Types)
//! - [`enum@Disiunctio`] / [`Disiunctio!`] - Sum type (enum and constructor macro)
//! - [`Absurdum`] - Disiunctio terminator (uninhabited; no English alias)
//!
//! ## Data Types
//! - [`Aut`] / [`Either`] - Sum of two types (Left/Right)
//!
//! ## Optics
//! - [`Aspectus`] / [`Lens`] - Focus on product fields
//! - [`Divisio`] / [`Prism`] - Focus on sum variants
//! - [`Aequivalentia`] / [`Iso`] - Bidirectional transformation
//!
//! ## Easy API
//! - State management: [`run_with_state`], [`State`]
//! - Reader/Config: [`run_with_config`], [`Reader`]
//! - Error handling: [`retry`], [`fallback`]
//! - IO: [`io`], [`IO`]
//!
//! # Naming Conventions
//!
//! `OrdoFP` uses scholastic Latin names by default, but provides English
//! aliases through the [`vernacular`](crate::vernacular) module:
//!
//! | Latin (Default) | English (Vernacular) |
//! |-----------------|----------------------|
//! | Coniunctio | Cons |
//! | Nihil | Nil |
//! | Absurdum (datatypes void) | Void |
//! | Aspectus | Lens |
//! | Divisio | Prism |
//! | Aequivalentia | Iso |
//! | Aut | Either |
//!
//! `Disiunctio` and its terminator `Absurdum` keep their Latin names even in
//! the vernacular module — no English aliases exist for them.

// =============================================================================
// Core Type Classes
// =============================================================================

pub use crate::foldable::Foldable;
pub use crate::gat::{Applicative, Apply, Functor, Monad};
// The traversable module is alloc-gated in lib.rs.
#[cfg(feature = "alloc")]
pub use crate::traversable::Traversable;

// =============================================================================
// HList
// =============================================================================

pub use crate::hlist::{Coniunctio, HList, Nihil};
// English aliases
pub use crate::hlist::Coniunctio as Cons;
pub use crate::hlist::Nihil as Nil;

// Re-export macros
pub use crate::{HList, hlist};

// =============================================================================
// Disiunctio
// =============================================================================

pub use crate::disiunctio::{
    Absurdum, Disiunctio, DisiunctioEmbedder, DisiunctioFoldable, DisiunctioInjector,
    DisiunctioMappable, DisiunctioSelector, DisiunctioSubsetter, DisiunctioTaker,
    DisiunctioUninjector,
};

// Re-export macro
pub use crate::Disiunctio;

// =============================================================================
// Data Types
// =============================================================================

pub use crate::datatypes::{Aut, Const, Identitas, Unitas};
// English aliases
pub use crate::datatypes::Aut as Either;
pub use crate::datatypes::Identitas as Identity;
pub use crate::datatypes::Unitas as Unit;

// =============================================================================
// Optics
// =============================================================================

pub use crate::optics::{
    Aequivalentia, AequivalentiaRef, Aspectus, AspectusRef, Divisio, DivisioRef, Iteratio,
    aequivalentia, aspectus, divisio,
};
// English aliases
pub use crate::optics::Aequivalentia as Iso;
pub use crate::optics::Aspectus as Lens;
pub use crate::optics::Divisio as Prism;
pub use crate::optics::Iteratio as Traversal;
pub use crate::optics::aequivalentia as iso;
pub use crate::optics::aspectus as lens;
pub use crate::optics::divisio as prism;

// =============================================================================
// Easy API (requires alloc)
// =============================================================================

#[cfg(feature = "alloc")]
pub use crate::easy::{
    IO,
    IOResult,
    MultiError,

    OptionExt,
    Reader,
    // Result extensions
    ResultExt,
    SimpleError,
    State,
    ask,
    asks,

    bool_to_option,
    bool_to_result,
    both,
    // Combinators
    chain,
    collect_results,
    eval_state,
    exec_state,
    fallback,
    get,
    gets,
    // IO
    io,
    io_both,

    io_sequence,
    modify,
    partition_results_vec,
    put,
    repeat,

    // Error
    retry,
    run_state,
    // Reader
    run_with_config,
    run_with_env,
    // State
    run_with_state,
    state_pure,

    try_all,
    when,
};

// =============================================================================
// Arena (requires alloc)
// =============================================================================

#[cfg(feature = "alloc")]
pub use crate::arena::{Arena, ArenaRef, with_arena};

// =============================================================================
// Specialization Hints
// =============================================================================

pub use crate::specialization::{black_box, cold_path, hot_path, likely, spin_loop_hint, unlikely};

// =============================================================================
// Refined Types (requires alloc)
// =============================================================================

#[cfg(feature = "alloc")]
pub use crate::refined::{
    NonNegativus as NonNegative,
    NonVacuus as NonEmpty,
    // Common predicates
    Positivus as Positive,
    Praedicatum as Predicate,
    Refinatus as Refined,
    RefinementError,
};

// =============================================================================
// Effect System (requires async)
// =============================================================================

#[cfg(feature = "async")]
pub use crate::effects::{
    Eff, EffResult, EffectSet, EffectSetVacuus as EmptyRow, assert_has_effect_type, pure_eff, send,
};

// =============================================================================
// Transformers (requires alloc)
// =============================================================================

#[cfg(feature = "alloc")]
pub use crate::transformers::{ContinuatioT, EitherT, OptionT, ReaderT, Scriptor, StateT};
// English aliases
#[cfg(feature = "alloc")]
pub use crate::transformers::ContinuatioT as ContT;
#[cfg(feature = "alloc")]
pub use crate::transformers::Scriptor as WriterT;
