#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

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
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    IntLit(i32),
    BoolLit(bool),

    Var(String),

    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
}

impl Expr {
    pub fn new(id: NodeId, kind: ExprKind) -> Self {
        Self { id, kind }
    }
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
}
#[derive(Debug, Clone)]
pub enum StmtKind {
    Return(Option<Expr>),
    VarDecl {
        name: String,
        ty: Ty,
        init: Option<Expr>,
    },
    Assign {
        target: Expr,
        expr: Expr,
    },
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
}

impl Stmt {
    pub fn new(id: NodeId, kind: StmtKind) -> Self {
        Self { id, kind }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub id: NodeId,
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
