use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::types::string::CefString;
use crate::{Browser, CefPoint, CefRange, CefRect, DragOperationsMask, PaintElementType, TextInputMode, ToCef};
use std::ptr::null_mut;
use std::slice::from_raw_parts;
use std::sync::Arc;
use zaplib_cef_sys::{
    cef_accessibility_handler_t, cef_browser_t, cef_drag_data_t, cef_drag_operations_mask_t, cef_paint_element_type_t,
    cef_range_t, cef_rect_t, cef_render_handler_t, cef_screen_info_t, cef_string_t, cef_text_input_mode_t,
};

pub trait RenderHandler {
    //get_accessibility_handler
    fn get_root_screen_rect(&self, browser: &Browser) -> Option<CefRect>;
    fn get_view_rect(&self, browser: &Browser) -> CefRect;
    fn get_screen_point(&self, _browser: &Browser, _view: CefPoint) -> Option<CefPoint> {
        None
    }
    //get_screen_info:
    fn on_popup_show(&self, _browser: &Browser, _show: bool) {}
    fn on_popup_size(&self, _browser: &Browser, _rect: CefRect) {}
    fn on_paint(
        &self,
        browser: &Browser,
        type_: PaintElementType,
        dirty_rects: &[CefRect],
        bytes: &[u8],
        width: i32,
        height: i32,
    );
    fn on_accelerated_paint(
        &self,
        _browser: &Browser,
        _type_: PaintElementType,
        _dirty_rects: &[CefRect],
        _shader_handle: *mut std::os::raw::c_void,
    ) {
    }
    fn update_drag_cursor(&self, _browser: &Browser, _operation: DragOperationsMask) {}
    fn on_scroll_offset_changed(&self, _browser: &Browser, _x: f64, _y: f64) {}
    fn on_ime_composition_range_changed(&self, _browser: &Browser, _selected_range: CefRange, _character_bounds: &[CefRect]) {}
    fn on_text_selection_changed(&self, _browser: &Browser, _selected_text: String, _selected_range: CefRange) {}
    fn on_virtual_keyboard_requested(&self, _browser: &Browser, _input_mode: TextInputMode) {}
}

impl RenderHandler for () {
    fn get_root_screen_rect(&self, _browser: &Browser) -> Option<CefRect> {
        None
    }

    fn get_view_rect(&self, _browser: &Browser) -> CefRect {
        CefRect::default()
    }

    fn on_paint(
        &self,
        _browser: &Browser,
        _type_: PaintElementType,
        _dirty_rects: &[CefRect],
        _bytes: &[u8],
        _width: i32,
        _height: i32,
    ) {
    }
}

struct RenderHandlerWrapper<T: RenderHandler> {
    _base: cef_render_handler_t,
    internal: Arc<T>,
}
unsafe impl<T: RenderHandler> WrapperFor<cef_render_handler_t> for RenderHandlerWrapper<T> {}
impl<T: RenderHandler> RenderHandlerWrapper<T> {
    fn from_ptr<'a>(ptr: *mut cef_render_handler_t) -> &'a mut BaseRefCountedExt<cef_render_handler_t, RenderHandlerWrapper<T>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    extern "C" fn get_accessibility_handler(_client: *mut cef_render_handler_t) -> *mut cef_accessibility_handler_t {
        // TODO
        null_mut()
    }

    extern "C" fn get_root_screen_rect(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        rect: *mut cef_rect_t,
    ) -> ::std::os::raw::c_int {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        if let Some(res) = client.internal.get_root_screen_rect(&browser) {
            if !rect.is_null() {
                unsafe {
                    (*rect).x = res.x;
                    (*rect).y = res.y;
                    (*rect).width = res.width;
                    (*rect).height = res.height;
                }
            }
            1
        } else {
            0
        }
    }
    extern "C" fn get_view_rect(client: *mut cef_render_handler_t, browser: *mut cef_browser_t, rect: *mut cef_rect_t) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let res = client.internal.get_view_rect(&browser);
        if !rect.is_null() {
            unsafe {
                (*rect).x = res.x;
                (*rect).y = res.y;
                (*rect).width = res.width;
                (*rect).height = res.height;
            }
        }
    }
    extern "C" fn get_screen_point(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        view_x: ::std::os::raw::c_int,
        view_y: ::std::os::raw::c_int,
        screen_x: *mut ::std::os::raw::c_int,
        screen_y: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let view_point = CefPoint { x: view_x, y: view_y };

        let screen_point = client.internal.get_screen_point(&browser, view_point);
        if let Some(screen_point) = screen_point {
            unsafe {
                *screen_x = screen_point.x;
                *screen_y = screen_point.y;
            }
            1
        } else {
            0
        }
    }

    extern "C" fn get_screen_info(
        _client: *mut cef_render_handler_t,
        _browser: *mut cef_browser_t,
        _screen_info: *mut cef_screen_info_t,
    ) -> ::std::os::raw::c_int {
        // TODO
        0
    }

    extern "C" fn on_popup_show(client: *mut cef_render_handler_t, browser: *mut cef_browser_t, show: ::std::os::raw::c_int) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        client.internal.on_popup_show(&browser, show > 0);
    }

    extern "C" fn on_popup_size(client: *mut cef_render_handler_t, browser: *mut cef_browser_t, rect: *const cef_rect_t) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let rect = CefRect::from_ptr(rect);
        client.internal.on_popup_size(&browser, rect);
    }

    extern "C" fn on_paint(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        type_: cef_paint_element_type_t,
        dirty_rects_count: u64,
        dirty_rects: *const cef_rect_t,
        buffer: *const ::std::os::raw::c_void,
        width: ::std::os::raw::c_int,
        height: ::std::os::raw::c_int,
    ) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let element_type = unsafe { std::mem::transmute(type_) };
        let dirty_rects = CefRect::from_array(dirty_rects_count as usize, dirty_rects);
        let bytes = unsafe { from_raw_parts(buffer as *const u8, (width * height * 4) as usize) };

        client.internal.on_paint(&browser, element_type, &dirty_rects, bytes, width, height);
    }

    extern "C" fn on_accelerated_paint(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        type_: cef_paint_element_type_t,
        dirty_rects_count: u64,
        dirty_rects: *const cef_rect_t,
        shared_handle: *mut ::std::os::raw::c_void,
    ) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let element_type = unsafe { std::mem::transmute(type_) };
        let dirty_rects = CefRect::from_array(dirty_rects_count as usize, dirty_rects);

        client.internal.on_accelerated_paint(&browser, element_type, &dirty_rects, shared_handle);
    }

    extern "C" fn start_dragging(
        _client: *mut cef_render_handler_t,
        _browser: *mut cef_browser_t,
        _drag_data: *mut cef_drag_data_t,
        _allowed_ops: cef_drag_operations_mask_t,
        _x: ::std::os::raw::c_int,
        _y: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        // TODO
        0
    }

    extern "C" fn update_drag_cursor(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        operation: cef_drag_operations_mask_t,
    ) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);

        client.internal.update_drag_cursor(&browser, operation);
    }

    extern "C" fn on_scroll_offset_changed(client: *mut cef_render_handler_t, browser: *mut cef_browser_t, x: f64, y: f64) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);

        client.internal.on_scroll_offset_changed(&browser, x, y);
    }

    extern "C" fn on_ime_composition_range_changed(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        selected_range: *const cef_range_t,
        character_bounds_count: u64,
        character_bounds: *const cef_rect_t,
    ) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let selected_range = CefRange::from_ptr(selected_range);
        let character_bounds = CefRect::from_array(character_bounds_count as usize, character_bounds);

        client.internal.on_ime_composition_range_changed(&browser, selected_range, &character_bounds);
    }

    extern "C" fn on_text_selection_changed(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        selected_text: *const cef_string_t,
        selected_range: *const cef_range_t,
    ) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let selected_text = unsafe { CefString::from_cef(selected_text) };
        let selected_range = CefRange::from_ptr(selected_range);

        client.internal.on_text_selection_changed(&browser, selected_text.to_string(), selected_range);
    }

    extern "C" fn on_virtual_keyboard_requested(
        client: *mut cef_render_handler_t,
        browser: *mut cef_browser_t,
        input_mode: cef_text_input_mode_t,
    ) {
        let client = Self::from_ptr(client);
        let browser = Browser::from(browser, false);
        let input_mode = unsafe { std::mem::transmute(input_mode) };
        client.internal.on_virtual_keyboard_requested(&browser, input_mode);
    }
}
impl<T: RenderHandler> ToCef<cef_render_handler_t> for Arc<T> {
    fn to_cef(&self) -> *mut cef_render_handler_t {
        wrap_ptr(|base| RenderHandlerWrapper {
            _base: cef_render_handler_t {
                base,
                get_accessibility_handler: Some(RenderHandlerWrapper::<T>::get_accessibility_handler),
                get_root_screen_rect: Some(RenderHandlerWrapper::<T>::get_root_screen_rect),
                get_view_rect: Some(RenderHandlerWrapper::<T>::get_view_rect),
                get_screen_point: Some(RenderHandlerWrapper::<T>::get_screen_point),
                get_screen_info: Some(RenderHandlerWrapper::<T>::get_screen_info),
                on_popup_show: Some(RenderHandlerWrapper::<T>::on_popup_show),
                on_popup_size: Some(RenderHandlerWrapper::<T>::on_popup_size),
                on_paint: Some(RenderHandlerWrapper::<T>::on_paint),
                on_accelerated_paint: Some(RenderHandlerWrapper::<T>::on_accelerated_paint),
                start_dragging: Some(RenderHandlerWrapper::<T>::start_dragging),
                update_drag_cursor: Some(RenderHandlerWrapper::<T>::update_drag_cursor),
                on_scroll_offset_changed: Some(RenderHandlerWrapper::<T>::on_scroll_offset_changed),
                on_ime_composition_range_changed: Some(RenderHandlerWrapper::<T>::on_ime_composition_range_changed),
                on_text_selection_changed: Some(RenderHandlerWrapper::<T>::on_text_selection_changed),
                on_virtual_keyboard_requested: Some(RenderHandlerWrapper::<T>::on_virtual_keyboard_requested),
            },
            internal: self.clone(),
        })
    }
}
