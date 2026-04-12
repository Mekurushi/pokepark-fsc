use crate::error::SemaResult;
use crate::frame::FrameLayout;
use crate::hir;
use crate::infer;
use crate::infer::infer_expr;
use crate::resolve::ScopeStack;
use fsc_parse::ast::{self};

// TODO: short-circuit

pub fn lower_func(
    func: &ast::FuncDef,
    frame: FrameLayout,
    scope: &ScopeStack,
) -> SemaResult<hir::FuncDef> {
    let body = lower_stmts(&func.body, &frame, scope)?;

    Ok(hir::FuncDef {
        name: func.name.clone(),
        exported: func.exported,
        ret_ty: lower_ty(&func.ret_ty),
        frame,
        body,
    })
}

fn lower_stmts(
    stmts: &[ast::Stmt],
    frame: &FrameLayout,
    scope: &ScopeStack,
) -> SemaResult<Vec<hir::Stmt>> {
    stmts.iter().map(|s| lower_stmt(s, frame, scope)).collect()
}

fn lower_stmt(stmt: &ast::Stmt, frame: &FrameLayout, scope: &ScopeStack) -> SemaResult<hir::Stmt> {
    match stmt {
        ast::Stmt::Return(expr) => match expr {
            Some(e) => Ok(hir::Stmt::Return(lower_expr(e, frame, scope)?)),
            None => Ok(hir::Stmt::ReturnVoid),
        },
        ast::Stmt::VarDecl { name, ty, init } => {
            let slot = frame.resolve(name)?;
            let init = init
                .as_ref()
                .map(|e| lower_expr(e, frame, scope))
                .transpose()?;
            Ok(hir::Stmt::VarDecl {
                name: name.clone(),
                slot,
                ty: lower_ty(ty),
                init,
            })
        }
        ast::Stmt::Assign { name, expr } => {
            let slot = frame.resolve(name)?;
            Ok(hir::Stmt::Assign {
                slot,
                value: lower_expr(expr, frame, scope)?,
            })
        }
    }
}

fn lower_expr(expr: &ast::Expr, frame: &FrameLayout, scope: &ScopeStack) -> SemaResult<hir::Expr> {
    match expr {
        ast::Expr::IntLit(value) => Ok(hir::Expr::IntLit {
            value: *value,
            ty: lower_ty(&ast::Ty::Int),
        }),

        ast::Expr::BoolLit(value) => Ok(hir::Expr::BoolLit {
            value: *value,
            ty: lower_ty(&ast::Ty::Bool),
        }),

        ast::Expr::Var(name) => {
            let slot = frame.resolve(name)?;
            let ty = scope.lookup(name)?.clone();

            Ok(hir::Expr::Var {
                name: name.clone(),
                slot,
                ty: lower_ty(&ty),
            })
        }

        ast::Expr::Unary {
            op,
            expr: expression,
        } => {
            let ty = infer_expr(expr, scope)?;
            Ok(hir::Expr::Unary {
                op: lower_unaryop(op),
                expr: Box::new(lower_expr(expression, frame, scope)?),
                ty: lower_ty(&ty),
            })
        }

        ast::Expr::BinOp { op, lhs, rhs } => {
            let ty = infer::infer_expr(expr, scope)?;

            Ok(hir::Expr::BinOp {
                op: lower_binop(op),
                lhs: Box::new(lower_expr(lhs, frame, scope)?),
                rhs: Box::new(lower_expr(rhs, frame, scope)?),
                ty: lower_ty(&ty),
            })
        }
    }
}

fn lower_ty(ty: &ast::Ty) -> hir::Ty {
    match ty {
        ast::Ty::Int => hir::Ty::Int,
        ast::Ty::Void => hir::Ty::Void,
        ast::Ty::Bool => hir::Ty::Bool,
    }
}

fn lower_unaryop(op: &ast::UnaryOp) -> hir::UnaryOp {
    match op {
        ast::UnaryOp::Not => hir::UnaryOp::Not,
        ast::UnaryOp::Neg => hir::UnaryOp::Neg,
    }
}

fn lower_binop(op: &ast::BinOp) -> hir::BinOp {
    match op {
        ast::BinOp::Add => hir::BinOp::Add,
        ast::BinOp::Sub => hir::BinOp::Sub,
        ast::BinOp::Mul => hir::BinOp::Mul,
        ast::BinOp::Div => hir::BinOp::Div,

        // comparison
        ast::BinOp::Lt => hir::BinOp::Lt,
        ast::BinOp::Gt => hir::BinOp::Gt,
        ast::BinOp::Le => hir::BinOp::Le,
        ast::BinOp::Ge => hir::BinOp::Ge,
        ast::BinOp::Eq => hir::BinOp::Eq,
        ast::BinOp::Neq => hir::BinOp::Neq,
        // logical
        ast::BinOp::And => hir::BinOp::And,
        ast::BinOp::Or => hir::BinOp::Or,
    }
}
