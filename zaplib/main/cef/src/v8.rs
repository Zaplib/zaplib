use crate::ptr::{wrap_ptr, BaseRefCountedExt, RefCounterGuard, WrapperFor};
use crate::types::string::CefString;
use crate::ToCef;
use std::ptr::null_mut;
use std::slice;
use std::sync::Arc;
use zaplib_cef_sys::{
    cef_string_utf16_t, cef_v8_propertyattribute_t, cef_v8array_buffer_release_callback_t, cef_v8context_get_current_context,
    cef_v8context_t, cef_v8handler_t, cef_v8value_create_array, cef_v8value_create_array_buffer, cef_v8value_create_function,
    cef_v8value_create_int, cef_v8value_create_object, cef_v8value_create_string, cef_v8value_create_uint, cef_v8value_t, size_t,
};

pub type V8PropertyAttribute = cef_v8_propertyattribute_t;

pub struct V8Context {
    ptr: RefCounterGuard<cef_v8context_t>,
}

impl V8Context {
    pub(crate) fn from(ptr: *mut cef_v8context_t, track_ref: bool) -> Self {
        unsafe { Self { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    // == Static Functions API ==
    pub fn get_current_context() -> V8Context {
        unsafe { V8Context::from(cef_v8context_get_current_context(), true) }
    }
    // pub fn cef_v8context_get_entered_context() -> *mut cef_v8context_t;
    // pub fn cef_v8context_in_context() -> ::std::os::raw::c_int;

    // == Class Methods API ==
    // TODO(Dmitry): implement more API when needed
    // pub fn get_task_runner(&self) -> *mut _cef_task_runner_t {}
    // pub fn is_valid(&self) -> bool {}
    // pub fn get_browser(&self) -> *mut _cef_browser_t {}
    // pub fn get_frame(&self) -> *mut _cef_frame_t {}

    pub fn get_global(&self) -> Option<V8Value> {
        if let Some(func) = self.ptr.as_ref().get_global {
            unsafe { Some(V8Value::from(func(self.ptr.get()), true)) }
        } else {
            None
        }
    }

    pub fn enter(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().enter {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    pub fn exit(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().exit {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }

    // pub fn is_same(&self, that: &V8Context) -> bool {}

    // pub fn eval(&self, code: *const cef_string_t,
    //         script_url: *const cef_string_t,
    //         start_line: ::std::os::raw::c_int,
    //         retval: *mut *mut _cef_v8value_t,
    //         exception: *mut *mut _cef_v8exception_t,
    //     ) -> bool {}
}

pub trait V8ArrayBufferReleaseCallback {
    fn release_buffer(&self, _buffer: *const u8) {}
}

impl V8ArrayBufferReleaseCallback for () {}

struct V8ArrayBufferReleaseCallbackWrapper<T: V8ArrayBufferReleaseCallback> {
    _base: cef_v8array_buffer_release_callback_t,
    internal: Arc<T>,
}

unsafe impl<T: V8ArrayBufferReleaseCallback> WrapperFor<cef_v8array_buffer_release_callback_t>
    for V8ArrayBufferReleaseCallbackWrapper<T>
{
}
impl<T: V8ArrayBufferReleaseCallback> V8ArrayBufferReleaseCallbackWrapper<T> {
    fn from_ptr<'a>(
        ptr: *mut cef_v8array_buffer_release_callback_t,
    ) -> &'a mut BaseRefCountedExt<cef_v8array_buffer_release_callback_t, V8ArrayBufferReleaseCallbackWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    unsafe extern "C" fn release_buffer(
        handler: *mut cef_v8array_buffer_release_callback_t,
        buffer: *mut ::std::os::raw::c_void,
    ) {
        let handler = Self::from_ptr(handler);
        handler.internal.release_buffer(buffer as *const u8);
    }
}

impl<T: V8ArrayBufferReleaseCallback> ToCef<cef_v8array_buffer_release_callback_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_v8array_buffer_release_callback_t {
        wrap_ptr(|base| V8ArrayBufferReleaseCallbackWrapper {
            _base: cef_v8array_buffer_release_callback_t {
                base,
                release_buffer: Some(V8ArrayBufferReleaseCallbackWrapper::<T>::release_buffer),
            },
            internal: self.clone(),
        })
    }
}

#[derive(Clone)]
pub struct V8Value {
    ptr: RefCounterGuard<cef_v8value_t>,
}

impl V8Value {
    pub(crate) fn from(ptr: *mut cef_v8value_t, track_ref: bool) -> Self {
        unsafe { Self { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    // == Static Functions API ==
    // TODO(Dmitry): implement more API when needed
    // pub fn cef_v8value_create_undefined() -> Self
    // pub fn cef_v8value_create_null() -> Self
    // pub fn cef_v8value_create_bool(value: ::std::os::raw::c_int) -> Self
    pub fn create_int(value: i32) -> Self {
        unsafe { Self::from(cef_v8value_create_int(value), true) }
    }

    pub fn create_uint(value: u32) -> Self {
        unsafe { Self::from(cef_v8value_create_uint(value), true) }
    }
    // pub fn cef_v8value_create_double(value: f64) -> Self
    // pub fn cef_v8value_create_date(date: *const cef_time_t) -> Self

    pub fn create_string(value: &str) -> Self {
        let value = CefString::from_str(value);
        unsafe { Self::from(cef_v8value_create_string(&value.into_cef()), true) }
    }

    pub fn create_object(/* accessor: *mut cef_v8accessor_t, interceptor: *mut cef_v8interceptor_t */) -> Self {
        unsafe { Self::from(cef_v8value_create_object(null_mut(), null_mut()), true) }
    }

    pub fn create_array(length: usize) -> Self {
        unsafe { Self::from(cef_v8value_create_array(length as ::std::os::raw::c_int), true) }
    }

    pub fn create_array_buffer<T: V8ArrayBufferReleaseCallback>(
        buffer: *const u8,
        length: usize,
        release_callback: Arc<T>,
    ) -> Self {
        unsafe {
            Self::from(
                cef_v8value_create_array_buffer(
                    buffer as *mut ::std::os::raw::c_void,
                    length as size_t,
                    release_callback.to_cef(),
                ),
                true,
            )
        }
    }

    pub fn create_function<TV8Handler: V8Handler>(name: &str, handler: &Arc<TV8Handler>) -> Self {
        let name = CefString::from_str(name);
        unsafe { V8Value::from(cef_v8value_create_function(&name.into_cef(), handler.to_cef()), true) }
    }

    /// WARNING(JP): this has to be `fn`; a static function that doesn't capture anything. I tried this before with `Fn` but
    /// Rust lets you capture things that shouldn't be captured, with the current architecture. Be very very careful if you want
    /// to change this back to `Fn` (or one of the other closure variants)! I added an `other_data` field so it's at least a bit
    /// easier to capture some arbitrary state.
    ///
    /// TODO(JP): Fix things so you can just use `Fn` here instead of having to use this this `other_data` hack.
    pub fn create_function_from_fn<T>(
        name: &str,
        other_data: T,
        func: fn(&CefString, &V8Value, &[V8Value], &T) -> Option<Result<V8Value, String>>,
    ) -> Self {
        struct FnHandler<T> {
            func: fn(&CefString, &V8Value, &[V8Value], &T) -> Option<Result<V8Value, String>>,
            other_data: T,
        }
        impl<T> V8Handler for FnHandler<T> {
            fn execute(&self, name: &CefString, object: &V8Value, arguments: &[V8Value]) -> Option<Result<V8Value, String>> {
                (self.func)(name, object, arguments, &self.other_data)
            }
        }

        let handler = Arc::new(FnHandler { func, other_data });
        Self::create_function(name, &handler)
    }

    // == Class Methods API ==
    // TODO(Dmitry): implement more API when needed
    // pub fn is_valid(&self) -> bool
    // pub fn is_undefined(&self) -> bool
    // pub fn is_null(&self) -> bool
    // pub fn is_bool(&self) -> bool
    // pub fn is_int(&self) -> bool
    // pub fn is_uint(&self) -> bool
    // pub fn is_double(&self) -> bool
    // pub fn is_date(&self) -> bool
    pub fn is_string(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_string {
            unsafe { func(self.ptr.get()) != 0 }
        } else {
            false
        }
    }
    // pub fn is_object(&self) -> bool
    pub fn is_array(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_array {
            unsafe { func(self.ptr.get()) != 0 }
        } else {
            false
        }
    }

    pub fn is_array_buffer(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_array_buffer {
            unsafe { func(self.ptr.get()) != 0 }
        } else {
            false
        }
    }

    // pub fn is_function(&self) -> bool
    // pub fn is_same(&self, that: *mut _cef_v8value_t) -> bool
    pub fn get_bool_value(&self) -> bool {
        let func = self.ptr.as_ref().get_bool_value.expect("get_bool_value called on a non-boolean value");
        unsafe { func(self.ptr.get()) != 0 }
    }

    pub fn get_int_value(&self) -> i32 {
        let func = self.ptr.as_ref().get_int_value.expect("get_int_value called on a non-integer value");
        unsafe { func(self.ptr.get()) }
    }
    pub fn get_uint_value(&self) -> u32 {
        let func = self.ptr.as_ref().get_uint_value.expect("get_uint_value called on a non-uint value");
        unsafe { func(self.ptr.get()) }
    }
    // pub fn get_double_value(&self) -> f64
    // pub fn get_date_value(&self) -> cef_time_t
    pub fn get_string_value(&self) -> String {
        if let Some(func) = self.ptr.as_ref().get_string_value {
            unsafe { CefString::from_userfree_cef(func(self.ptr.get())) }.to_string()
        } else {
            "".to_string()
        }
    }
    // pub fn is_user_created(&self) -> bool
    // pub fn has_exception(&self) -> bool
    // pub fn get_exception(&self) -> *mut _cef_v8exception_t
    // pub fn clear_exception(&self) -> bool
    // pub fn will_rethrow_exceptions(&self) -> bool
    // pub fn set_rethrow_exceptions(&self, rethrow: ::std::os::raw::c_int) -> bool
    // pub fn has_value_bykey(&self, key: *const cef_string_t) -> bool
    pub fn has_value_byindex(&self, index: usize) -> bool {
        if let Some(func) = self.ptr.as_ref().has_value_byindex {
            unsafe { func(self.ptr.get(), index as ::std::os::raw::c_int) > 0 }
        } else {
            false
        }
    }

    // pub fn delete_value_bykey(&self, key: *const cef_string_t) -> bool,
    // pub fn delete_value_byindex(&self, index: ::std::os::raw::c_int) -> bool,
    // pub fn get_value_bykey(&self, key: *const cef_string_t) -> *mut _cef_v8value_t
    pub fn get_value_byindex(&self, index: usize) -> Option<V8Value> {
        if let Some(func) = self.ptr.as_ref().get_value_byindex {
            unsafe { Some(V8Value::from(func(self.ptr.get(), index as ::std::os::raw::c_int), true)) }
        } else {
            None
        }
    }

    pub fn set_value_bykey(&self, key: &str, value: &V8Value, attribute: V8PropertyAttribute) -> bool {
        if let Some(func) = self.ptr.as_ref().set_value_bykey {
            let key = CefString::from_str(key);
            unsafe { func(self.ptr.get(), &key.into_cef(), value.ptr.to_cef_as_arg(), attribute) > 0 }
        } else {
            false
        }
    }

    /// Convenience wrapper for [`V8Value::set_value_bykey`] and [`V8Value::create_function_from_fn`].
    pub fn set_fn_value<T>(
        &self,
        key: &str,
        other_data: T,
        func: fn(&CefString, &V8Value, &[V8Value], &T) -> Option<Result<V8Value, String>>,
    ) -> bool {
        self.set_value_bykey(
            key,
            &V8Value::create_function_from_fn(key, other_data, func),
            V8PropertyAttribute::V8_PROPERTY_ATTRIBUTE_NONE,
        )
    }

    pub fn set_value_byindex(&self, index: usize, value: &V8Value) -> bool {
        if let Some(func) = self.ptr.as_ref().set_value_byindex {
            unsafe { func(self.ptr.get(), index as ::std::os::raw::c_int, value.ptr.to_cef_as_arg()) > 0 }
        } else {
            false
        }
    }
    // pub fn set_value_byaccessor(&self,
    //         key: *const cef_string_t,
    //         settings: cef_v8_accesscontrol_t,
    //         attribute: cef_v8_propertyattribute_t
    //     ) -> bool
    // pub fn get_keys(&self, keys: cef_string_list_t) -> bool
    // pub fn set_user_data(&self, user_data: *mut _cef_base_ref_counted_t) -> bool,
    // pub fn get_user_data(&self) -> *mut _cef_base_ref_counted_t
    // pub fn get_externally_allocated_memory(&self) -> bool
    // pub fn adjust_externally_allocated_memory(&self, change_in_bytes: ::std::os::raw::c_int) -> bool
    pub fn get_array_length(&self) -> usize {
        if let Some(func) = self.ptr.as_ref().get_array_length {
            unsafe { func(self.ptr.get()) as usize }
        } else {
            0
        }
    }

    pub fn get_array_buffer_release_callback<T: V8ArrayBufferReleaseCallback>(&self) -> Option<Arc<T>> {
        if let Some(func) = self.ptr.as_ref().get_array_buffer_release_callback {
            unsafe { Some(V8ArrayBufferReleaseCallbackWrapper::<T>::from_ptr(func(self.ptr.get())).internal.clone()) }
        } else {
            None
        }
    }

    // pub fn neuter_array_buffer(&self) -> bool
    // pub fn get_function_name(&self) -> cef_string_userfree_t
    // pub fn get_function_handler(&self) -> *mut _cef_v8handler_t
    // pub fn execute_function(&self,
    //         object: *mut _cef_v8value_t,
    //         argumentsCount: size_t,
    //         arguments: *const *mut _cef_v8value_t
    //     ) -> *mut _cef_v8value_t
    // pub fn execute_function_with_context(&self,
    //         context: *mut _cef_v8context_t,
    //         object: *mut _cef_v8value_t,
    //         argumentsCount: size_t,
    //         arguments: *const *mut _cef_v8value_t
    //     ) -> *mut _cef_v8value_t
}

pub trait V8Handler {
    fn execute(&self, _name: &CefString, _object: &V8Value, _arguments: &[V8Value]) -> Option<Result<V8Value, String>> {
        None
    }
}
impl V8Handler for () {}

struct V8HandlerWrapper<T: V8Handler> {
    _base: cef_v8handler_t,
    internal: Arc<T>,
}
unsafe impl<T: V8Handler> WrapperFor<cef_v8handler_t> for V8HandlerWrapper<T> {}
impl<T: V8Handler> V8HandlerWrapper<T> {
    fn from_ptr<'a>(ptr: *mut cef_v8handler_t) -> &'a mut BaseRefCountedExt<cef_v8handler_t, V8HandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    unsafe extern "C" fn execute(
        handler: *mut cef_v8handler_t,
        cef_name: *const cef_string_utf16_t,
        cef_object: *mut cef_v8value_t,
        cef_arguments_count: u64,
        cef_arguments: *const *mut cef_v8value_t,
        cef_retval: *mut *mut cef_v8value_t,
        cef_exception: *mut cef_string_utf16_t,
    ) -> i32 {
        let handler = Self::from_ptr(handler);
        let name = CefString::from_cef(cef_name);
        let object = V8Value::from(cef_object, false);
        let arguments: Vec<V8Value> = slice::from_raw_parts(cef_arguments, cef_arguments_count as usize)
            .iter()
            .map(|cef_argument| V8Value::from(*cef_argument, false))
            .collect();
        match handler.internal.execute(&name, &object, &arguments) {
            Some(result) => {
                match result {
                    Ok(value) => *cef_retval = *value.ptr,
                    Err(err) => {
                        *cef_retval = null_mut();
                        *cef_exception = CefString::convert_str_to_cef(Some(&err))
                    }
                }
                1
            }
            None => 0,
        }
    }
}
impl<T: V8Handler> ToCef<cef_v8handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_v8handler_t {
        wrap_ptr(|base| V8HandlerWrapper {
            _base: cef_v8handler_t { base, execute: Some(V8HandlerWrapper::<T>::execute) },
            internal: self.clone(),
        })
    }
}
