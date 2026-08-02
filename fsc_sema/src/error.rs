use fsc_diagnostics::{Diagnostic, Label, Span, Stage};
use fsc_parse::ast::Ty;

#[derive(Debug, PartialEq)]
pub enum SemaError {
    UndeclaredName {
        name: String,
        reference_span: Span,
    },

    DuplicateDeclaration {
        name: String,
        duplicate_span: Span,
        original_span: Span,
    },

    TypeMismatch {
        expected: Ty,
        found: Ty,
        span: Span,
        expected_span: Option<Span>,
    },

    ReturnTypeMismatch {
        expected: Ty,
        found: Ty,
        span: Span,
        return_type_span: Span,
    },

    VoidInValuePosition {
        span: Span,
    },
    NotCallable {
        name: String,
        callee_span: Span,
        declaration_span: Span,
    },
}

impl std::fmt::Display for SemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndeclaredName { name, .. } => {
                write!(f, "undeclared name `{name}`")
            }
            Self::DuplicateDeclaration { name, .. } => {
                write!(f, "`{name}` is already declared in this scope")
            }
            Self::TypeMismatch {
                expected, found, ..
            } => {
                write!(
                    f,
                    "type mismatch: expected `{expected:?}`, found `{found:?}`"
                )
            }
            Self::ReturnTypeMismatch {
                expected, found, ..
            } => {
                write!(
                    f,
                    "return type mismatch: \
                     function declares `{expected:?}` but expression has type `{found:?}`"
                )
            }
            Self::VoidInValuePosition { .. } => {
                write!(f, "expression has type `void` where a value is required")
            }
            Self::NotCallable { name, .. } => {
                write!(f, "function is not callable {name}")
            }
        }
    }
}

impl From<SemaError> for Diagnostic {
    fn from(error: SemaError) -> Self {
        let message = error.to_string();
        let diagnostic = Diagnostic::error(Stage::Semantic, message);
        match error {
            SemaError::UndeclaredName { reference_span, .. } => {
                diagnostic.with_label(Label::primary(reference_span, "not found in this scope"))
            }
            SemaError::DuplicateDeclaration {
                duplicate_span,
                original_span,
                ..
            } => diagnostic
                .with_label(Label::primary(duplicate_span, "redeclared here"))
                .with_label(Label::secondary(original_span, "first declared here")),
            SemaError::TypeMismatch {
                expected,
                found,
                span,
                expected_span,
            } => {
                let diagnostic =
                    diagnostic.with_label(Label::primary(span, format!("has type `{found:?}`")));
                if let Some(expected_span) = expected_span {
                    diagnostic.with_label(Label::secondary(
                        expected_span,
                        format!("expected `{expected:?}` because of this"),
                    ))
                } else {
                    diagnostic
                }
            }
            SemaError::ReturnTypeMismatch {
                expected,
                found,
                span,
                return_type_span,
            } => diagnostic
                .with_label(Label::primary(span, format!("returns `{found:?}`")))
                .with_label(Label::secondary(
                    return_type_span,
                    format!("function declares `{expected:?}` here"),
                )),
            SemaError::VoidInValuePosition { span } => {
                diagnostic.with_label(Label::primary(span, "has type `Void`"))
            }
            SemaError::NotCallable {
                callee_span,
                declaration_span,
                ..
            } => diagnostic
                .with_label(Label::primary(callee_span, "called here"))
                .with_label(Label::secondary(
                    declaration_span,
                    "this value is declared here",
                )),
        }
    }
}
pub type SemaResult<T> = Result<T, SemaError>;
