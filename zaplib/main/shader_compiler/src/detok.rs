use crate::error::ParseError;
use crate::ident::{Ident, IdentPath};
use crate::span::{CodeFragmentId, Span};
use crate::token::{Token, TokenWithSpan};
use std::iter::Cloned;
use std::slice::Iter;

pub(crate) trait DeTokParser {
    fn end(&self) -> usize;
    fn token_end(&self) -> usize;
    fn peek_span(&self) -> Span;
    fn peek_token(&self) -> Token;
    fn skip_token(&mut self);
    fn error(&mut self, msg: String) -> ParseError;
    fn parse_ident(&mut self) -> Result<Ident, ParseError>;
    fn parse_ident_path(&mut self) -> Result<IdentPath, ParseError>;
    fn accept_token(&mut self, token: Token) -> bool;
    fn expect_token(&mut self, expected: Token) -> Result<(), ParseError>;
    fn accept_ident(&mut self, ident_str: &str) -> bool;
    fn expect_ident(&mut self, ident_str: &str) -> Result<(), ParseError>;
    fn error_not_splattable(&mut self, what: &str) -> ParseError;
    fn error_missing_prop(&mut self, what: &str) -> ParseError;
    fn error_enum(&mut self, ident: Ident, what: &str) -> ParseError;
    fn begin_span(&self) -> SpanTracker;
}

pub(crate) struct DeTokParserImpl<'a> {
    token_clone: Vec<TokenWithSpan>,
    tokens_with_span: Cloned<Iter<'a, TokenWithSpan>>,
    token_with_span: TokenWithSpan,
    end: usize,
}

impl<'a> DeTokParserImpl<'a> {
    pub(crate) fn new(tokens_with_span: &'a [TokenWithSpan]) -> Self {
        let mut tokens_with_span = tokens_with_span.iter().cloned();
        let token_with_span = tokens_with_span.next().unwrap();
        DeTokParserImpl { token_clone: Vec::new(), tokens_with_span, token_with_span, end: 0 }
    }
}

impl<'a> DeTokParser for DeTokParserImpl<'a> {
    fn peek_span(&self) -> Span {
        self.token_with_span.span
    }

    fn peek_token(&self) -> Token {
        self.token_with_span.token
    }

    fn skip_token(&mut self) {
        self.end = self.token_with_span.span.end;
        self.token_clone.push(self.token_with_span);
        self.token_with_span = self.tokens_with_span.next().unwrap();
    }

    fn error(&mut self, message: String) -> ParseError {
        ParseError {
            span: Span {
                code_fragment_id: self.token_with_span.span.code_fragment_id,
                start: self.token_with_span.span.start,
                end: self.token_with_span.span.end,
            },
            message,
        }
    }

    fn error_missing_prop(&mut self, what: &str) -> ParseError {
        self.error(format!("Error missing property {}", what))
    }

    fn error_not_splattable(&mut self, what: &str) -> ParseError {
        self.error(format!("Error type {} not splattable", what))
    }

    fn error_enum(&mut self, ident: Ident, what: &str) -> ParseError {
        self.error(format!("Error missing {} for enum {}", ident, what))
    }

    fn parse_ident(&mut self) -> Result<Ident, ParseError> {
        match self.peek_token() {
            Token::Ident(ident) => {
                self.skip_token();
                Ok(ident)
            }
            token => Err(self.error(format!("expected ident, unexpected token `{}`", token))),
        }
    }

    fn parse_ident_path(&mut self) -> Result<IdentPath, ParseError> {
        let mut ident_path = IdentPath::default();
        let span = self.begin_span();
        match self.peek_token() {
            Token::Ident(ident) => {
                self.skip_token();
                ident_path.push(ident);
            }
            token => {
                return Err(span.error(self, format!("expected ident_path, unexpected token `{}`", token)));
            }
        };

        loop {
            if !self.accept_token(Token::PathSep) {
                return Ok(ident_path);
            }
            match self.peek_token() {
                Token::Ident(ident) => {
                    self.skip_token();
                    if !ident_path.push(ident) {
                        return Err(span.error(self, format!("identifier too long `{}`", ident_path)));
                    }
                }
                _ => {
                    return Ok(ident_path);
                }
            }
        }
    }

    fn end(&self) -> usize {
        self.end
    }

    fn token_end(&self) -> usize {
        self.token_with_span.span.end
    }

    fn accept_token(&mut self, token: Token) -> bool {
        if self.peek_token() != token {
            return false;
        }
        self.skip_token();
        true
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), ParseError> {
        let actual = self.peek_token();
        if actual != expected {
            return Err(self.error(format!("expected {} unexpected token `{}`", expected, actual)));
        }
        self.skip_token();
        Ok(())
    }

    fn accept_ident(&mut self, ident_str: &str) -> bool {
        if let Token::Ident(ident) = self.peek_token() {
            if ident == Ident::new(ident_str) {
                self.skip_token();
                return true;
            }
        }
        false
    }

    fn expect_ident(&mut self, ident_str: &str) -> Result<(), ParseError> {
        let actual = self.peek_token();
        if let Token::Ident(ident) = actual {
            if ident == Ident::new(ident_str) {
                self.skip_token();
                return Ok(());
            }
        }
        return Err(self.error(format!("expected {} unexpected token `{}`", ident_str, actual)));
    }

    fn begin_span(&self) -> SpanTracker {
        SpanTracker { code_fragment_id: self.token_with_span.span.code_fragment_id, start: self.token_with_span.span.start }
    }
}

pub(crate) struct SpanTracker {
    code_fragment_id: CodeFragmentId,
    start: usize,
}

impl SpanTracker {
    pub(crate) fn end<F, R>(&self, parser: &dyn DeTokParser, f: F) -> R
    where
        F: FnOnce(Span) -> R,
    {
        f(Span { code_fragment_id: self.code_fragment_id, start: self.start, end: parser.end() })
    }

    pub(crate) fn error(&self, parser: &dyn DeTokParser, message: String) -> ParseError {
        ParseError { span: Span { code_fragment_id: self.code_fragment_id, start: self.start, end: parser.token_end() }, message }
    }
}
