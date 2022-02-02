//! Defining native menus.

use crate::*;

impl Cx {
    pub const COMMAND_QUIT: CommandId = location_hash!();
    pub const COMMAND_UNDO: CommandId = location_hash!();
    pub const COMMAND_REDO: CommandId = location_hash!();
    pub const COMMAND_CUT: CommandId = location_hash!();
    pub const COMMAND_COPY: CommandId = location_hash!();
    pub const COMMAND_PASTE: CommandId = location_hash!();
    pub const COMMAND_ZOOM_IN: CommandId = location_hash!();
    pub const COMMAND_ZOOM_OUT: CommandId = location_hash!();
    pub const COMMAND_MINIMIZE: CommandId = location_hash!();
    pub const COMMAND_ZOOM: CommandId = location_hash!();
    pub const COMMAND_SELECT_ALL: CommandId = location_hash!();

    pub fn command_default_keymap(&mut self) {
        Cx::COMMAND_QUIT.set_key(self, KeyCode::KeyQ);
        Cx::COMMAND_UNDO.set_key(self, KeyCode::KeyZ);
        Cx::COMMAND_REDO.set_key_shift(self, KeyCode::KeyZ);
        Cx::COMMAND_CUT.set_key(self, KeyCode::KeyX);
        Cx::COMMAND_COPY.set_key(self, KeyCode::KeyC);
        Cx::COMMAND_PASTE.set_key(self, KeyCode::KeyV);
        Cx::COMMAND_SELECT_ALL.set_key(self, KeyCode::KeyA);
        Cx::COMMAND_ZOOM_OUT.set_key(self, KeyCode::Minus);
        Cx::COMMAND_ZOOM_IN.set_key(self, KeyCode::Equals);
        Cx::COMMAND_MINIMIZE.set_key(self, KeyCode::KeyM);
    }
}

/// An alias over LocationHash so we have a semantic type
/// but can change the underlying implementation whenever.
/// See [`Event::Command`].
pub type CommandId = LocationHash;

impl CommandId {
    pub fn set_enabled(&self, cx: &mut Cx, enabled: bool) {
        let mut s = if let Some(s) = cx.command_settings.get(self) { *s } else { CxCommandSetting::default() };
        s.enabled = enabled;
        cx.command_settings.insert(*self, s);
    }

    pub fn set_key(&self, cx: &mut Cx, key_code: KeyCode) {
        let mut s = if let Some(s) = cx.command_settings.get(self) { *s } else { CxCommandSetting::default() };
        s.shift = false;
        s.key_code = key_code;
        cx.command_settings.insert(*self, s);
    }

    pub fn set_key_shift(&self, cx: &mut Cx, key_code: KeyCode) {
        let mut s = if let Some(s) = cx.command_settings.get(self) { *s } else { CxCommandSetting::default() };
        s.shift = true;
        s.key_code = key_code;
        cx.command_settings.insert(*self, s);
    }
}

/// Represents a single menu, as well as all menus (recursively).
#[derive(PartialEq, Clone)]
pub enum Menu {
    Main { items: Vec<Menu> },
    Item { name: String, command: CommandId },
    Sub { name: String, items: Vec<Menu> },
    Line,
}

impl Menu {
    pub fn main(items: Vec<Menu>) -> Menu {
        Menu::Main { items }
    }

    pub fn sub(name: &str, items: Vec<Menu>) -> Menu {
        Menu::Sub { name: name.to_string(), items }
    }

    pub fn line() -> Menu {
        Menu::Line
    }

    pub fn item(name: &str, command: CommandId) -> Menu {
        Menu::Item { name: name.to_string(), command }
    }
}
