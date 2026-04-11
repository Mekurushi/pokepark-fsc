pub mod error;
pub mod token;

use crate::lexer::error::LexerError;
use crate::lexer::token::{LexOutput, Span, Token, TokenKind};
use logos::Logos;

pub fn tokenize(src: &str) -> LexOutput {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut lex = TokenKind::lexer_with_extras(src, vec![0usize]);

    while let Some(result) = lex.next() {
        let span = lex.span();
        match result {
            Ok(kind) => tokens.push(Token {
                kind,
                span: Span {
                    start: span.start,
                    end: span.end,
                },
            }),
            Err(()) => match lex.slice().chars().next() {
                Some(ch) => errors.push(LexerError::UnknownChar {
                    ch,
                    span: Span {
                        start: span.start,
                        end: span.end,
                    },
                }),
                None => errors.push(LexerError::InternalError {
                    span: Span {
                        start: span.start,
                        end: span.end,
                    },
                }),
            },
        }
    }

    LexOutput {
        tokens,
        line_starts: lex.extras,
        errors,
    }
}
