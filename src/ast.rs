#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub private: bool,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    GrowStack(i32),   // grow_stack 0x1
    LoadArg(i32),     // load_arg 0x0
    Add,              // add
    Retv(i32),        // retv -0x2
}