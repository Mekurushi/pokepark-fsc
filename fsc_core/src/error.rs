// Assembler Error
#[derive(Debug)]
pub enum AssemblerError {
    InvalidB40Char(char),
    OperandOutOfRange(i32),
    UndefinedSymbol(String),
    DuplicateSymbol(String),
    LabelOutsideFunction(String),
}

pub type AssemblerResult<T> = Result<T, AssemblerError>;

impl std::fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerError::InvalidB40Char(c) => write!(
                f,
                "invalid character '{c}' in script name, allowed: ' 0-9 A-Z _ - /`"
            ),
            AssemblerError::OperandOutOfRange(n) => {
                write!(f, "operand {n:#x} out of 16-bit signed range")
            }
            AssemblerError::UndefinedSymbol(name) => write!(f, "undefined symbol '{name}'"),
            AssemblerError::DuplicateSymbol(name) => write!(f, "duplicated symbol '{name}'"),
            AssemblerError::LabelOutsideFunction(name) => {
                write!(f, "label '{name}' outside of function")
            }
        }
    }
}
