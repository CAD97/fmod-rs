use {
    parking_lot::{const_rwlock, RwLock},
    std::{
        fmt,
        mem::ManuallyDrop,
        ops::{Deref, DerefMut},
    },
};

/// Only one system may be safely created, at a time, as system create and
/// release races with all FMOD API use. Additionally, this must be an actual
/// lock, so that it can synchronize with free functions as well.
///
/// - A write lock is acquired to perform system create/release.
/// - A read lock is acquired to perform free functions.
/// - `false` indicates that no system exists, and one may be created.
/// - `true` indicates that a system exists, and creating another is unsafe.
pub(crate) static GLOBAL_SYSTEM_STATE: RwLock<usize> = const_rwlock(0);

#[allow(clippy::missing_safety_doc)]
/// FMOD resources managed by a [Handle].
pub unsafe trait FmodResource: fmt::Debug + Sealed {
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(all(feature = "raw", feature = "unstable"), doc(cfg(raw)))]
    type Raw;

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(all(feature = "raw", feature = "unstable"), doc(cfg(raw)))]
    fn as_raw(&self) -> *mut Self::Raw {
        self as *const Self as *const Self::Raw as *mut Self::Raw
    }

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(all(feature = "raw", feature = "unstable"), doc(cfg(raw)))]
    unsafe fn from_raw<'a>(this: *mut Self::Raw) -> &'a Self;

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(all(feature = "raw", feature = "unstable"), doc(cfg(raw)))]
    unsafe fn from_raw_mut<'a>(this: *mut Self::Raw) -> &'a mut Self;

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(all(feature = "raw", feature = "unstable"), doc(cfg(raw)))]
    unsafe fn from_raw_opt<'a>(this: *mut Self::Raw) -> Option<&'a Self> {
        if this.is_null() {
            None
        } else {
            Some(Self::from_raw(this))
        }
    }

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(all(feature = "raw", feature = "unstable"), doc(cfg(raw)))]
    unsafe fn release(this: *mut Self::Raw) -> fmod::Result;
}

pub(crate) use sealed::Sealed;
mod sealed {
    pub trait Sealed {}
}

/// An owning handle to an FMOD resource. When this handle is dropped, the
/// underlying FMOD resource is released.
pub struct Handle<'a, T: ?Sized + FmodResource> {
    raw: &'a T::Raw,
}

impl<T: ?Sized + FmodResource> fmt::Debug for Handle<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + FmodResource> Drop for Handle<'_, T> {
    fn drop(&mut self) {
        let raw = self.as_raw();

        match unsafe { T::release(raw) } {
            Ok(()) => {
                #[cfg(feature = "log")]
                log::trace!("Released fmod::{self:?}");
            },
            Err(error) => {
                whoops!("Error releasing fmod::{self:?}: {error}");
            },
        }
    }
}

impl<'a, T: ?Sized + FmodResource> Handle<'a, T> {
    raw! {
        pub fn into_raw(this: Self) -> *mut T::Raw {
            let this = ManuallyDrop::new(this);
            this.as_raw()
        }
    }
    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut T::Raw) -> Self {
            Self { raw: &mut *raw }
        }
    }

    pub(crate) unsafe fn new(raw: *mut T::Raw) -> Self {
        let this = Self::from_raw(raw);

        #[cfg(feature = "log")]
        log::trace!("Created fmod::{this:?}");

        this
    }

    /// Forget to release this FMOD resource.
    pub fn leak(this: Self) -> &'a T {
        let this = ManuallyDrop::new(this);
        unsafe { T::from_raw(this.as_raw()) }
    }

    /// Claim responsibility to release this FMOD resource.
    ///
    /// # Safety
    ///
    /// No references to the resource may outlive the owning handle.
    pub unsafe fn unleak(this: &'a T) -> Self {
        Self::from_raw(this.as_raw())
    }
}

// Using references is scary to me, but required for ergonomics, and almost
// every other FFI binding does it with opaque types, so it's necessarily okay
// in practice. The theoretical problem is twofold:
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

impl<T: ?Sized + FmodResource> Deref for Handle<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { T::from_raw(self.raw as *const _ as *mut _) }
    }
}

impl<T: ?Sized + FmodResource> DerefMut for Handle<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { T::from_raw_mut(self.raw as *const _ as *mut _) }
    }
}
