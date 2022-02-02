use crate::{code_fragment::CodeFragment, span::Span};
use std::fmt;

#[derive(Clone, Debug)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ParseError {
    pub fn format_for_console(&self, code_fragments: &[CodeFragment]) -> String {
        let code_fragment = &code_fragments[self.span.code_fragment_id.0];
        let pos = self.span.start;
        // Hacky debugging: besides printing the file/line/col and so on, we also pull out a
        // string (without newlines) from the actual code, and position it such that `pos`
        // appears right under our "vvvvvv". :-)
        format!(
            "Error parsing shader at {} => {}\n\naround:             vvvvvv\n{}",
            code_fragment.name_line_col_at_offset(pos),
            self.message,
            code_fragment.code().chars().skip(pos - 20).take(50).collect::<String>().replace('\n', " "),
        )
    }
}
