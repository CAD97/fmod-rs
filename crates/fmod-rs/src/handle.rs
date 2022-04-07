use std::{mem::ManuallyDrop, ops::Deref, ptr, sync::atomic::AtomicUsize};

/// Only one system may be safely created, as system creation races with all
/// FMOD API use.
///
/// - `0`: No system has been created.
/// - `1`: A system is being created.
/// - `2`: A system has been created.
pub(crate) static GLOBAL_SYSTEM_STATE: AtomicUsize = AtomicUsize::new(0);

#[allow(clippy::missing_safety_doc)]
pub unsafe trait FmodResource: Sealed {
    type Raw;

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    fn as_raw(&self) -> *mut Self::Raw {
        self as *const Self as *mut Self as *mut Self::Raw
    }

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    unsafe fn from_raw<'a>(this: *mut Self::Raw) -> &'a Self;

    #[doc(hidden)]
    unsafe fn release(this: *mut Self::Raw) -> fmod::Result;
}

pub(crate) use sealed::Sealed;
mod sealed {
    pub trait Sealed {}
}

pub struct Handle<T: ?Sized + FmodResource> {
    raw: ptr::NonNull<T::Raw>,
}

impl<T: ?Sized + FmodResource> Drop for Handle<T> {
    fn drop(&mut self) {
        let raw = self.as_raw();

        match unsafe { T::release(raw) } {
            Ok(()) => {
                #[cfg(feature = "tracing")]
                tracing::trace!(
                    parent: crate::span(),
                    "Released {}({:p})",
                    std::any::type_name::<T>(),
                    raw,
                );
            },
            Err(error) => {
                let _ = error;
                #[cfg(feature = "tracing")]
                tracing::error!(
                    parent: crate::span(),
                    error = error.into_raw(),
                    "Error releasing {}({:p}): {error}",
                    std::any::type_name::<T>(),
                    raw,
                );
            },
        }
    }
}

impl<T: ?Sized + FmodResource> Handle<T> {
    raw! {
        pub fn into_raw(this: Self) -> *mut T::Raw {
            let this = ManuallyDrop::new(this);
            this.as_raw()
        }

        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut T::Raw) -> Self {
            Self { raw: ptr::NonNull::new_unchecked(raw) }
        }
    }

    pub(crate) unsafe fn new(raw: *mut T::Raw) -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            parent: crate::span(),
            "Created {}({:p})",
            std::any::type_name::<T>(),
            raw,
        );

        Self::from_raw(raw)
    }

    /// Forget to release this FMOD resource.
    pub fn leak(this: Self) -> &'static T {
        let this = ManuallyDrop::new(this);
        unsafe { T::from_raw(this.as_raw()) }
    }

    /// Claim responsibility to release this FMOD resource.
    ///
    /// # Safety
    ///
    /// No references to the resource may outlive the owning handle.
    ///
    /// # Special note for `System` and `studio::System`
    ///
    /// Releasing an FMOD system is *thread unsafe*. If a system's release can
    /// race with *any* FMOD API call, then that is a data race and potential
    /// UB. If you want to release a system, it is *on you* to ensure that *all*
    /// references to FMOD resources are gone, such that no API calls can be
    /// made during or after releasing the system.
    pub unsafe fn unleak(this: &'static T) -> Self {
        Self::from_raw(this.as_raw())
    }
}

// Deref impls are scary to me, but required for ergonomics, and every other
// FFI binding does it with opaque types, so it's necessarily okay in practice.
// The theoretical problem is twofold:
//  1. Rust references under Stacked Borrows 2021 only hold pointer provenance
//     for exactly as many bytes as mem::size_of_val. These are 1-ZSTs for the
//     type system, so in theory a reference has provenance over no memory, and
//     later trying to use that memory (say, by telling FMOD it's a valid ptr)
//     is thus UB. Luckily, this might be relaxed to allow using unsized tails.
//  2. Rust references have _strong_ guarantees about uniqueness and mutabiliy.
//     Manifesting &T claims that nobody else will change the value, and &mut T
//     claims that nobody else even has a pointer to the value (that is used).
//     C++ has no such rules, and FMOD doesn't even provide `const` annotation
//     to tell us what can change things; these are truly fully opaque handles.
// Both of these issues are dismissable for the same core reason:
//     FFI is hard, especially between languages with different memory models.
// In this case, we never actually alias the memory managed on the C++ side
// (our pointee types are ZSTs, as previously mentioned), so Rust doesn't claim
// provenance over any actual memory that the C++ side thinks it has access to.
// We take the ~~coward's~~ simple way out: we deal almost exclusively in &T,
// and rely on the FFI barrier to keep us safe.

impl<T: ?Sized + FmodResource> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { T::from_raw(self.raw.as_ptr()) }
    }
}
