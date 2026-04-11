use crate::frame::{FrameLayout, StackSlot};
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

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit {
        value: i32,
        ty: Ty,
    },

    Var {
        name: String,
        slot: StackSlot,
        ty: Ty,
    },

    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        ty: Ty,
    },
}

impl Expr {
    pub fn ty(&self) -> &Ty {
        match self {
            Self::IntLit { ty, .. } => ty,
            Self::Var { ty, .. } => ty,
            Self::BinOp { ty, .. } => ty,
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
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: String,
    pub exported: bool,
    pub ret_ty: Ty,
    pub frame: FrameLayout,
    pub body: Vec<Stmt>,
}
