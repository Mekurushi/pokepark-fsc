use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token {
    // --- Keywords ---
    #[token("fn")]
    Fn,

    #[token("private")]
    Private,

    // --- Opcodes ---
    #[token("grow_stack")]
    GrowStack,

    #[token("load_arg")]
    LoadArg,

    #[token("retv")]
    Retv,

    #[token("add")]
    Add,

    // --- Punctuation ---
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

}