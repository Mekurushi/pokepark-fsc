use crate::error::{SemaError, SemaResult};
use crate::resolve::ResolveOutput;
use crate::symbol::SymbolKind;
use fsc_parse::ast::{BinOp, Expr, ExprKind, Ty, UnaryOp};

pub fn infer_expr(expr: &Expr, resolved: &ResolveOutput) -> SemaResult<Ty> {
    match &expr.kind {
        ExprKind::IntLit(_) => Ok(Ty::Int),
        ExprKind::BoolLit(_) => Ok(Ty::Bool),
        ExprKind::StringLit(_) => Ok(Ty::Str),

        ExprKind::Var(_) => {
            let sym_id = resolved.resolutions.symbol(expr.id);
            Ok(resolved.symbols.get(sym_id).ty.clone())
        }
        ExprKind::BinOp { op, lhs, rhs } => infer_binop(op, lhs, rhs, resolved),

        ExprKind::Unary { op, expr } => infer_unary(op, expr, resolved),
        ExprKind::Call { .. } => {
            let sym_id = resolved.resolutions.symbol(expr.id);
            let sym = resolved.symbols.get(sym_id);
            match &sym.kind {
                SymbolKind::Function { ret_ty, .. } => Ok(ret_ty.clone()),
                _ => Err(SemaError::NotCallable(sym.name.clone())),
            }
        }
        // TODO: check this again; always Int and caller interprets?
        ExprKind::SysCall { .. } => Ok(Ty::Int),
    }
}
fn infer_binop(op: &BinOp, lhs: &Expr, rhs: &Expr, resolved: &ResolveOutput) -> SemaResult<Ty> {
    let lty = infer_expr(lhs, resolved)?;
    let rty = infer_expr(rhs, resolved)?;

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
        | BinOp::Lt
        | BinOp::Gt
        | BinOp::Le
        | BinOp::Ge
        | BinOp::And
        | BinOp::Or => Ok(Ty::Bool),
    }
}
fn infer_unary(op: &UnaryOp, expr: &Expr, resolved: &ResolveOutput) -> SemaResult<Ty> {
    match op {
        UnaryOp::Neg => {
            let ty = infer_expr(expr, resolved)?;
            if ty != Ty::Int {
                return Err(SemaError::TypeMismatch {
                    expected: Ty::Int,
                    found: ty,
                });
            }
            Ok(ty)
        }
        UnaryOp::Not => {
            let ty = infer_expr(expr, resolved)?;
            if ty == Ty::Void {
                return Err(SemaError::VoidInValuePosition);
            }
            Ok(Ty::Bool)
        }
    }
}
