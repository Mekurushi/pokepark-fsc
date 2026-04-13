use crate::error::{SemaError, SemaResult};
use crate::infer;
use crate::resolve::ResolveOutput;
use fsc_parse::ast::{self, Ty};

pub fn check_fn(func: &ast::FuncDef, resolved: &ResolveOutput) -> SemaResult<()> {
    check_stmts(&func.body, &func.ret_ty, resolved)
}

fn check_stmts(stmts: &[ast::Stmt], ret_ty: &Ty, resolved: &ResolveOutput) -> SemaResult<()> {
    for stmt in stmts {
        check_stmt(stmt, ret_ty, resolved)?;
    }
    Ok(())
}

fn check_stmt(stmt: &ast::Stmt, ret_ty: &Ty, resolved: &ResolveOutput) -> SemaResult<()> {
    match &stmt.kind {
        ast::StmtKind::Return(expr) => match expr {
            Some(e) => {
                let found = infer::infer_expr(e, resolved)?;
                check_return(ret_ty, &found)
            }
            None => check_void_return(ret_ty),
        },

        ast::StmtKind::VarDecl { ty, init, .. } => {
            if let Some(e) = init {
                let found = infer::infer_expr(e, resolved)?;
                check_assignable(ty, &found)?;
            }
            Ok(())
        }

        ast::StmtKind::Assign { target, expr } => {
            let sym_id = resolved.resolutions.symbol(target.id);
            let decl_ty = &resolved.symbols.get(sym_id).ty;
            let found = infer::infer_expr(expr, resolved)?;
            check_assignable(decl_ty, &found)
        }

        ast::StmtKind::If {
            cond,
            then_body,
            else_body,
        } => {
            let cond_ty = infer::infer_expr(cond, resolved)?;
            check_condition(&cond_ty)?;
            check_stmts(then_body, ret_ty, resolved)?;
            if let Some(else_stmts) = else_body {
                check_stmts(else_stmts, ret_ty, resolved)?;
            }
            Ok(())
        }
    }
}

pub fn check_assignable(declared: &Ty, found: &Ty) -> SemaResult<()> {
    if declared != found {
        return Err(SemaError::TypeMismatch {
            expected: declared.clone(),
            found: found.clone(),
        });
    }
    Ok(())
}

pub fn check_return(ret_ty: &Ty, found: &Ty) -> SemaResult<()> {
    if ret_ty != found {
        return Err(SemaError::ReturnTypeMismatch {
            expected: ret_ty.clone(),
            found: found.clone(),
        });
    }
    Ok(())
}

pub fn check_void_return(ret_ty: &Ty) -> SemaResult<()> {
    if *ret_ty != Ty::Void {
        return Err(SemaError::ReturnTypeMismatch {
            expected: ret_ty.clone(),
            found: Ty::Void,
        });
    }
    Ok(())
}

pub fn check_condition(ty: &Ty) -> SemaResult<()> {
    if *ty == Ty::Void {
        return Err(SemaError::VoidInValuePosition);
    }
    //TODO: check for boolean
    Ok(())
}
