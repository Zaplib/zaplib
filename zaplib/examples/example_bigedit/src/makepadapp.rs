//use syn::Type;
use crate::buildmanager::*;
use crate::makepadstorage::*;
use crate::makepadwindow::*;
use zaplib::*;

pub struct MakepadApp {
    menu: Menu,
    makepad_storage: MakepadStorage,
    build_manager: BuildManager,
    makepad_windows: Vec<ZapWindow>,
}

const COMMAND_ABOUT_MAKEPAD: CommandId = location_hash!();
const COMMAND_PREFERENCES: CommandId = location_hash!();
const COMMAND_NEW_FILE: CommandId = location_hash!();
const COMMAND_NEW_WINDOW: CommandId = location_hash!();
const COMMAND_ADD_FOLDER_TO_BUILDER: CommandId = location_hash!();
const COMMAND_SAVE_AS: CommandId = location_hash!();
const COMMAND_RENAME: CommandId = location_hash!();
const COMMAND_CLOSE_EDITOR: CommandId = location_hash!();
const COMMAND_REMOVE_FOLDER_FROM_BUILDER: CommandId = location_hash!();
const COMMAND_CLOSE_WINDOW: CommandId = location_hash!();
const COMMAND_FIND: CommandId = location_hash!();
const COMMAND_REPLACE: CommandId = location_hash!();
const COMMAND_FIND_IN_FILES: CommandId = location_hash!();
const COMMAND_REPLACE_IN_FILES: CommandId = location_hash!();
const COMMAND_TOGGLE_LINE_COMMENT: CommandId = location_hash!();
const COMMAND_TOGGLE_BLOCK_COMMENT: CommandId = location_hash!();
const COMMAND_START_PROGRAM: CommandId = location_hash!();
const COMMAND_STOP_PROGRAM: CommandId = location_hash!();
const COMMAND_BRING_ALL_TO_FRONT: CommandId = location_hash!();

impl MakepadApp {
    pub fn new(cx: &mut Cx) -> Self {
        // set up the keyboard map
        COMMAND_PREFERENCES.set_key(cx, KeyCode::Comma);
        COMMAND_NEW_FILE.set_key(cx, KeyCode::KeyN);
        COMMAND_NEW_WINDOW.set_key_shift(cx, KeyCode::KeyN);
        COMMAND_SAVE_AS.set_key_shift(cx, KeyCode::KeyS);
        COMMAND_CLOSE_EDITOR.set_key(cx, KeyCode::KeyW);
        COMMAND_CLOSE_WINDOW.set_key_shift(cx, KeyCode::KeyW);

        cx.command_default_keymap();

        Self {
            menu: Menu::main(vec![
                Menu::sub(
                    "Bigedit",
                    vec![
                        Menu::item("About Bigedit", COMMAND_ABOUT_MAKEPAD),
                        Menu::line(),
                        Menu::item("Preferences", COMMAND_PREFERENCES),
                        Menu::line(),
                        Menu::item("Quit Bigedit", Cx::COMMAND_QUIT),
                    ],
                ),
                Menu::sub(
                    "File",
                    vec![
                        Menu::item("New File", COMMAND_NEW_FILE),
                        Menu::item("New Window", COMMAND_NEW_WINDOW),
                        Menu::line(),
                        Menu::item("Add Folder to Builder", COMMAND_ADD_FOLDER_TO_BUILDER),
                        Menu::line(),
                        Menu::item("Save As", COMMAND_SAVE_AS),
                        Menu::line(),
                        Menu::item("Rename", COMMAND_RENAME),
                        Menu::line(),
                        Menu::item("Close Editor", COMMAND_CLOSE_EDITOR),
                        Menu::item("Remove Folder from Builder", COMMAND_REMOVE_FOLDER_FROM_BUILDER),
                        Menu::item("Close Window", COMMAND_CLOSE_WINDOW),
                    ],
                ),
                Menu::sub(
                    "Edit",
                    vec![
                        Menu::item("Undo", Cx::COMMAND_UNDO),
                        Menu::item("Redo", Cx::COMMAND_REDO),
                        Menu::line(),
                        Menu::item("Cut", Cx::COMMAND_CUT),
                        Menu::item("Copy", Cx::COMMAND_COPY),
                        Menu::item("Paste", Cx::COMMAND_PASTE),
                        Menu::line(),
                        Menu::item("Find", COMMAND_FIND),
                        Menu::item("Replace", COMMAND_REPLACE),
                        Menu::line(),
                        Menu::item("Find in Files", COMMAND_FIND_IN_FILES),
                        Menu::item("Replace in Files", COMMAND_REPLACE_IN_FILES),
                        Menu::line(),
                        Menu::item("Toggle Line Comment", COMMAND_TOGGLE_LINE_COMMENT),
                        Menu::item("Toggle Block Comment", COMMAND_TOGGLE_BLOCK_COMMENT),
                    ],
                ),
                Menu::sub("Selection", vec![Menu::item("Select All", Cx::COMMAND_SELECT_ALL)]),
                Menu::sub("View", vec![Menu::item("Zoom In", Cx::COMMAND_ZOOM_IN), Menu::item("Zoom Out", Cx::COMMAND_ZOOM_OUT)]),
                Menu::sub(
                    "Run",
                    vec![Menu::item("Start Program", COMMAND_START_PROGRAM), Menu::item("Stop Program", COMMAND_STOP_PROGRAM)],
                ),
                Menu::sub(
                    "Window",
                    vec![
                        Menu::item("Minimize", Cx::COMMAND_MINIMIZE),
                        Menu::item("Zoom", Cx::COMMAND_ZOOM),
                        Menu::line(),
                        Menu::item("Bring All to Front", COMMAND_BRING_ALL_TO_FRONT),
                    ],
                ),
                Menu::sub("Help", vec![Menu::item("About Bigedit", COMMAND_ABOUT_MAKEPAD)]),
            ]),
            makepad_windows: vec![],
            build_manager: BuildManager::new(cx),
            makepad_storage: MakepadStorage::new(cx),
        }
    }

    fn default_layout(&mut self, cx: &mut Cx) {
        self.makepad_windows = vec![ZapWindow::new(cx)];
        cx.request_draw();
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Construct => {
                #[cfg(not(target_arch = "wasm32"))]
                self.makepad_storage.init(cx);
                self.default_layout(cx);
            }
            Event::KeyDown(ke) => match ke.key_code {
                KeyCode::KeyD => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        //let size = self.build_manager.search_index.calc_total_size();
                        //println!("Text Index size {}", size);
                    }
                }
                KeyCode::KeyR => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        self.makepad_storage.reload_builders();
                    }
                }
                KeyCode::Key0 => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        cx.reset_font_atlas_and_redraw();
                        println!("IMPLEMENT SCALE");
                        //self.storage.settings.style_options.scale = 1.0;
                        //self.reload_style(cx);
                        //cx.reset_font_atlas_and_redraw();
                        //self.storage.save_settings(cx);
                    }
                }
                KeyCode::Equals => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        cx.reset_font_atlas_and_redraw();
                        println!("IMPLEMENT SCALE");
                        //let scale = self.storage.settings.style_options.scale * 1.1;
                        // self.storage.settings.style_options.scale = scale.min(3.0).max(0.3);
                        //self.reload_style(cx);
                        //cx.reset_font_atlas_and_redraw();
                        //self.storage.save_settings(cx);
                    }
                }
                KeyCode::Minus => {
                    if ke.modifiers.logo || ke.modifiers.control {
                        cx.reset_font_atlas_and_redraw();
                        println!("IMPLEMENT SCALE");
                        //let scale = self.storage.settings.style_options.scale / 1.1;
                        //self.storage.settings.style_options.scale = scale.min(3.0).max(0.3);
                        //self.reload_style(cx);
                        //cx.reset_font_atlas_and_redraw();
                        //self.storage.save_settings(cx);
                    }
                }
                _ => (),
            },
            Event::Signal(se) => {
                // process network messages for hub_ui
                if let Some(hub_ui) = &mut self.makepad_storage.hub_ui {
                    if let Some(_) = se.signals.get(&self.makepad_storage.hub_ui_message) {
                        if let Some(mut msgs) = hub_ui.get_messages() {
                            for htc in msgs.drain(..) {
                                self.makepad_storage.handle_hub_msg(cx, &htc, &mut self.makepad_windows);
                                self.build_manager.handle_hub_msg(cx, &mut self.makepad_storage, &htc);
                            }
                            return;
                        }
                    }
                }
                if let Some(_statusses) = se.signals.get(&self.makepad_storage.settings_changed) {
                    if self.makepad_storage.settings_old.builders != self.makepad_storage.settings.builders {
                        self.makepad_storage.reload_builders();
                    }
                    /*
                    if self.storage.settings_old.style_options != self.storage.settings.style_options {
                        self.reload_style(cx);
                        cx.reset_font_atlas_and_redraw();
                    }*/
                    if self.makepad_storage.settings_old.builds != self.makepad_storage.settings.builds {
                        // self.build_manager.restart_build(cx, &mut self.makepad_storage);
                    }
                }
            }
            _ => (),
        }
        for (window_index, window) in self.makepad_windows.iter_mut().enumerate() {
            window.handle(cx, event, window_index, &mut self.makepad_storage, &mut self.build_manager);
            // break;
        }
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        //return;
        for (window_index, window) in self.makepad_windows.iter_mut().enumerate() {
            window.draw(cx, &self.menu, window_index, &mut self.makepad_storage, &mut self.build_manager);
            // break;
        }
    }
}
