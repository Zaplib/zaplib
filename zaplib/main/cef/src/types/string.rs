use std::collections::HashMap;
use std::ptr::null_mut;
use widestring::U16CString;
use zaplib_cef_sys::{cef_string_list_t, cef_string_map_t, cef_string_utf16_t};

pub type CefString = CefStringUTF16;
pub struct CefStringUTF16 {
    str: U16CString,
}
impl CefStringUTF16 {
    pub fn from_str(s: &str) -> Self {
        Self {
            // TODO error safety
            str: U16CString::from_str(s).unwrap(),
        }
    }
    pub unsafe fn from_cef(ptr: *const cef_string_utf16_t) -> CefStringUTF16 {
        if ptr.is_null() {
            CefStringUTF16 { str: U16CString::from_str("").unwrap() }
        } else {
            // It's a pointer, so CEF retains ownership and will call the dtor

            CefStringUTF16 {
                // TODO error safety
                str: U16CString::from_ptr((*ptr).str_, (*ptr).length as usize).unwrap(),
            }
        }
    }
    pub unsafe fn from_userfree_cef(ptr: *mut cef_string_utf16_t) -> CefStringUTF16 {
        let res = Self::from_cef(ptr);
        // `ptr` can be null for empty strings.
        if !ptr.is_null() {
            zaplib_cef_sys::cef_string_userfree_utf16_free(ptr);
        }
        res
    }
    pub fn into_cef(self) -> cef_string_utf16_t {
        extern "C" fn free_str(ptr: *mut u16) {
            if ptr.is_null() {
                return;
            }
            // TODO - what about the cef_string_utf16_t wrapper?
            unsafe {
                // Restore and drop
                U16CString::from_raw(ptr);
            }
        }

        cef_string_utf16_t { length: self.str.len() as u64, str_: self.str.into_raw(), dtor: Some(free_str) }
    }
    pub fn convert_str_to_cef(s: Option<&str>) -> cef_string_utf16_t {
        s.map(|x| CefString::from_str(x).into_cef()).unwrap_or_else(|| unsafe { std::mem::zeroed() })
    }
    pub unsafe fn parse_string_list(ptr: cef_string_list_t) -> Vec<String> {
        let count = zaplib_cef_sys::cef_string_list_size(ptr);
        let mut res = Vec::with_capacity(count as usize);
        for i in 0..count {
            let value = null_mut();
            if zaplib_cef_sys::cef_string_list_value(ptr, i, value) > 0 {
                res.push(CefString::from_cef(value).to_string());
            }
        }
        res
    }
    pub unsafe fn parse_string_map(ptr: cef_string_map_t) -> HashMap<String, String> {
        let count = zaplib_cef_sys::cef_string_map_size(ptr);
        let mut res = HashMap::with_capacity(count as usize);
        for i in 0..count {
            let key = null_mut();
            let value = null_mut();
            zaplib_cef_sys::cef_string_map_key(ptr, i, key);
            zaplib_cef_sys::cef_string_map_value(ptr, i, value);

            res.insert(CefString::from_cef(key).to_string(), CefString::from_cef(value).to_string());
        }
        res
    }
}
impl ToString for CefStringUTF16 {
    fn to_string(&self) -> String {
        self.str.to_string_lossy()
    }
}
