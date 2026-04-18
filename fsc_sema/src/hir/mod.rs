use crate::frame::{FrameLayout, StackSlot};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Int,
    Void,
    Bool,
    Str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    //arithmetic
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

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit {
        value: i32,
        ty: Ty,
    },

    BoolLit {
        value: bool,
        ty: Ty,
    },
    StrLit {
        value: String,
        ty: Ty,
    },

    Var {
        name: String,
        slot: StackSlot,
        ty: Ty,
    },

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        ty: Ty,
    },

    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        ty: Ty,
    },

    Call {
        callee: String,
        args: Vec<Expr>,
        ty: Ty,
    },
}

impl Expr {
    pub fn ty(&self) -> &Ty {
        match self {
            Self::IntLit { ty, .. }
            | Self::Var { ty, .. }
            | Self::BinOp { ty, .. }
            | Self::BoolLit { ty, .. }
            | Self::Unary { ty, .. }
            | Self::Call { ty, .. }
            | Self::StrLit { ty, .. } => ty,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expr),

    ReturnVoid,
    VarDecl {
        name: String,
        slot: StackSlot,
        ty: Ty,
        init: Option<Expr>,
    },
    Assign {
        slot: StackSlot,
        value: Expr,
    },
    ExprStmt(Expr),
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: String,
    pub exported: bool,
    pub ret_ty: Ty,
    pub frame: FrameLayout,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Item {
    FuncDef(FuncDef),
}

#[derive(Debug)]
pub struct Script {
    pub items: Vec<Item>,
}
