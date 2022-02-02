//use syn::Type;
use zaplib::*;
use zaplib_components::*;

use crate::jseditor::*;
use crate::makepadstorage::*;
use crate::plaineditor::*;
use crate::rusteditor::*;
use crate::searchindex::*;

use std::collections::HashMap;

pub struct FileEditors {
    pub editors: HashMap<u64, FileEditor>, //text_editor: TextEditor
}

pub enum FileEditor {
    Rust(RustEditor),
    JS(JSEditor),
    Plain(PlainEditor), //Text(TextEditor)
}

impl FileEditor {
    pub fn handle(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        mtb: &mut MakepadTextBuffer,
        search_index: Option<&mut SearchIndex>,
    ) -> TextEditorEvent {
        match self {
            FileEditor::Rust(re) => re.handle(cx, event, mtb, search_index),
            FileEditor::JS(re) => re.handle(cx, event, mtb),
            FileEditor::Plain(re) => re.handle(cx, event, mtb),
        }
    }

    pub fn set_last_cursor(&mut self, cx: &mut Cx, cursor: (usize, usize), at_top: bool) {
        match self {
            FileEditor::Rust(re) => re.text_editor.set_last_cursor(cx, cursor, at_top),
            FileEditor::JS(re) => re.text_editor.set_last_cursor(cx, cursor, at_top),
            FileEditor::Plain(re) => re.text_editor.set_last_cursor(cx, cursor, at_top),
        }
    }

    pub fn set_key_focus(&mut self, cx: &mut Cx) {
        match self {
            FileEditor::Rust(re) => re.text_editor.set_key_focus(cx),
            FileEditor::JS(re) => re.text_editor.set_key_focus(cx),
            FileEditor::Plain(re) => re.text_editor.set_key_focus(cx),
        }
    }

    pub fn set_scroll_pos_on_load(&mut self, pos: Vec2) {
        match self {
            FileEditor::Rust(re) => re.text_editor._scroll_pos_on_load = Some(pos),
            FileEditor::JS(re) => re.text_editor._scroll_pos_on_load = Some(pos),
            FileEditor::Plain(re) => re.text_editor._scroll_pos_on_load = Some(pos),
        }
    }

    pub fn draw(&mut self, cx: &mut Cx, mtb: &mut MakepadTextBuffer, search_index: &mut SearchIndex) {
        match self {
            FileEditor::Rust(re) => re.draw(cx, mtb, Some(search_index)),
            FileEditor::JS(re) => re.draw(cx, mtb, Some(search_index)),
            FileEditor::Plain(re) => re.draw(cx, mtb, Some(search_index)),
        }
    }
}

impl FileEditors {
    pub fn does_path_match_editor_type(&mut self, path: &str, editor_id: u64) -> bool {
        if !self.editors.contains_key(&editor_id) {
            return false;
        }
        match self.editors.get(&editor_id).unwrap() {
            FileEditor::Rust(_) => path.ends_with(".rs") || path.ends_with(".toml") || path.ends_with(".ron"),
            FileEditor::JS(_) => path.ends_with(".js") || path.ends_with(".html"),
            FileEditor::Plain(_) => {
                !(path.ends_with(".rs")
                    || path.ends_with(".toml")
                    || path.ends_with(".ron")
                    || path.ends_with(".js")
                    || path.ends_with(".html"))
            }
        }
    }

    pub fn get_file_editor_for_path(&mut self, path: &str, editor_id: u64) -> (&mut FileEditor, bool) {
        // check which file extension we have to spawn a new editor
        let is_new = !self.editors.contains_key(&editor_id);
        if is_new {
            let editor = if path.ends_with(".rs") || path.ends_with(".toml") || path.ends_with(".ron") {
                FileEditor::Rust(RustEditor::new())
            } else if path.ends_with(".js") || path.ends_with(".html") {
                FileEditor::JS(JSEditor::new())
            } else {
                FileEditor::Plain(PlainEditor::new())
            };
            self.editors.insert(editor_id, editor);
        }
        (self.editors.get_mut(&editor_id).unwrap(), is_new)
    }

    pub fn highest_file_editor_id(&self) -> u64 {
        let mut max_id = 0;
        for (id, _) in &self.editors {
            if *id > max_id {
                max_id = *id;
            }
        }
        max_id
    }
}

pub fn path_file_name(path: &str) -> String {
    if let Some(pos) = path.rfind('/') {
        path[pos + 1..path.len()].to_string()
    } else {
        path.to_string()
    }
}
