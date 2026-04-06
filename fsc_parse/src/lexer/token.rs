use crate::lexer::error::LexerError;
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\f]+")] // whitespace
#[logos(extras = Vec<usize>)]
#[logos(skip(r"\n", newline_callback))]
#[logos(skip(r"//[^\n]*", allow_greedy = true))] // line comments TODO: check greedy
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")] // block comments
pub enum TokenKind {
    // --- Type Keywords ---
    #[token("int", priority = 10)]
    KwInt,

    // --- Control Flow ---
    #[token("return", priority = 10)]
    KwReturn,

    // --- Declaration keywords ---
    #[token("static", priority = 10)]
    KwStatic,

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

fn newline_callback(lex: &mut logos::Lexer<TokenKind>) -> logos::Skip {
    lex.extras.push(lex.span().end);
    logos::Skip
}

impl TokenKind {
    pub fn is_type_kw(&self) -> bool {
        matches!(self, Self::KwInt)
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::KwInt => "`int`",
            Self::KwReturn => "`return`",
            Self::KwStatic => "`static`",
            Self::IntLit(_) => "integer literal",
            Self::Ident(_) => "identifier",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Star => "`*`",
            Self::Slash => "`/`",
            Self::Percent => "`%`",
            Self::Eq => "`=`",
            Self::Comma => "`,`",
            Self::Semicolon => "`;`",
            Self::LParen => "`(`",
            Self::RParen => "`)`",
            Self::LBrace => "`{`",
            Self::RBrace => "`}`",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub line_starts: Vec<usize>,
    pub errors: Vec<LexerError>,
}
