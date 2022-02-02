use crate::analyse::analyse_shader;
use crate::builtin::generate_builtins;
use crate::builtin::Builtin;
use crate::code_fragment::CodeFragment;
use crate::detok::DeTokParserImpl;
use crate::error::ParseError;
use crate::ident::Ident;
use crate::lex::lex;
use crate::shaderast::ShaderAst;
use crate::span::CodeFragmentId;
use crate::token::{Token, TokenWithSpan};
use std::collections::HashMap;

/// TODO(JP): Would be nice if we can make [`ShaderAstGenerator::builtins`] a `const` so we don't
/// need to keep any state.
pub struct ShaderAstGenerator {
    builtins: HashMap<Ident, Builtin>,
}

impl ShaderAstGenerator {
    pub fn new() -> Self {
        Self { builtins: generate_builtins() }
    }

    /// Generate a complete [`ShaderAst`] from some code fragments.
    pub fn generate_shader_ast(&self, code_fragments: &[CodeFragment]) -> Result<ShaderAst, ParseError> {
        let mut tokens: Vec<TokenWithSpan> = vec![];
        let code_fragments_len = code_fragments.len();
        for (index, code_fragment) in code_fragments.iter().enumerate() {
            for token_result in lex(code_fragment.code().chars(), CodeFragmentId(index)) {
                let token = token_result?;
                // Skip intermediate `Eof` tokens, but keep the last one.
                if token.token != Token::Eof || index == code_fragments_len - 1 {
                    tokens.push(token);
                }
            }
        }
        let shader_ast = DeTokParserImpl::new(&tokens).parse_shader()?;
        analyse_shader(&self.builtins, &shader_ast)?;
        Ok(shader_ast)
    }
}
