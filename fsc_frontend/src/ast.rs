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
    GrowStack(i16),   // grow_stack 0x1
    LoadArg(i16),     // load_arg 0x0
    Add,              // add
    Sub,                // sub
    Push(i16),               // push
    PushResult,
    Call(String),
    Jmp(String),
    LStr(String),
    Retv(i16),        // retv -0x2
    Ret(i16),        // ret -0x2
}