use crate::diagnostic::{Diagnostic, Label};
use crate::lexer::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    UnknownChar { ch: char, span: Span },
    InternalError { span: Span },
}

impl From<LexerError> for Diagnostic {
    fn from(e: LexerError) -> Self {
        match e {
            LexerError::UnknownChar { ch, span } => {
                Diagnostic::error(format!("unknown character `{ch}`"))
                    .with_label(Label::primary(span, "not a valid token"))
            }
            LexerError::InternalError { span } => Diagnostic::error("internal lexer error")
                .with_label(Label::primary(span, "occurred here")),
        }
    }
}

pub type LexerResult<T> = Result<T, LexerError>;
