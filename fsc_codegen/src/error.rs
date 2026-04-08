use fsc_assembler::error::AssemblerError;

#[derive(Debug, PartialEq)]
pub enum CodegenError {
    // TODO: more check errors should wander into check module
    UndeclaredVariable(String),

    AlreadyDeclared(String),

    Assembler(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndeclaredVariable(name) => {
                write!(f, "undeclared variable `{name}`")
            }
            Self::AlreadyDeclared(name) => {
                write!(f, "variable `{name}` already declared in this scope")
            }
            Self::Assembler(msg) => {
                write!(f, "assembler error: {msg}")
            }
        }
    }
}

pub type CodegenResult<T> = Result<T, CodegenError>;

impl From<AssemblerError> for CodegenError {
    fn from(e: AssemblerError) -> Self {
        Self::Assembler(e.to_string())
    }
}
