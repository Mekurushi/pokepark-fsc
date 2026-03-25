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
