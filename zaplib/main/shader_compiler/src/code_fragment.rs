/// Represents a location in a file, and the code string itself. Generate easily
/// using the `zaplib::code_fragment!()` macro.
#[derive(Debug, Clone)]
pub enum CodeFragment {
    Static { filename: &'static str, line: usize, col: usize, code: &'static str },
    Dynamic { name: String, code: String },
}

impl CodeFragment {
    /// Offset the `line` and `col` fields by a certain number of characters.
    pub fn name_line_col_at_offset(&self, offset_chars: usize) -> String {
        let (name, mut line, mut col, code) = match self {
            CodeFragment::Static { filename, line, col, code } => (*filename, *line, *col, *code),
            CodeFragment::Dynamic { name, code } => (name.as_str(), 0 as usize, 0 as usize, code.as_str()),
        };

        for (char_index, ch) in code.chars().enumerate() {
            if char_index == offset_chars {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        format!("{}:{}:{}", name, line, col)
    }

    pub fn code(&self) -> &str {
        match self {
            CodeFragment::Static { code, .. } => code,
            CodeFragment::Dynamic { code, .. } => code,
        }
    }
}
