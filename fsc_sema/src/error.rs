use fsc_parse::ast::{BinOp, Ty};

#[derive(Debug)]
pub enum TypeCheckError {
    UnknownVar(String),

    TypeMismatch { expected: Ty, found: Ty },

    AlreadyDeclared(String),

    MissingReturnValue,

    UnexpectedReturnValue,

    InvalidOperandType { op: BinOp, ty: Ty },
}
impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeCheckError::UnknownVar(name) => {
                write!(f, "unknown variable `{name}`")
            }

            TypeCheckError::AlreadyDeclared(name) => {
                write!(f, "variable `{name}` is already declared")
            }

            TypeCheckError::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {expected:?}, found {found:?}")
            }

            TypeCheckError::MissingReturnValue => {
                write!(f, "missing return value")
            }

            TypeCheckError::UnexpectedReturnValue => {
                write!(f, "unexpected return value in void function")
            }

            TypeCheckError::InvalidOperandType { op, ty } => {
                write!(f, "invalid operand type {ty:?} for operator {op:?}")
            }
        }
    }
}

pub type TypeCheckResult<T> = Result<T, TypeCheckError>;
