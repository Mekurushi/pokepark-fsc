mod error;

use crate::lexer::error::{ParseError, ParseResult};
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")] // whitespace
#[logos(skip(r"//[^\n]*", allow_greedy = true))] // line comments TODO: check greedy
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")] // block comments
pub enum Token {
    // --- Type Keywords ---
    #[token("int", priority = 10)]
    KwInt,

    // --- Control Flow ---
    #[token("return", priority = 10)]
    KwReturn,

    // --- Integer literals ---
    /// Hexadecimal integer; higher priority so it's not splitting
    #[regex(r"0[xX][0-9a-fA-F]+", |lex| {
        i32::from_str_radix(&lex.slice()[2..], 16).ok()
    }, priority = 5)]
    /// pure decimal
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i32>().ok(), priority = 2)]
    IntLit(i32),

    // --- Identifier ---
    /// non-keyword identifier e.g. function names
    /// Must start with a letter or underscore
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),

    // --- Arithmetic ---
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    // --- Assignment ---
    #[token("=")]
    Eq,

    // --- Punctuation ---
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,

    // --- Delimiters ---
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
}

// TODO: support for span, lines, cols
pub fn tokenize(src: &str) -> ParseResult<Vec<(Token, std::ops::Range<usize>)>> {
    let mut tokens = Vec::new();
    let mut lex = Token::lexer(src);
    while let Some(result) = lex.next() {
        let span = lex.span();
        match result {
            Ok(tok) => tokens.push((tok, span)),
            Err(()) => return Err(ParseError::LexError { offset: span.start }),
        }
    }
    Ok(tokens)
}
