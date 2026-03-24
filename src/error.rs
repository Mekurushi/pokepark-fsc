// Parser Error
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { got: String, expected: &'static str, offset: usize },
    UnexpectedEof   { expected: &'static str },
    LexError        { offset: usize },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken { got, expected, offset } =>
                write!(f, "offset {offset}: expected {expected}, got `{got}`"),
            Self::UnexpectedEof { expected } =>
                write!(f, "unexpected end of file, expected {expected}"),
            Self::LexError { offset } =>
                write!(f, "unrecognised token at offset {offset}"),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

// Codegen Error
#[derive(Debug)]
pub enum AssemblerError {
    InvalidB40Char(char),
    OperandOutOfRange(i32),
    UndefinedSymbol(String),
    DuplicateSymbol(String)

}

pub type AssemblerResult<T> = Result<T, AssemblerError>;

impl std::fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerError::InvalidB40Char(c) =>
                write!(f, "invalid character '{c}' in script name, allowed: ' 0-9 A-Z _ - /`"),
            AssemblerError::OperandOutOfRange(n) =>
                write!(f, "operand {n:#x} out of 16-bit signed range"),
            AssemblerError::UndefinedSymbol(name) =>
                write!(f, "undefined symbol '{name}'"),
            AssemblerError::DuplicateSymbol(name) =>
                write!(f, "duplicated symbol '{name}'"),
        }
    }
}