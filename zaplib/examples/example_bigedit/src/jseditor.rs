use crate::makepadstorage::*;
use crate::searchindex::*;
use zaplib::*;
use zaplib_components::*;

pub struct JSEditor {
    pub text_editor: TextEditor,
}

impl JSEditor {
    pub fn new() -> Self {
        Self { text_editor: TextEditor { folding_depth: 3, ..TextEditor::default() } }
    }

    pub fn handle(&mut self, cx: &mut Cx, event: &mut Event, mtb: &mut MakepadTextBuffer) -> TextEditorEvent {
        self.text_editor.handle(cx, event, &mut mtb.text_buffer)
    }

    pub fn draw(&mut self, cx: &mut Cx, mtb: &mut MakepadTextBuffer, search_index: Option<&mut SearchIndex>) {
        JSTokenizer::update_token_chunks(mtb, search_index);

        self.text_editor.begin_text_editor(cx, &mut mtb.text_buffer, None);
        for (index, token_chunk) in mtb.text_buffer.token_chunks.iter_mut().enumerate() {
            self.text_editor.draw_chunk(cx, index, &mtb.text_buffer.flat_text, token_chunk, &mtb.text_buffer.markers);
        }

        self.text_editor.end_text_editor(cx, &mut mtb.text_buffer);
    }
}

#[derive(Default)]
pub struct JSTokenizer {
    comment_single: bool,
    comment_depth: usize,
}

impl JSTokenizer {
    pub fn update_token_chunks(mtb: &mut MakepadTextBuffer, mut _search_index: Option<&mut SearchIndex>) {
        let text_buffer = &mut mtb.text_buffer;
        if text_buffer.needs_token_chunks() && !text_buffer.lines.is_empty() {
            let mut state = TokenizerState::new(&text_buffer.lines);
            let mut tokenizer = JSTokenizer::default();
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
        token_chunks: &Vec<TokenChunk>,
    ) -> TokenType {
        let start = chunk.len();
        if self.comment_depth > 0 {
            // parse comments
            loop {
                if state.next == '\0' {
                    self.comment_depth = 0;
                    return TokenType::CommentChunk;
                } else if state.next == '*' {
                    chunk.push(state.next);
                    state.advance();
                    if state.next == '/' {
                        self.comment_depth -= 1;
                        chunk.push(state.next);
                        state.advance();
                        if self.comment_depth == 0 {
                            return TokenType::CommentMultiEnd;
                        }
                    }
                } else if state.next == '\n' {
                    if self.comment_single {
                        self.comment_depth = 0;
                    }
                    // output current line
                    if (chunk.len() - start) > 0 {
                        return TokenType::CommentChunk;
                    }

                    chunk.push(state.next);
                    state.advance();
                    return TokenType::Newline;
                } else if state.next == ' ' {
                    if (chunk.len() - start) > 0 {
                        return TokenType::CommentChunk;
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
        } else {
            if state.eof {
                return TokenType::Eof;
            }
            state.advance_with_cur();
            match state.cur {
                '\0' => {
                    // eof insert a terminating space and end
                    chunk.push('\0');
                    TokenType::Whitespace
                }
                '\n' => {
                    chunk.push('\n');
                    TokenType::Newline
                }
                ' ' | '\t' => {
                    // eat as many spaces as possible
                    chunk.push(state.cur);
                    while state.next == ' ' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Whitespace
                }
                '/' => {
                    // parse comment or regexp
                    chunk.push(state.cur);
                    if state.next == '/' {
                        chunk.push(state.next);
                        state.advance();
                        self.comment_depth = 1;
                        self.comment_single = true;
                        TokenType::CommentLine
                    } else if state.next == '*' {
                        // start parsing a multiline comment
                        //let mut comment_depth = 1;
                        chunk.push(state.next);
                        state.advance();
                        self.comment_single = false;
                        self.comment_depth = 1;
                        TokenType::CommentMultiBegin
                    } else {
                        let is_regexp = match TokenChunk::scan_last_token(token_chunks) {
                            TokenType::ParenOpen
                            | TokenType::Keyword
                            | TokenType::Operator
                            | TokenType::Delimiter
                            | TokenType::Colon
                            | TokenType::Looping => true,
                            _ => false,
                        };
                        if is_regexp {
                            while !state.eof && state.next != '\n' {
                                if state.next != '/' || state.prev != '\\' && state.cur == '\\' && state.next == '/' {
                                    chunk.push(state.next);
                                    state.advance_with_prev();
                                } else {
                                    chunk.push(state.next);
                                    state.advance();
                                    // lets see what characters we are followed by
                                    while state.next == 'g'
                                        || state.next == 'i'
                                        || state.next == 'm'
                                        || state.next == 's'
                                        || state.next == 'u'
                                        || state.next == 'y'
                                    {
                                        chunk.push(state.next);
                                        state.advance();
                                    }
                                    return TokenType::Regex;
                                }
                            }
                            return TokenType::Regex;
                        } else if state.next == '=' {
                            chunk.push(state.next);
                            state.advance();
                        }
                        TokenType::Operator
                    }
                }
                '"' | '\'' => {
                    // parse string
                    let end_char = state.cur;
                    chunk.push(state.cur);
                    state.prev = '\0';
                    while !state.eof && state.next != '\n' {
                        if state.next == '\\' {
                            Self::parse_js_escape_char(state, chunk);
                        } else if state.next != end_char || state.prev != '\\' && state.cur == '\\' && state.next == end_char {
                            chunk.push(state.next);
                            state.advance_with_prev();
                        } else {
                            // found the end
                            chunk.push(state.next);
                            state.advance();
                            return TokenType::String;
                        }
                    }
                    TokenType::String
                }
                '0'..='9' => {
                    // try to parse numbers
                    chunk.push(state.cur);
                    Self::parse_js_number_tail(state, chunk);
                    TokenType::Number
                }
                ':' => {
                    chunk.push(state.cur);
                    TokenType::Colon
                }
                '*' => {
                    chunk.push(state.cur);
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                        TokenType::Operator
                    } else if state.next == '/' {
                        chunk.push(state.next);
                        state.advance();
                        TokenType::Unexpected
                    } else {
                        TokenType::Operator
                    }
                }
                '+' => {
                    chunk.push(state.cur);
                    if state.next == '=' || state.next == '+' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '-' => {
                    chunk.push(state.cur);
                    if state.next == '>' || state.next == '=' || state.next == '-' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '=' => {
                    chunk.push(state.cur);
                    if state.next == '>' {
                        chunk.push(state.next);
                        state.advance();
                    } else if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                        if state.next == '=' {
                            chunk.push(state.next);
                            state.advance();
                        }
                    }

                    TokenType::Operator
                }
                '.' => {
                    chunk.push(state.cur);
                    if state.next == '.' {
                        chunk.push(state.next);
                        state.advance();
                        return TokenType::Splat;
                    }
                    TokenType::Operator
                }
                ';' => {
                    chunk.push(state.cur);
                    if state.next == '.' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Delimiter
                }
                '&' => {
                    chunk.push(state.cur);
                    if state.next == '&' || state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '|' => {
                    chunk.push(state.cur);
                    if state.next == '|' || state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '!' => {
                    chunk.push(state.cur);
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                        if state.next == '=' {
                            chunk.push(state.next);
                            state.advance();
                        }
                    }
                    TokenType::Operator
                }
                '<' => {
                    chunk.push(state.cur);
                    if state.next == '=' || state.next == '<' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '>' => {
                    chunk.push(state.cur);
                    if state.next == '=' || state.next == '>' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                ',' => {
                    chunk.push(state.cur);
                    TokenType::Delimiter
                }
                '(' | '{' | '[' => {
                    chunk.push(state.cur);
                    TokenType::ParenOpen
                }
                ')' | '}' | ']' => {
                    chunk.push(state.cur);
                    TokenType::ParenClose
                }
                '_' | '$' => {
                    chunk.push(state.cur);
                    Self::parse_js_ident_tail(state, chunk);
                    if state.next == '(' {
                        TokenType::Call
                    } else {
                        TokenType::Identifier
                    }
                }
                'a'..='z' | 'A'..='Z' => {
                    // try to parse keywords or identifiers
                    chunk.push(state.cur);

                    let keyword_type = Self::parse_js_keyword(state, chunk, token_chunks);

                    if Self::parse_js_ident_tail(state, chunk) {
                        if state.next == '(' {
                            TokenType::Call
                        } else {
                            TokenType::Identifier
                        }
                    } else {
                        keyword_type
                    }
                }
                _ => {
                    chunk.push(state.cur);
                    TokenType::Operator
                }
            }
        }
    }

    fn parse_js_ident_tail<'a>(state: &mut TokenizerState<'a>, chunk: &mut Vec<char>) -> bool {
        let mut ret = false;
        while state.next_is_digit() || state.next_is_letter() || state.next == '_' || state.next == '$' {
            ret = true;
            chunk.push(state.next);
            state.advance();
        }
        ret
    }

    fn parse_js_escape_char<'a>(state: &mut TokenizerState<'a>, chunk: &mut Vec<char>) -> bool {
        if state.next == '\\' {
            chunk.push(state.next);
            state.advance();
            if state.next == 'u' {
                chunk.push(state.next);
                state.advance();
                // ! TODO LIMIT THIS TO MAX UNICODE
                while state.next_is_hex() {
                    chunk.push(state.next);
                    state.advance();
                }
            } else if state.next != '\n' && state.next != '\0' {
                // its a single char escape TODO limit this to valid escape chars
                chunk.push(state.next);
                state.advance();
            }
            return true;
        }
        false
    }
    fn parse_js_number_tail<'a>(state: &mut TokenizerState<'a>, chunk: &mut Vec<char>) {
        if state.next == 'x' {
            // parse a hex number
            chunk.push(state.next);
            state.advance();
            while state.next_is_hex() || state.next == '_' {
                chunk.push(state.next);
                state.advance();
            }
        } else if state.next == 'b' {
            // parse a binary
            chunk.push(state.next);
            state.advance();
            while state.next == '0' || state.next == '1' || state.next == '_' {
                chunk.push(state.next);
                state.advance();
            }
        } else {
            while state.next_is_digit() || state.next == '_' {
                chunk.push(state.next);
                state.advance();
            }
            if state.next == '.' {
                chunk.push(state.next);
                state.advance();
                // again eat as many numbers as possible
                while state.next_is_digit() || state.next == '_' {
                    chunk.push(state.next);
                    state.advance();
                }
            }
        }
    }

    fn parse_js_keyword<'a>(state: &mut TokenizerState<'a>, chunk: &mut Vec<char>, _token_chunks: &Vec<TokenChunk>) -> TokenType {
        match state.cur {
            'b' => {
                if state.keyword(chunk, "reak") {
                    return TokenType::Flow;
                }
            }
            'c' => {
                if state.keyword(chunk, "ase") {
                    return TokenType::Flow;
                }
                if state.keyword(chunk, "lass") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "o") {
                    if state.keyword(chunk, "nst") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "ntinue") {
                        return TokenType::Flow;
                    }
                }
            }
            'd' => {
                if state.keyword(chunk, "o") {
                    return TokenType::Looping;
                }
                if state.keyword(chunk, "e") {
                    if state.keyword(chunk, "bugger") {
                        return TokenType::Flow;
                    }
                    if state.keyword(chunk, "fault") {
                        return TokenType::Flow;
                    }
                    if state.keyword(chunk, "lete") {
                        return TokenType::Keyword;
                    }
                }
            }
            'e' => {
                if state.keyword(chunk, "lse") {
                    return TokenType::Flow;
                }
                if state.keyword(chunk, "num") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "xte") {
                    if state.keyword(chunk, "rn") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "nds") {
                        return TokenType::Keyword;
                    }
                    return TokenType::TypeDef;
                }
            }
            'f' => {
                if state.keyword(chunk, "alse") {
                    return TokenType::Bool;
                }
                if state.keyword(chunk, "inally") {
                    return TokenType::Fn;
                }
                if state.keyword(chunk, "or") {
                    return TokenType::Looping;
                }
                if state.keyword(chunk, "unction") {
                    return TokenType::Fn;
                }
            }
            'g' => {
                if state.keyword(chunk, "et") {
                    return TokenType::Keyword;
                }
            }
            'i' => {
                if state.keyword(chunk, "f") {
                    return TokenType::Flow;
                } else if state.keyword(chunk, "mport") {
                    return TokenType::TypeDef;
                } else if state.keyword(chunk, "in") {
                    if state.next_is_letter() || state.next_is_digit() {
                        if state.keyword(chunk, "stanceof") {
                            return TokenType::BuiltinType;
                        }
                    } else {
                        return TokenType::Keyword;
                    }
                }
            }
            'l' => {
                if state.keyword(chunk, "et") {
                    return TokenType::Keyword;
                }
            }
            'n' => {
                if state.keyword(chunk, "ew") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "ull") {
                    return TokenType::Keyword;
                }
            }
            'r' => {
                if state.keyword(chunk, "eturn") {
                    return TokenType::Flow;
                }
            }
            's' => {
                if state.keyword(chunk, "uper") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "witch") {
                    return TokenType::Flow;
                }
                if state.keyword(chunk, "et") {
                    return TokenType::Keyword;
                }
            }
            't' => {
                if state.keyword(chunk, "r") {
                    if state.keyword(chunk, "y") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "ue") {
                        return TokenType::Bool;
                    }
                }
                if state.keyword(chunk, "ypeof") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "h") {
                    if state.keyword(chunk, "is") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "row") {
                        return TokenType::Flow;
                    }
                }
            }
            'v' => {
                // use
                if state.keyword(chunk, "ar") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "oid") {
                    return TokenType::Keyword;
                }
            }
            'w' => {
                // use
                if state.keyword(chunk, "hile") {
                    return TokenType::Looping;
                }
                if state.keyword(chunk, "ith") {
                    return TokenType::Keyword;
                }
            }
            'u' => {
                // use
                if state.keyword(chunk, "ndefined") {
                    return TokenType::Keyword;
                }
            }
            'y' => {
                // use
                if state.keyword(chunk, "ield") {
                    return TokenType::Flow;
                }
            }
            'N' => {
                if state.keyword(chunk, "aN") {
                    return TokenType::Keyword;
                }
            }
            'I' => {
                if state.keyword(chunk, "nfinity") {
                    return TokenType::Keyword;
                }
            }
            _ => {}
        }
        if state.next == '(' {
            TokenType::Call
        } else {
            TokenType::Identifier
        }
    }
}
