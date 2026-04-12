use crate::error::{SemaError, SemaResult};
use crate::resolve::ScopeStack;
use fsc_parse::ast::{BinOp, Expr, Ty, UnaryOp};

pub fn infer_expr(expr: &Expr, scope: &ScopeStack) -> SemaResult<Ty> {
    match expr {
        Expr::IntLit(_) => Ok(Ty::Int),
        Expr::BoolLit(_) => Ok(Ty::Bool),

        Expr::Var(name) => scope.lookup(name).cloned(),

        Expr::BinOp { op, lhs, rhs } => infer_binop(op, lhs, rhs, scope),

        Expr::Unary { op, expr } => infer_unary(op, expr, scope),
    }
}
fn infer_unary(op: &UnaryOp, expr: &Expr, scope: &ScopeStack) -> SemaResult<Ty> {
    match op {
        UnaryOp::Neg => {
            let ty = infer_expr(expr, scope)?;
            if ty != Ty::Int {
                return Err(SemaError::TypeMismatch {
                    expected: Ty::Int,
                    found: ty,
                });
            }
            Ok(ty)
        }
        UnaryOp::Not => {
            let ty = infer_expr(expr, scope)?;
            if ty == Ty::Void {
                return Err(SemaError::VoidInValuePosition);
            }
            Ok(Ty::Bool)
        }
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
        BinOp::Eq
        | BinOp::Neq
        | BinOp::And
        | BinOp::Or
        | BinOp::Ge
        | BinOp::Gt
        | BinOp::Le
        | BinOp::Lt => Ok(Ty::Bool),
    }
}
