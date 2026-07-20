//! Monad Transformers for composing monadic effects.
//!
//! This module provides monad transformer implementations, which allow composing different
//! monadic effects into a single monad. Monad transformers solve the problem of using multiple
//! monads together without excessive nesting.
//!
//! # What are Monad Transformers?
//!
//! Monad transformers allow you to:
//!
//! - Combine multiple monadic effects (like option, state, reader, etc.)
//! - Access the operations of all combined monads through a unified interface
//! - Avoid deeply nested monadic types
//!
//! # Core Concepts
//!
//! The key components of the monad transformer pattern:
//!
//! - **Base Monad**: The innermost monad being transformed (e.g., `Result`, `Option`, `Vec`)
//! - **Transformer**: A wrapper that adds new effects while preserving the interface
//! - **Lift**: Operations to promote values from the base monad to the transformer
//! - **Stack**: The combination of transformers and base monad creates a "stack" of effects
//!
//! # Transformer Stacks
//!
//! Transformers are typically used in stacks, with each transformer adding a new capability:
//!
//! ```text
//! ReaderT<StateT<Option<_>>> = Environment + State + Optionality
//! ```
//!
//! In this example:
//! - `Option<_>` is the base monad, providing optional computation
//! - `StateT<_>` transforms it to add state management
//! - `ReaderT<_>` adds environment access on top
//!
//! # Available Transformers
//!
//! This module provides the following monad transformers:
//!
//! - [`OptionT`]: Adds optionality to any base monad
//! - [`EitherT`]: Adds error handling with a specific error type
//! - [`ReaderT`]: Adds environment/configuration reading capabilities
//! - [`StateT`]: Adds stateful computation capabilities
//!
//! It also exports two monads that carry transformer-style names but do not
//! (yet) take a base monad:
//!
//! - [`Scriptor`]: a plain Writer monad (value + accumulated log)
//! - [`ContinuatioT`]: a plain continuation monad (CPS)
//!
//! # CPS Transformers (Phase 6)
//!
//! For O(1) bind composition, use the CPS (Church-encoded) variant
//! (requires the `transformers-cps` feature):
//! - [`ecclesia::LectorEcclesiaT`]: CPS `ReaderT`
//!
//! # Example Usage
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use ordofp_core::transformers::{OptionT, MonadTransformer};
//!
//! // OptionT over Result - combines optionality with error handling
//! let opt_result: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
//! let mapped = opt_result.map(|x| x * 2);
//! assert_eq!(mapped.run(), Ok(Some(84)));
//!
//! // Chaining computations
//! let chained = OptionT::<Result<Option<i32>, &str>>::some(10)
//!     .flat_map(|x| {
//!         if x > 5 { OptionT::some(x * 2) }
//!         else { OptionT::none() }
//!     });
//! assert_eq!(chained.run(), Ok(Some(20)));
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```
//!
//! # Implementation Pattern
//!
//! Monad transformers generally follow this implementation pattern:
//!
//! 1. Define a new type that wraps a function or value with the base monad inside
//! 2. Implement the [`MonadTransformer`] trait to provide lifting capabilities
//! 3. Implement `Functor`, `Apply`, `Applicative`, and `Monad` operations
//! 4. Provide additional methods specific to the transformer (like `run`, `exec`, etc.)

#[cfg(feature = "alloc")]
extern crate alloc;

mod either_t;
mod option_t;

#[cfg(feature = "alloc")]
mod reader_t;

#[cfg(feature = "alloc")]
mod state_t;

#[cfg(feature = "alloc")]
pub mod writer_t;

#[cfg(feature = "alloc")]
mod cont_t;

// Async transformers (OrdoFP 2.0)
// Requires the "async" feature flag
#[cfg(feature = "async")]
pub mod async_transforms;

#[cfg(feature = "transformers-cps")]
pub mod ecclesia;

pub use either_t::EitherT;
pub use option_t::OptionT;

#[cfg(feature = "alloc")]
pub use reader_t::ReaderT;

#[cfg(feature = "alloc")]
pub use state_t::StateT;

#[cfg(feature = "alloc")]
pub use writer_t::{LogScriptor, Scriptor};

#[cfg(feature = "alloc")]
pub use cont_t::ContinuatioT;

/// Trait for monad transformers.
///
/// This trait provides a common interface for all monad transformers, allowing
/// them to be used in a Universalis way regardless of the specific transformer type.
/// By implementing this trait, a type declares its capability to lift values
/// from a base monad into the transformer context.
///
/// # Laws
///
/// Implementations must satisfy these laws:
///
/// 1. **Lift Preserves Identity:**
///    ```text
///    lift(pure(x)) == pure(x)
///    ```
///
/// 2. **Lift Preserves Bind:**
///    ```text
///    lift(m).flat_map(|x| lift(f(x))) == lift(m.flat_map(f))
///    ```
///
/// # Example
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::{OptionT, MonadTransformer};
///
/// // Lift a Result into OptionT
/// let base: Result<i32, &str> = Ok(42);
/// let lifted: OptionT<Result<Option<i32>, &str>> = OptionT::lift_m(base);
/// assert_eq!(lifted.run(), Ok(Some(42)));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
pub trait MonadTransformer {
    /// The type of the base monad.
    type BaseMonad;

    /// Lifts a value from the base monad into the transformer.
    ///
    /// This method takes a value from the base monad and wraps it in the
    /// transformer's context, preserving the monadic properties.
    fn lift(base: Self::BaseMonad) -> Self;
}

/// Helper function to lift a value from a base monad into a monad transformer.
///
/// This function provides a convenient way to lift values without needing to
/// specify the transformer type explicitly in many cases.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::{OptionT, lift};
///
/// let base: Result<i32, &str> = Ok(42);
/// let lifted: OptionT<Result<Option<i32>, &str>> = lift(base);
/// assert_eq!(lifted.run(), Ok(Some(42)));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
pub fn lift<T, M>(m: M) -> T
where
    T: MonadTransformer<BaseMonad = M>,
{
    T::lift(m)
}
