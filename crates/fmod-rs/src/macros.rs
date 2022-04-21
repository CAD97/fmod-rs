macro_rules! opaque {
    {
        $(#[$meta:meta])*
        class $Name:ident {
            type Raw = $Raw:ident;
            fn release = $release:expr;
        }
    } => {
        use fmod::FmodResource;

        #[cfg(not(feature = "unstable"))]
        pub struct $Name {
            _data: ::std::cell::Cell<[u8; 0]>,
            _marker: ::std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
        }

        #[cfg(feature = "unstable")]
        extern {
            $(#[$meta])*
            pub type $Name;
        }

        unsafe impl Send for $Name {}
        unsafe impl Sync for $Name {}
        impl fmod::Sealed for $Name {}
        unsafe impl FmodResource for $Name {
            type Raw = $Raw;

            unsafe fn from_raw<'a>(this: *mut Self::Raw) -> &'a Self {
                &*(this as *mut Self)
            }

            #[allow(clippy::redundant_closure_call)]
            unsafe fn release(this: *mut Self::Raw) -> fmod::Result {
                std::ptr::drop_in_place(Self::from_raw(this) as *const Self as *mut Self);
                fmod_try!(($release)(this));
                Ok(())
            }
        }
    };

    ($(#[$meta:meta])* class $Name:ident = $Raw:ident, $raw:ident*) => {
        opaque! {
            $(#[$meta:meta])*
            class $Name {
                type Raw = $Raw;
                fn release = paste::paste!([<$raw Release>]);
            }
        }
    };

    ($(#[$meta:meta])* weak class $Name:ident = $Raw:ident, $raw:ident*) => {
        opaque! {
            $(#[$meta:meta])*
            class $Name {
                type Raw = $Raw;
                fn release = |_| FMOD_OK;
            }
        }
    };
}

#[cfg(feature = "raw")]
macro_rules! raw {
    ($(#[$meta:meta])* pub $($tt:tt)*) => {
        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[doc(cfg(feature = "raw"))]
        $(#[$meta])* pub $($tt)*
    };
    ($mac:ident! { $(#[$meta:meta])* pub $($tt:tt)* }) => {
        $mac! {
            #[allow(clippy::missing_safety_doc, missing_docs)]
            #[doc(cfg(feature = "raw"))]
            $(#[$meta])* pub $($tt)*
        }
    };

}

#[cfg(not(feature = "raw"))]
macro_rules! raw {
    ($(#[$meta:meta])* pub $($tt:tt)*) => {
        #[allow(clippy::missing_safety_doc, dead_code, missing_docs)]
        $(#[$meta])* pub(crate) $($tt)*
    };
    ($mac:ident! { $(#[$meta:meta])* pub $($tt:tt)* }) => {
        $mac! {
            #[allow(clippy::missing_safety_doc, dead_code, missing_docs)]
            $(#[$meta])* pub(crate) $($tt)*
        }
    };
}

macro_rules! fmod_try {
    ($e:expr) => {{
        #[allow(unused_unsafe)]
        let result = unsafe { $e };
        if let Some(error) = fmod::Error::from_raw(result) {
            return Err(error);
        }
    }};
}
