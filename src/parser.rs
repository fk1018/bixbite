use crate::{ast::CompilationUnit, lexer::TokenStream};

pub fn parse(tokens: TokenStream) -> CompilationUnit {
    CompilationUnit::from_source(tokens.into_source())
}
