macro_rules! static_assert {
    ($cond:expr $(,)?) => {
        #[allow(deprecated)]
        const _: () = { ::std::assert!($cond) };
    };
    ($cond:expr, $msg:expr $(,)?) => {
        #[allow(deprecated)]
        const _: () = { ::std::assert!($cond, $msg) };
    };
}

macro_rules! yeet {
    ($err:expr) => {
        return Err($err)?
    };
}

macro_rules! ix {
    ($ix:expr) => {
        match $ix {
            ix => match usize::try_from(ix) {
                Ok(ix) => ix,
                Err(_) => panic!("index out of bounds: {ix} is not a valid index"),
            },
        }
    };
}

macro_rules! group_syntax { ($($tt:tt)*) => ($($tt)*) }

macro_rules! whoops {
    {
        panic: $($args:tt)*
    } => {{
        #[cfg(feature = "log")]
        log::error!($($args)*);
        if cfg!(debug_assertions) {
            if !::std::thread::panicking() {
                panic!($($args)*);
            }
            use ::std::io::prelude::*;
            let _ = writeln!(::std::io::stderr(), $($args)*);
        }
    }};
    {
        no_panic: $($args:tt)*
    } => {{
        #[cfg(feature = "log")]
        log::error!($($args)*);
        if cfg!(debug_assertions) {
            use ::std::io::prelude::*;
            let _ = writeln!(::std::io::stderr(), $($args)*);
        }
    }};
    ($($args:tt)*) => { whoops!{panic: $($args)*} };
}

macro_rules! opaque_type {
    {
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident $(;)?
    } => {
        #[cfg(not(feature = "unstable"))]
        $(#[$meta])*
        $vis struct $Name {
            _data: ::std::cell::Cell<[u8; 0]>,
            _marker: ::std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
        }

        #[cfg(feature = "unstable")]
        #[allow(unused_doc_comments)] // false positive vvvvvv
        #[doc(cfg(all()))] // doesn't actually require cfg(feature = "unstable")
        extern {
            $(#[$meta])*
            $vis type $Name;
        }
    };
}

macro_rules! fmod_class {
    {
        #[doc = $doc:expr]
        $(#[$meta:meta])*
        class $prefix:literal $Name:ident {
            type Raw = $Raw:ty;
            fn release = $release:expr;
        }
    } => {
        opaque_type! {
            #[doc = $doc]
            $(#[$meta])*
            pub struct $Name;
        }

        unsafe impl Send for $Name {}
        unsafe impl Sync for $Name {}
        impl ::std::panic::UnwindSafe for $Name {}
        impl ::std::panic::RefUnwindSafe for $Name {}
        impl ::fmod::Sealed for $Name {}
        unsafe impl ::fmod::Resource for $Name {
            type Raw = $Raw;

            unsafe fn from_raw<'a>(this: *mut Self::Raw) -> &'a Self {
                debug_assert!(!this.is_null());
                &*(this as *mut Self)
            }

            #[allow(clippy::redundant_closure_call)]
            unsafe fn release(this: *mut Self::Raw) -> fmod::Result {
                ::std::ptr::drop_in_place(Self::from_raw(this) as *const Self as *mut Self);
                ffi!(($release)(this))?;
                Ok(())
            }
        }

        impl ::std::fmt::Debug for $Name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, concat!($prefix, stringify!($Name), "({:p})"), self)
            }
        }
    };

    {
        #[doc = $doc:expr]
        $(#[$meta:meta])*
        class $prefix:literal $Name:ident {
            type Raw = $Raw:ty;
            fn release = $release:expr;
        }

        mod $($module:ident),+;
    } => {
        fmod_class! {
            #[doc = $doc]
            $(#[$meta])*
            class $prefix $Name {
                type Raw = $Raw;
                fn release = $release;
            }
        }

        ::paste::paste! {
            #[doc = $doc]
            pub mod [<$Name:snake>] {
                $(mod $module;)+
                pub use super::$Name;
                #[allow(unused_imports)]
                pub use /*self::*/{
                    $($module::*,)+
                };
            }
            pub use self::[<$Name:snake>]::*;
        }
    };

    {
        #[doc = $doc:expr]
        $(#[$meta:meta])*
        class $prefix:literal $Name:ident {
            type Raw = $Raw:ty;
            fn release = $release:expr;
        }

        mod;
    } => {
        fmod_class! {
            #[doc = $doc]
            $(#[$meta])*
            class $prefix $Name {
                type Raw = $Raw;
                fn release = $release;
            }
        }

        ::paste::paste! {
            #[doc = $doc]
            pub mod [<$Name:snake>];
            pub use self::[<$Name:snake>]::*;
        }
    };

    {
        #[doc = $doc:expr]
        $(#[$meta:meta])*
        class $Name:ident = $Raw:ident;
        $(mod $($module:ident),*;)?
    } => {
        fmod_class! {
            #[doc = $doc]
            $(#[$meta])*
            class "fmod::" $Name {
                type Raw = ::fmod::raw::$Raw;
                fn release = $Name::raw_release;
            }
            $(mod $($module),*;)?
        }
    };

    {
        #[doc = $doc:expr]
        $(#[$meta:meta])*
        weak class $Name:ident = $Raw:ident;
        $(mod $($module:ident),*;)?
    } => {
        fmod_class! {
            #[doc = $doc]
            $(#[$meta])*
            class "fmod::" $Name {
                type Raw = ::fmod::raw::$Raw;
                fn release = |_| ::fmod::raw::FMOD_OK;
            }
            $(mod $($module),*;)?
        }
    };

    {
        #[doc = $doc:expr]
        $(#[$meta:meta])*
        class studio::$Name:ident = $Raw:ident;
        $(mod $($module:ident),*;)?
    } => {
        fmod_class! {
            #[doc = $doc]
            $(#[$meta])*
            class "fmod::studio::" $Name {
                type Raw = ::fmod::raw::$Raw;
                fn release = $Name::raw_release;
            }
            $(mod $($module),*;)?
        }
    };

    {
        #[doc = $doc:expr]
        $(#[$meta:meta])*
        weak class studio::$Name:ident = $Raw:ident;
        $(mod $($module:ident),*;)?
    } => {
        fmod_class! {
            #[doc = $doc]
            $(#[$meta])*
            class "fmod::studio::" $Name {
                type Raw = ::fmod::raw::$Raw;
                fn release = |_| ::fmod::raw::FMOD_OK;
            }
            $(mod $($module),*;)?
        }
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

macro_rules! fmod_flags_ops {
    ($Name:ty: $($Op:ident)::+ $fn_op:ident $op:tt) => {
        #[allow(deprecated)]
        impl $($Op)::+ for $Name {
            type Output = $Name;
            fn $fn_op(self) -> $Name {
                let raw = $op <$Name>::into_raw(self);
                <$Name>::from_raw(raw)
            }
        }

        #[allow(deprecated)]
        impl $($Op)::+ for &'_ $Name {
            type Output = $Name;
            fn $fn_op(self) -> $Name {
                $op *self
            }
        }
    };
    ($Name:ty: $($Op:ident)::+ $fn_op:ident $op:tt $($OpAssign:ident)::+ $fn_op_assign:ident) => {
        #[allow(deprecated)]
        impl $($Op)::+ for $Name {
            type Output = $Name;
            fn $fn_op(self, rhs: $Name) -> $Name {
                let raw = <$Name>::into_raw(self) $op <$Name>::into_raw(rhs);
                <$Name>::from_raw(raw)
            }
        }

        #[allow(deprecated)]
        impl $($Op)::+<&$Name> for $Name {
            type Output = $Name;
            fn $fn_op(self, rhs: &$Name) -> $Name {
                self $op *rhs
            }
        }

        #[allow(deprecated)]
        impl $($Op)::+<$Name> for &$Name {
            type Output = $Name;
            fn $fn_op(self, rhs: $Name) -> $Name {
                *self $op rhs
            }
        }

        #[allow(deprecated)]
        impl $($Op)::+<&$Name> for &$Name {
            type Output = $Name;
            fn $fn_op(self, rhs: &$Name) -> $Name {
                *self $op *rhs
            }
        }

        #[allow(deprecated)]
        impl $($OpAssign)::+ for $Name {
            fn $fn_op_assign(&mut self, rhs: $Name) {
                *self = *self $op rhs;
            }
        }

        #[allow(deprecated)]
        impl $($OpAssign)::+<&$Name> for $Name {
            fn $fn_op_assign(&mut self, rhs: &$Name) {
                *self = *self $op *rhs;
            }
        }
    };
}

macro_rules! fmod_flags {
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

        #[allow(deprecated)]
        impl $Name {
            $(
                fmod_flags! {@stripdefault
                    $(#[$($vmeta)*])*
                    #[allow(non_upper_case_globals)]
                    pub const $Variant: Self = Self::from_raw($value);
                }
            )*
        }

        #[allow(deprecated)]
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

            /// Check whether *all* flags of the argument are set.
            pub fn is_set(self, variant: Self) -> bool {
                self & variant == variant
            }
        }

        fmod_flags_ops!($Name: std::ops::BitAnd bitand & std::ops::BitAndAssign bitand_assign);
        fmod_flags_ops!($Name: std::ops::BitOr bitor | std::ops::BitOrAssign bitor_assign);
        fmod_flags_ops!($Name: std::ops::BitXor bitxor ^ std::ops::BitXorAssign bitxor_assign);
        fmod_flags_ops!($Name: std::ops::Not not !);

        #[allow(deprecated)]
        impl std::fmt::Debug for $Name {
            #[allow(unreachable_patterns)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $($Name::$Variant => f.debug_struct(stringify!($Variant)).finish(),)*
                    _ => f.debug_struct(stringify!($Name)).field("raw", &self.raw).finish(),
                }
            }
        }

        fmod_flags! {@default $Name {$(
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
        fmod_flags! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@default $Name:ident {
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        fmod_flags! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@stripdefault #[default] $($tt:tt)*} => { $($tt)* };
    {@stripdefault $($tt:tt)*} => { $($tt)* };
}

macro_rules! fmod_enum {
    {
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty
        $(where)?
        {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    } => {
        ::paste::paste! {
            fmod_enum! {
                $(#[$meta])*
                $vis enum $Name: $Raw
                where
                    const { self < [<$Raw _MAX>] },
                    const { self >= 0 },
                {$(
                    $(#[$($vmeta)*])*
                    $Variant = $value,
                )*}
            }
        }
    };
    {
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty
        where const { self <= $MAX:expr } $(,)?
        {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    } => {
        fmod_enum! {
            $(#[$meta])*
            $vis enum $Name: $Raw
            where
                const { self < $MAX + 1 },
                const { self >= 0 },
            {$(
                $(#[$($vmeta)*])*
                $Variant = $value,
            )*}
        }
    };
    {
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty
        where
            const { self < $MAX:expr } $(,)?
        {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    } => {
        fmod_enum! {
            $(#[$meta])*
            $vis enum $Name: $Raw
            where
                const { self < $MAX },
                const { self >= 0 },
            {$(
                $(#[$($vmeta)*])*
                $Variant = $value,
            )*}
        }
    };
    {
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty
        where
            const { self >= $MIN:expr } $(,)?
        {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    } => {
        ::paste::paste! {
            fmod_enum! {
                $(#[$meta])*
                $vis enum $Name: $Raw
                where
                    const { self < [<$Raw _MAX>] },
                    const { self >= $MIN },
                {$(
                    $(#[$($vmeta)*])*
                    $Variant = $value,
                )*}
            }
        }
    };
    {
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty
        where
            const { self <= $MAX:expr },
            const { self >= $MIN:expr },
        {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    } => {
        fmod_enum! {
            $(#[$meta])*
            $vis enum $Name: $Raw
            where
                const { self < $MAX + 1 },
                const { self >= $MIN },
            {$(
                $(#[$($vmeta)*])*
                $Variant = $value,
            )*}
        }
    };

    {
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty
        where
            const { self < $MAX:expr },
            const { self >= $MIN:expr },
        {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    } => {
        $(#[$meta])*
        #[repr(i32)]
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        $vis enum $Name {
            $(
                $(#[$($vmeta)*])*
                $Variant = $value,
            )*
        }

        impl $Name {
            raw! {
                pub const RAW_RANGE: ::std::ops::Range<i32> = $MIN..$MAX;
            }
            raw! {
                pub const fn zeroed() -> $Name {
                    unsafe { Self::from_raw(0) }
                }
            }
            raw! {
                pub const unsafe fn from_raw(raw: $Raw) -> $Name {
                    debug_assert!($MIN <= raw && raw < $MAX);
                    unsafe { ::std::mem::transmute(raw) }
                }
            }
            raw! {
                pub const fn try_from_raw(raw: $Raw) -> Result<$Name> {
                    if $MIN <= raw && raw < $MAX {
                        Ok(unsafe { Self::from_raw(raw) })
                    } else {
                        Err(Error::InvalidParam)
                    }
                }
            }
            raw! {
                pub const unsafe fn from_raw_ref(raw: &$Raw) -> &$Name {
                    debug_assert!($MIN <= *raw && *raw < $MAX);
                    unsafe { &*(raw as *const $Raw as *const $Name ) }
                }
            }
            raw! {
                pub unsafe fn from_raw_mut(raw: &mut $Raw) -> &mut $Name {
                    debug_assert!($MIN <= *raw && *raw < $MAX);
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
                pub unsafe fn as_raw_mut(&mut self) -> &mut $Raw {
                    unsafe { &mut *(self as *mut $Name as *mut $Raw ) }
                }
            }
        }

        $(
            static_assert!($Name::$Variant.into_raw() < $Name::RAW_RANGE.end);
            static_assert!($Name::$Variant.into_raw() >= $Name::RAW_RANGE.start);
        )*

        static_assert! {
            [$($Name::$Variant),*].len() == ($Name::RAW_RANGE.end - $Name::RAW_RANGE.start) as usize,
            "fmod_enum! is missing some variant(s)",
        }

        impl fmod::effect::DspParamType for $Name {
            fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &Self) -> Result {
                dsp.set_parameter::<$Raw>(index, value.into_raw())
            }

            fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<Self> {
                let value = dsp.get_parameter::<$Raw>(index)?;
                Self::try_from_raw(value)
            }

            fn get_dsp_parameter_string(dsp: &Dsp, index: i32) -> Result<fmod::ArrayString<32>> {
                dsp.get_parameter_string::<$Raw>(index)
            }
        }
    };
}

macro_rules! fmod_typedef {
    {
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Raw:ty {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    } => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $Name {
            raw: $Raw,
        }

        impl $Name {
            $(
                fmod_typedef! {@stripdefault
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

        fmod_typedef! {@default $Name {$(
            $(#[$($vmeta)*])*
            $Variant = $value,
        )*}}
    };

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
        fmod_typedef! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@default $Name:ident {
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        fmod_typedef! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@stripdefault #[default] $($tt:tt)*} => { $($tt)* };
    {@stripdefault $($tt:tt)*} => { $($tt)* };
}

macro_rules! fmod_struct {
    {
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident$(<$lt:lifetime>)? = $Raw:ident {
            $($body:tt)*
        }
    } => {
        fmod_struct! {
            #![fmod_no_default]
            $(#[$meta])*
            #[derive(::smart_default::SmartDefault)]
            $vis struct $Name$(<$lt>)? = $Raw {
                $($body)*
            }
        }
    };
    {
        #![fmod_no_default]
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident$(<$lt:lifetime>)? = $Raw:ident {
            $($body:tt)*
        }
    } => {
        #[repr(C)]
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $Name$(<$lt>)? {
            $($body)*
        }

        static_assert!(::std::mem::size_of::<$Name>() == ::std::mem::size_of::<$Raw>());
        static_assert!(::std::mem::align_of::<$Name>() == ::std::mem::align_of::<$Raw>());

        impl$(<$lt>)? $Name$(<$lt>)? {
            raw! {
                pub const fn from_raw(raw: $Raw) -> $Name$(<$lt>)? {
                    unsafe { ::std::mem::transmute(raw) }
                }
            }
            raw! {
                pub const fn from_raw_ref(raw: &$Raw) -> &$Name$(<$lt>)? {
                    unsafe { &*(raw as *const $Raw as *const $Name$(<$lt>)? ) }
                }
            }
            raw! {
                pub fn from_raw_mut(raw: &mut $Raw) -> &mut $Name$(<$lt>)? {
                    unsafe { &mut *(raw as *mut $Raw as *mut $Name$(<$lt>)? ) }
                }
            }
            raw! {
                pub const fn into_raw(self) -> $Raw {
                    unsafe { ::std::mem::transmute(self) }
                }
            }
            raw! {
                pub const fn as_raw(&self) -> &$Raw {
                    unsafe { &*(self as *const $Name$(<$lt>)? as *const $Raw ) }
                }
            }
            raw! {
                pub fn as_raw_mut(&mut self) -> &mut $Raw {
                    unsafe { &mut *(self as *mut $Name$(<$lt>)? as *mut $Raw ) }
                }
            }
        }
    };
}
