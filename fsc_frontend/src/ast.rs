#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub _params: Vec<String>,
    pub private: bool,
    pub body: Vec<Statement>,
}
#[derive(Debug, Clone)]
pub enum Statement {
    Label(String),
    Instruction(Instruction),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    SysCall { argc: u8, page: u8, func: u8 },
    GrowStack(i16),
    ShrinkStack(i16),
    LoadArg(i16),
    StoreArg(i16),
    ArgAddi(i16),
    ArgSubi(i16),
    Add,
    Sub,
    Mul,
    Div,
    Push(i16),
    PushImm(i32),
    PushResult,
    DelayLoad,
    DelayNeq0,
    Exit1,
    Exit2,
    SetArgMode,
    Call(String),
    Jmp(String),
    Jnz(String),
    JnzPause(String),
    JnzSet(String),
    JzSet(String),
    JzPause(String),
    Jz(String),
    Jeq(String),
    JeqImm{ imm: i8, label: String },
    Eq0,
    Eq,
    LStr(String),
    Retv(i16),        // retv -0x2
    Ret(i16),        // ret -0x2
}