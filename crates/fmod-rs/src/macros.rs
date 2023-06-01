macro_rules! whoops {
    {
        trace($($trace:tt)*),
        panic($($panic:tt)*) $(,)?
    } => {{
        #[cfg(feature = "tracing")]
        tracing::error!(parent: crate::span(), $($trace)*);
        if cfg!(debug_assertions) {
            if !::std::thread::panicking() {
                panic!($($panic)*);
            }
            use ::std::io::prelude::*;
            let _ = writeln!(::std::io::stderr(), $($panic)*);
        }
    }};
    {
        trace($($trace:tt)*),
        stderr($($panic:tt)*) $(,)?
    } => {{
        #[cfg(feature = "tracing")]
        tracing::error!(parent: crate::span(), $($trace)*);
        if cfg!(debug_assertions) {
            use ::std::io::prelude::*;
            let _ = writeln!(::std::io::stderr(), $($panic)*);
        }
    }};
    ($($args:tt)*) => { whoops!{trace($($args)*), panic($($args)*)} };
}

macro_rules! opaque {
    {
        $(#[$meta:meta])*
        class $Name:ident {
            type Raw = $Raw:ident;
            fn release = $release:expr;
        }
    } => {
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
        unsafe impl fmod::FmodResource for $Name {
            type Raw = $Raw;

            unsafe fn from_raw<'a>(this: *mut Self::Raw) -> &'a Self {
                &*(this as *mut Self)
            }

            #[allow(clippy::redundant_closure_call)]
            unsafe fn release(this: *mut Self::Raw) -> fmod::Result {
                std::ptr::drop_in_place(Self::from_raw(this) as *const Self as *mut Self);
                ffi!(($release)(this))?;
                Ok(())
            }
        }
    };

    ($(#[$meta:meta])* class $Name:ident = $Raw:ident, $raw:ident* ($release:expr) $(;)?) => {
        opaque! {
            $(#[$meta])*
            class $Name {
                type Raw = $Raw;
                fn release = $release;
            }
        }
    };

    ($(#[$meta:meta])* class $Name:ident = $Raw:ident, $raw:ident* $(;)?) => {
        opaque! { $(#[$meta])* class $Name = $Raw, $raw* (paste::paste!([<$raw Release>])) }
    };

    ($(#[$meta:meta])* weak class $Name:ident = $Raw:ident, $raw:ident* $(;)?) => {
        opaque! { $(#[$meta])* class $Name = $Raw, $raw* (|_| FMOD_OK) }
    };
}

#[cfg(feature = "raw")]
macro_rules! raw {
    ($(#[$meta:meta])* pub $($tt:tt)*) => {
        #[allow(clippy::missing_safety_doc, missing_docs)]
        #[cfg_attr(feature = "unstable", doc(cfg(feature = "raw")))]
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

macro_rules! ffi {
    ($e:expr) => {{
        #[allow(unused_unsafe)]
        fmod::Error::from_raw(unsafe { $e })
    }};
}

macro_rules! flags_ops {
    ($Name:ty: $($Op:ident)::+ $fn_op:ident $op:tt) => {
        impl $($Op)::+ for $Name {
            type Output = $Name;
            fn $fn_op(self) -> $Name {
                let raw = $op <$Name>::into_raw(self);
                <$Name>::from_raw(raw)
            }
        }

        impl $($Op)::+ for &'_ $Name {
            type Output = $Name;
            fn $fn_op(self) -> $Name {
                $op *self
            }
        }
    };
    ($Name:ty: $($Op:ident)::+ $fn_op:ident $op:tt $($OpAssign:ident)::+ $fn_op_assign:ident) => {
        impl $($Op)::+ for $Name {
            type Output = $Name;
            fn $fn_op(self, rhs: $Name) -> $Name {
                let raw = <$Name>::into_raw(self) $op <$Name>::into_raw(rhs);
                <$Name>::from_raw(raw)
            }
        }

        impl $($Op)::+<&$Name> for $Name {
            type Output = $Name;
            fn $fn_op(self, rhs: &$Name) -> $Name {
                self $op *rhs
            }
        }

        impl $($Op)::+<$Name> for &$Name {
            type Output = $Name;
            fn $fn_op(self, rhs: $Name) -> $Name {
                *self $op rhs
            }
        }

        impl $($Op)::+<&$Name> for &$Name {
            type Output = $Name;
            fn $fn_op(self, rhs: &$Name) -> $Name {
                *self $op *rhs
            }
        }

        impl $($OpAssign)::+ for $Name {
            fn $fn_op_assign(&mut self, rhs: $Name) {
                *self = *self $op rhs;
            }
        }

        impl $($OpAssign)::+<&$Name> for $Name {
            fn $fn_op_assign(&mut self, rhs: &$Name) {
                *self = *self $op *rhs;
            }
        }
    };
}

macro_rules! flags {
    {$(
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident: $Raw:ty {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    )*} => {$(

        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $Name {
            raw: $Raw,
        }

        impl $Name {
            $(
                flags! {@stripdefault
                    $(#[$($vmeta)*])*
                    #[allow(non_upper_case_globals)]
                    pub const $Variant: Self = Self::from_raw($value);
                }
            )*
        }

        impl $Name {
            raw! {
                pub const fn zeroed() -> $Name {
                    Self::from_raw(0)
                }
            }
            raw! {
                pub const fn from_raw(raw: $Raw) -> $Name {
                    unsafe { ::std::mem::transmute(raw) }
                }
            }
            raw! {
                pub const fn from_raw_ref(raw: &$Raw) -> &$Name {
                    unsafe { &*(raw as *const $Raw as *const $Name ) }
                }
            }
            raw! {
                pub fn from_raw_mut(raw: &mut $Raw) -> &mut $Name {
                    unsafe { &mut *(raw as *mut $Raw as *mut $Name ) }
                }
            }
            raw! {
                pub const fn into_raw(self) -> $Raw {
                    unsafe { ::std::mem::transmute(self) }
                }
            }
            raw! {
                pub const fn as_raw(&self) -> &$Raw {
                    unsafe { &*(self as *const $Name as *const $Raw ) }
                }
            }
            raw! {
                pub fn as_raw_mut(&mut self) -> &mut $Raw {
                    unsafe { &mut *(self as *mut $Name as *mut $Raw ) }
                }
            }

            pub fn is_set(self, variant: Self) -> bool {
                self & variant == variant
            }
        }

        flags_ops!($Name: std::ops::BitAnd bitand & std::ops::BitAndAssign bitand_assign);
        flags_ops!($Name: std::ops::BitOr bitor | std::ops::BitOrAssign bitor_assign);
        flags_ops!($Name: std::ops::BitXor bitxor ^ std::ops::BitXorAssign bitxor_assign);
        flags_ops!($Name: std::ops::Not not !);

        impl std::fmt::Debug for $Name {
            #[allow(unreachable_patterns)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $($Name::$Variant => f.debug_struct(stringify!($Variant)).finish(),)*
                    _ => f.debug_struct(stringify!($Name)).field("raw", &self.raw).finish(),
                }
            }
        }

        flags! {@default $Name {$(
            $(#[$($vmeta)*])*
            $Variant = $value,
        )*}}
    )*};

    {@default $Name:ident {}} => {};

    {@default $Name:ident {
        #[default]
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        #[doc = concat!("[`", stringify!($Name), "::", stringify!($Variant), "`]")]
        impl Default for $Name {
            fn default() -> $Name {
                $Name::$Variant
            }
        }
        flags! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@default $Name:ident {
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        flags! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@stripdefault #[default] $($tt:tt)*} => { $($tt)* };
    {@stripdefault $($tt:tt)*} => { $($tt)* };
}

macro_rules! enum_struct {
    {$(
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    )*} => {$(
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $Name {
            raw: $Raw,
        }

        impl $Name {
            $(
                enum_struct! {@stripdefault
                    $(#[$($vmeta)*])*
                    #[allow(non_upper_case_globals)]
                    pub const $Variant: Self = Self::from_raw($value);
                }
            )*
        }

        impl $Name {
            raw! {
                pub const fn zeroed() -> $Name {
                    Self::from_raw(0)
                }
            }
            raw! {
                pub const fn from_raw(raw: $Raw) -> $Name {
                    unsafe { ::std::mem::transmute(raw) }
                }
            }
            raw! {
                pub const fn from_raw_ref(raw: &$Raw) -> &$Name {
                    unsafe { &*(raw as *const $Raw as *const $Name ) }
                }
            }
            raw! {
                pub fn from_raw_mut(raw: &mut $Raw) -> &mut $Name {
                    unsafe { &mut *(raw as *mut $Raw as *mut $Name ) }
                }
            }
            raw! {
                pub const fn into_raw(self) -> $Raw {
                    unsafe { ::std::mem::transmute(self) }
                }
            }
            raw! {
                pub const fn as_raw(&self) -> &$Raw {
                    unsafe { &*(self as *const $Name as *const $Raw ) }
                }
            }
            raw! {
                pub fn as_raw_mut(&mut self) -> &mut $Raw {
                    unsafe { &mut *(self as *mut $Name as *mut $Raw ) }
                }
            }
        }

        impl std::fmt::Debug for $Name {
            #[allow(deprecated, unreachable_patterns)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $($Name::$Variant => f.debug_struct(stringify!($Variant)).finish(),)*
                    _ => f.debug_struct(stringify!($Name)).field("raw", &self.raw).finish(),
                }
            }
        }

        enum_struct! {@default $Name {$(
            $(#[$($vmeta)*])*
            $Variant = $value,
        )*}}
    )*};

    {@default $Name:ident {}} => {};

    {@default $Name:ident {
        #[default]
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        #[doc = concat!("[`", stringify!($Name), "::", stringify!($Variant), "`]")]
        impl Default for $Name {
            fn default() -> $Name {
                $Name::$Variant
            }
        }
        enum_struct! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@default $Name:ident {
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        enum_struct! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@stripdefault #[default] $($tt:tt)*} => { $($tt)* };
    {@stripdefault $($tt:tt)*} => { $($tt)* };
}

macro_rules! fmod_struct {
    {$(
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident = $Raw:ident {
            $($body:tt)*
        }
    )*} => {$(
        #[repr(C)]
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, ::smart_default::SmartDefault, PartialEq)]
        pub struct $Name {
            $($body)*
        }

        ::static_assertions::assert_eq_size!($Name, $Raw);
        ::static_assertions::assert_eq_align!($Name, $Raw);

        impl $Name {
            raw! {
                pub const fn from_raw(raw: $Raw) -> $Name {
                    unsafe { ::std::mem::transmute(raw) }
                }
            }
            raw! {
                pub const fn from_raw_ref(raw: &$Raw) -> &$Name {
                    unsafe { &*(raw as *const $Raw as *const $Name ) }
                }
            }
            raw! {
                pub fn from_raw_mut(raw: &mut $Raw) -> &mut $Name {
                    unsafe { &mut *(raw as *mut $Raw as *mut $Name ) }
                }
            }
            raw! {
                pub const fn into_raw(self) -> $Raw {
                    unsafe { ::std::mem::transmute(self) }
                }
            }
            raw! {
                pub const fn as_raw(&self) -> &$Raw {
                    unsafe { &*(self as *const $Name as *const $Raw ) }
                }
            }
            raw! {
                pub fn as_raw_mut(&mut self) -> &mut $Raw {
                    unsafe { &mut *(self as *mut $Name as *mut $Raw ) }
                }
            }
        }
    )*};
}
