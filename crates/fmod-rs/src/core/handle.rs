//! Implementation of owning handles to FMOD resources which handles calling
//! FMOD_*_Release. All handles own a strong reference to the gloabl System in
//! static HEART: Weak<BeatingHeart>.

use {
    cfg_if::cfg_if,
    std::{
        ops::{Deref, DerefMut},
        ptr,
        sync::{Arc, Weak},
    },
};

struct TrustMe<T>(T);
unsafe impl<T> Send for TrustMe<T> {}
unsafe impl<T> Sync for TrustMe<T> {}

enum BeatingHeart {
    Core(ptr::NonNull<fmod::System>),
    // #[cfg(feature = "studio")]
    // Studio(ptr::NonNull<fmod::studio::System>),
}

cfg_if! {
    if #[cfg(feature = "unstable")] {
        #[cfg(not(feature = "parking_lot"))]
        use std::lazy::SyncLazy as Lazy;
    } else if #[cfg(feature = "once_cell")] {
        use once_cell::sync::Lazy;
    } else {
        compile_error!("FMOD.rs requires either the `unstable` or `once_cell` feature to be enabled");
    }
}

cfg_if! {
    if #[cfg(feature = "parking_lot")] {
        use parking_lot::{RwLock, RwLockUpgradableReadGuard};
        cfg_if! {
            if #[cfg(feature = "unstable")] {
                static HEART: RwLock<Weak<TrustMe<BeatingHeart>>> =
                    parking_lot::const_rwlock(Weak::new());
            } else {
                static HEART: Lazy<RwLock<Weak<TrustMe<BeatingHeart>>>> =
                    Lazy::new(|| RwLock::new(Weak::new()));
            }
        }
    } else {
        use std::sync::RwLock;
        static HEART: Lazy<RwLock<Weak<TrustMe<BeatingHeart>>>> =
            Lazy::new(|| RwLock::new(Weak::new()));
    }
}

impl Drop for BeatingHeart {
    fn drop(&mut self) {
        debug_assert!(heart().is_none());
        // SAFETY: once BeatingHeart lies still, all FMOD resources have gone.
        //         we can now safely release the System without racing anything.
        match self {
            BeatingHeart::Core(system) => unsafe { fmod::System::raw_release(system.as_ptr()) }
                .unwrap_or_else(|error| {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        parent: crate::span(),
                        error = error.into_raw(),
                        "Error releasing System({:p}): {error}",
                        system,
                    );
                }),
        }
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait FmodResource {
    type Raw;
    #[doc(hidden)]
    unsafe fn release(this: *mut Self::Raw) -> fmod::Result<()>;
    #[doc(hidden)]
    unsafe fn cast_raw(this: *mut Self::Raw) -> *mut Self;
}

/// An owning handle to an FMOD managed resource.
pub struct Handle<T: ?Sized + FmodResource> {
    raw: ptr::NonNull<T>,
    _own: Arc<TrustMe<BeatingHeart>>,
}

impl<T: ?Sized + FmodResource> Handle<T> {
    raw! {
        pub fn as_raw(&self) -> *mut T::Raw {
            self.raw.as_ptr() as *mut T::Raw
        }
    }
}

impl<T: ?Sized + FmodResource> Drop for Handle<T> {
    fn drop(&mut self) {
        unsafe { T::release(self.as_raw()) }.unwrap_or_else(|error| {
            let _ = error;
            #[cfg(feature = "tracing")]
            tracing::error!(
                parent: crate::span(),
                error = error.into_raw(),
                "Error releasing {}({:p}): {error}",
                std::any::type_name::<T>(),
                self.as_raw(),
            );
        });
    }
}

fn heart() -> Option<Arc<TrustMe<BeatingHeart>>> {
    cfg_if! {
        if #[cfg(feature = "parking_lot")] {
            HEART.read().upgrade()
        } else {
            HEART.read().unwrap().upgrade()
        }
    }
}

impl<T: ?Sized + FmodResource> Handle<T> {
    pub(super) unsafe fn new_raw(raw: *mut T::Raw) -> fmod::Result<Handle<T>> {
        match heart() {
            Some(heart) => Ok(Handle {
                raw: ptr::NonNull::new_unchecked(T::cast_raw(raw)),
                _own: heart,
            }),
            None => {
                cfg_if! {
                    if #[cfg(debug_assertions)] {
                        unreachable!(
                            "`Handle::new_raw` should only be called when the FMOD System is alive",
                        );
                    } else {
                        #[cfg(feature = "tracing")]
                        tracing::error!(
                            parent: &crate::span(),
                            "`Handle::new_raw` called improperly somehow",
                        );
                        Err(fmod::Error::InternalRs)
                    }
                }
            },
        }
    }
}

impl Handle<fmod::System> {
    pub(super) fn new_system() -> fmod::Result<Handle<fmod::System>> {
        // aquire a read lock on the heart
        cfg_if! {
            if #[cfg(feature = "parking_lot")] {
                let heart = HEART.read();
            } else {
                let heart = HEART.read().unwrap();
            }
        }

        // if we already have an initialized system, clone it
        if let Some(heart) = heart.upgrade() {
            return Ok(match heart.0 {
                BeatingHeart::Core(system) => Handle {
                    raw: system,
                    _own: heart.clone(),
                },
            });
        }

        // it looks like we're first to the system, so get a write lock
        drop(heart);
        cfg_if! {
            if #[cfg(feature = "parking_lot")] {
                let heart = HEART.upgradable_read();
            } else {
                let mut heart = HEART.write().unwrap();
            }
        }

        // but if we were beat to initialization, don't make a new system
        if let Some(heart) = heart.upgrade() {
            return Ok(match heart.0 {
                BeatingHeart::Core(system) => Handle {
                    raw: system,
                    _own: heart.clone(),
                },
            });
        }

        // otherwise, we're the first, so create the actual system
        #[cfg(feature = "parking_lot")]
        let mut heart = RwLockUpgradableReadGuard::upgrade(heart);

        // SAFETY: all of the above is to guarantee that this cannot possibly
        //         race any other FMOD API functions, and is thus safe to call.
        let raw = unsafe {
            ptr::NonNull::new_unchecked(fmod::System::raw_create()? as *mut fmod::System)
        };

        // register the global system heart
        let own = Arc::new(TrustMe(BeatingHeart::Core(raw)));
        *heart = Arc::downgrade(&own);

        // and now we're initialized
        Ok(Handle { raw, _own: own })
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
        unsafe { self.raw.as_ref() }
    }
}

impl<T: ?Sized + FmodResource> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut() }
    }
}
