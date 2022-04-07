macro_rules! opaque {
    ($($(#[$meta:meta])* class $Name:ident = $Raw:ident, $release:ident;)*) => {$(
        #[cfg(not(feature = "unstable"))]
        #[repr(C)]
        $(#[$meta])*
        pub struct $Name {
            _data: std::cell::Cell<[u8; 0]>,
            _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
        }

        #[cfg(feature = "unstable")]
        extern {
            $(#[$meta])*
            pub type $Name;
        }

        unsafe impl fmod::FmodResource for $Name {
            type Raw = $Raw;

            unsafe fn release(this: *mut $Raw) -> fmod::Result {
                fmod_try!($release(this));
                Ok(())
            }

            unsafe fn cast_raw(this: *mut $Raw) -> *mut Self {
                this as *mut Self
            }
        }

        impl $Name {
            raw! {
                #[allow(clippy::missing_safety_doc)]
                pub unsafe fn from_raw(raw: *mut $Raw) -> &'static $Name {
                    &*(raw as *mut _)
                }

                pub fn as_raw(&self) -> *mut $Raw {
                    self as *const _ as *const _ as *mut _
                }
            }
        }
    )*};
}

#[cfg(feature = "raw")]
macro_rules! raw {
    ($(#[$meta:meta])* pub $($tt:tt)*) => {
        $(#[$meta])* pub $($tt)*
    };
}

#[cfg(not(feature = "raw"))]
macro_rules! raw {
    ($(#[$meta:meta])* pub $($tt:tt)*) => {
        #[allow(dead_code)]
        $(#[$meta])* pub(crate) $($tt)*
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
