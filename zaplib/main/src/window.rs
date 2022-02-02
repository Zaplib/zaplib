//! Managing individual windows on native platforms.

use crate::*;

#[cfg(feature = "cef-server")]
use crate::cef_browser::GetResourceUrlCallback;

/// A pointer to a [`CxWindow`] (indexed in [`Cx::windows`] using [`Window::window_id`]),
#[derive(Clone, Default)]
pub struct Window {
    /// The id referring to [`CxWindow`], once instantiated. It's an index in [`Cx::windows`].
    pub window_id: Option<usize>,

    /// The inner dimensions of the native window when it's created for the first time.
    pub create_inner_size: Option<Vec2>,

    /// The position on the screen of the native window when it's created for the first time.
    pub create_position: Option<Vec2>,

    /// The title that the window will get once created.
    pub create_title: String,

    /// Set to true if the window should as a drop target for a AppOpenFiles events.
    ///
    /// TODO(JP): only works on the wasm32 and mac targets for now.
    pub create_add_drop_target_for_app_open_files: bool,

    /// If set, will run CEF with the given URL.
    ///
    /// TODO(JP): only works on the mac target for now.
    #[cfg(feature = "cef")]
    pub create_cef_url: Option<String>,

    #[cfg(feature = "cef-server")]
    pub get_resource_url_callback: Option<GetResourceUrlCallback>,
}

impl Window {
    pub fn begin_window(&mut self, cx: &mut Cx) {
        // if we are not at ground level for viewports,
        if self.window_id.is_none() {
            let new_window = CxWindow {
                window_state: CxWindowState::Create {
                    title: self.create_title.clone(),
                    inner_size: if let Some(inner_size) = self.create_inner_size {
                        inner_size
                    } else {
                        cx.get_default_window_size()
                    },
                    position: self.create_position,
                    add_drop_target_for_app_open_files: self.create_add_drop_target_for_app_open_files,

                    #[cfg(feature = "cef")]
                    cef_url: self.create_cef_url.clone(),

                    #[cfg(feature = "cef-server")]
                    get_resource_url_callback: self.get_resource_url_callback,
                },
                ..Default::default()
            };
            let window_id;
            if !cx.windows_free.is_empty() {
                window_id = cx.windows_free.pop().unwrap();
                cx.windows[window_id] = new_window
            } else {
                window_id = cx.windows.len();
                cx.windows.push(new_window);
            }
            self.window_id = Some(window_id);
        }
        let window_id = self.window_id.unwrap();
        cx.windows[window_id].main_pass_id = None;
        cx.window_stack.push(window_id);
    }

    pub fn get_inner_size(&mut self, cx: &mut Cx) -> Vec2 {
        if let Some(window_id) = self.window_id {
            return cx.windows[window_id].get_inner_size();
        }
        Vec2::default()
    }

    pub fn get_position(&mut self, cx: &mut Cx) -> Option<Vec2> {
        if let Some(window_id) = self.window_id {
            return cx.windows[window_id].get_position();
        }
        None
    }

    pub fn set_position(&mut self, cx: &mut Cx, pos: Vec2) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_set_position = Some(pos)
        }
    }

    pub fn handle_window(&mut self, _cx: &mut Cx, _event: &mut Event) -> bool {
        false
    }

    pub fn end_window(&mut self, cx: &mut Cx) -> Area {
        cx.window_stack.pop();
        Area::Empty
    }

    pub fn minimize_window(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_command = CxWindowCmd::Minimize;
        }
    }

    pub fn maximize_window(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_command = CxWindowCmd::Maximize;
        }
    }

    pub fn fullscreen_window(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_command = CxWindowCmd::FullScreen;
        }
    }

    pub fn normal_window(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_command = CxWindowCmd::NormalScreen;
        }
    }

    pub fn can_fullscreen(&mut self, cx: &mut Cx) -> bool {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_geom.can_fullscreen
        } else {
            false
        }
    }

    pub fn xr_can_present(&mut self, cx: &mut Cx) -> bool {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_geom.xr_can_present
        } else {
            false
        }
    }

    pub fn is_fullscreen(&mut self, cx: &mut Cx) -> bool {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_geom.is_fullscreen
        } else {
            false
        }
    }

    pub fn xr_is_presenting(&mut self, cx: &mut Cx) -> bool {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_geom.xr_is_presenting
        } else {
            false
        }
    }

    pub fn xr_start_presenting(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_command = CxWindowCmd::XrStartPresenting;
        }
    }

    pub fn xr_stop_presenting(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_command = CxWindowCmd::XrStopPresenting;
        }
    }

    pub fn is_topmost(&mut self, cx: &mut Cx) -> bool {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_geom.is_topmost
        } else {
            false
        }
    }

    pub fn set_topmost(&mut self, cx: &mut Cx, topmost: bool) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_topmost = Some(topmost);
        }
    }

    pub fn restore_window(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_command = CxWindowCmd::Restore;
        }
    }

    pub fn close_window(&mut self, cx: &mut Cx) {
        if let Some(window_id) = self.window_id {
            cx.windows[window_id].window_state = CxWindowState::Close;
        }
    }
}

/// Information on the geometry and capabilities of a particular native window.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WindowGeom {
    pub dpi_factor: f32,
    pub can_fullscreen: bool,
    pub xr_can_present: bool,
    pub xr_is_presenting: bool,
    pub is_fullscreen: bool,
    pub is_topmost: bool,
    pub position: Vec2,
    pub inner_size: Vec2,
    pub outer_size: Vec2,
}

#[derive(Clone)]
pub(crate) enum CxWindowState {
    Create {
        title: String,
        inner_size: Vec2,
        position: Option<Vec2>,

        #[allow(dead_code)] // Not supported in all platforms yet.
        add_drop_target_for_app_open_files: bool,

        #[allow(dead_code)] // Not supported in all platforms yet.
        #[cfg(feature = "cef")]
        cef_url: Option<String>,

        #[cfg(feature = "cef-server")]
        get_resource_url_callback: Option<GetResourceUrlCallback>,
    },
    Created,
    Close,
    Closed,
}

#[derive(Clone)]
pub(crate) enum CxWindowCmd {
    None,
    Restore,
    Maximize,
    Minimize,
    XrStartPresenting,
    XrStopPresenting,
    FullScreen,
    NormalScreen,
}

impl Default for CxWindowCmd {
    fn default() -> Self {
        CxWindowCmd::None
    }
}

impl Default for CxWindowState {
    fn default() -> Self {
        CxWindowState::Closed
    }
}

#[derive(Clone, Default)]
pub(crate) struct CxWindow {
    pub(crate) window_state: CxWindowState,
    pub(crate) window_command: CxWindowCmd,
    pub(crate) window_set_position: Option<Vec2>,
    pub(crate) window_topmost: Option<bool>,
    pub(crate) window_geom: WindowGeom,
    pub(crate) main_pass_id: Option<usize>,
}

impl CxWindow {
    pub(crate) fn get_inner_size(&mut self) -> Vec2 {
        match &self.window_state {
            CxWindowState::Create { inner_size, .. } => *inner_size,
            CxWindowState::Created => self.window_geom.inner_size,
            _ => Vec2::default(),
        }
    }

    pub(crate) fn get_position(&mut self) -> Option<Vec2> {
        match &self.window_state {
            CxWindowState::Create { position, .. } => *position,
            CxWindowState::Created => Some(self.window_geom.position),
            _ => None,
        }
    }
}
