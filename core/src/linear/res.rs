//! Resource Management - Safe acquisition and release of resources
//!
//! > *"Res custodienda"*
//! > — A resource to be guarded. (Latin)
//!
//! This module provides the `Res` type for safe resource management,
//! inspired by ZIO's bracket pattern and Haskell's `ResourceT`.

use alloc::boxed::Box;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;

/// A RAII guard that ensures the release function is called when dropped.
struct Guard<A, R: FnOnce(A)> {
    resource: Option<A>,
    release: Option<R>,
}

impl<A, R: FnOnce(A)> Drop for Guard<A, R> {
    fn drop(&mut self) {
        if let (Some(resource), Some(release)) = (self.resource.take(), self.release.take()) {
            release(resource);
        }
    }
}

/// A managed resource with guaranteed cleanup.
///
/// `Res<R, A>` represents a resource of type `A` that requires explicit
/// acquisition and release. The resource is guaranteed to be released
/// even if the computation using it fails.
///
/// # Type Parameters
///
/// * `R` - A marker type for the resource category (for type-level documentation)
/// * `A` - The actual resource type
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::Res;
///
/// // Define a resource that must be released after use (e.g. a file handle)
/// let file_res = Res::<(), String>::make(
///     || String::from("contents of data.txt"),
///     |_file| { /* e.g. close the file handle */ }
/// );
///
/// // Use the resource - `release` above is guaranteed to run even on panic
/// let len = file_res.use_res(|file| file.len());
/// assert_eq!(len, "contents of data.txt".len());
/// ```
pub struct Res<R, A> {
    acquire: Box<dyn FnOnce() -> A + Send>,
    release: Box<dyn FnOnce(A) + Send>,
    _resource: PhantomData<R>,
}

impl<R, A> Res<R, A> {
    /// Create a new managed resource.
    ///
    /// # Parameters
    ///
    /// * `acquire` - Function to acquire the resource
    /// * `release` - Function to release the resource (always called)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Res;
    ///
    /// let res = Res::<(), i32>::make(
    ///     || 42,
    ///     |_| println!("Resource released")
    /// );
    /// ```
    #[inline]
    pub fn make<Acq, Rel>(acquire: Acq, release: Rel) -> Self
    where
        Acq: FnOnce() -> A + Send + 'static,
        Rel: FnOnce(A) + Send + 'static,
    {
        Res {
            acquire: Box::new(acquire),
            release: Box::new(release),
            _resource: PhantomData,
        }
    }

    /// Use the resource with guaranteed cleanup.
    ///
    /// The resource is acquired, passed to the function, and then
    /// released regardless of whether the function succeeds or panics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Res;
    ///
    /// let res = Res::<(), String>::make(
    ///     || "hello".to_string(),
    ///     |s| println!("Released: {}", s)
    /// );
    ///
    /// let len = res.use_res(|s| s.len());
    /// assert_eq!(len, 5);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the internal guard invariant (the resource is present
    /// until the guard drops) is violated, which indicates a bug in this
    /// crate. Panics from `f` itself propagate; the release function still
    /// runs during unwinding because it lives in the guard's destructor.
    #[inline]
    pub fn use_res<B, F>(self, f: F) -> B
    where
        F: FnOnce(&A) -> B,
    {
        let resource = (self.acquire)();
        let guard = Guard {
            resource: Some(resource),
            release: Some(self.release),
        };
        // SAFETY: The resource is always present until the guard is dropped.
        let result = f(guard.resource.as_ref().unwrap());
        drop(guard);
        result
    }

    /// Use the resource mutably with guaranteed cleanup.
    ///
    /// # Panics
    ///
    /// Panics only if the internal guard invariant (the resource is present
    /// until the guard drops) is violated, which indicates a bug in this
    /// crate. Panics from `f` itself propagate; the release function still
    /// runs during unwinding because it lives in the guard's destructor.
    #[inline]
    pub fn use_res_mut<B, F>(self, f: F) -> B
    where
        F: FnOnce(&mut A) -> B,
    {
        let resource = (self.acquire)();
        let mut guard = Guard {
            resource: Some(resource),
            release: Some(self.release),
        };
        // SAFETY: The resource is always present until the guard is dropped.
        let result = f(guard.resource.as_mut().unwrap());
        drop(guard);
        result
    }

    /// Map a function over the resource type.
    ///
    /// This creates a new `Res` that acquires the original resource,
    /// transforms it, and releases the transformed resource.
    #[inline]
    pub fn fmap<B, F, G>(self, transform: F, inverse: G) -> Res<R, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        G: FnOnce(B) -> A + Send + 'static,
        A: 'static,
    {
        let acquire = self.acquire;
        let release = self.release;

        Res {
            acquire: Box::new(move || transform((acquire)())),
            release: Box::new(move |b| (release)(inverse(b))),
            _resource: PhantomData,
        }
    }
}

impl<R, A: 'static + Send> Res<R, A> {
    /// Combine two resources into a single resource.
    ///
    /// Both resources are acquired, and both are released when done.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Res;
    ///
    /// let res1 = Res::<(), i32>::make(|| 1, |_| {});
    /// let res2 = Res::<(), String>::make(|| "hello".to_string(), |_| {});
    ///
    /// let combined = res1.zip(res2);
    /// let result = combined.use_res(|(n, s)| format!("{}: {}", n, s));
    /// assert_eq!(result, "1: hello");
    /// ```
    #[inline]
    pub fn zip<B: 'static + Send>(self, other: Res<R, B>) -> Res<R, (A, B)> {
        let acquire1 = self.acquire;
        let release1 = self.release;
        let acquire2 = other.acquire;
        let release2 = other.release;

        Res {
            acquire: Box::new(move || ((acquire1)(), (acquire2)())),
            release: Box::new(move |(a, b)| {
                (release1)(a);
                (release2)(b);
            }),
            _resource: PhantomData,
        }
    }
}

/// Type alias for async release function to reduce type complexity.
type AsyncReleaseFn<A> = Box<dyn FnOnce(A) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// Async resource management with guaranteed cleanup.
///
/// `ResAsync<R, A>` is the async version of `Res`, allowing for
/// async acquisition and release of resources.
pub struct ResAsync<R, A> {
    acquire: Pin<Box<dyn Future<Output = A> + Send>>,
    release: AsyncReleaseFn<A>,
    _resource: PhantomData<R>,
}

impl<R, A> ResAsync<R, A> {
    /// Create a new async managed resource.
    #[inline]
    pub fn make<Acq, Rel, RelFut>(acquire: Acq, release: Rel) -> Self
    where
        Acq: Future<Output = A> + Send + 'static,
        Rel: FnOnce(A) -> RelFut + Send + 'static,
        RelFut: Future<Output = ()> + Send + 'static,
    {
        ResAsync {
            acquire: Box::pin(acquire),
            release: Box::new(move |a| Box::pin(release(a))),
            _resource: PhantomData,
        }
    }

    /// Use the async resource with guaranteed cleanup.
    pub async fn use_res_async<B, F, Fut>(self, f: F) -> B
    where
        F: FnOnce(A) -> Fut,
        Fut: Future<Output = (A, B)>,
    {
        let resource = self.acquire.await;
        let (resource, result) = f(resource).await;
        (self.release)(resource).await;
        result
    }
}

/// Simple bracket function for any value.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::bracket;
///
/// let result = bracket(
///     || 42,
///     |resource| resource * 2,
///     |resource| println!("cleaning up {resource}")
/// );
/// assert_eq!(result, 84);
/// ```
///
/// # Panics
///
/// Panics only if the internal guard invariant (the resource is present
/// until the guard drops) is violated, which indicates a bug in this crate.
/// Panics from `use_fn` propagate; `release` still runs during unwinding
/// because it lives in the guard's destructor.
#[inline]
pub fn bracket<A, B, Acquire, Use, Release>(acquire: Acquire, use_fn: Use, release: Release) -> B
where
    Acquire: FnOnce() -> A,
    Use: FnOnce(&A) -> B,
    Release: FnOnce(A),
{
    let resource = acquire();
    let guard = Guard {
        resource: Some(resource),
        release: Some(release),
    };
    // SAFETY: The resource is always present until the guard is dropped.
    let result = use_fn(guard.resource.as_ref().unwrap());
    drop(guard);
    result
}

/// Bracket that handles panics and ensures cleanup.
///
/// Uses `std::panic::catch_unwind` to ensure the release function
/// is called even if the use function panics.
///
/// # Errors
///
/// Returns `Err` carrying the panic payload if `use_fn` panicked; `release`
/// has already run on the resource by the time the error is returned.
#[cfg(feature = "std")]
#[inline]
pub fn bracket_safe<A, B, Acquire, Use, Release>(
    acquire: Acquire,
    use_fn: Use,
    release: Release,
) -> Result<B, Box<dyn std::any::Any + Send>>
where
    A: std::panic::UnwindSafe,
    Acquire: FnOnce() -> A,
    Use: FnOnce(&A) -> B + std::panic::UnwindSafe,
    Release: FnOnce(A),
{
    let resource = acquire();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| use_fn(&resource)));
    release(resource);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::{String, ToString};
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use core::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_res_basic() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = Arc::clone(&released);

        let res = Res::<(), i32>::make(
            || 42,
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
            },
        );

        let result = res.use_res(|&n| n * 2);
        assert_eq!(result, 84);
        assert!(released.load(Ordering::SeqCst));
    }

    #[test]
    fn test_res_use_mut() {
        let res = Res::<(), Vec<i32>>::make(Vec::new, |_| {});

        let result = res.use_res_mut(|v| {
            v.push(1);
            v.push(2);
            v.len()
        });

        assert_eq!(result, 2);
    }

    #[test]
    fn test_res_zip() {
        let res1 = Res::<(), i32>::make(|| 1, |_| {});
        let res2 = Res::<(), String>::make(|| "hello".to_string(), |_| {});

        let combined = res1.zip(res2);
        let result = combined.use_res(|(n, s)| format!("{n}: {s}"));

        assert_eq!(result, "1: hello");
    }

    #[test]
    fn test_bracket() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = Arc::clone(&released);

        let result = bracket(
            || 42,
            |&n| n * 2,
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
            },
        );

        assert_eq!(result, 84);
        assert!(released.load(Ordering::SeqCst));
    }

    #[test]
    fn test_res_fmap() {
        let res = Res::<(), i32>::make(|| 42, |_| {});

        let mapped = res.fmap(
            |n| n.to_string(),
            |s| {
                s.parse()
                    .expect("fmap inverse should parse '42' back to i32")
            },
        );

        let result = mapped.use_res(std::string::String::len);
        assert_eq!(result, 2); // "42" has length 2
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_res_panic_safety() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = Arc::clone(&released);

        let res = Res::<(), i32>::make(
            || 42,
            move |_| {
                released_clone.store(true, Ordering::SeqCst);
            },
        );

        let _ = catch_unwind(AssertUnwindSafe(|| {
            res.use_res(|_| {
                panic!("Oops");
            });
        }));

        assert!(
            released.load(Ordering::SeqCst),
            "Resource should be released on panic"
        );
    }
}
