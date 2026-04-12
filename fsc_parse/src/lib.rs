use crate::ast::Script;
use crate::parser::error::{ParseError, ParseResult};
use crate::parser::{TokenStream, parse_item};

pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;

pub fn parse(source: &str) -> ParseResult<Script> {
    let lex_output = lexer::tokenize(source);

    if !lex_output.errors.is_empty() {
        return Err(ParseError::Lex(lex_output.errors));
    }

    let mut ts = TokenStream::new(lex_output.tokens, source.to_string());
    let mut items = Vec::new();

    while !ts.is_at_end() {
        items.push(parse_item(&mut ts)?);
    }

    Ok(Script { items })
}
