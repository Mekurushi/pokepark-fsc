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
pub enum CodegenError {
    InvalidB40Char(char),
    OperandOutOfRange(i32),
    UndefinedSymbol(String),

}

pub type CodegenResult<T> = Result<T, CodegenError>;

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::InvalidB40Char(c) =>
                write!(f, "invalid character '{c}' in script name, allowed: ' 0-9 A-Z _ - /`"),
            CodegenError::OperandOutOfRange(n) =>
                write!(f, "operand {n:#x} out of 16-bit signed range"),
            CodegenError::UndefinedSymbol(name) =>
                write!(f, "undefined symbol '{name}'"),
        }
    }
}