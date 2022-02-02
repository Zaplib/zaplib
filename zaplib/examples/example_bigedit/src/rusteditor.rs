use crate::colorpicker::*;
use crate::makepadstorage::*;
use crate::mprstokenizer::*;
use crate::searchindex::*;
use zaplib::*;
use zaplib_components::*;

pub struct RustEditor {
    pub view: View,
    pub splitter: Splitter,
    pub text_editor: TextEditor,
    pub dummy_fold_caption: FoldCaption,
    pub dummy_color_picker: ColorPicker,
    pub dummy_color: Vec4,
}

impl RustEditor {
    pub fn new() -> Self {
        //tab.animator.default = tab.anim_default(cx);
        Self {
            view: View::default(),
            splitter: Splitter {
                pos: 125.0,
                align: SplitterAlign::Last,
                _hit_state_margin: Some(Padding { l: 3., t: 0., r: 7., b: 0. }),
                ..Splitter::default()
            },
            text_editor: TextEditor::default(),
            dummy_fold_caption: FoldCaption::default(),
            dummy_color_picker: ColorPicker::default(),
            dummy_color: vec4(0.5, 0.3, 0.2, 0.8),
        }
    }

    pub fn handle(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        mtb: &mut MakepadTextBuffer,
        search_index: Option<&mut SearchIndex>,
    ) -> TextEditorEvent {
        self.dummy_fold_caption.handle_fold_caption(cx, event);

        if let ColorPickerEvent::Change { hsva } = self.dummy_color_picker.handle(cx, event) {
            self.dummy_color = hsva;
            cx.request_draw();
        }

        if let SplitterEvent::Moving { .. } = self.splitter.handle(cx, event) {
            cx.request_draw();
        }
        let ce = self.text_editor.handle(cx, event, &mut mtb.text_buffer);
        if let TextEditorEvent::Change = ce {
            Self::update_token_chunks(mtb, search_index);
        }
        ce
    }

    pub fn draw(&mut self, cx: &mut Cx, mtb: &mut MakepadTextBuffer, search_index: Option<&mut SearchIndex>) {
        self.view.begin_view(cx, LayoutSize::FILL);
        cx.begin_row(Width::Fill, Height::Fill);

        self.splitter.begin_draw(cx);
        Self::update_token_chunks(mtb, search_index);
        self.text_editor.begin_text_editor(cx, &mtb.text_buffer, None);
        for (index, token_chunk) in mtb.text_buffer.token_chunks.iter_mut().enumerate() {
            self.text_editor.draw_chunk(cx, index, &mtb.text_buffer.flat_text, token_chunk, &mtb.text_buffer.markers);
        }
        self.text_editor.end_text_editor(cx, &mtb.text_buffer);

        self.splitter.mid_draw(cx);

        cx.begin_column(Width::Fill, Height::Fill);
        let height_scale = self.dummy_fold_caption.draw_fold_caption(cx, "dummy fold caption");
        self.dummy_color_picker.draw(cx, self.dummy_color, height_scale);
        cx.end_column();

        self.splitter.end_draw(cx);

        cx.end_row();
        self.view.end_view(cx);
    }

    pub fn update_token_chunks(mtb: &mut MakepadTextBuffer, mut search_index: Option<&mut SearchIndex>) {
        if mtb.text_buffer.needs_token_chunks() && !mtb.text_buffer.lines.is_empty() {
            let mut state = TokenizerState::new(&mtb.text_buffer.lines);
            let mut tokenizer = MprsTokenizer::default();
            let mut pair_stack = Vec::new();
            loop {
                let offset = mtb.text_buffer.flat_text.len();
                let token_type = tokenizer.next_token(&mut state, &mut mtb.text_buffer.flat_text, &mtb.text_buffer.token_chunks);
                if TokenChunk::push_with_pairing(
                    &mut mtb.text_buffer.token_chunks,
                    &mut pair_stack,
                    state.next,
                    offset,
                    mtb.text_buffer.flat_text.len(),
                    token_type,
                ) {
                    mtb.text_buffer.was_invalid_pair = true;
                }

                if token_type == TokenType::Eof {
                    break;
                }
                if let Some(search_index) = search_index.as_mut() {
                    search_index.new_rust_token(mtb);
                }
            }
            if !pair_stack.is_empty() {
                mtb.text_buffer.was_invalid_pair = true;
            }
        }
    }
}
