//! Abstraction for reading and seeking.

use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};

/// A trait for the combination of reading and seeking.
///
/// TODO(JP): [`BufReader`] is explicitly not included here since its [`Seek`]
/// behavior sucks: it causes clearing its internal buffer. Instead you have
/// to use [`BufReader::seek_relative`], but that breaks this nice transparent abstraction,
/// so maybe we should make our own [`BufReader`] variant that doesn't clear
/// on seek, and can be transparently passed into anything that accepts
/// [`ReadSeek`].
pub trait ReadSeek: Read + Seek {}
impl ReadSeek for File {}
impl ReadSeek for Cursor<Vec<u8>> {}
impl ReadSeek for Cursor<&Vec<u8>> {}

/// Convenient alias for a [`BufReader`] that contains a dynamic dispatch pointer
/// to a [`ReadSeek`].
pub type ReadSeekBufReader = BufReader<Box<dyn ReadSeek>>;
