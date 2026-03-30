use crate::error::{ParseError, ParseResult};
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

    #[token("shrink_stack")]
    ShrinkStack,

    #[token("load_arg")]
    LoadArg,

    #[token("store_arg")]
    StoreArg,

    #[token("arg_addi")]
    ArgAddi,

    #[token("arg_subi")]
    ArgSubi,

    #[token("retv")]
    Retv,

    #[token("ret")]
    Ret,

    #[token("delay_load")]
    DelayLoad,

    #[token("delay")]
    Delay,

    #[token("delay_neq0")]
    DelayNeq0,

    #[token("exit_1")]
    Exit1, //unimpl

    #[token("exit_2")]
    Exit2, //unimpl

    #[token("set_arg_mode")]
    SetArgMode,

    #[token("add")]
    Add,

    #[token("sub")]
    Sub,

    #[token("mul")]
    Mul,

    #[token("div")]
    Div,

    #[token("mod")]
    Mod,

    #[token("and")]
    And,

    #[token("or")]
    Or,

    #[token("xor")]
    Xor,

    #[token("not")]
    Not,

    #[token("neg")]
    Neg,

    #[token("fadd")]
    Fadd,

    #[token("fsub")]
    Fsub,

    #[token("fmul")]
    Fmul,

    #[token("fdiv")]
    Fdiv,

    #[token("feq0")]
    Feq0,

    #[token("fneg")]
    Fneg,

    #[token("feq")]
    Feq,

    #[token("fneq")]
    Fneq,

    #[token("flt")]
    Flt,

    #[token("fgt")]
    Fgt,

    #[token("fle")]
    Fle,

    #[token("fge")]
    Fge,

    #[token("push")]
    Push,

    #[token("push_imm")]
    PushImm,

    #[token("call")]
    Call,

    #[token("jmp")]
    Jmp,

    #[token("jnz")]
    Jnz,

    #[token("jnz_pause")]
    JnzPause,

    #[token("jz_pause")]
    JzPause,

    #[token("jnz_set")]
    JnzSet,

    #[token("jz_set")]
    JzSet,

    #[token("jz")]
    Jz,

    #[token("jeq")]
    Jeq,

    #[token("jeq_imm")]
    JeqImm,

    #[token("eq0")]
    Eq0,

    #[token("eq")]
    Eq,

    #[token("neq")]
    Neq,

    #[token("lt")]
    Lt,

    #[token("gt")]
    Gt,

    #[token("le")]
    Le,

    #[token("ge")]
    Ge,

    #[token("sl")]
    Sl,

    #[token("srm")]
    Srm,

    #[token("sr")]
    Sr,

    #[token("lea")]
    Lea,

    #[token("lbi")]
    Lbi,
    #[token("lsi")]
    Lsi,
    #[token("lwi")]
    Lwi,
    #[token("lb")]
    Lb,
    #[token("ls")]
    Ls,
    #[token("lw")]
    Lw,

    #[token("sb")]
    Sb,
    #[token("ss")]
    Ss,
    #[token("sw")]
    Sw,
    #[token("sbadd")]
    SbAdd,
    #[token("sbiadd")]
    SbiAdd,
    #[token("sbsub")]
    SbSub,
    #[token("sbisub")]
    SbiSub,

    #[token("ssadd")]
    SsAdd,
    #[token("ssiadd")]
    SsiAdd,
    #[token("sssub")]
    SsSub,
    #[token("ssisub")]
    SsiSub,

    #[token("swadd")]
    SwAdd,
    #[token("swiadd")]
    SwiAdd,
    #[token("swsub")]
    SwSub,
    #[token("swisub")]
    SwiSub,

    #[token("itof")]
    ItoF,

    #[token("ftoi")]
    FtoI,

    #[token("push_result")]
    PushResult,

    #[token("lstr")]
    LStr,
    #[regex(r"SC([0-9]+)", |lex| {
        lex.slice()[2..].parse::<u8>().ok()
    })]
    SysCall(u8),

    #[regex(r#""[^"]*""#, |lex| {
    let s = lex.slice();
    s[1..s.len()-1].to_string() // strip quotes
})]
    StringLiteral(String),

    // --- Punctuation ---
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*:", |lex| {
    let s = lex.slice();
    s[..s.len()-1].to_string() // strip the colon
})]
    LabelDef(String),

    #[regex(r"-?0x[0-9a-fA-F]{1,4}", |lex| {
    let s = lex.slice();
    if let Some(digits) = s.strip_prefix("-0x") {
        i16::from_str_radix(digits, 16).ok().map(|n| -n)
    } else {
        i16::from_str_radix(&s[2..], 16).ok()
    }
})]
    Int(i16),

    #[regex(r"0x[0-9a-fA-F]{5,8}", |lex| {
    i32::from_str_radix(&lex.slice()[2..], 16).ok()
})]
    Int32(i32),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}

pub fn tokenize(src: &str) -> ParseResult<Vec<(Token, std::ops::Range<usize>)>> {
    let mut tokens = Vec::new();
    let mut lex = Token::lexer(src);
    while let Some(result) = lex.next() {
        let span = lex.span();
        match result {
            Ok(tok) => tokens.push((tok, span)),
            Err(()) => return Err(ParseError::LexError { offset: span.start }),
        }
    }
    Ok(tokens)
}
