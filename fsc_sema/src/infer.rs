use crate::check::check_assignable;
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
        ExprKind::Call {
            args, callee_span, ..
        } => infer_call(expr, args, *callee_span, resolved),
        // TODO: check this again; always Int and caller interprets?
        ExprKind::SysCall { .. } => Ok(Ty::Int),
    }
}

fn infer_call(
    expr: &Expr,
    args: &[Expr],
    callee_span: fsc_diagnostics::Span,
    resolved: &ResolveOutput,
) -> SemaResult<Ty> {
    let sym_id = resolved.resolutions.symbol(expr.id);
    let symbol = resolved.symbols.get(sym_id);
    let SymbolKind::Function { ret_ty, params } = &symbol.kind else {
        return Err(SemaError::NotCallable {
            name: symbol.name.clone(),
            callee_span,
            declaration_span: symbol.name_span,
        });
    };

    if args.len() != params.len() {
        return Err(SemaError::ArgumentCountMismatch {
            name: symbol.name.clone(),
            expected: params.len(),
            found: args.len(),
            call_span: expr.span,
            declaration_span: symbol.name_span,
        });
    }

    for (argument, parameter) in args.iter().zip(params) {
        let found = infer_expr(argument, resolved)?;
        check_assignable(
            &parameter.ty,
            &found,
            argument.span,
            Some(parameter.type_span),
        )?;
    }

    Ok(ret_ty.clone())
}

fn infer_binop(op: &BinOp, lhs: &Expr, rhs: &Expr, resolved: &ResolveOutput) -> SemaResult<Ty> {
    let lty = infer_expr(lhs, resolved)?;
    let rty = infer_expr(rhs, resolved)?;

    if lty == Ty::Void || rty == Ty::Void {
        return Err(SemaError::VoidInValuePosition {
            span: if lty == Ty::Void { lhs.span } else { rhs.span },
        });
    }
    if lty != rty {
        return Err(SemaError::TypeMismatch {
            expected: lty,
            found: rty,
            span: rhs.span,
            expected_span: Some(lhs.span),
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
                    span: expr.span,
                    expected_span: None,
                });
            }
            Ok(ty)
        }
        UnaryOp::Not => {
            let ty = infer_expr(expr, resolved)?;
            if ty == Ty::Void {
                return Err(SemaError::VoidInValuePosition { span: expr.span });
            }
            Ok(Ty::Bool)
        }
    }
}
