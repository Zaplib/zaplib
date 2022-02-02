use crate::makepadstorage::*;
use zaplib::*;
use zaplib_components::*;

pub struct Keyboard {
    view: ScrollView,
    modifiers: KeyModifiers,
    key_down: Option<KeyCode>,
    key_up: Option<KeyCode>,
    buttons: Vec<Button>,
}

#[derive(Clone)]
pub enum KeyboardEvent {
    None,
}

#[derive(Clone, PartialEq, PartialOrd, Hash, Ord)]
enum KeyType {
    Control,
    Alt,
    Shift,
}
impl Eq for KeyType {}

impl KeyType {
    fn name(&self) -> String {
        match self {
            KeyType::Control => "Control".to_string(),
            KeyType::Alt => "Alternate".to_string(),
            KeyType::Shift => "Shift".to_string(),
        }
    }
}

const KEYS: &[KeyType] = &[KeyType::Alt, KeyType::Control, KeyType::Shift];

impl Keyboard {
    pub fn new() -> Self {
        Self {
            view: ScrollView::new_standard_vh(),
            buttons: KEYS.iter().map(|_| Button::default()).collect(),
            modifiers: KeyModifiers::default(),
            key_down: None,
            key_up: None,
        }
    }

    fn send_textbuffers_update(&mut self, cx: &mut Cx, makepad_storage: &mut MakepadStorage) {
        // clear all files we missed
        for mtb in &mut makepad_storage.text_buffers {
            mtb.text_buffer.keyboard.modifiers = self.modifiers.clone();
            mtb.text_buffer.keyboard.key_down = self.key_down;
            mtb.text_buffer.keyboard.key_up = self.key_up;
            cx.send_signal(mtb.text_buffer.signal, TextBuffer::STATUS_KEYBOARD_UPDATE);
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event, makepad_storage: &mut MakepadStorage) -> KeyboardEvent {
        // do shit here
        if self.view.handle(cx, event) {}
        let mut update_textbuffers = false;
        for (index, btn) in self.buttons.iter_mut().enumerate() {
            match btn.handle(cx, event) {
                ButtonEvent::Down => {
                    match KEYS[index] {
                        KeyType::Control => {
                            self.modifiers.control = true;
                            self.key_up = None;
                            self.key_down = Some(KeyCode::Control);
                        }
                        KeyType::Alt => {
                            self.modifiers.alt = true;
                            self.key_up = None;
                            self.key_down = Some(KeyCode::Alt);
                        }
                        KeyType::Shift => {
                            self.modifiers.shift = true;
                            self.key_up = None;
                            self.key_down = Some(KeyCode::Shift);
                        }
                    }
                    update_textbuffers = true;
                }
                ButtonEvent::Up | ButtonEvent::Clicked => {
                    match KEYS[index] {
                        KeyType::Control => {
                            self.modifiers.control = false;
                            self.key_down = None;
                            self.key_up = Some(KeyCode::Control);
                        }
                        KeyType::Alt => {
                            self.modifiers.alt = false;
                            self.key_down = None;
                            self.key_up = Some(KeyCode::Alt);
                        }
                        KeyType::Shift => {
                            self.modifiers.shift = false;
                            self.key_down = None;
                            self.key_up = Some(KeyCode::Shift);
                        }
                    }
                    update_textbuffers = true;
                }
                _ => (),
            }
        }
        if update_textbuffers {
            self.send_textbuffers_update(cx, makepad_storage);
        }

        KeyboardEvent::None
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        self.view.begin_view(cx, LayoutSize::FILL);
        cx.begin_row(Width::Fill, Height::Fill);

        for (index, button) in self.buttons.iter_mut().enumerate() {
            button.draw(cx, &KEYS[index].name());
        }

        cx.end_row();
        self.view.end_view(cx);
    }
}
