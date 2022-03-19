//! Zerde is our lightweight manual serialization/deserialization system.
//!
//! Keep in sync with zerde.ts!
//!
//! With Zerde you manually push data into a buffer, and then parse it out in the same manner
//! on the other side. Buffers are used for internal communication only and use system endianness.
//!
//! Zerde buffers have to be 64-bit aligned at the start. They start with 8 bytes indicating
//! the size of the buffer in bytes (which has to be a multiple of 4).
//!
//! After that, a number of "slots" follow (4 bytes each). The following datatypes are supported:
//! * u32: 1 slot
//! * f32: 1 slot
//! * u64: 2 slots, might be preceded with an empty slot to get 64-bit alignment
//! * f64: 2 slots, might be preceded with an empty slot to get 64-bit alignment
//! * string: 1 slot indicating number of slots to follow; then 1 slot per character, encoded as UTF-32. No terminator.
//! * byte slice: 1 slot indicating the number of bytes in the slice. Then bytes/4 slots (rounded up), where each
//!     slot contains 4 bytes, with the first byte of a 4-byte tuple being put in the least-significant byte of the
//!     slot, and the last byte of the 4-byte tuple ending up in the most-significant byte of the slot. The final slot
//!     might have some zeroes in its most significant bytes, if the length of the byte slice was not a multiple of 4.
//!     TODO(JP): This is all a little complicated and makes reading out the data a bit slow / requiring a copy, since
//!     the bytes are not in a nice order. We might want to get rid of this entirely and use pointers instead, though
//!     that would make the format less self-contained; we could also just reorder the bytes.
//!
//! Pointers are not natively supported but can be put in `u64`.
//!
//! Zerde buffers will automatically grow as you put more data in it.
//!
//! Arrays are not natively supported, but it's typical to pack arrays of data in a similar way to strings, with a
//! preceding length value.
//!
//! Packing heterogeneous data is also possible, by prefixing your data with a number representing the data type of the
//! data that follows.
//!
//! Another common pattern is arrays of heterogeneous data, where you don't have to prefix the array with a size, but
//! can instead have a special data type (e.g. "0") representing the end of the array. This is what we use at the top
//! level for communicating messages between WebAssembly and Rust in the main event loop.

// Not everything in this file is used in all contexts.
#![allow(dead_code)]

use std::alloc;
use std::mem;
use std::ptr;
use std::sync::Arc;

use crate::*;

/// Serializing data into a Zerde buffer.
pub(crate) struct ZerdeBuilder {
    mu32: *mut u32,
    mf32: *mut f32,
    mu64: *mut u64,
    mf64: *mut f64,
    slots: usize,
    used_slots: isize,
}

impl ZerdeBuilder {
    pub(crate) fn new() -> Self {
        unsafe {
            let slots = 1024;
            let buf =
                alloc::alloc(alloc::Layout::from_size_align(4 * slots as usize, mem::align_of::<u32>()).unwrap()) as *mut u32;
            (buf as *mut u64).write(4 * slots as u64);

            Self {
                mu32: buf as *mut u32,
                mf32: buf as *mut f32,
                mu64: buf as *mut u64,
                mf64: buf as *mut f64,
                slots,
                used_slots: 2,
            }
        }
    }

    /// If necessary, grows the buffer (exponentially). Returns the slot offset to write to.
    fn fit(&mut self, slots: usize) -> isize {
        if self.used_slots as usize + slots > self.slots {
            let mut new_slots = usize::max(self.used_slots as usize + slots, self.slots * 2);
            if new_slots & 1 != 0 {
                // f64 align
                new_slots += 1;
            }
            let new_bytes = new_slots << 2;
            let old_bytes = self.slots << 2;
            let new_buf = unsafe {
                let new_buf =
                    alloc::alloc(alloc::Layout::from_size_align(new_bytes as usize, mem::align_of::<u64>()).unwrap()) as *mut u32;
                ptr::copy_nonoverlapping(self.mu32, new_buf, self.slots);
                alloc::dealloc(
                    self.mu32 as *mut u8,
                    alloc::Layout::from_size_align(old_bytes as usize, mem::align_of::<u64>()).unwrap(),
                );
                (new_buf as *mut u64).write(new_bytes as u64);
                new_buf
            };
            self.slots = new_slots;
            self.mu32 = new_buf;
            self.mf32 = new_buf as *mut f32;
            self.mu64 = new_buf as *mut u64;
            self.mf64 = new_buf as *mut f64;
        }
        let pos = self.used_slots;
        self.used_slots += slots as isize;
        pos
    }

    pub(crate) fn send_u32(&mut self, v: u32) {
        // NOTE(JP): Cannot inline `pos` here!
        let pos = self.fit(1);
        unsafe {
            self.mu32.offset(pos).write(v);
        }
    }

    pub(crate) fn send_f32(&mut self, v: f32) {
        // NOTE(JP): Cannot inline `pos` here!
        let pos = self.fit(1);
        unsafe {
            self.mf32.offset(pos).write(v);
        }
    }

    pub(crate) fn send_f64(&mut self, v: f64) {
        if self.used_slots & 1 != 0 {
            // 64-bit alignment.
            self.fit(1);
        }
        // NOTE(JP): Cannot inline `pos` here!
        let pos = self.fit(2);
        unsafe {
            self.mf64.offset(pos >> 1).write(v);
        }
    }

    pub(crate) fn send_u64(&mut self, v: u64) {
        if self.used_slots & 1 != 0 {
            // 64-bit alignment.
            self.fit(1);
        }
        // NOTE(JP): Cannot inline `pos` here!
        let pos = self.fit(2);
        unsafe {
            self.mu64.offset(pos >> 1).write(v);
        }
    }

    pub(crate) fn send_string(&mut self, msg: &str) {
        let len = msg.chars().count();
        self.send_u32(len as u32);
        for c in msg.chars() {
            self.send_u32(c as u32);
        }
    }

    pub(crate) fn send_u8slice(&mut self, msg: &[u8]) {
        let u8_len = msg.len();
        let len = u8_len >> 2;
        let spare = u8_len & 3;
        self.send_u32(u8_len as u32);
        // this is terrible. im sure this can be done so much nicer
        for i in 0..len {
            self.send_u32(
                (msg[(i << 2)] as u32)
                    | ((msg[(i << 2) + 1] as u32) << 8)
                    | ((msg[(i << 2) + 2] as u32) << 16)
                    | ((msg[(i << 2) + 3] as u32) << 24),
            );
        }
        match spare {
            1 => self.send_u32(msg[(len << 2)] as u32),
            2 => self.send_u32((msg[(len << 2)] as u32) | ((msg[(len << 2) + 1] as u32) << 8)),
            3 => self
                .send_u32((msg[(len << 2)] as u32) | ((msg[(len << 2) + 1] as u32) << 8) | ((msg[(len << 2) + 2] as u32) << 16)),
            _ => (),
        }
    }

    fn build_mutable_buffer<T>(&mut self, buffer_type: u32, buffer: Vec<T>) {
        let mut buffer = std::mem::ManuallyDrop::new(buffer);

        self.send_u32(buffer_type);

        self.send_u32(buffer.as_mut_ptr() as u32);
        self.send_u32((buffer.len() * std::mem::size_of::<T>()) as u32);
        self.send_u32((buffer.capacity() * std::mem::size_of::<T>()) as u32);
    }

    fn build_read_only_buffer<T>(&mut self, buffer_type: u32, buffer: Arc<Vec<T>>) {
        self.send_u32(buffer_type);

        self.send_u32(buffer.as_ptr() as u32);
        self.send_u32((buffer.len() * std::mem::size_of::<T>()) as u32);
        // releasing buffer from Arc memory management
        let arc_ptr = Arc::into_raw(Arc::clone(&buffer)) as u32;
        self.send_u32(arc_ptr);
    }

    pub(crate) fn build_zap_params(&mut self, params: Vec<ZapParam>) {
        self.send_u32(params.len() as u32);

        for param in params {
            match param {
                ZapParam::String(str) => {
                    self.send_u32(ZAP_PARAM_STRING);

                    self.send_string(&str);
                }
                ZapParam::ReadOnlyU8Buffer(buffer) => {
                    self.build_read_only_buffer(ZAP_PARAM_READ_ONLY_UINT8_BUFFER, buffer);
                }
                ZapParam::MutableU8Buffer(buffer) => {
                    self.build_mutable_buffer(ZAP_PARAM_UINT8_BUFFER, buffer);
                }
                ZapParam::ReadOnlyF32Buffer(buffer) => {
                    self.build_read_only_buffer(ZAP_PARAM_READ_ONLY_FLOAT32_BUFFER, buffer);
                }
                ZapParam::MutableF32Buffer(buffer) => {
                    self.build_mutable_buffer(ZAP_PARAM_FLOAT32_BUFFER, buffer);
                }
                ZapParam::ReadOnlyU32Buffer(buffer) => {
                    self.build_read_only_buffer(ZAP_PARAM_READ_ONLY_UINT32_BUFFER, buffer);
                }
                ZapParam::MutableU32Buffer(buffer) => {
                    self.build_mutable_buffer(ZAP_PARAM_UINT32_BUFFER, buffer);
                }
            }
        }
    }

    pub(crate) fn take_ptr(self /* move! */) -> u64 {
        self.mu32 as u64
    }
}

/// Parsing a Zerde buffer.
pub(crate) struct ZerdeParser {
    mu32: *mut u32,
    mf32: *mut f32,
    mu64: *mut u64,
    mf64: *mut f64,
    slots: usize,
    used_slots: isize,
}

impl Drop for ZerdeParser {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(
                self.mu32 as *mut u8,
                alloc::Layout::from_size_align((self.slots * mem::size_of::<u64>()) as usize, mem::align_of::<u32>()).unwrap(),
            );
        }
    }
}

impl ZerdeParser {
    pub(crate) fn from(buf: u64) -> ZerdeParser {
        unsafe {
            let bytes = (buf as *mut u64).read() as usize;
            ZerdeParser {
                mu32: buf as *mut u32,
                mf32: buf as *mut f32,
                mu64: buf as *mut u64,
                mf64: buf as *mut f64,
                used_slots: 2,
                slots: bytes >> 2,
            }
        }
    }

    pub(crate) fn parse_u32(&mut self) -> u32 {
        unsafe {
            let ret = self.mu32.offset(self.used_slots).read();
            self.used_slots += 1;
            ret
        }
    }

    pub(crate) fn parse_f32(&mut self) -> f32 {
        unsafe {
            let ret = self.mf32.offset(self.used_slots).read();
            self.used_slots += 1;
            ret
        }
    }

    pub(crate) fn parse_f64(&mut self) -> f64 {
        unsafe {
            if self.used_slots & 1 != 0 {
                // 64-bit alignment.
                self.used_slots += 1;
            }
            let ret = self.mf64.offset(self.used_slots >> 1).read();
            self.used_slots += 2;
            ret
        }
    }

    pub(crate) fn parse_u64(&mut self) -> u64 {
        unsafe {
            if self.used_slots & 1 != 0 {
                // 64-bit alignment.
                self.used_slots += 1;
            }
            let ret = self.mu64.offset(self.used_slots >> 1).read();
            self.used_slots += 2;
            ret
        }
    }

    pub(crate) fn parse_string(&mut self) -> String {
        let len = self.parse_u32();
        let mut out = String::with_capacity(len as usize);
        for _ in 0..len {
            if let Some(c) = std::char::from_u32(self.parse_u32()) {
                out.push(c);
            }
        }
        out
    }

    pub(crate) fn parse_vec_ptr(&mut self) -> Vec<u8> {
        let vec_ptr = self.parse_u32() as *mut u8;
        let vec_len = self.parse_u32() as usize;
        unsafe { Vec::<u8>::from_raw_parts(vec_ptr, vec_len, vec_len) }
    }

    pub(crate) fn parse_arc_vec<T>(&mut self) -> Arc<Vec<T>> {
        let arc_ptr = self.parse_u32() as *const Vec<T>;
        unsafe { Arc::from_raw(arc_ptr) }
    }

    fn parse_buffer<T>(&mut self) -> Vec<T> {
        let vec_ptr = self.parse_u32();
        let vec_len = self.parse_u32() as usize / std::mem::size_of::<T>();
        let vec_cap = self.parse_u32() as usize / std::mem::size_of::<T>();
        unsafe { Vec::from_raw_parts(vec_ptr as *mut T, vec_len, vec_cap) }
    }

    pub(crate) fn parse_zap_params(&mut self) -> Vec<ZapParam> {
        let len = self.parse_u32();
        (0..len)
            .map(|_| {
                let param_type = self.parse_u32();
                match param_type {
                    ZAP_PARAM_STRING => self.parse_string().into_param(),
                    ZAP_PARAM_READ_ONLY_UINT8_BUFFER => self.parse_arc_vec::<u8>().into_param(),
                    ZAP_PARAM_READ_ONLY_FLOAT32_BUFFER => self.parse_arc_vec::<f32>().into_param(),
                    ZAP_PARAM_READ_ONLY_UINT32_BUFFER => self.parse_arc_vec::<u32>().into_param(),
                    ZAP_PARAM_UINT8_BUFFER => self.parse_buffer::<u8>().into_param(),
                    ZAP_PARAM_FLOAT32_BUFFER => self.parse_buffer::<f32>().into_param(),
                    ZAP_PARAM_UINT32_BUFFER => self.parse_buffer::<u32>().into_param(),
                    value => panic!("Unexpected param type: {}", value),
                }
            })
            .collect()
    }
}
