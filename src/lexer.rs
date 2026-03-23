use logos::Logos;
use crate::error::{ParseError, ParseResult};

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token {
    // --- Keywords ---
    #[token("fn")]
    Fn,

    #[token("private")]
    Private,

    // --- Opcodes ---
    #[token("grow_stack")]
    GrowStack,

    #[token("load_arg")]
    LoadArg,

    #[token("retv")]
    Retv,

    #[token("ret")]
    Ret,

    #[token("add")]
    Add,

    #[token("sub")]
    Sub,

    #[token("push")]
    Push,

    #[token("call")]
    Call,

    // --- Punctuation ---
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[regex(r"-?0x[0-9a-fA-F]+", |lex| {
        let s = lex.slice();
        if let Some(digits) = s.strip_prefix("-0x") {
            i16::from_str_radix(digits, 16).ok().map(|n| -n)
        } else {
            i16::from_str_radix(&s[2..], 16).ok()
        }
    })]
    Int(i16),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String)

}

pub fn tokenize(src: &str) -> ParseResult<Vec<(Token, std::ops::Range<usize>)>> {
    let mut tokens = Vec::new();
    let mut lex = Token::lexer(src);
    while let Some(result) = lex.next() {
        let span = lex.span();
        match result {
            Ok(tok) => tokens.push((tok, span)),
            Err(_) => return Err(ParseError::LexError { offset: span.start }),
        }
    }
    Ok(tokens)
}