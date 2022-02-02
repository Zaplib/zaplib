use std::ptr::null_mut;

use crate::ptr::RefCounterGuard;
use crate::types::string::CefString;
use crate::{window_handle_default, Browser, CefWindowHandle, PaintElementType};
use zaplib_cef_sys::{cef_browser_host_t, cef_file_dialog_mode_t};

pub type FileDialogMode = cef_file_dialog_mode_t;

pub struct BrowserHost {
    ptr: RefCounterGuard<cef_browser_host_t>,
}
impl BrowserHost {
    pub(crate) fn from(ptr: *mut cef_browser_host_t, track_ref: bool) -> BrowserHost {
        unsafe { BrowserHost { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub fn get_browser(&self) -> Option<Browser> {
        if let Some(func) = self.ptr.as_ref().get_browser {
            Some(Browser::from(unsafe { func(self.ptr.get()) }, true))
        } else {
            None
        }
    }

    pub fn close_browser(&self, force_close: bool) {
        if let Some(func) = self.ptr.as_ref().close_browser {
            unsafe { func(self.ptr.get(), force_close as i32) }
        }
    }

    pub fn try_close_browser(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().try_close_browser {
            unsafe { func(self.ptr.get()) != 0 }
        } else {
            false
        }
    }

    pub fn set_focus(&self, focus: bool) {
        if let Some(func) = self.ptr.as_ref().set_focus {
            unsafe { func(self.ptr.get(), focus as i32) }
        }
    }

    pub fn get_window_handle(&self) -> CefWindowHandle {
        if let Some(func) = self.ptr.as_ref().get_window_handle {
            unsafe { func(self.ptr.get()) }
        } else {
            window_handle_default()
        }
    }

    pub fn get_opener_window_handle(&self) -> CefWindowHandle {
        if let Some(func) = self.ptr.as_ref().get_opener_window_handle {
            unsafe { func(self.ptr.get()) }
        } else {
            window_handle_default()
        }
    }

    pub fn has_view(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().has_view {
            unsafe { func(self.ptr.get()) != 0 }
        } else {
            false
        }
    }

    //    pub fn get_client(&self) -> *mut cef_client_t {}

    //    pub fn get_request_context(&self) -> *mut cef_request_context_t {}

    pub fn get_zoom_level(&self) -> f64 {
        if let Some(func) = self.ptr.as_ref().get_zoom_level {
            unsafe { func(self.ptr.get()) }
        } else {
            0.0
        }
    }

    pub fn set_zoom_level(&self, zoom_level: f64) {
        if let Some(func) = self.ptr.as_ref().set_zoom_level {
            unsafe { func(self.ptr.get(), zoom_level) }
        }
    }

    //    pub fn run_file_dialog(
    //        &self,
    //        mode: FileDialogMode,
    //        title: &str,
    //        default_file_path: &str,
    //        accept_filters: cef_string_list_t,
    //        selected_accept_filter: i32,
    //        callback: *mut cef_run_file_dialog_callback_t,
    //    ) {
    //    }

    pub fn start_download(&self, url: &str) {
        if let Some(func) = self.ptr.as_ref().start_download {
            let url = CefString::from_str(url);
            unsafe { func(self.ptr.get(), &url.into_cef()) }
        }
    }

    //    pub fn download_image(
    //        &self,
    //        image_url: *const cef_string_t,
    //        is_favicon: ::std::os::raw::c_int,
    //        max_image_size: u32,
    //        bypass_cache: ::std::os::raw::c_int,
    //        callback: *mut cef_download_image_callback_t,
    //    ) {
    //    }

    pub fn print(&self) {
        if let Some(func) = self.ptr.as_ref().print {
            unsafe { func(self.ptr.get()) }
        }
    }

    //    pub fn print_to_pdf(
    //        &self,
    //        path: *const cef_string_t,
    //        settings: *const cef_pdf_print_settings_t,
    //        callback: *mut cef_pdf_print_callback_t,
    //    ) {
    //    }

    pub fn find(&self, identifier: i32, search_text: &str, forward: bool, match_case: bool, find_next: bool) {
        if let Some(func) = self.ptr.as_ref().find {
            let search_text = CefString::from_str(search_text);
            unsafe {
                func(self.ptr.get(), identifier, &search_text.into_cef(), forward as i32, match_case as i32, find_next as i32)
            }
        }
    }

    pub fn stop_finding(&self, clear_selection: bool) {
        if let Some(func) = self.ptr.as_ref().stop_finding {
            unsafe { func(self.ptr.get(), clear_selection as i32) }
        }
    }

    pub fn show_dev_tools(
        &self,
        // windowInfo: *const cef_window_info_t,
        // client: *mut cef_client_t,
        // settings: *const cef_browser_settings_t,
        // inspect_element_at: *const cef_point_t,
    ) {
        if let Some(func) = self.ptr.as_ref().show_dev_tools {
            unsafe { func(self.ptr.get(), null_mut(), null_mut(), null_mut(), null_mut()) }
        }
    }

    pub fn close_dev_tools(&self) {
        if let Some(func) = self.ptr.as_ref().close_dev_tools {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn has_dev_tools(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().has_dev_tools {
            unsafe { func(self.ptr.get()) != 0 }
        } else {
            false
        }
    }

    pub fn replace_misspelling(&self, word: &str) {
        if let Some(func) = self.ptr.as_ref().replace_misspelling {
            let word = CefString::from_str(word);
            unsafe { func(self.ptr.get(), &word.into_cef()) }
        }
    }

    pub fn add_word_to_dictionary(&self, word: &str) {
        if let Some(func) = self.ptr.as_ref().add_word_to_dictionary {
            let word = CefString::from_str(word);
            unsafe { func(self.ptr.get(), &word.into_cef()) }
        }
    }

    pub fn is_window_rendering_disabled(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().is_window_rendering_disabled {
            unsafe { func(self.ptr.get()) != 0 }
        } else {
            false
        }
    }

    pub fn was_resized(&self) {
        if let Some(func) = self.ptr.as_ref().was_resized {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn was_hidden(&self, hidden: bool) {
        if let Some(func) = self.ptr.as_ref().was_hidden {
            unsafe { func(self.ptr.get(), hidden as i32) }
        }
    }

    pub fn notify_screen_info_changed(&self) {
        if let Some(func) = self.ptr.as_ref().notify_screen_info_changed {
            unsafe { func(self.ptr.get()) }
        }
    }

    pub fn invalidate(&self, type_: PaintElementType) {
        if let Some(func) = self.ptr.as_ref().invalidate {
            unsafe { func(self.ptr.get(), type_) }
        }
    }

    pub fn send_external_begin_frame(&self) {
        if let Some(func) = self.ptr.as_ref().send_external_begin_frame {
            unsafe { func(self.ptr.get()) }
        }
    }

    //    pub fn send_key_event(&self, event: *const cef_key_event_t) {}
    //
    //    pub fn send_mouse_click_event(
    //        &self,
    //        event: *const cef_mouse_event_t,
    //        type_: cef_mouse_button_type_t,
    //        mouseUp: ::std::os::raw::c_int,
    //        clickCount: ::std::os::raw::c_int,
    //    ) {
    //    }
    //
    //    pub fn send_mouse_move_event(
    //        &self,
    //        event: *const cef_mouse_event_t,
    //        mouseLeave: ::std::os::raw::c_int,
    //    ) {
    //    }
    //
    //    pub fn send_mouse_wheel_event(
    //        &self,
    //        event: *const cef_mouse_event_t,
    //        deltaX: ::std::os::raw::c_int,
    //        deltaY: ::std::os::raw::c_int,
    //    ) {
    //    }
    //
    //    pub fn send_touch_event(&self, event: *const cef_touch_event_t) {}
    //
    //    pub fn send_focus_event(&self, setFocus: ::std::os::raw::c_int) {}
    //
    //    pub fn send_capture_lost_event(&self) {}
    //
    //    pub fn notify_move_or_resize_started(&self) {}
    //
    //    pub fn get_windowless_frame_rate(&self) -> ::std::os::raw::c_int {}
    //
    //    pub fn set_windowless_frame_rate(&self, frame_rate: ::std::os::raw::c_int) {}
    //
    //    pub fn ime_set_composition(
    //        &self,
    //        text: *const cef_string_t,
    //        underlinesCount: usize,
    //        underlines: *const cef_composition_underline_t,
    //        replacement_range: *const cef_range_t,
    //        selection_range: *const cef_range_t,
    //    ) {
    //    }
    //
    //    pub fn ime_commit_text(
    //        &self,
    //        text: *const cef_string_t,
    //        replacement_range: *const cef_range_t,
    //        relative_cursor_pos: ::std::os::raw::c_int,
    //    ) {
    //    }
    //
    //    pub fn ime_finish_composing_text(&self, keep_selection: ::std::os::raw::c_int) {}
    //
    //    pub fn ime_cancel_composition(&self) {}
    //
    //    pub fn drag_target_drag_enter(
    //        &self,
    //        drag_data: *mut cef_drag_data_t,
    //        event: *const cef_mouse_event_t,
    //        allowed_ops: cef_drag_operations_mask_t,
    //    ) {
    //    }
    //
    //    pub fn drag_target_drag_over(
    //        &self,
    //        event: *const cef_mouse_event_t,
    //        allowed_ops: cef_drag_operations_mask_t,
    //    ) {
    //    }
    //
    //    pub fn drag_target_drag_leave(&self) {}
    //
    //    pub fn drag_target_drop(&self, event: *const cef_mouse_event_t) {}
    //
    //    pub fn drag_source_ended_at(
    //        &self,
    //        x: ::std::os::raw::c_int,
    //        y: ::std::os::raw::c_int,
    //        op: cef_drag_operations_mask_t,
    //    ) {
    //    }
    //
    //    pub fn drag_source_system_drag_ended(&self) {}
    //
    //    pub fn get_visible_navigation_entry(&self) -> *mut cef_navigation_entry_t {}
    //
    //    pub fn set_accessibility_state(&self, accessibility_state: cef_state_t) {}
    //
    //    pub fn set_auto_resize_enabled(
    //        &self,
    //        enabled: ::std::os::raw::c_int,
    //        min_size: *const cef_size_t,
    //        max_size: *const cef_size_t,
    //    ) {
    //    }
    //
    //    pub fn get_extension(&self) -> *mut cef_extension_t {}
    //
    //    pub fn is_background_host(&self) -> ::std::os::raw::c_int {}
    //
    //    pub fn set_audio_muted(&self, mute: ::std::os::raw::c_int) {}
    //
    //    pub fn is_audio_muted(&self) -> ::std::os::raw::c_int {}
}
