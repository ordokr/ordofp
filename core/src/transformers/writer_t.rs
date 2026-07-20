//! # Scriptor (Writer Monad)
//!
//! The Scriptor monad represents computations that produce a value along with an accumulated log.
//! It's a way to carry auxiliary data alongside the main computation result in a purely functional way.
//!
//! Note: despite living in the `transformers` module, `Scriptor<W, A>` is a
//! plain Writer *monad*, not a transformer — it has no base-monad parameter
//! `M`. A true `WriterT` is possible future work.
//!
//! ## Quick Start
//!
//! Accumulate logs alongside computations:
//!
//! ```rust
//! use ordofp_core::transformers::Scriptor;
//! use ordofp_core::typeclasses::Unitas;
//!
//! // Create a Scriptor with a value and log
//! let writer = Scriptor::new(vec!["Starting".to_string()], 42);
//!
//! // Extract the value and log
//! let (log, value) = writer.run();
//! assert_eq!(value, 42);
//! assert_eq!(log, vec!["Starting".to_string()]);
//! ```
//!
//! ## Core Concepts
//!
//! - **Value and Log**: Each Scriptor computation produces both a primary value and a log/output
//! - **Log Accumulation**: When Scriptor computations are chained, their logs are combined using the monoid operation
//! - **Pure Functional Logging**: Allows for logging without side effects
//!
//! ## Scholastic Naming
//!
//! Following `OrdoFP`'s Scholastic naming convention:
//! - `Scriptor` - Latin for "writer", the main type
//! - `dictum` - Latin for "tell/say", adds to log without changing value
//! - `ausculta` - Latin for "listen", returns both result and log

use alloc::vec::Vec;

use crate::typeclasses::Compositio;
use crate::typeclasses::Unitas;

/// The Scriptor (Writer) monad represents computations that produce a value along with an accumulated log.
///
/// # Type Parameters
///
/// - `W`: The log type, which must implement the Unitas (Monoid) trait
/// - `A`: The value type
///
/// # Use Cases
///
/// - Logging operations in a purely functional way
/// - Accumulating data alongside computations
/// - Tracking the history of operations
/// - Building audit trails for computations
/// - Collecting metrics or statistics
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Scriptor<W, A> {
    /// The log accumulated during computation
    log: W,
    /// The value produced by the computation
    value: A,
}

impl<W: Unitas + Clone, A> Scriptor<W, A> {
    /// Creates a new Scriptor with the given log and value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer: Scriptor<Vec<String>, i32> = Scriptor::new(
    ///     vec!["Created value 42".to_string()],
    ///     42
    /// );
    ///
    /// let (log, value) = writer.run();
    /// assert_eq!(value, 42);
    /// assert_eq!(log, vec!["Created value 42".to_string()]);
    /// ```
    #[inline]
    pub const fn new(log: W, value: A) -> Self {
        Scriptor { log, value }
    }

    /// Creates a Scriptor with the given log and the unit value `()`.
    ///
    /// This is useful when you only care about logging something without producing a meaningful value.
    /// Named `dictum` (Latin for "tell/say") following Scholastic naming.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::<Vec<String>, ()>::dictum(
    ///     vec!["Important message".to_string()]
    /// );
    ///
    /// let (log, value) = writer.run();
    /// assert_eq!(value, ());
    /// assert_eq!(log, vec!["Important message".to_string()]);
    /// ```
    #[inline]
    pub const fn dictum(log: W) -> Scriptor<W, ()> {
        Scriptor::new(log, ())
    }

    /// Alias for `dictum` - Creates a Scriptor with the given log and unit value.
    #[inline]
    pub const fn tell(log: W) -> Scriptor<W, ()> {
        Self::dictum(log)
    }

    /// Extracts both the value and the log from the Scriptor.
    ///
    /// Returns a tuple `(log, value)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::new(vec!["Log entry".to_string()], 42);
    /// let (log, value) = writer.run();
    /// assert_eq!(value, 42);
    /// assert_eq!(log, vec!["Log entry".to_string()]);
    /// ```
    #[inline]
    pub fn run(self) -> (W, A) {
        (self.log, self.value)
    }

    /// Extracts just the value from the Scriptor, discarding the log.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::new(vec!["Log entry".to_string()], 42);
    /// let value = writer.unwrap();
    /// assert_eq!(value, 42);
    /// ```
    #[inline]
    pub fn unwrap(self) -> A {
        self.value
    }

    /// Creates a new Scriptor with the given value and an empty log.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer: Scriptor<Vec<String>, i32> = Scriptor::pure(42);
    /// let (log, value) = writer.run();
    /// assert_eq!(value, 42);
    /// assert!(log.is_empty());
    /// ```
    #[inline]
    pub fn pure(value: A) -> Self {
        Self::new(W::empty(), value)
    }

    /// Extracts just the log from the Scriptor, discarding the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::new(vec!["Log entry".to_string()], 42);
    /// let log = writer.log();
    /// assert_eq!(log, vec!["Log entry".to_string()]);
    /// ```
    #[inline]
    pub fn log(self) -> W {
        self.log
    }

    /// Returns a reference to the value.
    #[inline]
    pub fn value_ref(&self) -> &A {
        &self.value
    }

    /// Returns a reference to the log.
    #[inline]
    pub fn log_ref(&self) -> &W {
        &self.log
    }

    /// Maps a function over the value inside this Scriptor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::new(vec!["Start".to_string()], 21);
    /// let doubled = writer.map(|x| x * 2);
    /// let (log, value) = doubled.run();
    /// assert_eq!(value, 42);
    /// assert_eq!(log, vec!["Start".to_string()]);
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> Scriptor<W, B>
    where
        F: FnOnce(A) -> B,
    {
        Scriptor {
            log: self.log,
            value: f(self.value),
        }
    }

    /// Monadic bind operation for the Scriptor monad.
    ///
    /// Sequences Scriptor computations, combining their logs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::new(vec!["Step 1".to_string()], 10);
    /// let result = writer.flat_map(|x| {
    ///     Scriptor::new(vec!["Step 2".to_string()], x * 2)
    /// });
    ///
    /// let (log, value) = result.run();
    /// assert_eq!(value, 20);
    /// assert_eq!(log, vec!["Step 1".to_string(), "Step 2".to_string()]);
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Scriptor<W, B>
    where
        F: FnOnce(A) -> Scriptor<W, B>,
    {
        let Scriptor { log, value } = f(self.value);
        Scriptor {
            log: self.log.combine(&log),
            value,
        }
    }

    /// Alias for `flat_map`.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> Scriptor<W, B>
    where
        F: FnOnce(A) -> Scriptor<W, B>,
    {
        self.flat_map(f)
    }

    /// Applies a function stored in a Scriptor to a value stored in another Scriptor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let fn_writer: Scriptor<Vec<String>, fn(i32) -> i32> = Scriptor::new(
    ///     vec!["Function".to_string()],
    ///     |x| x + 10
    /// );
    /// let val_writer = Scriptor::new(vec!["Value".to_string()], 32);
    ///
    /// let result = val_writer.apply(fn_writer);
    /// let (log, value) = result.run();
    /// assert_eq!(value, 42);
    /// assert_eq!(log, vec!["Function".to_string(), "Value".to_string()]);
    /// ```
    #[inline]
    pub fn apply<B, F>(self, wf: Scriptor<W, F>) -> Scriptor<W, B>
    where
        F: FnOnce(A) -> B,
    {
        Scriptor {
            log: wf.log.combine(&self.log),
            value: (wf.value)(self.value),
        }
    }

    /// Executes the computation and returns both the result and the accumulated log
    /// as a pair within the Scriptor. Named `ausculta` (Latin for "listen").
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::new(vec!["Log".to_string()], 42);
    /// let listened = writer.ausculta();
    /// let (outer_log, (value, inner_log)) = listened.run();
    /// assert_eq!(value, 42);
    /// assert_eq!(inner_log, vec!["Log".to_string()]);
    /// ```
    #[inline]
    pub fn ausculta(self) -> Scriptor<W, (A, W)>
    where
        W: Clone,
    {
        let log_clone = self.log.clone();
        Scriptor {
            log: self.log,
            value: (self.value, log_clone),
        }
    }

    /// Alias for `ausculta`.
    #[inline]
    pub fn listen(self) -> Scriptor<W, (A, W)>
    where
        W: Clone,
    {
        self.ausculta()
    }

    /// Modifies the log using a function that also has access to the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::writer_t::Scriptor;
    ///
    /// let writer = Scriptor::new(vec!["Original".to_string()], 42);
    /// let modified = writer.censor(|log| {
    ///     log.into_iter().map(|s| s.to_uppercase()).collect()
    /// });
    /// let (log, value) = modified.run();
    /// assert_eq!(value, 42);
    /// assert_eq!(log, vec!["ORIGINAL".to_string()]);
    /// ```
    #[inline]
    pub fn censor<F>(self, f: F) -> Scriptor<W, A>
    where
        F: FnOnce(W) -> W,
    {
        Scriptor {
            log: f(self.log),
            value: self.value,
        }
    }
}

impl<W: Unitas + Clone, A: Clone + Unitas> Compositio for Scriptor<W, A> {
    #[inline]
    fn combine(&self, other: &Self) -> Self {
        Scriptor {
            log: self.log.combine(&other.log),
            value: self.value.combine(&other.value),
        }
    }
}

impl<W: Unitas + Clone, A: Clone + Unitas> Unitas for Scriptor<W, A> {
    #[inline]
    fn empty() -> Self {
        Scriptor {
            log: W::empty(),
            value: A::empty(),
        }
    }
}

impl<W, A> IntoIterator for Scriptor<W, A> {
    type Item = A;
    type IntoIter = core::option::IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        Some(self.value).into_iter()
    }
}

/// Convenience type alias for Scriptor with `Vec<String>` as log.
pub type LogScriptor<A> = Scriptor<Vec<alloc::string::String>, A>;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec;

    #[test]
    fn test_scriptor_new() {
        let writer: Scriptor<Vec<String>, i32> = Scriptor::new(vec!["test".to_string()], 42);
        let (log, value) = writer.run();
        assert_eq!(value, 42);
        assert_eq!(log, vec!["test".to_string()]);
    }

    #[test]
    fn test_scriptor_pure() {
        let writer: Scriptor<Vec<String>, i32> = Scriptor::pure(42);
        let (log, value) = writer.run();
        assert_eq!(value, 42);
        assert!(log.is_empty());
    }

    #[test]
    fn test_scriptor_dictum() {
        let writer: Scriptor<Vec<String>, ()> =
            Scriptor::<Vec<String>, ()>::dictum(vec!["message".to_string()]);
        let (log, value) = writer.run();
        assert_eq!(value, ());
        assert_eq!(log, vec!["message".to_string()]);
    }

    #[test]
    fn test_scriptor_map() {
        let writer = Scriptor::new(vec!["start".to_string()], 21);
        let doubled = writer.map(|x| x * 2);
        let (log, value) = doubled.run();
        assert_eq!(value, 42);
        assert_eq!(log, vec!["start".to_string()]);
    }

    #[test]
    fn test_scriptor_flat_map() {
        let writer = Scriptor::new(vec!["step1".to_string()], 10);
        let result = writer.flat_map(|x| Scriptor::new(vec!["step2".to_string()], x + 5));

        let (log, value) = result.run();
        assert_eq!(value, 15);
        assert_eq!(log, vec!["step1".to_string(), "step2".to_string()]);
    }

    #[test]
    fn test_scriptor_chain() {
        let result = Scriptor::new(vec!["A".to_string()], 1)
            .flat_map(|a| Scriptor::new(vec!["B".to_string()], a + 1))
            .flat_map(|b| Scriptor::new(vec!["C".to_string()], b + 1));

        let (log, value) = result.run();
        assert_eq!(value, 3);
        assert_eq!(log, vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    }

    #[test]
    fn test_scriptor_listen() {
        let writer = Scriptor::new(vec!["log".to_string()], 42);
        let listened = writer.listen();
        let (outer_log, (value, inner_log)) = listened.run();
        assert_eq!(value, 42);
        assert_eq!(inner_log, vec!["log".to_string()]);
        assert_eq!(outer_log, vec!["log".to_string()]);
    }

    #[test]
    fn test_scriptor_censor() {
        let writer = Scriptor::new(vec!["hello".to_string()], 42);
        let censored = writer.censor(|log| log.into_iter().map(|s| s.to_uppercase()).collect());
        let (log, value) = censored.run();
        assert_eq!(value, 42);
        assert_eq!(log, vec!["HELLO".to_string()]);
    }

    #[test]
    fn test_scriptor_left_identity() {
        // pure(a).bind(f) == f(a)
        let a = 5;
        let f = |x: i32| Scriptor::new(vec!["f".to_string()], x * 2);

        let left: Scriptor<Vec<String>, _> = Scriptor::pure(a).bind(f);
        let right = f(a);

        assert_eq!(left.run(), right.run());
    }

    #[test]
    fn test_scriptor_right_identity() {
        // m.bind(pure) == m
        let m = Scriptor::new(vec!["m".to_string()], 42);
        let bound = m.clone().bind(Scriptor::pure);

        assert_eq!(m.run(), bound.run());
    }

    #[test]
    fn test_scriptor_associativity() {
        // (m.bind(f)).bind(g) == m.bind(|x| f(x).bind(g))
        let m = Scriptor::new(vec!["m".to_string()], 5);
        let f = |x: i32| Scriptor::new(vec!["f".to_string()], x + 1);
        let g = |x: i32| Scriptor::new(vec!["g".to_string()], x * 2);

        let left = m.clone().bind(f).bind(g);
        let right = m.bind(|x| f(x).bind(g));

        assert_eq!(left.run(), right.run());
    }
}
