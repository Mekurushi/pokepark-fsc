use crate::error::{SemaError, SemaResult};
use crate::infer::infer_expr;
use crate::resolve::ScopeStack;
use fsc_parse::ast::{Expr, FuncDef, Stmt, Ty};

// TODO: forbid instructions after return or missing returns

pub fn check_func(func: &FuncDef, scope: &ScopeStack) -> SemaResult<()> {
    for stmt in &func.body {
        check_stmt(stmt, &func.ret_ty, scope)?;
    }
    Ok(())
}

fn check_stmt(stmt: &Stmt, ret_ty: &Ty, scope: &ScopeStack) -> SemaResult<()> {
    match stmt {
        Stmt::Return(expr) => match expr {
            Some(expr) => check_return(expr, ret_ty, scope),
            None => {
                if *ret_ty == Ty::Void {
                    Ok(())
                } else {
                    Err(SemaError::ReturnTypeMismatch {
                        expected: ret_ty.clone(),
                        found: Ty::Void,
                    })
                }
            }
        },

        Stmt::Assign { name, expr } => {
            let decl_ty = scope.lookup(name)?;
            let found = infer_expr(expr, scope)?;
            check_assignable(decl_ty, &found)
        }
        Stmt::VarDecl {
            name: _name,
            ty,
            init,
        } => match init {
            Some(expr) => {
                let found = infer_expr(expr, scope)?;
                check_assignable(ty, &found)
            }
            None => Ok(()),
        },
    }
}

fn check_assignable(decl_ty: &Ty, infered_ty: &Ty) -> SemaResult<()> {
    if decl_ty == infered_ty {
        return Ok(());
    }
    Err(SemaError::TypeMismatch {
        expected: decl_ty.clone(),
        found: infered_ty.clone(),
    })
}

fn check_return(expr: &Expr, ret_ty: &Ty, scope: &ScopeStack) -> SemaResult<()> {
    let found = infer_expr(expr, scope)?;
    if &found != ret_ty {
        return Err(SemaError::ReturnTypeMismatch {
            expected: ret_ty.clone(),
            found,
        });
    }
    Ok(())
}
