use zaplib_components::*;

#[derive(Default)]
pub struct MprsTokenizer {
    comment_single: bool,
    comment_depth: usize,
    in_string_code: bool,
    in_string: bool,
}

impl MprsTokenizer {
    pub fn next_token<'a>(
        &mut self,
        state: &mut TokenizerState<'a>,
        chunk: &mut Vec<char>,
        token_chunks: &[TokenChunk],
    ) -> TokenType {
        let start = chunk.len();
        //chunk.truncate(0);
        if self.in_string {
            if state.next == ' ' || state.next == '\t' {
                while state.next == ' ' || state.next == '\t' {
                    chunk.push(state.next);
                    state.advance_with_cur();
                }
                return TokenType::Whitespace;
            }
            loop {
                if state.eof {
                    self.in_string = false;
                    return TokenType::StringChunk;
                } else if state.next == '\n' {
                    if (chunk.len() - start) > 0 {
                        return TokenType::StringChunk;
                    }
                    chunk.push(state.next);
                    state.advance_with_cur();
                    return TokenType::Newline;
                } else if state.next == '"' && state.cur != '\\' {
                    if (chunk.len() - start) > 0 {
                        return TokenType::StringChunk;
                    }
                    chunk.push(state.next);
                    state.advance_with_cur();
                    self.in_string = false;
                    return TokenType::StringMultiEnd;
                } else {
                    chunk.push(state.next);
                    state.advance_with_cur();
                }
            }
        } else if self.comment_depth > 0 {
            // parse comments
            loop {
                if state.eof {
                    self.comment_depth = 0;
                    return TokenType::CommentChunk;
                }
                if state.next == '/' {
                    chunk.push(state.next);
                    state.advance();
                    if state.next == '*' {
                        chunk.push(state.next);
                        state.advance();
                        self.comment_depth += 1;
                    }
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
                    while state.next == ' ' || state.next == '\t' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Whitespace
                }
                '/' => {
                    // parse comment
                    chunk.push(state.cur);
                    if state.next == '/' {
                        chunk.push(state.next);
                        state.advance();
                        self.comment_depth = 1;
                        self.comment_single = true;
                        return TokenType::CommentLine;
                    }
                    if state.next == '*' {
                        // start parsing a multiline comment
                        //let mut comment_depth = 1;
                        chunk.push(state.next);
                        state.advance();
                        self.comment_single = false;
                        self.comment_depth = 1;
                        return TokenType::CommentMultiBegin;
                    }
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '\'' => {
                    // parse char literal or lifetime annotation
                    chunk.push(state.cur);

                    if Self::parse_rust_escape_char(state, chunk) {
                        // escape char or unicode
                        if state.next == '\'' {
                            // parsed to closing '
                            chunk.push(state.next);
                            state.advance();
                            return TokenType::String;
                        }
                        TokenType::TypeName
                    } else {
                        // parse a single char or lifetime
                        let offset = state.offset;
                        let (is_ident, _) = Self::parse_rust_ident_tail(state, chunk);
                        if is_ident && ((state.offset - offset) > 1 || state.next != '\'') {
                            return TokenType::TypeName;
                        }
                        if state.next != '\n' {
                            if (state.offset - offset) == 0 {
                                // not an identifier char
                                chunk.push(state.next);
                                state.advance();
                            }
                            if state.next == '\'' {
                                // lifetime identifier
                                chunk.push(state.next);
                                state.advance();
                            }
                            return TokenType::String;
                        }
                        TokenType::String
                    }
                }
                '"' => {
                    // parse string
                    // we have to scan back, skip all whitespacey things
                    // see if we find a shader!(
                    // we have to backparse.

                    chunk.push(state.cur);

                    if chunk.len() >= 3 && chunk[chunk.len() - 3] == 'r' && chunk[chunk.len() - 2] == '#' {
                        self.in_string_code = true;
                        return TokenType::ParenOpen;
                    }
                    if state.next == '#' && self.in_string_code {
                        self.in_string_code = false;
                        return TokenType::ParenClose;
                    }

                    state.prev = '\0';
                    while !state.eof && state.next != '\n' {
                        chunk.push(state.next);
                        if state.next != '"' || state.cur == '\\' && state.prev != '\\' {
                            state.advance_with_prev();
                        } else {
                            state.advance();
                            return TokenType::String;
                        }
                    }
                    if state.next == '\n' {
                        self.in_string = true;
                        return TokenType::StringMultiBegin;
                    }
                    TokenType::String
                }
                '0'..='9' => {
                    // try to parse numbers
                    chunk.push(state.cur);
                    Self::parse_rust_number_tail(state, chunk);
                    TokenType::Number
                }
                ':' => {
                    chunk.push(state.cur);
                    if state.next == ':' {
                        chunk.push(state.next);
                        state.advance();
                        return TokenType::Namespace;
                    }
                    TokenType::Colon
                }
                '*' => {
                    chunk.push(state.cur);
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                        return TokenType::Operator;
                    }
                    if state.next == '/' {
                        chunk.push(state.next);
                        state.advance();
                        return TokenType::Unexpected;
                    }
                    TokenType::Operator
                }
                '^' => {
                    chunk.push(state.cur);
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '+' => {
                    chunk.push(state.cur);
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '-' => {
                    chunk.push(state.cur);
                    if state.next == '>' || state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    TokenType::Operator
                }
                '=' => {
                    chunk.push(state.cur);
                    if state.next == '>' || state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }

                    TokenType::Operator
                }
                '.' => {
                    chunk.push(state.cur);
                    if state.next == '.' {
                        chunk.push(state.next);
                        state.advance();
                        if state.next == '=' {
                            chunk.push(state.next);
                            state.advance();
                            return TokenType::Splat;
                        }
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
                    }
                    TokenType::Operator
                }
                '<' => {
                    chunk.push(state.cur);
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    if state.next == '<' {
                        chunk.push(state.next);
                        state.advance();
                        if state.next == '=' {
                            chunk.push(state.next);
                            state.advance();
                        }
                    }
                    TokenType::Operator
                }
                '>' => {
                    chunk.push(state.cur);
                    if state.next == '=' {
                        chunk.push(state.next);
                        state.advance();
                    }
                    if state.next == '>' {
                        chunk.push(state.next);
                        state.advance();
                        if state.next == '=' {
                            chunk.push(state.next);
                            state.advance();
                        }
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
                '#' => {
                    chunk.push(state.cur);
                    // if followed by 0-9A-Fa-f parse untill not one of those
                    if state.next >= '0' && state.next <= '9'
                        || state.next >= 'a' && state.next <= 'f'
                        || state.next >= 'A' && state.next <= 'F'
                    {
                        // parse a hex number
                        chunk.push(state.next);
                        state.advance();
                        while state.next_is_hex() {
                            chunk.push(state.next);
                            state.advance();
                        }
                        TokenType::Color
                    } else {
                        TokenType::Hash
                    }
                }
                '_' => {
                    chunk.push(state.cur);
                    Self::parse_rust_ident_tail(state, chunk);
                    if state.next == '(' {
                        return TokenType::Call;
                    }
                    if state.next == '!' {
                        return TokenType::Macro;
                    }
                    TokenType::Identifier
                }
                'a'..='z' => {
                    // try to parse keywords or identifiers
                    chunk.push(state.cur);

                    let keyword_type = Self::parse_rust_lc_keyword(state, chunk, token_chunks);
                    let (is_ident, _) = Self::parse_rust_ident_tail(state, chunk);
                    if is_ident {
                        if state.next == '(' {
                            return TokenType::Call;
                        }
                        if state.next == '!' {
                            return TokenType::Macro;
                        }
                        TokenType::Identifier
                    } else {
                        keyword_type
                    }
                }
                'A'..='Z' => {
                    chunk.push(state.cur);
                    let mut is_keyword = false;
                    if state.cur == 'S' && state.keyword(chunk, "elf") {
                        is_keyword = true;
                    }
                    let (is_ident, has_underscores) = Self::parse_rust_ident_tail(state, chunk);
                    if is_ident {
                        is_keyword = false;
                    }
                    if has_underscores {
                        return TokenType::ThemeName;
                    }
                    if is_keyword {
                        return TokenType::Keyword;
                    }
                    TokenType::TypeName
                }
                _ => {
                    chunk.push(state.cur);
                    TokenType::Operator
                }
            }
        }
    }

    fn parse_rust_ident_tail<'a>(state: &mut TokenizerState<'a>, chunk: &mut Vec<char>) -> (bool, bool) {
        let mut ret = false;
        let mut has_underscores = false;
        while state.next_is_digit() || state.next_is_letter() || state.next == '_' || state.next == '$' {
            if state.next == '_' {
                has_underscores = true;
            }
            ret = true;
            chunk.push(state.next);
            state.advance();
        }
        (ret, has_underscores)
    }

    fn parse_rust_escape_char<'a>(state: &mut TokenizerState<'a>, chunk: &mut Vec<char>) -> bool {
        if state.next == '\\' {
            chunk.push(state.next);
            state.advance();
            if state.next == 'u' {
                chunk.push(state.next);
                state.advance();
                if state.next == '{' {
                    chunk.push(state.next);
                    state.advance();
                    while state.next_is_hex() {
                        chunk.push(state.next);
                        state.advance();
                    }
                    if state.next == '}' {
                        chunk.push(state.next);
                        state.advance();
                    }
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
    fn parse_rust_number_tail<'a>(state: &mut TokenizerState<'a>, chunk: &mut Vec<char>) {
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
        } else if state.next == 'o' {
            // parse a octal
            chunk.push(state.next);
            state.advance();
            while state.next == '0'
                || state.next == '1'
                || state.next == '2'
                || state.next == '3'
                || state.next == '4'
                || state.next == '5'
                || state.next == '6'
                || state.next == '7'
                || state.next == '_'
            {
                chunk.push(state.next);
                state.advance();
            }
        } else {
            while state.next_is_digit() || state.next == '_' {
                chunk.push(state.next);
                state.advance();
            }
            if state.next == 'u' || state.next == 'i' {
                chunk.push(state.next);
                state.advance();
                // if state.keyword(chunk, "8") {
                // } else if state.keyword(chunk, "16") {
                // } else if state.keyword(chunk, "32") {
                // } else if state.keyword(chunk, "64") {
                // }
            } else if state.next == '.' || state.next == 'f' || state.next == 'e' || state.next == 'E' {
                if state.next == '.' || state.next == 'f' {
                    chunk.push(state.next);
                    state.advance();
                    while state.next_is_digit() || state.next == '_' {
                        chunk.push(state.next);
                        state.advance();
                    }
                }
                if state.next == 'E' || state.next == 'e' {
                    chunk.push(state.next);
                    state.advance();
                    if state.next == '+' || state.next == '-' {
                        chunk.push(state.next);
                        state.advance();
                        while state.next_is_digit() || state.next == '_' {
                            chunk.push(state.next);
                            state.advance();
                        }
                    } else {
                        return;
                    }
                }
                if state.next == 'f' {
                    // the f32, f64 postfix
                    chunk.push(state.next);
                    state.advance();
                    // if state.keyword(chunk, "32") {
                    // } else if state.keyword(chunk, "64") {
                    // }
                }
            }
        }
    }

    fn parse_rust_lc_keyword<'a>(
        state: &mut TokenizerState<'a>,
        chunk: &mut Vec<char>,
        token_chunks: &[TokenChunk],
    ) -> TokenType {
        match state.cur {
            'a' => {
                if state.keyword(chunk, "s") {
                    return TokenType::Keyword;
                }
            }
            'b' => {
                if state.keyword(chunk, "reak") {
                    return TokenType::Flow;
                }
                if state.keyword(chunk, "ool") {
                    return TokenType::BuiltinType;
                }
            }
            'c' => {
                if state.keyword(chunk, "on") {
                    if state.keyword(chunk, "st") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "tinue") {
                        return TokenType::Flow;
                    }
                }
                if state.keyword(chunk, "rate") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "har") {
                    return TokenType::BuiltinType;
                }
            }
            'd' => {
                if state.keyword(chunk, "yn") {
                    return TokenType::Keyword;
                }
            }
            'e' => {
                if state.keyword(chunk, "lse") {
                    return TokenType::Flow;
                }
                if state.keyword(chunk, "num") {
                    return TokenType::TypeDef;
                }
                if state.keyword(chunk, "xtern") {
                    return TokenType::Keyword;
                }
            }
            'f' => {
                if state.keyword(chunk, "alse") {
                    return TokenType::Bool;
                }
                if state.keyword(chunk, "n") {
                    return TokenType::Fn;
                }
                if state.keyword(chunk, "or") {
                    // check if we are first on a line
                    if token_chunks.len() < 2
                        || token_chunks[token_chunks.len() - 1].token_type == TokenType::Newline
                        || token_chunks[token_chunks.len() - 2].token_type == TokenType::Newline
                            && token_chunks[token_chunks.len() - 1].token_type == TokenType::Whitespace
                    {
                        return TokenType::Looping;
                        //self.code_editor.set_indent_color(self.code_editor.colors.indent_line_looping);
                    }

                    return TokenType::Keyword;
                    // self.code_editor.set_indent_color(self.code_editor.colors.indent_line_def);
                }

                if state.keyword(chunk, "32") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "64") {
                    return TokenType::BuiltinType;
                }
            }
            'i' => {
                if state.keyword(chunk, "f") {
                    return TokenType::Flow;
                }
                if state.keyword(chunk, "mpl") {
                    return TokenType::Impl;
                }
                if state.keyword(chunk, "size") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "n") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "8") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "16") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "32") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "64") {
                    return TokenType::BuiltinType;
                }
            }
            'l' => {
                if state.keyword(chunk, "et") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "oop") {
                    return TokenType::Looping;
                }
            }
            'm' => {
                if state.keyword(chunk, "atch") {
                    return TokenType::Flow;
                }
                if state.keyword(chunk, "ut") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "o") {
                    if state.keyword(chunk, "d") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "ve") {
                        return TokenType::Keyword;
                    }
                }
            }
            'p' => {
                // pub
                if state.keyword(chunk, "ub") {
                    return TokenType::Keyword;
                }
            }
            'r' => {
                if state.keyword(chunk, "e") {
                    if state.keyword(chunk, "f") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "turn") {
                        return TokenType::Flow;
                    }
                }
            }
            's' => {
                if state.keyword(chunk, "elf") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "uper") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "t") {
                    if state.keyword(chunk, "atic") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "r") {
                        if state.keyword(chunk, "uct") {
                            return TokenType::TypeDef;
                        }
                        return TokenType::BuiltinType;
                    }
                }
            }
            't' => {
                if state.keyword(chunk, "ype") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "r") {
                    if state.keyword(chunk, "ait") {
                        return TokenType::TypeDef;
                    }
                    if state.keyword(chunk, "ue") {
                        return TokenType::Bool;
                    }
                }
            }
            'u' => {
                // use

                if state.keyword(chunk, "nsafe") {
                    return TokenType::Keyword;
                }
                if state.keyword(chunk, "8") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "16") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "32") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "64") {
                    return TokenType::BuiltinType;
                }
                if state.keyword(chunk, "s") {
                    if state.keyword(chunk, "ize") {
                        return TokenType::BuiltinType;
                    }
                    if state.keyword(chunk, "e") {
                        return TokenType::Keyword;
                    }
                }
            }
            'w' => {
                // use
                if state.keyword(chunk, "h") {
                    if state.keyword(chunk, "ere") {
                        return TokenType::Keyword;
                    }
                    if state.keyword(chunk, "ile") {
                        return TokenType::Looping;
                    }
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
