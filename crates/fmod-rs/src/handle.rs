#![allow(private_bounds)]

use {
    parking_lot::RwLock,
    std::{
        fmt,
        marker::PhantomData,
        mem::ManuallyDrop,
        ops::{Deref, DerefMut},
        panic::{RefUnwindSafe, UnwindSafe},
        ptr,
    },
};

/// Only one system may be safely created at a time, as system create and
/// release races with all of the FMOD API. Additionally, this must be an
/// actual lock, so that it can synchronize with free functions as well.
///
/// - A write lock is acquired to perform system create/release.
/// - A read lock is acquired to perform free functions.
/// - `0` indicates that no system exists, and one may be created.
/// - `>= 1` indicates that systems exist, and creating another is unsafe.
pub(crate) static GLOBAL_SYSTEM_STATE: RwLock<usize> = RwLock::new(0);

#[cfg(feature = "unstable")]
pub(crate) use std::marker::{MetaSized, PointeeSized};

#[cfg(all(doc, not(feature = "unstable")))]
/// Shim for [`std::marker::PointeeSized`] bounds on stable.
pub trait PointeeSized {}

#[cfg(all(not(doc), not(feature = "unstable")))]
trait PointeeSized {}

#[cfg(not(feature = "unstable"))]
impl<T: ?Sized> PointeeSized for T {}

#[cfg(all(doc, not(feature = "unstable")))]
/// Shim for [`std::marker::MetaSized`] bounds on stable.
pub trait MetaSized {}

#[cfg(all(not(doc), not(feature = "unstable")))]
trait MetaSized {}

#[cfg(not(feature = "unstable"))]
impl<T: ?Sized> MetaSized for T {}

#[allow(clippy::missing_safety_doc)]
/// FMOD resources managed by a [Handle].
pub unsafe trait Resource: PointeeSized + fmt::Debug + Sealed {
    #[allow(missing_docs)]
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    type Raw;

    #[inline(always)]
    #[allow(missing_docs)]
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    fn as_raw(&self) -> *mut Self::Raw {
        self as *const Self as *mut Self::Raw
    }

    #[doc(hidden)]
    fn cast_from_raw(this: *mut Self::Raw) -> *mut Self;

    #[track_caller]
    #[inline(always)]
    #[allow(missing_docs)]
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    unsafe fn from_raw<'a>(this: *mut Self::Raw) -> &'a Self {
        debug_assert!(!this.is_null());
        &*Self::cast_from_raw(this)
    }

    #[track_caller]
    #[inline(always)]
    #[allow(missing_docs)]
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    unsafe fn from_raw_mut<'a>(this: *mut Self::Raw) -> &'a mut Self {
        debug_assert!(!this.is_null());
        &mut *Self::cast_from_raw(this)
    }

    #[track_caller]
    #[inline(always)]
    #[allow(missing_docs)]
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    unsafe fn try_from_raw<'a>(this: *mut Self::Raw) -> Option<&'a Self> {
        if this.is_null() {
            None
        } else {
            Some(Self::from_raw(this))
        }
    }

    #[track_caller]
    #[inline(always)]
    #[allow(missing_docs)]
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    unsafe fn try_from_raw_mut<'a>(this: *mut Self::Raw) -> Option<&'a mut Self> {
        if this.is_null() {
            None
        } else {
            Some(Self::from_raw_mut(this))
        }
    }

    #[allow(missing_docs)]
    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    unsafe fn release(this: *mut Self::Raw) -> fmod::Result;
}

pub(crate) trait Sealed: PointeeSized {}

/// An owning handle to an FMOD resource.
///
/// When this handle is dropped, the underlying FMOD resource is released.
pub struct Handle<'a, T: ?Sized + PointeeSized + Resource> {
    raw: ptr::NonNull<T::Raw>,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<T: ?Sized + PointeeSized + Resource> Send for Handle<'_, T> where T: Send {}
unsafe impl<T: ?Sized + PointeeSized + Resource> Sync for Handle<'_, T> where T: Sync {}
impl<T: ?Sized + PointeeSized + Resource> UnwindSafe for Handle<'_, T> where T: UnwindSafe {}
impl<T: ?Sized + PointeeSized + Resource> RefUnwindSafe for Handle<'_, T> where T: RefUnwindSafe {}

impl<T: ?Sized + PointeeSized + Resource> fmt::Debug for Handle<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self).as_ref().fmt(f)
    }
}

impl<T: ?Sized + PointeeSized + Resource> Drop for Handle<'_, T> {
    fn drop(&mut self) {
        let this = unsafe { Self::from_raw(self.as_raw()) };
        match this.release() {
            Ok(()) => {}, // all good
            Err(error) => whoops!("Error releasing {self:?}: {error}"),
        }
    }
}

impl<'a, T: ?Sized + PointeeSized + Resource> Handle<'a, T> {
    raw! {
        pub fn as_raw(&self) -> *mut T::Raw {
            self.as_ref().as_raw()
        }
    }

    raw! {
        pub fn into_raw(self) -> *mut T::Raw {
            let this = ManuallyDrop::new(self);
            this.as_raw()
        }
    }

    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut T::Raw) -> Self {
            Self { raw: ptr::NonNull::new_unchecked(raw), _phantom: PhantomData }
        }
    }

    pub(crate) unsafe fn new(raw: *mut T::Raw) -> Self {
        let this = Self::from_raw(raw);

        #[cfg(feature = "log")]
        log::trace!("Created {this:?}");

        this
    }

    /// Manually release this FMOD resource.
    ///
    /// If an error occurs, the resource is leaked for convenience with `?`. If
    /// you want to avoid this, use [HandleExt::release] with `Option<Handle>`
    /// instead.
    pub fn release(self) -> fmod::Result {
        ManuallyDrop::new(Some(self)).release()
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

impl<T: ?Sized + PointeeSized + Resource> AsRef<T> for Handle<'_, T> {
    fn as_ref(&self) -> &T {
        unsafe { T::from_raw(self.raw.as_ptr()) }
    }
}

impl<T: ?Sized + PointeeSized + Resource> AsMut<T> for Handle<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { T::from_raw_mut(self.raw.as_ptr()) }
    }
}

impl<T: ?Sized + MetaSized + Resource> Deref for Handle<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized + MetaSized + Resource> DerefMut for Handle<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

/// Extension trait for <code>Option&lt;[Handle]&gt;</code>.
#[allow(private_bounds)]
pub trait HandleExt<T: ?Sized + PointeeSized + Resource>: Sealed {
    /// Manually release this FMOD resource.
    ///
    /// This is only necessary if you want to handle potential errors yourself;
    /// the resource handle is automatically released when dropped.
    fn release(&mut self) -> fmod::Result;

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    #[allow(missing_docs)]
    fn into_raw(self) -> *mut T::Raw;

    #[cfg_attr(not(feature = "raw"), doc(hidden))]
    #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
    #[allow(missing_docs, clippy::missing_safety_doc)]
    unsafe fn from_raw(raw: *mut T::Raw) -> Self;
}

impl<T: ?Sized + PointeeSized + Resource> Sealed for Option<Handle<'_, T>> {}
impl<T: ?Sized + PointeeSized + Resource> HandleExt<T> for Option<Handle<'_, T>> {
    fn release(&mut self) -> fmod::Result {
        let this = match self {
            Some(this) => this.as_raw(),
            None => yeet!(fmod::Error::InvalidHandle),
        };
        unsafe { T::release(this) }?;
        #[cfg(feature = "log")]
        log::trace!("Released {this:?}");
        Ok(())
    }

    fn into_raw(self) -> *mut T::Raw {
        self.map(Handle::into_raw).unwrap_or_else(ptr::null_mut)
    }

    unsafe fn from_raw(raw: *mut T::Raw) -> Self {
        if raw.is_null() {
            None
        } else {
            Some(Handle::from_raw(raw))
        }
    }
}
