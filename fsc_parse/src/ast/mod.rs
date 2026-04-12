#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Int,
    Void,
    Bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    // comparison
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    // logical
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not, // !
    Neg, // -
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i32),

    Var(String),

    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    BoolLit(bool),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Option<Expr>),
    VarDecl {
        name: String,
        ty: Ty,
        init: Option<Expr>,
    },
    Assign {
        name: String,
        expr: Expr,
    },
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Ty,
    pub body: Vec<Stmt>,
    pub exported: bool,
}

#[derive(Debug, Clone)]
pub enum Item {
    FuncDef(FuncDef),
}

#[derive(Debug, Clone)]
pub struct Script {
    pub items: Vec<Item>,
}
