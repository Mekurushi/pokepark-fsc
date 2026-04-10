#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Int,
    Void,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
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
