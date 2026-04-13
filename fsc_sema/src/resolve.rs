use crate::error::{SemaError, SemaResult};
use crate::symbol::{Symbol, SymbolId, SymbolKind, SymbolTable};
use fsc_parse::ast;
use fsc_parse::ast::NodeId;
use std::collections::HashMap;

pub struct Resolutions {
    references: HashMap<NodeId, SymbolId>,
}

impl Resolutions {
    fn new() -> Self {
        Self {
            references: HashMap::new(),
        }
    }

    fn insert(&mut self, node: NodeId, sym: SymbolId) {
        self.references.insert(node, sym);
    }
    pub fn symbol(&self, node: NodeId) -> SymbolId {
        let sym_id = self.references.get(&node);
        match sym_id {
            Some(sym_id) => *sym_id,
            None => todo!("Symbol not found"), //TODO: explicit error
        }
    }
}

pub struct ResolveOutput {
    pub symbols: SymbolTable,
    pub resolutions: Resolutions,
    pub params_in_order: Vec<SymbolId>,
}

struct ScopeStack {
    scopes: Vec<HashMap<String, SymbolId>>,
}

impl ScopeStack {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop(&mut self) {
        debug_assert!(self.scopes.len() > 1, "cannot pop the outermost scope");
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, sym_id: SymbolId) -> SemaResult<()> {
        let current = self.scopes.last_mut();
        let Some(map) = current else {
            unreachable!("unreachable initialized with at least one scope")
        };
        if map.contains_key(name) {
            return Err(SemaError::DuplicateDeclaration(name.to_string()));
        }
        map.insert(name.to_string(), sym_id);
        Ok(())
    }

    fn lookup(&self, name: &str) -> SemaResult<SymbolId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Ok(id);
            }
        }
        Err(SemaError::UndeclaredName(name.to_string()))
    }
}

pub fn resolve_fn(func: &ast::FuncDef) -> SemaResult<ResolveOutput> {
    let mut symbols = SymbolTable::new();
    let mut resolutions = Resolutions::new();
    let mut scope = ScopeStack::new();

    let mut params: Vec<SymbolId> = Vec::new();
    for (index, param) in func.params.iter().enumerate() {
        let sym_id = symbols.insert(Symbol {
            name: param.name.clone(),
            ty: param.ty.clone(),
            kind: SymbolKind::Param {
                // TODO: unused, original idea not used check if this can be removed
                index: index as u32,
            },
        });
        scope.declare(&param.name, sym_id)?;
        params.push(sym_id);
    }

    resolve_stmts(&func.body, &mut scope, &mut symbols, &mut resolutions)?;

    Ok(ResolveOutput {
        symbols,
        resolutions,
        params_in_order: params,
    })
}

fn resolve_stmts(
    stmts: &[ast::Stmt],
    scope: &mut ScopeStack,
    symbols: &mut SymbolTable,
    resolutions: &mut Resolutions,
) -> SemaResult<()> {
    for stmt in stmts {
        resolve_stmt(stmt, scope, symbols, resolutions)?;
    }
    Ok(())
}

fn resolve_stmt(
    stmt: &ast::Stmt,
    scope: &mut ScopeStack,
    symbols: &mut SymbolTable,
    resolutions: &mut Resolutions,
) -> SemaResult<()> {
    match &stmt.kind {
        ast::StmtKind::Return(expr) => {
            if let Some(e) = expr {
                resolve_expr(e, scope, resolutions)?;
            }
            Ok(())
        }

        ast::StmtKind::VarDecl { name, ty, init } => {
            if let Some(e) = init {
                resolve_expr(e, scope, resolutions)?;
            }
            let sym_id = symbols.insert(Symbol {
                name: name.clone(),
                ty: ty.clone(),
                kind: SymbolKind::Local,
            });
            scope.declare(name, sym_id)?;
            resolutions.insert(stmt.id, sym_id);
            Ok(())
        }

        ast::StmtKind::Assign { target, expr } => {
            resolve_expr(target, scope, resolutions)?;
            resolve_expr(expr, scope, resolutions)?;
            Ok(())
        }

        ast::StmtKind::If {
            cond,
            then_body,
            else_body,
        } => {
            resolve_expr(cond, scope, resolutions)?;

            scope.push();
            resolve_stmts(then_body, scope, symbols, resolutions)?;
            scope.pop();

            if let Some(else_stmts) = else_body {
                scope.push();
                resolve_stmts(else_stmts, scope, symbols, resolutions)?;
                scope.pop();
            }
            Ok(())
        }
    }
}

fn resolve_expr(
    expr: &ast::Expr,
    scope: &ScopeStack,
    resolutions: &mut Resolutions,
) -> SemaResult<()> {
    match &expr.kind {
        ast::ExprKind::IntLit(_) | ast::ExprKind::BoolLit(_) => Ok(()),

        ast::ExprKind::Var(name) => {
            let sym_id = scope.lookup(name)?;
            resolutions.insert(expr.id, sym_id);
            Ok(())
        }

        ast::ExprKind::BinOp { lhs, rhs, .. } => {
            resolve_expr(lhs, scope, resolutions)?;
            resolve_expr(rhs, scope, resolutions)
        }

        ast::ExprKind::Unary { expr: inner, .. } => resolve_expr(inner, scope, resolutions),
    }
}
