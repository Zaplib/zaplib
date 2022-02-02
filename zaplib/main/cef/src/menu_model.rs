use zaplib_cef_sys::cef_menu_model_t;

use crate::ptr::RefCounterGuard;

#[derive(Clone)]
pub struct MenuModel {
    ptr: RefCounterGuard<cef_menu_model_t>,
}
impl MenuModel {
    pub(crate) fn from(ptr: *mut cef_menu_model_t, track_ref: bool) -> MenuModel {
        unsafe { MenuModel { ptr: RefCounterGuard::from(&mut (*ptr).base, ptr, track_ref) } }
    }

    pub fn clear(&self) -> bool {
        if let Some(func) = self.ptr.as_ref().clear {
            unsafe { func(self.ptr.get()) > 0 }
        } else {
            false
        }
    }
    // TODO: implement other methods
}
