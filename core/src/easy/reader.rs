//! Easy Reader/Configuration Pattern
//!
//! Simplified configuration and environment handling.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::easy::*;
//!
//! struct Config {
//!     timeout_ms: u64,
//!     max_retries: u32,
//! }
//!
//! let result = run_with_config(&Config { timeout_ms: 1000, max_retries: 3 }, |config| {
//!     config.timeout_ms * config.max_retries as u64
//! });
//! assert_eq!(result, 3000);
//! ```

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;

// =============================================================================
// Basic Reader Operations
// =============================================================================

/// Run a computation with read-only configuration.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_with_config;
///
/// let result = run_with_config(&42, |config| *config * 2);
/// assert_eq!(result, 84);
/// ```
#[inline]
pub fn run_with_config<R, A, F>(config: &R, computation: F) -> A
where
    F: FnOnce(&R) -> A,
{
    computation(config)
}

/// Run a computation with owned configuration.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_with_env;
///
/// #[derive(Default)]
/// struct Config {
///     value: i32,
/// }
///
/// let result = run_with_env(Config::default(), |config: &Config| config.value);
/// assert_eq!(result, 0);
/// ```
// `env` is taken by value for call-site ergonomics: temporaries and owned
// configs can be passed without a borrow dance.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn run_with_env<R, A, F>(env: R, computation: F) -> A
where
    F: FnOnce(&R) -> A,
{
    computation(&env)
}

/// Run a computation with a locally modified environment.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_with_local;
///
/// let result = run_with_local(&42, |x| x + 10, |config| *config * 2);
/// assert_eq!(result, 104);  // (42 + 10) * 2
/// ```
#[inline]
pub fn run_with_local<R, A, M, F>(env: &R, modifier: M, computation: F) -> A
where
    R: Clone,
    M: FnOnce(R) -> R,
    F: FnOnce(&R) -> A,
{
    let local_env = modifier(env.clone());
    computation(&local_env)
}

// =============================================================================
// Reader Monad Style
// =============================================================================

/// A computation that reads from an environment.
pub struct Reader<R, A> {
    run: Box<dyn FnOnce(&R) -> A>,
}

impl<R: 'static, A: 'static> Reader<R, A> {
    /// Create a new reader computation.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&R) -> A + 'static,
    {
        Reader { run: Box::new(f) }
    }

    /// Run the computation with an environment.
    #[inline]
    pub fn run(self, env: &R) -> A {
        (self.run)(env)
    }

    /// Map over the result.
    #[inline]
    pub fn map<B: 'static, F>(self, f: F) -> Reader<R, B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        Reader::new(move |r| f((self.run)(r)))
    }

    /// Chain with another reader computation.
    #[inline]
    pub fn and_then<B: 'static, F>(self, f: F) -> Reader<R, B>
    where
        F: FnOnce(A) -> Reader<R, B> + 'static,
    {
        Reader::new(move |r| {
            let a = (self.run)(r);
            f(a).run(r)
        })
    }

    /// Sequence two computations, keeping the second result.
    #[inline]
    pub fn then<B: 'static>(self, next: Reader<R, B>) -> Reader<R, B> {
        Reader::new(move |r| {
            let _ = (self.run)(r);
            next.run(r)
        })
    }
}

/// Create a pure reader computation.
#[inline]
pub fn reader_pure<R: 'static, A: 'static>(value: A) -> Reader<R, A> {
    Reader::new(move |_| value)
}

/// Read the entire environment.
#[inline]
pub fn ask<R: Clone + 'static>() -> Reader<R, R> {
    Reader::new(|r: &R| r.clone())
}

/// Read a value derived from the environment.
#[inline]
pub fn asks<R: 'static, A: 'static, F>(f: F) -> Reader<R, A>
where
    F: FnOnce(&R) -> A + 'static,
{
    Reader::new(f)
}

/// Run a computation with a locally modified environment.
pub fn local<R: Clone + 'static, A: 'static, M, C>(modifier: M, computation: C) -> Reader<R, A>
where
    M: FnOnce(R) -> R + 'static,
    C: FnOnce() -> Reader<R, A> + 'static,
{
    Reader::new(move |r: &R| {
        let local_r = modifier(r.clone());
        computation().run(&local_r)
    })
}

// =============================================================================
// Dependency Injection Pattern
// =============================================================================

/// A simple dependency container.
pub struct Dependencies<D> {
    deps: Arc<D>,
}

impl<D> Dependencies<D> {
    /// Create a new dependency container.
    pub fn new(deps: D) -> Self {
        Dependencies {
            deps: Arc::new(deps),
        }
    }

    /// Run a computation with the dependencies.
    pub fn run<A, F>(&self, f: F) -> A
    where
        F: FnOnce(&D) -> A,
    {
        f(&self.deps)
    }

    /// Get a reference to the dependencies.
    pub fn get(&self) -> &D {
        &self.deps
    }

    /// Map the dependencies to a derived value.
    pub fn map<B, F>(&self, f: F) -> B
    where
        F: FnOnce(&D) -> B,
    {
        f(&self.deps)
    }
}

impl<D> Clone for Dependencies<D> {
    fn clone(&self) -> Self {
        Dependencies {
            deps: Arc::clone(&self.deps),
        }
    }
}

/// A trait for types that can provide dependencies.
pub trait HasDependencies {
    /// The concrete dependency bundle this type carries.
    type Deps;

    /// Borrow the dependencies for use by a reader-style computation.
    fn deps(&self) -> &Self::Deps;
}

// =============================================================================
// Configuration Builder
// =============================================================================

/// A builder for configuration with validation.
pub struct ConfigBuilder<C> {
    config: C,
    errors: Vec<alloc::string::String>,
}

impl<C: Default> ConfigBuilder<C> {
    /// Create a new config builder with default values.
    pub fn new() -> Self {
        ConfigBuilder {
            config: C::default(),
            errors: Vec::new(),
        }
    }
}

impl<C> ConfigBuilder<C> {
    /// Create a builder from an existing config.
    pub fn from(config: C) -> Self {
        ConfigBuilder {
            config,
            errors: Vec::new(),
        }
    }

    /// Modify the configuration.
    pub fn with<F>(mut self, modifier: F) -> Self
    where
        F: FnOnce(&mut C),
    {
        modifier(&mut self.config);
        self
    }

    /// Validate a condition, recording an error if it fails.
    pub fn validate<F>(mut self, predicate: F, error: &str) -> Self
    where
        F: FnOnce(&C) -> bool,
    {
        if !predicate(&self.config) {
            self.errors.push(alloc::string::String::from(error));
        }
        self
    }

    /// Build the configuration, returning errors if validation failed.
    ///
    /// # Errors
    ///
    /// Returns the messages of every [`ConfigBuilder::validate`] call
    /// whose predicate failed, in the order the validations were
    /// declared; the built configuration is discarded in that case.
    pub fn build(self) -> Result<C, Vec<alloc::string::String>> {
        if self.errors.is_empty() {
            Ok(self.config)
        } else {
            Err(self.errors)
        }
    }

    /// Build the configuration, panicking on validation errors.
    ///
    /// # Panics
    ///
    /// Panics if any [`ConfigBuilder::validate`] predicate failed; the
    /// panic message lists all recorded validation errors. Use
    /// [`ConfigBuilder::build`] for a recoverable `Result` instead.
    pub fn build_or_panic(self) -> C {
        if self.errors.is_empty() {
            self.config
        } else {
            panic!("Configuration validation failed: {:?}", self.errors)
        }
    }
}

impl<C: Default> Default for ConfigBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Environment Layers
// =============================================================================

/// A layered environment for scoped configurations.
pub struct LayeredEnv<E> {
    layers: Vec<E>,
}

impl<E> LayeredEnv<E> {
    /// Create a new layered environment.
    pub fn new() -> Self {
        LayeredEnv { layers: Vec::new() }
    }

    /// Push a new layer.
    pub fn push(&mut self, layer: E) {
        self.layers.push(layer);
    }

    /// Pop the top layer.
    pub fn pop(&mut self) -> Option<E> {
        self.layers.pop()
    }

    /// Get the top layer.
    pub fn top(&self) -> Option<&E> {
        self.layers.last()
    }

    /// Run with a temporary layer.
    pub fn with_layer<A, F>(&mut self, layer: E, f: F) -> A
    where
        F: FnOnce(&Self) -> A,
    {
        self.push(layer);
        let result = f(self);
        self.pop();
        result
    }

    /// Get all layers from bottom to top.
    pub fn layers(&self) -> &[E] {
        &self.layers
    }
}

impl<E> Default for LayeredEnv<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Clone> LayeredEnv<E> {
    /// Merge all layers using a combiner function.
    pub fn merge<F>(&self, combiner: F) -> Option<E>
    where
        F: Fn(E, &E) -> E,
    {
        if self.layers.is_empty() {
            return None;
        }

        let mut result = self.layers[0].clone();
        for layer in &self.layers[1..] {
            result = combiner(result, layer);
        }
        Some(result)
    }
}

// =============================================================================
// Scoped Local Environment
// =============================================================================

/// A scoped environment cell for implicit parameter passing.
///
/// This is a plain `RefCell`, **not** thread-local storage — to keep one
/// instance per thread, place it in a `thread_local!` block yourself.
/// `run` is also **not re-entrant**: nesting `run` calls on the same cell
/// overwrites the outer environment, and the environment is not restored
/// if the closure panics.
pub struct LocalEnv<E> {
    cell: RefCell<Option<E>>,
}

impl<E> LocalEnv<E> {
    /// Create a new local environment.
    pub const fn new() -> Self {
        LocalEnv {
            cell: RefCell::new(None),
        }
    }

    /// Run a computation with this environment.
    pub fn run<A, F>(&self, env: E, f: F) -> A
    where
        F: FnOnce() -> A,
    {
        *self.cell.borrow_mut() = Some(env);
        let result = f();
        *self.cell.borrow_mut() = None;
        result
    }

    /// Get a value from the current environment.
    ///
    /// # Panics
    /// Panics if called outside of a `run` context.
    pub fn with<A, F>(&self, f: F) -> A
    where
        F: FnOnce(&E) -> A,
    {
        let borrow = self.cell.borrow();
        f(borrow.as_ref().expect("LocalEnv: not in run context"))
    }
}

impl<E> Default for LocalEnv<E> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_with_config() {
        let result = run_with_config(&42, |config| *config * 2);
        assert_eq!(result, 84);
    }

    #[test]
    fn test_run_with_local() {
        let result = run_with_local(&42, |x| x + 10, |config| *config * 2);
        assert_eq!(result, 104);
    }

    #[test]
    fn test_reader_monad() {
        let comp = ask::<i32>().map(|x| x * 2);
        let result = comp.run(&21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_asks() {
        let comp = asks::<i32, _, _>(|x| x + 1);
        let result = comp.run(&41);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_reader_and_then() {
        let comp = ask::<i32>().and_then(|x| asks(move |y: &i32| x + y));
        let result = comp.run(&21);
        assert_eq!(result, 42); // 21 + 21
    }

    #[test]
    fn test_dependencies() {
        struct Deps {
            value: i32,
        }

        let deps = Dependencies::new(Deps { value: 42 });
        let result = deps.run(|d| d.value * 2);
        assert_eq!(result, 84);
    }

    #[test]
    fn test_config_builder() {
        #[derive(Default)]
        struct Config {
            timeout: u64,
        }

        let config = ConfigBuilder::<Config>::new()
            .with(|c| c.timeout = 1000)
            .validate(|c| c.timeout > 0, "timeout must be positive")
            .build()
            .expect("ConfigBuilder with valid timeout should build successfully");

        assert_eq!(config.timeout, 1000);
    }

    #[test]
    fn test_config_builder_validation_error() {
        #[derive(Default)]
        struct Config {
            timeout: u64,
        }

        let result = ConfigBuilder::<Config>::new()
            .validate(|c| c.timeout > 0, "timeout must be positive")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_layered_env() {
        let mut env = LayeredEnv::new();
        env.push(10);
        env.push(20);

        let result = env.with_layer(30, |e| {
            *e.top()
                .expect("layer stack should have top element while inside with_layer")
        });

        assert_eq!(result, 30);
        assert_eq!(
            *env.top()
                .expect("original layer should remain after with_layer exits"),
            20
        );
    }

    #[test]
    fn test_local_env() {
        let env: LocalEnv<i32> = LocalEnv::new();

        let result = env.run(42, || env.with(|x| *x * 2));

        assert_eq!(result, 84);
    }
}
