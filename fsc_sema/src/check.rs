use crate::error::{SemaError, SemaResult};
use crate::infer;
use crate::resolve::Resolutions;
use crate::symbol::SymbolTable;
use fsc_diagnostics::Span;
use fsc_parse::ast::{self, Ty};

pub fn check_fn(
    func: &ast::FuncDef,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
) -> SemaResult<()> {
    check_stmts(
        &func.body,
        &func.header.ret_ty,
        func.header.ret_ty_span,
        resolutions,
        symbols,
    )
}

fn check_stmts(
    stmts: &[ast::Stmt],
    ret_ty: &Ty,
    ret_ty_span: Span,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
) -> SemaResult<()> {
    for stmt in stmts {
        check_stmt(stmt, ret_ty, ret_ty_span, resolutions, symbols)?;
    }
    Ok(())
}

fn check_stmt(
    stmt: &ast::Stmt,
    ret_ty: &Ty,
    ret_ty_span: Span,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
) -> SemaResult<()> {
    match &stmt.kind {
        ast::StmtKind::Return(expr) => match expr {
            Some(e) => {
                let found = infer::infer_expr(e, resolutions, symbols)?;
                check_return(ret_ty, &found, e.span, ret_ty_span)
            }
            None => check_void_return(ret_ty, stmt.span, ret_ty_span),
        },
        ast::StmtKind::Break => {
            //TODO: check is in loop
            Ok(())
        }

        ast::StmtKind::VarDecl {
            ty, ty_span, init, ..
        } => {
            if let Some(e) = init {
                let found = infer::infer_expr(e, resolutions, symbols)?;
                check_assignable(ty, &found, e.span, Some(*ty_span))?;
            }
            Ok(())
        }

        ast::StmtKind::Assign { target, expr } => {
            let sym_id = resolutions.symbol(target.id);
            let symbol = symbols.get(sym_id);
            match symbol.kind {
                crate::symbol::SymbolKind::Const { .. } => {
                    return Err(SemaError::AssignmentToConstant {
                        name: symbol.name.clone(),
                        assignment_span: target.span,
                        declaration_span: symbol.name_span,
                    });
                }
                crate::symbol::SymbolKind::Config => {
                    return Err(SemaError::AssignmentToConfig {
                        name: symbol.name.clone(),
                        assignment_span: target.span,
                        declaration_span: symbol.name_span,
                    });
                }
                _ => {}
            }
            let decl_ty = &symbol.ty;
            let found = infer::infer_expr(expr, resolutions, symbols)?;
            check_assignable(decl_ty, &found, expr.span, Some(symbol.type_span))
        }

        ast::StmtKind::If {
            cond,
            then_body,
            else_body,
        } => {
            let cond_ty = infer::infer_expr(cond, resolutions, symbols)?;
            check_condition(&cond_ty, cond.span)?;
            check_stmts(then_body, ret_ty, ret_ty_span, resolutions, symbols)?;
            if let Some(else_stmts) = else_body {
                check_stmts(else_stmts, ret_ty, ret_ty_span, resolutions, symbols)?;
            }
            Ok(())
        }

        ast::StmtKind::While { cond, body } => {
            let cond_ty = infer::infer_expr(cond, resolutions, symbols)?;
            check_condition(&cond_ty, cond.span)?;

            check_stmts(body, ret_ty, ret_ty_span, resolutions, symbols)?;
            Ok(())
        }
        ast::StmtKind::ExprStmt(expr) => {
            let ty = infer::infer_expr(expr, resolutions, symbols)?;
            if ty != Ty::Void {
                if !matches!(expr.kind, ast::ExprKind::Call { .. }) {
                    // TODO: emit warning: "expression result unused"
                }
            }
            Ok(())
        }
        ast::StmtKind::Pause(expr) => {
            let ty = infer::infer_expr(expr, resolutions, symbols)?;
            if ty != Ty::Int {
                return Err(SemaError::TypeMismatch {
                    expected: Ty::Int,
                    found: ty.clone(),
                    span: expr.span,
                    expected_span: None,
                });
            }
            Ok(())
        }
    }
}
//TODO: check Syscall

pub fn check_assignable(
    declared: &Ty,
    found: &Ty,
    span: Span,
    expected_span: Option<Span>,
) -> SemaResult<()> {
    if declared != found {
        return Err(SemaError::TypeMismatch {
            expected: declared.clone(),
            found: found.clone(),
            span,
            expected_span,
        });
    }
    Ok(())
}

pub fn check_return(ret_ty: &Ty, found: &Ty, span: Span, return_type_span: Span) -> SemaResult<()> {
    if ret_ty != found {
        return Err(SemaError::ReturnTypeMismatch {
            expected: ret_ty.clone(),
            found: found.clone(),
            span,
            return_type_span,
        });
    }
    Ok(())
}

pub fn check_void_return(ret_ty: &Ty, span: Span, return_type_span: Span) -> SemaResult<()> {
    if *ret_ty != Ty::Void {
        return Err(SemaError::ReturnTypeMismatch {
            expected: ret_ty.clone(),
            found: Ty::Void,
            span,
            return_type_span,
        });
    }
    Ok(())
}

pub fn check_condition(ty: &Ty, span: Span) -> SemaResult<()> {
    if *ty == Ty::Void {
        return Err(SemaError::VoidInValuePosition { span });
    }
    //TODO: check for boolean
    Ok(())
}
