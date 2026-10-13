#![allow(nonstandard_style, unused_parens, clippy::needless_return)]
#![allow(clippy::missing_safety_doc, clippy::unnecessary_cast)]
#![cfg_attr(docsrs, doc = "This crate is intentionally empty on doc.rs.")]

include!(concat!(env!("OUT_DIR"), "/fsbank.rs"));
include!(concat!(env!("OUT_DIR"), "/fsbank_errors.rs"));
