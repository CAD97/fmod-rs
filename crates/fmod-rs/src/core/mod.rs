use {
    crate::Result,
    std::{
        mem::ManuallyDrop,
        ops::{Deref, DerefMut},
    },
};

mod channel;
mod channel_group;
mod dsp;
mod sound;
mod system;

pub use self::{channel::*, channel_group::*, dsp::*, sound::*, system::*};

#[allow(clippy::missing_safety_doc)]
pub unsafe trait FmodResource {
    type Raw;
    #[allow(clippy::missing_safety_doc)]
    unsafe fn release(this: *mut Self) -> Result<()>;
}

/// A handle to an FMOD managed resource.
pub struct Handle<T: FmodResource> {
    raw: *mut T,
}

impl<T: FmodResource> Handle<T> {
    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut T::Raw) -> Self {
            Self { raw: raw as *mut T }
        }

        pub fn into_raw(this: Self) -> *mut T::Raw {
            let this = ManuallyDrop::new(this);
            this.raw as *mut T::Raw
        }
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

impl<T: FmodResource> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.raw }
    }
}

impl<T: FmodResource> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.raw }
    }
}

impl<T: FmodResource> Drop for Handle<T> {
    fn drop(&mut self) {
        unsafe { T::release(self.raw) }.unwrap_or_else(|error| {
            #[cfg(feature = "tracing")]
            tracing::error!(
                parent: &*crate::SPAN,
                error = error.into_raw(),
                "FMOD error releasing {}({:p}): {error}",
                std::any::type_name::<T>(),
                &mut **self as *mut T,
            );
        });
    }
}
