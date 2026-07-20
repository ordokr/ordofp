//! Effect Trait - Core effect type definition
//!
//! > *"Effectus sequitur causam"*
//! > — The effect follows the cause. (Scholastic axiom)

use core::fmt::Debug;

/// Marker trait for effect types.
///
/// `Effectus` marks types that represent computational effects, such as
/// IO operations, state manipulation, error handling, or async operations.
///
/// Effects are not executed directly; instead, they are interpreted by
/// effect handlers ([`EffectusHandler`](super::EffectusHandler) or
/// [`EffectusHandlerAsync`](super::EffectusHandlerAsync)).
///
/// # Laws
///
/// Effect types should be:
/// 1. **Pure descriptions** - An effect value describes an action, not executes it
/// 2. **Composable** - Effects can be combined into larger effect descriptions
/// 3. **Interpretable** - Effects have at least one valid interpretation (handler)
///
/// # Implementing Effectus
///
/// Most effect types are simple marker types:
///
/// ```rust
/// use ordofp_core::effects::Effectus;
///
/// /// Custom logging effect
/// pub struct LogEffectus;
/// impl Effectus for LogEffectus {}
///
/// /// Parameterized database effect
/// pub struct DatabaseEffectus<C> {
///     _phantom: std::marker::PhantomData<C>,
/// }
/// impl<C: Send + Sync + 'static> Effectus for DatabaseEffectus<C> {}
/// ```
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{Effectus, IoEffectus};
///
/// fn requires_io<E: Effectus>() {
///     // Function that works with any effect type
/// }
///
/// requires_io::<IoEffectus>();
/// ```
pub trait Effectus: Send + Sync + 'static {}

/// A trait for effects that carry a value.
///
/// Some effects need to carry data, such as a log message for a logging effect
/// or a query for a database effect.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{Effectus, EffectusWithValue};
///
/// pub enum LogLevel {
///     Info,
///     Warn,
///     Error,
/// }
///
/// pub struct LogEffect {
///     pub message: String,
///     pub level: LogLevel,
/// }
///
/// impl Effectus for LogEffect {}
///
/// impl EffectusWithValue for LogEffect {
///     type Value = String;
///
///     fn value(&self) -> &Self::Value {
///         &self.message
///     }
/// }
///
/// let effect = LogEffect { message: "hello".to_string(), level: LogLevel::Info };
/// assert_eq!(effect.value(), "hello");
/// ```
pub trait EffectusWithValue: Effectus {
    /// The type of value carried by this effect.
    type Value;

    /// Get a reference to the carried value.
    fn value(&self) -> &Self::Value;
}

/// A trait for effects that can produce a result type.
///
/// This is used for effects that, when handled, produce a specific result type.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{Effectus, EffectusProduces};
///
/// pub struct ReadFileEffect {
///     pub path: String,
/// }
///
/// impl Effectus for ReadFileEffect {}
///
/// impl EffectusProduces for ReadFileEffect {
///     type Result = Result<String, std::io::Error>;
/// }
/// ```
pub trait EffectusProduces: Effectus {
    /// The type produced when this effect is handled.
    type Result;
}

/// Trait for effect types that can be combined.
///
/// This allows composing multiple effects into a single effect type.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{CombinedEffectus, Effectus, EffectusCombine};
///
/// struct MyIoEffectus;
/// impl Effectus for MyIoEffectus {}
///
/// struct MyErrorEffectus;
/// impl Effectus for MyErrorEffectus {}
///
/// impl EffectusCombine<MyErrorEffectus> for MyIoEffectus {
///     type Combined = CombinedEffectus<MyIoEffectus, MyErrorEffectus>;
///
///     fn combine(self, _other: MyErrorEffectus) -> Self::Combined {
///         CombinedEffectus::new()
///     }
/// }
///
/// // Combine IO and Error effects
/// let combined = MyIoEffectus.combine(MyErrorEffectus);
/// let _ = combined;
/// ```
pub trait EffectusCombine<Other: Effectus>: Effectus {
    /// The combined effect type.
    type Combined: Effectus;

    /// Combine this effect with another.
    fn combine(self, other: Other) -> Self::Combined;
}

/// A combined effect type representing both effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombinedEffectus<E1, E2> {
    _e1: core::marker::PhantomData<E1>,
    _e2: core::marker::PhantomData<E2>,
}

impl<E1, E2> CombinedEffectus<E1, E2> {
    /// Create a new combined effect.
    #[inline]
    pub const fn new() -> Self {
        CombinedEffectus {
            _e1: core::marker::PhantomData,
            _e2: core::marker::PhantomData,
        }
    }
}

impl<E1, E2> Default for CombinedEffectus<E1, E2> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<E1: Effectus, E2: Effectus> Effectus for CombinedEffectus<E1, E2> {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEffect;
    impl Effectus for TestEffect {}

    #[test]
    fn test_effectus_marker() {
        fn accepts_effectus<E: Effectus>() {}
        accepts_effectus::<TestEffect>();
    }

    #[test]
    fn test_combined_effectus() {
        struct Effect1;
        impl Effectus for Effect1 {}

        struct Effect2;
        impl Effectus for Effect2 {}

        fn accepts_effectus<E: Effectus>() {}
        accepts_effectus::<CombinedEffectus<Effect1, Effect2>>();
    }
}
