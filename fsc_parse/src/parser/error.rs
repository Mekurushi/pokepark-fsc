use crate::diagnostic::{Diagnostic, Label};
use crate::lexer::token::Span;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        got: String,
        expected: &'static str,
        span: Span,
    },
    UnexpectedEof {
        expected: &'static str,
        span: Span,
    },
}

impl From<ParseError> for Diagnostic {
    fn from(e: ParseError) -> Self {
        match e {
            ParseError::UnexpectedToken {
                got,
                expected,
                span,
            } => Diagnostic::error(format!("expected {expected}, found {got}"))
                .with_label(Label::primary(span, format!("expected {expected}"))),

            ParseError::UnexpectedEof { expected, span } => {
                Diagnostic::error(format!("expected {expected}, found end of file"))
                    .with_label(Label::primary(span, "file ends here"))
            }
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
