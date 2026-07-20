//! Resource Management - Res
//!
//! > *"Res est quod secure acquiritur et dimittetur"*
//! > — A resource is what is safely acquired and released. (Latin)
//!
//! This module provides resource management with guaranteed cleanup,
//! inspired by ZIO's `ZManaged` and Cats Effect's Resource.
//!
//! # Overview
//!
//! Resources are values that need cleanup after use. This module provides
//! the `Res` type for synchronous resources and `ResAsync` for async resources.
//! `Res` guarantees that cleanup runs even if the computation fails or panics.
//! `ResAsync` does **not** yet run its release action automatically: `uti` and
//! `uti_safe` consume the resource in `f`, so automatic cleanup awaits an API
//! redesign (see the `ResAsync::_release` field doc).
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Resource | Res | *res* = thing, matter, resource |
//! | Acquire | Acquirere | *acquirere* = to acquire |
//! | Release | Dimittere | *dimittere* = to release, dismiss |
//! | Bracket | Amplexus | *amplexus* = embrace, encompass |
//! | Use | Uti | *uti* = to use |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::async_core::res::Res;
//!
//! // Synchronous resource
//! let file = Res::make(
//!     || String::from("file handle"),
//!     |f| println!("closing {f}")
//! );
//!
//! let bytes_read = file.use_res(|f| {
//!     // Use the file
//!     f.len()
//! });
//! // File is automatically closed
//! assert_eq!(bytes_read, "file handle".len());
//! ```

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;

use super::fibra::{Fibra, FibraExitus};

// =============================================================================
// Res - Synchronous Resource
// =============================================================================

/// A managed resource with guaranteed cleanup.
///
/// `Res` encapsulates the lifecycle of a resource:
/// - **Acquire**: Obtain the resource
/// - **Use**: Work with the resource
/// - **Release**: Clean up, even on failure
///
/// # Latin Etymology
/// *Res* = thing, matter, substance, resource.
///
/// # Type Parameters
///
/// - `A`: The type of the resource
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::res::Res;
///
/// let counter = Res::make(
///     || {
///         println!("Acquiring counter");
///         0
///     },
///     |_| println!("Releasing counter")
/// );
///
/// let result = counter.use_res(|n| *n + 42);
/// assert_eq!(result, 42);
/// ```
pub struct Res<A> {
    acquire: Box<dyn FnOnce() -> A + Send>,
    release: Box<dyn FnOnce(A) + Send>,
}

/// Drop guard shared by every bracket entry point: releases the resource
/// even if the use-function unwinds.
///
/// # Latin Etymology
/// *Custos* = guardian.
struct Custos<A, Rel: FnOnce(A)> {
    resource: Option<A>,
    release: Option<Rel>,
}

impl<A, Rel: FnOnce(A)> Drop for Custos<A, Rel> {
    fn drop(&mut self) {
        if let (Some(resource), Some(release)) = (self.resource.take(), self.release.take()) {
            release(resource);
        }
    }
}

impl<A> Res<A> {
    /// Create a managed resource.
    ///
    /// # Arguments
    ///
    /// - `acquire`: Function to acquire the resource
    /// - `release`: Function to release/cleanup the resource
    ///
    /// # Latin Etymology
    /// *Fabricare* = to make, construct.
    #[inline]
    pub fn make<Acq, Rel>(acquire: Acq, release: Rel) -> Self
    where
        Acq: FnOnce() -> A + Send + 'static,
        Rel: FnOnce(A) + Send + 'static,
    {
        Res {
            acquire: Box::new(acquire),
            release: Box::new(release),
        }
    }

    /// Create a pure resource (no cleanup needed).
    ///
    /// # Latin Etymology
    /// *Purus* = pure.
    #[inline]
    pub fn purus(value: A) -> Self
    where
        A: Send + 'static,
    {
        Res {
            acquire: Box::new(move || value),
            release: Box::new(|_| {}),
        }
    }

    /// Use the resource with guaranteed cleanup.
    ///
    /// The resource is acquired, passed to `f`, and then released.
    /// Cleanup runs even if `f` panics.
    ///
    /// # Panics
    ///
    /// Panics only if the internal guard invariant (the resource slot is
    /// occupied until the guard drops) is violated, which would indicate a
    /// bug in this crate. A panic from `f` itself propagates after the
    /// release action has run.
    ///
    /// # Latin Etymology
    /// *Uti* = to use.
    #[inline]
    pub fn uti<B, F>(self, f: F) -> B
    where
        F: FnOnce(&A) -> B,
    {
        let custos = Custos {
            resource: Some((self.acquire)()),
            release: Some(self.release),
        };
        let result = f(custos
            .resource
            .as_ref()
            .expect("resource is held until the guard drops"));
        drop(custos);
        result
    }

    /// Alias for `uti`.
    #[inline]
    pub fn use_res<B, F>(self, f: F) -> B
    where
        F: FnOnce(&A) -> B,
    {
        self.uti(f)
    }

    /// Use the resource mutably.
    ///
    /// Like [`uti`](Self::uti) but hands `f` a `&mut A`; release still
    /// runs even if `f` panics.
    ///
    /// # Panics
    ///
    /// Panics only if the internal guard invariant (the resource slot is
    /// occupied until the guard drops) is violated, which would indicate a
    /// bug in this crate.
    #[inline]
    pub fn uti_mut<B, F>(self, f: F) -> B
    where
        F: FnOnce(&mut A) -> B,
    {
        let mut custos = Custos {
            resource: Some((self.acquire)()),
            release: Some(self.release),
        };
        let result = f(custos
            .resource
            .as_mut()
            .expect("resource is held until the guard drops"));
        drop(custos);
        result
    }

    /// Map over the resource type.
    ///
    /// The mapping function is applied during acquisition.
    ///
    /// # Latin Etymology
    /// *Mutare* = to change.
    #[inline]
    pub fn mutare<B, F, G>(self, f: F, g: G) -> Res<B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        G: FnOnce(B) -> A + Send + 'static,
        A: 'static,
        B: Send + 'static,
    {
        let Res { acquire, release } = self;
        Res {
            acquire: Box::new(move || f((acquire)())),
            release: Box::new(move |b| (release)(g(b))),
        }
    }
}

/// Combine two resources.
///
/// Both are acquired in order, and released in reverse order.
#[inline]
pub fn zip_res<A, B>(ra: Res<A>, rb: Res<B>) -> Res<(A, B)>
where
    A: Send + 'static,
    B: Send + 'static,
{
    let Res {
        acquire: acq_a,
        release: rel_a,
    } = ra;
    let Res {
        acquire: acq_b,
        release: rel_b,
    } = rb;

    Res {
        acquire: Box::new(move || {
            let a = (acq_a)();
            let b = (acq_b)();
            (a, b)
        }),
        release: Box::new(move |(a, b)| {
            // Release in reverse order
            (rel_b)(b);
            (rel_a)(a);
        }),
    }
}

/// Combine multiple resources.
#[inline]
pub fn zip_all_res<A>(resources: Vec<Res<A>>) -> Res<Vec<A>>
where
    A: Send + 'static,
{
    if resources.is_empty() {
        return Res::purus(Vec::new());
    }

    let n = resources.len();
    let mut acquirers: Vec<Box<dyn FnOnce() -> A + Send>> = Vec::with_capacity(n);
    let mut releasers: Vec<Box<dyn FnOnce(A) + Send>> = Vec::with_capacity(n);

    for res in resources {
        acquirers.push(res.acquire);
        releasers.push(res.release);
    }

    Res {
        acquire: Box::new(move || acquirers.into_iter().map(|acq| acq()).collect()),
        release: Box::new(move |items: Vec<A>| {
            // Release in reverse order
            for (item, rel) in items.into_iter().zip(releasers).rev() {
                rel(item);
            }
        }),
    }
}

// =============================================================================
// ResAsync - Asynchronous Resource
// =============================================================================

/// Type alias for async acquire function.
type AsyncAcquireFn<A> = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = A> + Send>> + Send>;

/// Type alias for async release function.
type AsyncReleaseFn<A> = Box<dyn FnOnce(A) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// An asynchronous managed resource.
///
/// Like `Res`, but for async acquisition and release. **Unlike `Res`, the
/// release action is currently never invoked** — see the `_release` field doc.
///
/// # Latin Etymology
/// *Res Asynchrona* = asynchronous resource.
///
/// # Example
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # use core::task::{Context, Poll, Waker};
/// #
/// # fn block_on<F: Future>(fut: F) -> F::Output {
/// #     let mut fut = Box::pin(fut);
/// #     let mut cx = Context::from_waker(Waker::noop());
/// #     loop {
/// #         if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
/// #             return out;
/// #         }
/// #     }
/// # }
/// #
/// use ordofp_core::async_core::res::ResAsync;
///
/// let connection = ResAsync::make(
///     || async { 42u32 },
///     |_conn| async move { /* close connection */ }
/// );
///
/// let result = block_on(connection.uti(|conn| async move {
///     conn + 1
/// }));
/// assert_eq!(result.unwrap(), 43);
/// ```
pub struct ResAsync<A> {
    acquire: AsyncAcquireFn<A>,
    /// Held but not yet invoked: `uti`'s `FnOnce(A)` consumes the resource,
    /// so cleanup needs an API redesign (e.g. `FnOnce(&mut A)`) before the
    /// release path can run. Underscore-named until that lands.
    _release: AsyncReleaseFn<A>,
}

impl<A> ResAsync<A> {
    /// Create an async managed resource.
    #[inline]
    pub fn make<Acq, AcqFut, Rel, RelFut>(acquire: Acq, release: Rel) -> Self
    where
        Acq: FnOnce() -> AcqFut + Send + 'static,
        AcqFut: Future<Output = A> + Send + 'static,
        Rel: FnOnce(A) -> RelFut + Send + 'static,
        RelFut: Future<Output = ()> + Send + 'static,
    {
        ResAsync {
            acquire: Box::new(move || Box::pin(acquire())),
            _release: Box::new(move |a| Box::pin(release(a))),
        }
    }

    /// Create a pure async resource (no cleanup needed).
    #[inline]
    pub fn purus(value: A) -> Self
    where
        A: Send + 'static,
    {
        ResAsync {
            acquire: Box::new(move || Box::pin(async move { value })),
            _release: Box::new(|_| Box::pin(async {})),
        }
    }

    /// Create from a synchronous resource.
    #[inline]
    pub fn from_sync(res: Res<A>) -> Self
    where
        A: Send + 'static,
    {
        let Res { acquire, release } = res;
        ResAsync {
            acquire: Box::new(move || Box::pin(async move { (acquire)() })),
            _release: Box::new(move |a| Box::pin(async move { (release)(a) })),
        }
    }

    /// Use the resource.
    ///
    /// Returns a fiber that acquires and uses the resource. The release
    /// action is **not** run: `f` consumes the resource, so cleanup awaits
    /// an API redesign (see the `_release` field doc).
    #[inline]
    pub fn uti<B, F, Fut>(self, f: F) -> Fibra<B>
    where
        F: FnOnce(A) -> Fut + Send + 'static,
        Fut: Future<Output = B> + Send + 'static,
        A: Send + 'static,
        B: Send + 'static,
    {
        Fibra::new(async move {
            let resource = (self.acquire)().await;
            // The acquired resource is not retained for cleanup here; use
            // `bracket` when release must be guaranteed.
            f(resource).await
        })
    }

    /// Use the resource, returning a `FibraExitus`.
    ///
    /// Like `uti`, the release action is **not** run (see the `_release`
    /// field doc).
    #[inline]
    pub fn uti_safe<B, F, Fut>(self, f: F) -> Fibra<B>
    where
        F: FnOnce(A) -> Fut + Send + 'static,
        Fut: Future<Output = FibraExitus<B>> + Send + 'static,
        A: Send + 'static,
        B: Send + 'static,
    {
        Fibra::new(async move {
            let resource = (self.acquire)().await;
            // Note: release should happen regardless of result
            f(resource).await
        })
        .flat_map(|res_b| match res_b {
            Ok(b) => Fibra::purus(b),
            Err(e) => Fibra::deficere(e),
        })
    }
}

// =============================================================================
// Bracket Pattern
// =============================================================================

/// The bracket pattern for resource management.
///
/// `bracket(acquire, use, release)` ensures `release` always runs,
/// even if `use` fails.
///
/// # Latin Etymology
/// *Amplexus* = embrace, encompass - like brackets embracing code.
///
/// # Panics
///
/// Panics only if the internal guard invariant (the resource slot is
/// occupied until the guard drops) is violated, which would indicate a
/// bug in this crate. A panic from `use_fn` propagates after `release`
/// has run.
#[inline]
pub fn amplexus<A, B, Acq, Use, Rel>(acquire: Acq, use_fn: Use, release: Rel) -> B
where
    Acq: FnOnce() -> A,
    Use: FnOnce(&A) -> B,
    Rel: FnOnce(A),
{
    let custos = Custos {
        resource: Some(acquire()),
        release: Some(release),
    };
    let result = use_fn(
        custos
            .resource
            .as_ref()
            .expect("resource is held until the guard drops"),
    );
    drop(custos);
    result
}

/// Alias for `amplexus`.
#[inline]
pub fn bracket<A, B, Acq, Use, Rel>(acquire: Acq, use_fn: Use, release: Rel) -> B
where
    Acq: FnOnce() -> A,
    Use: FnOnce(&A) -> B,
    Rel: FnOnce(A),
{
    amplexus(acquire, use_fn, release)
}

/// Async bracket pattern.
///
/// Returns a fiber that runs the bracketed computation.
///
/// # No `Clone` requirement, and a documented cancellation/panic gap
///
/// Every `UseFut`/`RelFut` here is bounded by `'static`, which forbids
/// `use_fn` from borrowing the resource (a `&mut A` couldn't outlive the
/// call, since it isn't `'static`). That means `use_fn` must own the
/// resource outright — so, to hand the *real* resource to `release` instead
/// of a duplicate, `use_fn` now returns it back alongside its result rather
/// than this function cloning it (the previous implementation released a
/// `.clone()`, which is wrong for any `A` whose clone doesn't denote "the
/// same underlying thing").
///
/// This still can't guarantee `release` runs if `use_fn`'s future panics
/// mid-poll or the `Fibra` is dropped/cancelled before `use_fn` completes:
/// the resource is owned inside that future's opaque state at that point,
/// and Rust has no "async `Drop`" to intercept it (a synchronous `Drop` impl
/// can construct a release future but cannot poll it to run its cleanup
/// logic). This is the same class of caveat already documented on
/// `ResAsync::_release`. Only the ordinary completion path is guaranteed to
/// invoke `release`, and when it does, it is guaranteed to receive the same
/// value `acquire` produced.
#[inline]
pub fn amplexus_async<A, B, Acq, AcqFut, Use, UseFut, Rel, RelFut>(
    acquire: Acq,
    use_fn: Use,
    release: Rel,
) -> Fibra<B>
where
    Acq: FnOnce() -> AcqFut + Send + 'static,
    AcqFut: Future<Output = A> + Send + 'static,
    Use: FnOnce(A) -> UseFut + Send + 'static,
    UseFut: Future<Output = (A, B)> + Send + 'static,
    Rel: FnOnce(A) -> RelFut + Send + 'static,
    RelFut: Future<Output = ()> + Send + 'static,
    A: Send + 'static,
    B: Send + 'static,
{
    Fibra::new(async move {
        let resource = acquire().await;
        let (resource, result) = use_fn(resource).await;
        release(resource).await;
        result
    })
}

/// Alias for `amplexus_async`.
#[inline]
pub fn bracket_async<A, B, Acq, AcqFut, Use, UseFut, Rel, RelFut>(
    acquire: Acq,
    use_fn: Use,
    release: Rel,
) -> Fibra<B>
where
    Acq: FnOnce() -> AcqFut + Send + 'static,
    AcqFut: Future<Output = A> + Send + 'static,
    Use: FnOnce(A) -> UseFut + Send + 'static,
    UseFut: Future<Output = (A, B)> + Send + 'static,
    Rel: FnOnce(A) -> RelFut + Send + 'static,
    RelFut: Future<Output = ()> + Send + 'static,
    A: Send + 'static,
    B: Send + 'static,
{
    amplexus_async(acquire, use_fn, release)
}

// =============================================================================
// Finalizer Pattern
// =============================================================================

/// Ensure a finalizer runs after a computation.
///
/// # Latin Etymology
/// *Finalis* = final.
#[inline]
pub fn finaliter<A, F, Fin>(computation: F, finalizer: Fin) -> A
where
    F: FnOnce() -> A,
    Fin: FnOnce(),
{
    // `Custos` needs a resource to guard; there isn't one here, so a unit
    // placeholder stands in and the finalizer is adapted to `FnOnce(())`.
    let custos = Custos {
        resource: Some(()),
        release: Some(move |()| finalizer()),
    };
    let result = computation();
    drop(custos);
    result
}

/// Async finalizer.
#[inline]
pub fn finaliter_async<A, F, FFut, Fin, FinFut>(computation: F, finalizer: Fin) -> Fibra<A>
where
    F: FnOnce() -> FFut + Send + 'static,
    FFut: Future<Output = A> + Send + 'static,
    Fin: FnOnce() -> FinFut + Send + 'static,
    FinFut: Future<Output = ()> + Send + 'static,
    A: Send + 'static,
{
    Fibra::new(async move {
        let result = computation().await;
        finalizer().await;
        result
    })
}

// =============================================================================
// Resource Pool
// =============================================================================

/// A pool of reusable resources.
///
/// # Latin Etymology
/// *Piscina* = pool, reservoir.
pub struct Piscina<A> {
    create: Box<dyn Fn() -> A + Send + Sync>,
    destroy: Box<dyn Fn(A) + Send + Sync>,
    max_size: usize,
    _marker: PhantomData<A>,
}

impl<A> Piscina<A> {
    /// Create a new resource pool.
    #[inline]
    pub fn new<C, D>(create: C, destroy: D, max_size: usize) -> Self
    where
        C: Fn() -> A + Send + Sync + 'static,
        D: Fn(A) + Send + Sync + 'static,
    {
        Piscina {
            create: Box::new(create),
            destroy: Box::new(destroy),
            max_size,
            _marker: PhantomData,
        }
    }

    /// Get the maximum pool size.
    #[inline]
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Create a new resource (for pool implementations).
    #[inline]
    pub fn create_resource(&self) -> A {
        (self.create)()
    }

    /// Destroy a resource (for pool implementations).
    #[inline]
    pub fn destroy_resource(&self, resource: A) {
        (self.destroy)(resource);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec;

    #[test]
    fn test_res_make_and_use() {
        let _acquired = core::cell::Cell::new(false);
        let _released = core::cell::Cell::new(false);

        // We can't capture non-Send types in the resource closures,
        // so we'll just test basic functionality
        let res = Res::make(|| 42, |_| {});

        let result = res.use_res(|n| *n * 2);
        assert_eq!(result, 84);
    }

    #[test]
    fn test_res_purus() {
        let res = Res::purus(42);
        let result = res.use_res(|n| *n + 1);
        assert_eq!(result, 43);
    }

    #[test]
    fn test_zip_res() {
        let ra = Res::purus(1);
        let rb = Res::purus(2);
        let combined = zip_res(ra, rb);
        let result = combined.use_res(|(a, b)| *a + *b);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_zip_all_res_empty() {
        let resources: Vec<Res<i32>> = vec![];
        let combined = zip_all_res(resources);
        let result = combined.use_res(std::vec::Vec::len);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_bracket() {
        let result = bracket(|| 42, |n| *n * 2, |_| {});
        assert_eq!(result, 84);
    }

    #[test]
    fn test_finaliter() {
        let mut finalized = false;
        let result = finaliter(
            || 42,
            || {
                finalized = true;
            },
        );
        assert_eq!(result, 42);
        assert!(finalized);
    }

    #[test]
    fn test_piscina_new() {
        let pool: Piscina<i32> = Piscina::new(|| 0, |_| {}, 10);
        assert_eq!(pool.max_size(), 10);
    }

    #[test]
    fn test_piscina_create_destroy() {
        let pool: Piscina<String> = Piscina::new(|| String::from("resource"), |_| {}, 5);
        let res = pool.create_resource();
        assert_eq!(res, "resource");
        pool.destroy_resource(res); // Should not panic
    }

    #[test]
    fn test_res_uti_releases_on_panic() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicBool, Ordering};

        let released = Arc::new(AtomicBool::new(false));
        let released_clone = Arc::clone(&released);
        let res = Res::make(|| 42, move |_| released_clone.store(true, Ordering::SeqCst));

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            res.uti(|_| -> i32 { panic!("boom") })
        }));

        assert!(outcome.is_err());
        assert!(
            released.load(Ordering::SeqCst),
            "release must run even when `f` panics"
        );
    }

    #[test]
    fn test_res_uti_mut_releases_on_panic() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicBool, Ordering};

        let released = Arc::new(AtomicBool::new(false));
        let released_clone = Arc::clone(&released);
        let res = Res::make(|| 42, move |_| released_clone.store(true, Ordering::SeqCst));

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            res.uti_mut(|_| -> i32 { panic!("boom") })
        }));

        assert!(outcome.is_err());
        assert!(
            released.load(Ordering::SeqCst),
            "release must run even when `f` panics"
        );
    }

    #[test]
    fn test_amplexus_releases_on_panic() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicBool, Ordering};

        let released = Arc::new(AtomicBool::new(false));
        let released_clone = Arc::clone(&released);

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            amplexus(
                || 42,
                |_: &i32| -> i32 { panic!("boom") },
                move |_| released_clone.store(true, Ordering::SeqCst),
            )
        }));

        assert!(outcome.is_err());
        assert!(
            released.load(Ordering::SeqCst),
            "release must run even when `use_fn` panics"
        );
    }

    #[test]
    fn test_finaliter_releases_on_panic() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicBool, Ordering};

        let finalized = Arc::new(AtomicBool::new(false));
        let finalized_clone = Arc::clone(&finalized);

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            finaliter(
                || -> i32 { panic!("boom") },
                move || finalized_clone.store(true, Ordering::SeqCst),
            )
        }));

        assert!(outcome.is_err());
        assert!(
            finalized.load(Ordering::SeqCst),
            "finalizer must run even when `computation` panics"
        );
    }

    #[test]
    fn test_amplexus_async_release_gets_same_resource_not_clone() {
        use alloc::sync::Arc;
        use core::sync::atomic::{AtomicUsize, Ordering};
        use core::task::{Context, Poll};

        // `Marker` deliberately does NOT derive `Clone`: `amplexus_async`'s
        // signature has no `A: Clone` bound, so this test only compiles if
        // `release` can be handed the *same* resource `acquire` produced
        // rather than a duplicate. Boxing it gives a heap address that's
        // stable across moves, so pointer equality proves it's literally
        // the same allocation, not merely an equal-looking copy (an `Arc`
        // wouldn't distinguish this, since `Arc::clone` shares its pointee's
        // address too).
        struct Marker(#[allow(dead_code)] u64); // payload keeps the Box non-ZST; only its address is read

        // Busy-poll a future to completion; nothing here ever returns
        // `Pending`, so this always terminates.
        fn drive<F: Future>(fut: F) -> F::Output {
            let mut fut = core::pin::pin!(fut);
            let mut cx = Context::from_waker(core::task::Waker::noop());
            loop {
                if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                    return out;
                }
            }
        }

        let resource = Box::new(Marker(0xC0_FFEE));
        let acquired_addr = core::ptr::from_ref::<Marker>(Box::as_ref(&resource)) as usize;

        let released_addr = Arc::new(AtomicUsize::new(0));
        let released_addr_clone = Arc::clone(&released_addr);

        let fiber = amplexus_async(
            move || async move { resource },
            |res: Box<Marker>| async move { (res, ()) },
            move |res: Box<Marker>| {
                released_addr_clone.store(
                    core::ptr::from_ref::<Marker>(Box::as_ref(&res)) as usize,
                    Ordering::SeqCst,
                );
                async move { drop(res) }
            },
        );

        let outcome = drive(fiber);
        assert!(outcome.is_ok());
        assert_eq!(
            released_addr.load(Ordering::SeqCst),
            acquired_addr,
            "release must receive the exact resource acquire produced, not a clone"
        );
    }
}
