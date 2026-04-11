use crate::error::{SemaError, SemaResult};
use fsc_parse::ast::{Expr, FuncDef, Stmt, Ty};
use std::collections::HashMap;

pub struct ScopeStack {
    scopes: Vec<HashMap<String, Ty>>,
}

impl ScopeStack {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }
    // unused as long as there is only one block
    pub fn _push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn _pop_scope(&mut self) {
        debug_assert!(self.scopes.len() > 1, "cannot pop the outermost scope");
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: &str, ty: Ty) -> SemaResult<()> {
        let current = self.scopes.last_mut();
        let Some(map) = current else {
            unreachable!("unreachable initialized with at least one scope")
        };
        if map.contains_key(name) {
            return Err(SemaError::DuplicateDeclaration(name.to_string()));
        }
        map.insert(name.to_string(), ty);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> SemaResult<&Ty> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(ty);
            }
        }
        Err(SemaError::UndeclaredName(name.to_string()))
    }
}

pub fn resolve_func(func: &FuncDef) -> SemaResult<ScopeStack> {
    let mut scope = ScopeStack::new();

    for param in &func.params {
        scope.declare(&param.name, param.ty.clone())?;
    }

    resolve_stmts(&func.body, &mut scope)?;

    Ok(scope)
}

fn resolve_stmts(stmts: &[Stmt], scope: &mut ScopeStack) -> SemaResult<()> {
    for stmt in stmts {
        resolve_stmt(stmt, scope)?;
    }
    Ok(())
}

fn resolve_stmt(stmt: &Stmt, scope: &mut ScopeStack) -> SemaResult<()> {
    match stmt {
        Stmt::Return(expr) => match expr {
            Some(expr) => resolve_expr(expr, scope),
            None => Ok(()),
        },

        Stmt::VarDecl { name, ty, init } => {
            if let Some(expr) = init {
                resolve_expr(expr, scope)?;
            }
            scope.declare(name, ty.clone())?;
            Ok(())
        }
        Stmt::Assign { name, expr } => {
            scope.lookup(name)?;
            resolve_expr(expr, scope)
        }
    }
}

fn resolve_expr(expr: &Expr, scope: &ScopeStack) -> SemaResult<()> {
    match expr {
        Expr::IntLit(_) => Ok(()),

        Expr::Var(name) => {
            scope.lookup(name)?;
            Ok(())
        }

        Expr::BinOp { lhs, rhs, .. } => {
            resolve_expr(lhs, scope)?;
            resolve_expr(rhs, scope)
        }
    }
}
