use crate::textbuffer::*;
use crate::texteditor::*;
use crate::tokentype::*;
use zaplib::*;

pub struct TextInput {
    pub text_editor: TextEditor,
    pub text_buffer: TextBuffer,
    pub empty_message: String,
}

#[derive(Default)]
pub struct TextInputOptions {
    pub multiline: bool,
    pub read_only: bool,
    pub empty_message: String,
}

const COLOR_EMPTY_MESSAGE: Vec4 = vec4(102.0 / 255.0, 102.0 / 255.0, 102.0 / 255.0, 1.0);

impl TextInput {
    pub fn new(opt: TextInputOptions) -> Self {
        Self {
            text_editor: TextEditor {
                read_only: opt.read_only,
                multiline: opt.multiline,
                draw_line_numbers: false,
                draw_cursor_row: false,
                highlight_area_on: false,
                mark_unmatched_parens: false,
                folding_depth: 3,
                line_number_width: 0.,
                top_padding: 0.,
                ..TextEditor::default()
            },
            empty_message: opt.empty_message,
            text_buffer: TextBuffer::from_utf8(""),
        }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event) -> TextEditorEvent {
        let text_buffer = &mut self.text_buffer;

        self.text_editor.handle(cx, event, text_buffer)
    }

    pub fn set_value(&mut self, cx: &mut Cx, text: &str) {
        let text_buffer = &mut self.text_buffer;
        text_buffer.load_from_utf8(text);
        cx.request_draw();
    }

    pub fn get_value(&self) -> String {
        self.text_buffer.get_as_string()
    }

    pub fn select_all(&mut self, cx: &mut Cx) {
        self.text_editor.cursors.select_all(&mut self.text_buffer);
        cx.request_draw();
    }

    pub fn draw_str_input_static(&mut self, cx: &mut Cx, text: &str) {
        let text_buffer = &mut self.text_buffer;
        text_buffer.load_from_utf8(text);
        self.draw(cx);
    }

    pub fn draw(&mut self, cx: &mut Cx) {
        let text_buffer = &mut self.text_buffer;
        if text_buffer.needs_token_chunks() && !text_buffer.lines.is_empty() {
            let mut state = TokenizerState::new(&text_buffer.lines);
            let mut tokenizer = TextInputTokenizer::default();
            let mut pair_stack = Vec::new();
            loop {
                let offset = text_buffer.flat_text.len();
                let token_type = tokenizer.next_token(&mut state, &mut text_buffer.flat_text, &text_buffer.token_chunks);
                TokenChunk::push_with_pairing(
                    &mut text_buffer.token_chunks,
                    &mut pair_stack,
                    state.next,
                    offset,
                    text_buffer.flat_text.len(),
                    token_type,
                );
                if token_type == TokenType::Eof {
                    break;
                }
            }
        }
        cx.begin_padding_box(Padding { t: 11., b: 7., r: 7., l: 7. }); // all (7.0) + top (4.0)

        // Overriding view layout for text inputs to prevent it from consuming all available space.
        // TODO(Dmitry): get rid of this special handling
        self.text_editor.begin_text_editor(cx, text_buffer, Some(LayoutSize::new(Width::Compute, Height::Compute)));
        if text_buffer.is_empty() {
            let pos = cx.get_draw_pos();

            // TODO(Shobhit): We should move this into TextEditor.
            TextIns::draw_walk(
                cx,
                &self.empty_message,
                &TextInsProps { color: COLOR_EMPTY_MESSAGE, text_style: TEXT_STYLE_MONO, ..TextInsProps::DEFAULT },
            );

            cx.set_draw_pos(pos);
        }

        for (index, token_chunk) in text_buffer.token_chunks.iter_mut().enumerate() {
            self.text_editor.draw_chunk(cx, index, &text_buffer.flat_text, token_chunk, &text_buffer.markers);
        }

        self.text_editor.end_text_editor(cx, text_buffer);
        cx.end_padding_box();
    }
}

#[derive(Default)]
pub struct TextInputTokenizer {}

impl TextInputTokenizer {
    pub fn next_token<'a>(
        &mut self,
        state: &mut TokenizerState<'a>,
        chunk: &mut Vec<char>,
        _token_chunks: &[TokenChunk],
    ) -> TokenType {
        let start = chunk.len();
        loop {
            if state.next == '\0' {
                if (chunk.len() - start) > 0 {
                    return TokenType::Identifier;
                }
                state.advance();
                chunk.push(' ');
                return TokenType::Eof;
            } else if state.next == '\n' {
                // output current line
                if (chunk.len() - start) > 0 {
                    return TokenType::Identifier;
                }

                chunk.push(state.next);
                state.advance();
                return TokenType::Newline;
            } else if state.next == ' ' {
                if (chunk.len() - start) > 0 {
                    return TokenType::Identifier;
                }
                while state.next == ' ' {
                    chunk.push(state.next);
                    state.advance();
                }
                return TokenType::Whitespace;
            } else {
                chunk.push(state.next);
                state.advance();
            }
        }
    }
}
