// Clippy TODO
#![allow(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]

#[cfg(not(target_arch = "wasm32"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
