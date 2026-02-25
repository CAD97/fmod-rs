//! We could use bindgen to translate the C headers, but that requires libclang
//! at build time, and the FMOD headers are straightforward and simple enough
//! that parsing with regex is a tenable solution. As a bonus, this maintains
//! the original header organization.
//!
//! NB: The FMOD headers are the same on all platforms, so we could transpile
//! them once and not require downstream users to transpile the headers again.
//! However, it's important that the translated headers match the headers for
//! the specific version of FMOD being linked against to maintain the soundness
//! critical version verification, and the simplest way to resolve this is to
//! just transpile the entire set of headers every time. This also allows us to
//! avoid redistributing the FMOD headers, which is legally questionable, unlike
//! redistributing the examples or a bindings library which links against FMOD.

use build_rs::{input::out_dir, output::rerun_if_changed};
use regex::Regex;
use std::{borrow::Cow, fs, path::Path, sync::LazyLock};

pub fn transpile(inc: impl AsRef<Path>, header: &str, extra_fixup: &[(&str, &str)]) {
    let path = inc.as_ref().join(header);
    rerun_if_changed(&path);
    let mut src = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("read FMOD header file {}", path.display()));

    macro_rules! regex {
        ($regex:literal) => {{
            static REGEX: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(concat!("(?m)", $regex)).unwrap());
            &*REGEX
        }};
    }

    macro_rules! replace {
        ($find:literal, $replace:literal $(,)?) => {
            src = regex!($find).replace_all(&src, $replace).into_owned();
        };
    }

    macro_rules! replace_fixpoint {
        ($find:literal, $replace:literal $(,)?) => {
            loop {
                match regex!($find).replace_all(&src, $replace) {
                    Cow::Owned(replaced) => src = replaced,
                    Cow::Borrowed(_) => break,
                }
            }
        };
    }

    // normalize line endings
    replace!("\r\n", "\n");

    // remove static forward declarations
    replace!(r"^static .*?;$", "");
    // static storage items become pub
    replace!(r"^static ", "pub ");

    // translate #define
    replace_fixpoint!(
        r"typedef ([\w ]*) (\w+);\n(/\*.*?\*/\n)?#define +(\w+) (\s*)(.*)\n",
        "${3}pub const $4: $5$2 = $6;\ntypedef $1 $2;\n",
    );
    replace!(
        r"^#define +(\w+) *(0x\w+) *(?:/\*.*\*/)?$",
        "pub const $1: c_uint = $2;"
    );
    replace!(
        r"^#define +(\w+) (\s*)(\w+) *(?:/\*.*\*/)?$",
        "pub const $1: ${2}c_int = $3;",
    );

    // translate switch
    replace!(r"switch ", "match ");
    replace!(r"case (\w+): ", "$1 =>");
    replace!(r"default ?:", "_ => ");

    // translate enum
    replace!(
        r"typedef enum(.*)\n\{([^}]*?),? *\n\} (\w+)",
        "typedef enum $3\n{$2,\n} $3",
    );
    replace!(
        r"\n\ntypedef enum (\w+)\n\{\n *(\w+), *",
        "\n\npub const $2: $1 = 0;\ntypedef enum $1\n{",
    );
    replace_fixpoint!(
        r"typedef enum (\w+)\n\{\n *(\w+) *= ([-\w]+), *",
        "pub const $2: $1 = $3;\ntypedef enum $1\n{",
    );
    replace_fixpoint!(
        r"const (\w+)(.*)\ntypedef enum (\w+)\n\{\n(\n)? *(\w+), *",
        "const $1$2\n${4}pub const $5: $3 = $1 + 1;\ntypedef enum $3\n{",
    );

    // translate opaque typedefs
    replace!(
        r"typedef struct +(\w+ *) .*;",
        "pub type $1 = self::_$1::$1; mod _$1 { #[repr(C)] pub struct $1 { _pin: ::core::marker::PhantomData<::core::marker::PhantomPinned>, _data: ::core::cell::UnsafeCell<[u8; 0]> } }",
    );

    // translate fn items
    replace!(
        r"([\w*]+) +FB?_API ([\w (),*]*?)\);",
        r#"pub unsafe fn $2) -> $1;"#,
    );

    // translate fn type aliases
    replace!(
        r"typedef (\w+[ *]*) *\(FB?_CALL \*(\w+)\)( *)",
        r#"pub type $2$3 = unsafe extern "system" fn $1"#,
    );

    // translate fn signatures
    replace!(
        r"(?:(pub)|fn) (\w[\w*]+ [\w *]*?)(?: *|(\w+))\(([\w (),*.]*?)\)(;|$)",
        "$1 fn $3($4) -> $2$5"
    );
    replace!(r" -> void *(;|$)", "$1");
    replace!(
        r#"extern "system" +fn \((.*) \.\.\.\)"#,
        r#"extern "C" fn ($1 ...)"#,
    );

    // make fn ptrs optional
    replace!(r#"= (unsafe extern "system" +fn.*?);"#, "= Option<$1>;");

    // translate type aliases
    replace!(
        r"typedef enum.*\n\{\n+.*(= 65536.*\n)?\} (\w+);",
        "typedef int $2;",
    );
    replace!(
        r"(?:typedef )?struct (\w+)\n\{([^}]*)\}.*;",
        "#[repr(C)] #[derive(Copy, Clone)] pub struct $1 {$2}",
    );
    replace!(r"typedef (\w[\w ]*\w) +(\w+);", "pub type $2 = $1;");

    // translate fields
    replace_fixpoint!(
        r"(\(|, |^(?:    )+)([\w--\d][\w ]*?)\b *(\*[ \w*]*?)? *(\w+)(\[\w+\])?([,);])",
        "$1$4: $2$3$5$6",
    );
    replace!(r"^(?:    )+(\w+):(.*);$", "    pub $1:$2,");
    // undo collateral damage to comments
    replace!(r"Pty: Firelight Technologies", "Firelight Technologies Pty");
    replace!(r"interface: C", "C interface");

    // translate pointers
    replace!(
        r"(->|:) const (\w[\w ]*?)((?:(?: *const *)?\*)+)",
        "$1 ${3}const $2",
    );
    replace!(r"(->|:) (\w[\w ]*)(\*+)", "$1 ${3}mut $2");
    replace_fixpoint!(r"\*\*", "*mut *");

    // translate arrays
    replace!(r": ([\w *]+)\[(.*?)\]", ": [$1; ($2) as usize]");

    // translate anonymous union fields
    replace!(
        r"\n\n    union\s*\{([^}]*?) *\}\n\} (FMOD_\w+);",
        "
    pub value: ${2}_VALUE,
}

#[repr(C)] #[derive(Copy, Clone)] pub union ${2}_VALUE {$1}",
    );

    // translate C primitive types
    replace!(r"\<int\>", "c_int");
    replace!(r"\<long long\>", "c_longlong");
    replace!(r"\<short\>", "c_short");
    replace!(r"\<char\>", "c_char");
    replace!(r"\<unsigned c_", "c_u");
    replace!(r"\<float\>", "c_float");
    replace!(r"\<void\>", "c_void");
    replace!(r"\<c_", "::core::ffi::c_");

    // translate string literals
    replace!(r#"(".*?");"#, "c$1.as_ptr(),");

    // escape reserved names
    replace!(r"\<type:", "r#type:");
    replace!(r"\<loop:", "r#loop:");

    // remove preprocessor directives
    replace!(r"^(\s*)#\w(.*\\\n)*.*\n", "");

    // translate extern block
    replace!(r#"extern "C" *(\n|\{)"#, r#"unsafe extern "system" $1"#);

    // fix typedefs incorrectly in extern block
    replace_fixpoint!(
        r#"\n(unsafe extern "system" \{)\n+(pub type .*)"#,
        "\n$2\n$1\n",
    );

    // mark as @generated
    replace!(r#"\A"#, "// @generated\n");

    for &(find, replace) in extra_fixup {
        let find = "(?m)".to_string() + find;
        src = Regex::new(&find)
            .unwrap()
            .replace_all(&src, replace)
            .into_owned();
    }

    fs::write(out_dir().join(header.replace(".h", ".rs")), src)
        .unwrap_or_else(|_| panic!("write FMOD bindings for {}", header));
}
