//! ⚡ This crate is the core of Zaplib. It contains all the
//! fundamental rendering primitives.
//!
//! Internally it depends on [`zaplib_shader_compiler`] and [`zaplib_vector`],
//! for shader compilation and vector graphics (mostly for fonts) respectively.
//!
//! If you need to use higher-level widgets, use `zaplib_components`.

// Necessary in cx_xlib
#![allow(temporary_cstring_as_ptr)]
// Not great but we do these comparisons all over the place..
#![allow(clippy::float_cmp)]
// We want to use links to private fields, since we use `--document-private-items`.
#![allow(rustdoc::private_intra_doc_links)]
// For using [`std::alloc::set_alloc_error_hook`].
#![cfg_attr(target_arch = "wasm32", feature(alloc_error_hook))]
// For using [`core::arch::wasm32`].
#![cfg_attr(target_arch = "wasm32", feature(stdsimd))]

#[macro_use]
mod macros;

#[cfg(any(target_os = "linux"))]
mod cx_linux;
#[cfg(target_os = "linux")]
mod cx_opengl;
#[cfg(target_os = "linux")]
mod cx_xlib;
#[cfg(target_os = "linux")]
pub(crate) use cx_linux::*;
#[cfg(target_os = "linux")]
pub(crate) use cx_opengl::*;

#[cfg(any(target_os = "macos"))]
mod cx_apple;
#[cfg(target_os = "macos")]
mod cx_cocoa;
#[cfg(any(target_os = "macos"))]
mod cx_macos;
#[cfg(target_os = "macos")]
mod cx_metal;
#[cfg(target_os = "macos")]
pub(crate) use cx_macos::*;
#[cfg(target_os = "macos")]
pub(crate) use cx_metal::*;

#[cfg(target_os = "windows")]
mod cx_dx11;
#[cfg(target_os = "windows")]
mod cx_win32;
#[cfg(any(target_os = "windows"))]
mod cx_windows;
#[cfg(target_os = "windows")]
pub(crate) use cx_dx11::*;
#[cfg(target_os = "windows")]
pub(crate) use cx_windows::*;

#[cfg(target_arch = "wasm32")]
mod cx_wasm32;
#[cfg(target_arch = "wasm32")]
mod cx_webgl;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod cx_desktop;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub(crate) use cx_desktop::*;

#[cfg(target_arch = "wasm32")]
pub use cx_wasm32::*;
#[cfg(target_arch = "wasm32")]
pub(crate) use cx_webgl::*;

#[cfg(feature = "cef")]
mod cef_browser;
#[cfg(feature = "cef")]
mod cef_utils;

#[cfg(any(target_arch = "wasm32", feature = "cef"))]
mod cx_web;
#[cfg(any(target_arch = "wasm32", feature = "cef"))]
mod zerde;

mod animator;
mod area;
pub mod byte_extract;
pub mod cast;
mod colors;
mod component_id;
mod cursor;
mod cx;
pub mod debug_log;
mod debugger;
mod draw_tree;
mod events;
mod fonts;
mod geometry;
mod hash;
mod layout;
mod layout_api;
mod layout_internal;
mod param;
mod pass;
mod profile;
mod read_seek;
mod shader;
mod texture;
pub mod universal_file;
pub mod universal_http_stream;
mod universal_instant;
pub mod universal_rand;
pub mod universal_thread;
mod window;

mod cube_ins;
mod image_ins;
mod menu;
mod quad_ins;
mod std_shader;
mod text_ins;

use cast::*;

pub use area::*;
pub use cast::*;
pub use cube_ins::*;
pub use cursor::*;
pub use cx::*;
pub use debugger::*;
pub use events::*;
pub use image_ins::*;
pub use param::*;
pub use quad_ins::*;
pub use std_shader::*;
pub use text_ins::*;
pub use texture::*;
pub use window::*;
pub use zaplib_shader_compiler::code_fragment::CodeFragment;
pub use zaplib_shader_compiler::math::*;
pub use zaplib_shader_compiler::ty::Ty;

pub use animator::*;
pub use colors::*;
pub use component_id::*;
pub use draw_tree::*;
pub use fonts::*;
pub use geometry::*;
pub use hash::*;
pub use layout::*;
pub use layout_api::*;
pub use layout_internal::*;
pub use macros::*;
pub use menu::*;
pub use pass::*;
pub use read_seek::*;
pub use shader::*;
pub use universal_file::*;
pub use universal_instant::*;
