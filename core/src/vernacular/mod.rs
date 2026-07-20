//! Vernacular API - English Aliases for `OrdoFP`
//!
//! This module provides English-language aliases for `OrdoFP`'s Latin-named types,
//! making the library more accessible to developers who prefer conventional naming.
//!
//! # Overview
//!
//! `OrdoFP` uses scholastic Latin naming to honor the philosophical traditions
//! underlying functional programming. However, we recognize this can be a barrier.
//! This module bridges the gap with familiar English names.
//!
//! # Usage
//!
//! ```rust
//! // Instead of:
//! // use ordofp_core::effects::{Computatio, Effectus, TractatorAlgebraicus};
//!
//! // You can use (the effect aliases require the `async` feature):
//! # #[cfg(feature = "async")]
//! # {
//! #[allow(unused_imports)]
//! use ordofp_core::vernacular::{Computation, Effect, AlgebraicHandler};
//!
//! // `Computation`, `Effect`, and `AlgebraicHandler` are aliases for
//! // `Computatio`, `Effectus`, and `TractatorAlgebraicus` respectively.
//! fn _type_check<E: Effect>() {}
//! # }
//! ```
//!
//! # Naming Philosophy
//!
//! | Latin (Scholastic) | English (Vernacular) | Meaning |
//! |--------------------|----------------------|---------|
//! | Computatio | Computation | A computation with effects |
//! | Effectus | Effect | A side effect |
//! | Tractator | Handler | An effect handler |
//! | Continuatio | Continuation | A continuation |
//! | Praedicatum | Predicate | A predicate for refinement |
//! | Refinatus | Refined | A refined type |

// =============================================================================
// Effect System Aliases
// =============================================================================

#[cfg(feature = "async")]
pub use crate::effects::{
    AsyncEffectus as AsyncEffect,
    ComposedHandler,
    // Computation types
    Computatio as Computation,

    ComputatioStatus as ComputationStatus,

    ConsolaOp as ConsoleOp,

    // Multi-shot continuations
    Continuatio as Continuation,
    ContinuatioAffinis as AffineContinuation,
    ContinuatioPluries as MultiContinuation,
    // Continuation types
    ContinuatioSemel as OnceContinuation,
    // Handler utilities
    DefaultHandler,
    // Eff monad
    Eff,
    EffResult,
    // Row types (v2)
    EffectRow,
    EffectSet,
    EffectSetVacuus as EmptyRow,
    // Core effect traits
    Effectus as Effect,
    // Algebraic effects
    EffectusAlgebraicus as AlgebraicEffect,
    EffectusHandler as EffectHandler,
    EffectusHandlerAsync as AsyncEffectHandler,

    ErrorEffectus as ErrorEffect,
    ErrorHandler,

    ErrorOp,
    IdentityHandler,

    // Common effects
    IoEffectus as IoEffect,
    LectorHandler as ReaderHandler,
    LectorOp as ReaderOp,
    Operatio as Operation,
    PurusEffectus as PureEffect,

    RandomEffectus as RandomEffect,
    ReaderEffectus as ReaderEffect,
    ResourceEffectus as ResourceEffect,
    ScriptorEffectus as WriterEffect,
    ScriptorHandler as WriterHandler,
    ScriptorOp as WriterOp,
    // Sem (freer monad)
    Sem,
    SemResult,
    StatusEffectus as StateEffect,
    // Built-in handlers
    StatusHandler as StateHandler,
    // Built-in operations
    StatusOp as StateOp,
    TempusEffectus as TimeEffect,
    TractatorAlgebraicus as AlgebraicHandler,
    TractatorContinuatio as ContinuationHandler,
    TractatorMulti as MultiHandler,

    TractatorResult as HandlerResult,

    pure_eff as pure,
    pure_sem,
    raise,
    send,
};

// =============================================================================
// Refined Types Aliases
// =============================================================================

#[cfg(feature = "alloc")]
pub use crate::refined::{
    Aut as Xor,
    Et as And,
    Falsum as False,

    Impar as Odd,
    Implicatio as Implies,
    IntraFines as InRange,
    MagnitudoExacta as ExactSize,
    MagnitudoMaxima as MaxSize,
    MagnitudoMinima as MinSize,
    MaiorQuam as GreaterThan,
    MinorQuam as LessThan,
    Negativus as Negative,
    // Combinators
    Non as Not,
    NonNegativus as NonNegative,
    NonNullus as NonZero,
    NonVacuus as NonEmpty,
    Par as Even,
    // Common predicates
    Positivus as Positive,
    // Core types
    Praedicatum as Predicate,
    Refinatus as Refined,
    RefinementError,

    Vel as Or,
    Verum as True,
};

// =============================================================================
// HList Aliases
// =============================================================================

pub use crate::hlist::{Coniunctio as Cons, HList, Nihil as Nil};

// =============================================================================
// Disiunctio Aliases
// =============================================================================

pub use crate::disiunctio::{
    Absurdum, Disiunctio, DisiunctioInjector as Injector, DisiunctioUninjector as Uninjector,
};

// =============================================================================
// Functor/Monad Aliases
// =============================================================================

pub use crate::gat::{Applicative, Apply, Functor, Monad};

// =============================================================================
// Optics Aliases
// =============================================================================

pub use crate::optics::{
    // At (Ad)
    Ad as At,
    AdExt as AtExt,
    AdInserere as AtInsert,
    AdRemovere as AtRemove,
    // Iso (Aequivalentia)
    Aequivalentia as Iso,
    AequivalentiaRef as IsoRef,
    // Lens (Aspectus)
    Aspectus as Lens,
    AspectusAd as AtLens,
    AspectusRef as LensRef,
    ComposedAequivalentia as ComposedIso,
    ComposedAspectus as ComposedLens,
    ComposedDivisio as ComposedPrism,
    // Prism (Divisio)
    Divisio as Prism,
    DivisioRef as PrismRef,
    // Traversal (Iteratio)
    Iteratio as Traversal,

    // Affine
    IteratioAffinis as Affine,
    Ix as Index,
    aequivalentia as iso,
    aspectus as lens,

    aspectus_ad as at_lens,
    divisio as prism,

    identitas as identity_iso,
    iteratio_affinis as affine,
    iteratio_option as affine_option,

    permutatio as swap_iso,
};

// =============================================================================
// Transformer Aliases
// =============================================================================

pub use crate::transformers::{EitherT, MonadTransformer, OptionT as MaybeT};

#[cfg(feature = "alloc")]
pub use crate::transformers::{ContinuatioT as ContT, ReaderT, Scriptor as Writer, StateT};

// =============================================================================
// Data Types Aliases
// =============================================================================

pub use crate::datatypes::{
    Absurdum as Void,

    // Either variant
    Aut as Either,

    // Constant functor
    Const,

    // Identity
    Identitas as Identity,
    // Phantom
    Phantasma as Phantom,

    // Lazy evaluation
    Pigritia as Lazy,

    // Unit and Void
    Unitas as Unit,
};

// =============================================================================
// Distributed Aliases
// =============================================================================

#[cfg(feature = "distributed")]
pub use crate::distributed::{
    AdministratorGregis as ClusterManager,
    AffinitasNodi as NodeAffinity,

    CaputNuntii as MessageHeader,
    ComputatioDistributa as DistributedComputation,

    // Cluster types
    ConfiguratioGregis as ClusterConfig,
    CorpusNuntii as MessageBody,
    // Distributed effects
    EffectusDistributus as DistributedEffect,
    Expressio as Expression,
    FacultatesNodi as NodeCapabilities,
    GenusNuntii as MessageType,
    InformationesNodi as NodeInfo,
    InscriptioNodi as NodeAddress,
    MethodusInventionis as DiscoveryMethod,
    MunusNodi as NodeRole,
    // Node types
    NodusIdentitas as NodeId,
    // Serializable types
    NodusSerializabilis as SerializableNode,
    // Protocol types
    Nuntius as Message,
    ProtocollumConsensus as ConsensusProtocol,
    SalusGregis as ClusterHealth,

    StatusGregis as ClusterState,
    StatusNodi as NodeStatus,
    TabulaDirigendi as RoutingTable,
    TractatorDirigens as RoutingHandler,
    TractatorDistributus as DistributedHandler,
    ValorConstans as ConstValue,
    VersioProtocolli as ProtocolVersion,
};

// =============================================================================
// Supervision Aliases
// =============================================================================

#[cfg(feature = "supervision")]
pub use crate::supervision::{
    Arbor as SupervisionTree, StrategiaSupervisionis as SupervisionStrategy,
};

// =============================================================================
// Tracing Aliases
// =============================================================================

#[cfg(feature = "alloc")]
pub use crate::tracing::{
    Attributum as Attribute, AttributumValue as AttributeValue,
    CollectorMemoriae as MemoryCollector, CollectorNullus as NullCollector,
    CollectorVestigium as TraceCollector, ConfigVestigium as TraceConfig,
    ContextusVestigium as TraceContext, EventusId as EventId, EventusKind as EventKind,
    EventusVestigium as TraceEvent, Gradus as Level, Samplator as Sampler, SpatiumId as SpanId,
    TractatorVestigians as TracingHandler, VestigiumId as TraceId,
};

// =============================================================================
// Async Aliases
// =============================================================================

#[cfg(feature = "async")]
pub use crate::async_core::{Fibra as Fiber, FibraId as FiberId, FibraStatus as FiberStatus};

// =============================================================================
// Comonad Aliases
// =============================================================================

pub use crate::comonad::Vestigium as Traced;

// =============================================================================
// Foldable Aliases
// =============================================================================

pub use crate::foldable::Foldable;

// =============================================================================
// Prelude - Common Imports
// =============================================================================

/// A prelude module containing the most commonly used types and traits.
///
/// # Usage
///
/// ```rust
/// use ordofp_core::vernacular::prelude::*;
/// ```
pub mod prelude {
    #[cfg(feature = "async")]
    pub use super::{
        AlgebraicEffect,

        // Handler types
        AlgebraicHandler,
        Computation,

        // Eff monad
        Eff,
        // Core effect types
        Effect,
        EffectHandler,
        // Row types
        EffectRow,
        EmptyRow,
        ErrorEffect,

        // Common effects
        IoEffect,
        ReaderEffect,

        StateEffect,
        pure,
        send,
    };

    #[cfg(feature = "alloc")]
    pub use super::{
        InRange,
        NonEmpty,
        NonNegative,
        Positive,
        Predicate,
        // Refined types
        Refined,
    };

    // Always available
    pub use super::{
        Applicative,
        Cons,
        // Data types
        Either,

        // Foldable
        Foldable,

        // Functor hierarchy
        Functor,
        // HList
        HList,
        Iso,
        // Optics
        Lens,
        Monad,

        Nil,

        Prism,
        iso,
        lens,
        prism,
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlist_aliases() {
        let list: HList![i32, bool, &str] = crate::hlist![1, true, "hello"];
        assert_eq!(list.head, 1);
    }

    #[test]
    fn test_either_alias() {
        let e: Either<&str, i32> = Either::Dexter(42);
        assert!(e.is_dexter());
    }
}
