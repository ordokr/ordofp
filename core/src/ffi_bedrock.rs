//! # FFI Bedrock
//!
//! Foundational patterns for ergonomic and safe C library wrapping.
//! Adapted third-party patterns are inventoried in the repo-root
//! `THIRD_PARTY_NOTICES.md`.

use core::ptr::NonNull;

/// Triage a pointer to `Some(NonNull<T>)` or `None`.
pub trait PointerUpgrade<T>: Sized {
    /// Convert this raw pointer into `Some(NonNull<T>)`, or `None` when it
    /// is null. No validity beyond non-nullness is checked or implied.
    fn upgrade(self) -> Option<NonNull<T>>;
}

impl<T> PointerUpgrade<T> for *const T {
    #[inline]
    fn upgrade(self) -> Option<NonNull<T>> {
        NonNull::new(self.cast_mut())
    }
}

impl<T> PointerUpgrade<T> for *mut T {
    #[inline]
    fn upgrade(self) -> Option<NonNull<T>> {
        NonNull::new(self)
    }
}

/// Triage a C return code (negative for error, zero/positive for success)
/// into a `Result`.
pub trait RetUpgrade {
    /// Split a C return code into `Ok`/`Err` by sign.
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the original code when it is negative (the C
    /// error convention); zero and positive codes are returned in `Ok`.
    fn upgrade(self) -> Result<core::ffi::c_int, core::ffi::c_int>;
}

impl RetUpgrade for core::ffi::c_int {
    #[inline]
    fn upgrade(self) -> Result<core::ffi::c_int, core::ffi::c_int> {
        if self < 0 { Err(self) } else { Ok(self) }
    }
}

/// Windows-specific result triage for APIs that return 0 on failure.
#[cfg(target_os = "windows")]
pub enum WindowsResult<T, E> {
    /// The API call succeeded; carries the non-zero return value.
    Ok(T),
    /// The API call failed (returned 0); carries the error description.
    Err(E),
}

/// Macro to implement conversion from integer types to `WindowsResult`.
///
/// Adapted to use &'static str as a Universalis error type.
#[cfg(target_os = "windows")]
#[macro_export]
macro_rules! impl_from_integer_for_windows_result {
    ( $( $integer_type:ty ),+ ) => {
        $(
            impl From<$integer_type> for $crate::ffi_bedrock::WindowsResult<$integer_type, &'static str> {
                fn from(return_value: $integer_type) -> Self {
                    match return_value {
                        0 => Self::Err("Windows API error: 0"),
                        _ => Self::Ok(return_value),
                    }
                }
            }
        )+
    };
}

/// Trait for processing Windows-specific results into standard Results.
#[cfg(target_os = "windows")]
pub trait ProcessWindowsResult<T, E> {
    /// Convert a [`WindowsResult`] into a standard [`Result`], preserving
    /// the payload of whichever variant it holds.
    ///
    /// # Errors
    ///
    /// Returns `Err` exactly when `self` is [`WindowsResult::Err`], i.e.
    /// when the originating Windows API call returned 0.
    fn process(self) -> Result<T, E>;
}

#[cfg(target_os = "windows")]
impl<T, E> ProcessWindowsResult<T, E> for WindowsResult<T, E> {
    #[inline]
    fn process(self) -> Result<T, E> {
        match self {
            Self::Ok(v) => Ok(v),
            Self::Err(e) => Err(e),
        }
    }
}

/// Foundation for all FFI wrappers.
/// Defines the basic structure and pointer management.
#[macro_export]
macro_rules! wrap_pure {
    (
        $(#[$meta:meta])*
        ($wrapped_type: ident): $ffi_type: ty
    ) => {
        $(#[$meta])*
        pub struct $wrapped_type {
            inner: core::ptr::NonNull<$ffi_type>,
        }

        impl $wrapped_type {
            /// Return a read-only raw pointer to the underlying FFI value.
            ///
            /// The pointer is valid for as long as `self` is alive and no
            /// exclusive reference to `self` exists.
            #[inline]
            pub fn as_ptr(&self) -> *const $ffi_type {
                self.inner.as_ptr().cast_const()
            }

            /// Return a mutable raw pointer to the underlying FFI value.
            ///
            /// The caller must ensure that no other references to the same
            /// value exist for the duration of any write through this pointer.
            #[inline]
            pub fn as_mut_ptr(&mut self) -> *mut $ffi_type {
                self.inner.as_ptr()
            }

            /// # Safety
            /// The pointer must satisfy all of the following:
            ///
            /// - **Valid for the wrapper's entire lifetime:** The pointed-to data must remain
            ///   validly initialized and properly aligned for all reads and writes through
            ///   this wrapper, for the entire duration from `from_raw` until `drop` or
            ///   `into_raw`. Dereferencing the wrapper (see [`Deref` impl](#deref)) requires
            ///   this validity.
            ///
            /// - **Exclusively owned:** The caller guarantees this is the sole wrapper
            ///   over this pointer. Constructing multiple wrappers over the same pointer
            ///   breaks the safety justifications for the `Send`/`Sync` impls below:
            ///   two wrappers allow a data race through concurrent `&`-access to shared
            ///   mutable state, and both will call `Drop::drop` on the same data if either
            ///   outlives the raw pointer's allocation.
            ///
            /// - **Cross-thread validity:** If the wrapper is moved to another thread
            ///   (`Send` impl), the pointed-to data must remain valid and have `Send`
            ///   semantics. Dropping the wrapper on a different thread than it was created
            ///   is valid only if the data can be safely dropped from that thread.
            ///
            /// The data must be able to be dropped (i.e., dropping is safe and well-defined
            /// for the FFI type).
            #[inline]
            pub unsafe fn from_raw(raw: core::ptr::NonNull<$ffi_type>) -> Self {
                Self { inner: raw }
            }

            /// Consume the wrapper and return the underlying `NonNull` pointer.
            ///
            /// The destructor is **not** run.  The caller takes ownership of the
            /// allocation and is responsible for eventually freeing or
            /// reconstructing it (e.g. via [`Self::from_raw`]).
            #[inline]
            pub fn into_raw(self) -> core::ptr::NonNull<$ffi_type> {
                let raw = self.inner;
                core::mem::forget(self);
                raw
            }
        }

        impl core::ops::Deref for $wrapped_type {
            type Target = $ffi_type;
            #[inline]
            fn deref(&self) -> &Self::Target {
                // SAFETY: `self.inner` is a `NonNull` pointer established at construction by
                // `from_raw`, whose safety contract requires the pointer to be valid for the
                // lifetime of the wrapper. The returned reference borrows `self`, so it cannot
                // outlive the owning wrapper, maintaining soundness.
                unsafe { self.inner.as_ref() }
            }
        }

        // SAFETY: `$wrapped_type` is a newtype wrapper around `NonNull<$ffi_type>`.
        // Raw pointers opt out of `Send`/`Sync` by default, but this wrapper
        // models *exclusive ownership* of the pointed-to value (the `from_raw`
        // safety contract forbids sharing the raw pointer). Transferring that
        // ownership to another thread is therefore safe whenever `$ffi_type: Send`.
        unsafe impl Send for $wrapped_type where $ffi_type: Send {}
        // SAFETY: Sharing `&$wrapped_type` across threads exposes a shared
        // reference to the inner `$ffi_type`. This is sound whenever `$ffi_type:
        // Sync`, which the bound enforces. The `Deref` impl yields only `&$ffi_type`
        // (never `&mut`), so concurrent shared access causes no data races.
        unsafe impl Sync for $wrapped_type where $ffi_type: Sync {}
    };
}

/// Defines a non-owning reference wrapper for a FFI type.
#[macro_export]
macro_rules! wrap_ref {
    ($wrapped_type: ident, $wrapped_ref: ident, $ffi_type: ty) => {
        #[repr(transparent)]
        pub struct $wrapped_ref<'a> {
            inner: core::mem::ManuallyDrop<$wrapped_type>,
            _marker: core::marker::PhantomData<&'a $ffi_type>,
        }

        impl<'a> core::ops::Deref for $wrapped_ref<'a> {
            type Target = $wrapped_type;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<'a> $wrapped_ref<'a> {
            /// # Safety
            /// The pointer must be valid for the lifetime 'a.
            pub unsafe fn from_raw(raw: core::ptr::NonNull<$ffi_type>) -> Self {
                Self {
                    // SAFETY: The caller upholds the same contract as `$wrapped_type::from_raw`:
                    // `raw` is a valid, non-null, properly-aligned pointer that remains live for
                    // at least lifetime `'a`. ManuallyDrop prevents double-free since the
                    // `$wrapped_ref` borrows rather than owns the pointee.
                    inner: core::mem::ManuallyDrop::new(unsafe { $wrapped_type::from_raw(raw) }),
                    _marker: core::marker::PhantomData,
                }
            }
        }
    };
}

/// Probing specific memory pattern and return the offset.
///
/// # Safety
/// The caller must ensure:
///
/// - **Terminated sequence:** `ptr` must point to a sequence of `T` terminated by a
///   sentinel value equal to `tail`. If the buffer is not properly terminated, the
///   scanning loop will read out of bounds indefinitely until it encounters data
///   equal to `tail` or the program crashes.
///
/// - **Non-zero-sized elements:** `T` must not be a zero-sized type (ZST). For ZSTs,
///   `ptr.add(1)` does not advance the pointer, causing an infinite loop that never
///   reaches `tail`.
// `tail` is a sentinel, typically a Copy scalar (`0u8`, NUL); by-value keeps
// call sites free of `&0`-style noise.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub unsafe fn probe_len<T: PartialEq>(mut ptr: *const T, tail: T) -> usize {
    for len in 0.. {
        // SAFETY: The caller guarantees `ptr` points to a sequence terminated by
        // `tail`, so this dereference is valid for at least `len` more elements.
        if unsafe { *ptr == tail } {
            return len;
        }
        // SAFETY: We have not yet reached the `tail` sentinel, so the pointer
        // has not gone past the end of the valid sequence; advancing by one
        // element stays within bounds.
        unsafe {
            ptr = ptr.add(1);
        }
    }
    usize::MAX
}

/// Building a memory slice starting with `ptr` and ending with given `tail`.
///
/// # Safety
/// `ptr` must be terminated by `tail`.
#[inline]
pub unsafe fn build_array<'a, T: PartialEq>(ptr: *const T, tail: T) -> Option<&'a [T]> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: The caller guarantees `ptr` is non-null and terminated by
        // `tail`, satisfying `probe_len`'s safety contract.
        let len = unsafe { probe_len(ptr, tail) };
        // SAFETY: `ptr` is non-null (checked above), valid for `len` reads of
        // `T` (guaranteed by the `tail`-termination contract), and the returned
        // lifetime `'a` is bounded by the caller's guarantee that the memory
        // lives at least as long as `'a`.
        Some(unsafe { core::slice::from_raw_parts(ptr, len) })
    }
}

#[cfg(test)]
// The wrap_pure!/wrap_ref! expansions below exist to typecheck the full
// generated API; not every generated item is called by the tests.
#[allow(dead_code)]
mod tests {
    use super::*;

    #[repr(C)]
    pub struct MockCStruct {
        pub value: i32,
    }

    wrap_pure!((SafeHandle): MockCStruct);
    wrap_ref!(SafeHandle, SafeHandleRef, MockCStruct);

    impl Drop for SafeHandle {
        fn drop(&mut self) {
            // In a real handle, we'd call a C free function here.
        }
    }

    #[test]
    fn test_pointer_upgrade() {
        let x = 42i32;
        let ptr: *const i32 = &raw const x;
        assert!(ptr.upgrade().is_some());

        let null_ptr: *const i32 = core::ptr::null();
        assert!(null_ptr.upgrade().is_none());
    }

    #[test]
    fn test_ret_upgrade() {
        let success: core::ffi::c_int = 0;
        let failure: core::ffi::c_int = -1;

        assert!(success.upgrade().is_ok());
        assert!(failure.upgrade().is_err());
    }

    #[test]
    fn test_wrapper_deref() {
        let mut x = MockCStruct { value: 100 };
        let ptr = NonNull::new(&raw mut x).expect("mutable reference is always non-null");
        // SAFETY: `ptr` is derived from a live, exclusively-borrowed local
        // variable, so it is non-null, properly aligned, and valid for the
        // duration of this test.  The `into_raw` call at the end prevents
        // `SafeHandle::drop` from running on a stack-allocated value.
        let handle = unsafe { SafeHandle::from_raw(ptr) };

        assert_eq!(handle.value, 100);
        // We into_raw to avoid calling drop on our local stack variable
        let _ = handle.into_raw();
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_result_triage() {
        let res: WindowsResult<i32, &'static str> = WindowsResult::Ok(42);
        assert_eq!(res.process(), Ok(42));

        let err: WindowsResult<i32, &'static str> = WindowsResult::Err("Windows API error: 0");
        assert_eq!(err.process(), Err("Windows API error: 0"));
    }

    #[cfg(target_os = "windows")]
    impl_from_integer_for_windows_result!(i32);

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_result_macro() {
        let success: WindowsResult<i32, &'static str> = 42.into();
        assert!(matches!(success, WindowsResult::Ok(42)));

        let failure: WindowsResult<i32, &'static str> = 0.into();
        assert!(matches!(failure, WindowsResult::Err(_)));
    }
}
