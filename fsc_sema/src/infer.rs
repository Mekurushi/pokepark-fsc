use crate::error::{SemaError, SemaResult};
use crate::resolve::ScopeStack;
use fsc_parse::ast::{BinOp, Expr, Ty};

pub fn infer_expr(expr: &Expr, scope: &ScopeStack) -> SemaResult<Ty> {
    match expr {
        Expr::IntLit(_) => Ok(Ty::Int),

        Expr::Var(name) => scope.lookup(name).cloned(),

        Expr::BinOp { op, lhs, rhs } => infer_binop(op, lhs, rhs, scope),
    }
}
fn infer_binop(op: &BinOp, lhs: &Expr, rhs: &Expr, scope: &ScopeStack) -> SemaResult<Ty> {
    let lty = infer_expr(lhs, scope)?;
    let rty = infer_expr(rhs, scope)?;
    //TODO: feels kinda like mixed concern; rethink if this need restructure
    if lty == Ty::Void || rty == Ty::Void {
        return Err(SemaError::VoidInValuePosition);
    }
    if lty != rty {
        return Err(SemaError::TypeMismatch {
            expected: lty,
            found: rty,
        });
    }
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => Ok(lty),
    }
}
