mod error;

use crate::check::error::{TypeCheckError, TypeCheckResult};
use fsc_parse::ast::{Expr, FuncDef, Stmt, Ty};
use std::collections::HashMap;

//TODO: just a simple typecheck for prototyping right now; should be extended prob separate crate
// later on

//TODO: casting to typed AST would make codegen checks easier
struct CheckCtx<'a> {
    vars: HashMap<&'a str, Ty>,
    ret_ty: Ty,
}
impl<'a> CheckCtx<'a> {
    pub fn new(func: &'a FuncDef) -> Self {
        let mut vars = HashMap::new();

        for param in &func.params {
            vars.insert(param.name.as_str(), param.ty.clone());
        }

        Self {
            vars,
            ret_ty: func.ret_ty.clone(),
        }
    }

    pub fn resolve_ty(&self, name: &str) -> TypeCheckResult<Ty> {
        self.vars
            .get(name)
            .cloned()
            .ok_or(TypeCheckError::UnknownVar(name.to_string()))
    }
}

pub fn check_func(func: &FuncDef) -> TypeCheckResult<()> {
    let mut cx = CheckCtx::new(func);
    for stmt in &func.body {
        check_stmt(stmt, &mut cx)?;
    }
    Ok(())
}

fn check_stmt(stmt: &Stmt, cx: &mut CheckCtx) -> TypeCheckResult<()> {
    match stmt {
        Stmt::Return(expr) => {
            match expr {
                Some(e) => {
                    let ty = check_expr(e, cx)?;

                    if ty != cx.ret_ty {
                        return Err(TypeCheckError::TypeMismatch {
                            expected: cx.ret_ty.clone(),
                            found: ty,
                        });
                    }
                }
                None => {
                    if cx.ret_ty != Ty::Void {
                        return Err(TypeCheckError::MissingReturnValue);
                    }
                }
            }

            Ok(())
        }
    }
}

fn check_expr(expr: &Expr, cx: &CheckCtx) -> TypeCheckResult<Ty> {
    match expr {
        Expr::IntLit(_) => Ok(Ty::Int),

        Expr::Var(name) => cx.resolve_ty(name),

        Expr::BinOp { op: _, lhs, rhs } => {
            let lty = check_expr(lhs, cx)?;
            let rty = check_expr(rhs, cx)?;

            if lty != rty {
                return Err(TypeCheckError::TypeMismatch {
                    expected: lty,
                    found: rty,
                });
            }

            Ok(lty)
        }
    }
}
