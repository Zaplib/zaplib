use crate::makepadstorage::*;
use crate::searchindex::*;
use zaplib::*;
use zaplib_components::*;

pub struct PlainEditor {
    pub text_editor: TextEditor,
}

impl PlainEditor {
    pub fn new() -> Self {
        Self { text_editor: TextEditor { folding_depth: 3, ..TextEditor::default() } }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event, mtb: &mut MakepadTextBuffer) -> TextEditorEvent {
        self.text_editor.handle(cx, event, &mut mtb.text_buffer)
    }

    pub fn draw(&mut self, cx: &mut Cx, mtb: &mut MakepadTextBuffer, search_index: Option<&mut SearchIndex>) {
        PlainTokenizer::update_token_chunks(&mut mtb.text_buffer, search_index);
        self.text_editor.begin_text_editor(cx, &mtb.text_buffer, None);
        for (index, token_chunk) in mtb.text_buffer.token_chunks.iter_mut().enumerate() {
            self.text_editor.draw_chunk(cx, index, &mtb.text_buffer.flat_text, token_chunk, &mtb.text_buffer.markers);
        }

        self.text_editor.end_text_editor(cx, &mtb.text_buffer);
    }
}

#[derive(Default)]
pub struct PlainTokenizer;

impl PlainTokenizer {
    pub fn update_token_chunks(text_buffer: &mut TextBuffer, mut _search_index: Option<&mut SearchIndex>) {
        if text_buffer.needs_token_chunks() && !text_buffer.lines.is_empty() {
            let mut state = TokenizerState::new(&text_buffer.lines);
            let mut tokenizer = PlainTokenizer::default();
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
    }

    fn next_token<'a>(
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
