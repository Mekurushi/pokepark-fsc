use fsc_parse::ast::Ty;

// TODO: diagnostic for Span context
#[derive(Debug, PartialEq)]
pub enum SemaError {
    // TODO: span for error location.
    UndeclaredName(String),

    // TODO: span for duplicate declaration location.
    // TODO: span for original declaration location.
    DuplicateDeclaration(String),

    // TODO: span for operation location.
    TypeMismatch { expected: Ty, found: Ty },

    // TODO: span for return expression location.
    // TODO: span for return type declaration.
    ReturnTypeMismatch { expected: Ty, found: Ty },

    VoidInValuePosition,
    NotCallable(String),
}

impl std::fmt::Display for SemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndeclaredName(name) => {
                write!(f, "undeclared name `{name}`")
            }
            Self::DuplicateDeclaration(name) => {
                write!(f, "`{name}` is already declared in this scope")
            }
            Self::TypeMismatch { expected, found } => {
                write!(
                    f,
                    "type mismatch: expected `{expected:?}`, found `{found:?}`"
                )
            }
            Self::ReturnTypeMismatch { expected, found } => {
                write!(
                    f,
                    "return type mismatch: \
                     function declares `{expected:?}` but expression has type `{found:?}`"
                )
            }
            Self::VoidInValuePosition => {
                write!(f, "expression has type `void` where a value is required")
            }
            Self::NotCallable(name) => {
                write!(f, "function is not callable {name}")
            }
        }
    }
}
pub type SemaResult<T> = Result<T, SemaError>;
