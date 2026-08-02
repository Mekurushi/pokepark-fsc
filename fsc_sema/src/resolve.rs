use crate::error::{SemaError, SemaResult};
use crate::symbol::{ParamInfo, Symbol, SymbolId, SymbolKind, SymbolTable};
use fsc_diagnostics::Span;
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
    pub symbols: SymbolTable, // TODO: borrow is probably better now
    pub resolutions: Resolutions,
    pub params_in_order: Vec<SymbolId>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    File,
    Function,
    Block,
}
#[derive(Debug)]
struct Scope {
    kind: ScopeKind,
    map: HashMap<String, ScopeEntry>,
}
#[derive(Debug, Clone, Copy)]
struct ScopeEntry {
    symbol: SymbolId,
    declaration_span: Span,
}
#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

// TODO: check assertions; currently more development check than state enforcements
impl ScopeStack {
    pub(crate) fn new() -> Self {
        Self { scopes: vec![] }
    }
    fn push_scope(&mut self, kind: ScopeKind) {
        self.scopes.push(Scope {
            kind,
            map: HashMap::new(),
        });
    }
    fn current_kind(&self) -> Option<ScopeKind> {
        self.scopes.last().map(|s| s.kind)
    }

    fn enter_file_scope(&mut self) {
        assert!(
            self.scopes.is_empty(),
            "file scope already \
        initialized"
        );

        self.push_scope(ScopeKind::File);
    }

    fn enter_function_scope(&mut self) {
        assert_eq!(
            self.current_kind(),
            Some(ScopeKind::File),
            "function scope must be entered from file scope"
        );
        self.push_scope(ScopeKind::Function);
    }

    fn enter_block_scope(&mut self) {
        assert!(
            matches!(
                self.current_kind(),
                Some(ScopeKind::Function | ScopeKind::Block)
            ),
            "block scope must be inside function or block"
        );
        self.push_scope(ScopeKind::Block);
    }

    fn exit_scope(&mut self) {
        assert!(!self.scopes.is_empty(), "cannot exit empty scope stack");
        assert_ne!(
            self.current_kind(),
            Some(ScopeKind::File),
            "cannot exit root scope"
        );

        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, sym_id: SymbolId, name_span: Span) -> SemaResult<()> {
        let current = self.scopes.last_mut();
        let Some(scope) = current else {
            unreachable!("unreachable initialized with at least one scope")
        };
        if let Some(original) = scope.map.get(name) {
            return Err(SemaError::DuplicateDeclaration {
                name: name.to_string(),
                duplicate_span: name_span,
                original_span: original.declaration_span,
            });
        }
        scope.map.insert(
            name.to_string(),
            ScopeEntry {
                symbol: sym_id,
                declaration_span: name_span,
            },
        );
        Ok(())
    }

    fn lookup(&self, name: &str, reference_span: Span) -> SemaResult<SymbolId> {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.map.get(name) {
                return Ok(entry.symbol);
            }
        }
        Err(SemaError::UndeclaredName {
            name: name.to_string(),
            reference_span,
        })
    }
}

pub fn declare_items(
    script: &ast::Script,
    scope: &mut ScopeStack,
    symbols: &mut SymbolTable,
) -> SemaResult<()> {
    scope.enter_file_scope();
    for item in &script.items {
        if let ast::Item::FuncDef(func) = item {
            let mut params: Vec<ParamInfo> = Vec::new();
            for param in func.params.iter() {
                params.push(ParamInfo {
                    name: param.name.clone(),
                    name_span: param.name_span,
                    ty: param.ty.clone(),
                    type_span: param.ty_span,
                });
            }
            let fn_sym_id = symbols.insert(Symbol {
                name: func.name.clone(),
                name_span: func.name_span,
                ty: func.ret_ty.clone(),
                type_span: func.ret_ty_span,
                kind: SymbolKind::Function {
                    ret_ty: func.ret_ty.clone(),
                    params,
                },
            });
            scope.declare(&func.name, fn_sym_id, func.name_span)?;
        }
    }
    Ok(())
}

pub fn resolve_params(
    params: &[ParamInfo],
    scope: &mut ScopeStack,
    symbol_table: &mut SymbolTable,
) -> SemaResult<Vec<SymbolId>> {
    let mut params_in_order: Vec<SymbolId> = Vec::new();
    for (index, param) in params.iter().enumerate() {
        let sym_id = symbol_table.insert(Symbol {
            name: param.name.clone(),
            name_span: param.name_span,
            ty: param.ty.clone(),
            type_span: param.type_span,
            kind: SymbolKind::Param {
                index: index as u32,
            },
        });
        scope.declare(&param.name, sym_id, param.name_span)?;
        params_in_order.push(sym_id);
    }
    Ok(params_in_order)
}

pub fn resolve_fn(
    func: &ast::FuncDef,
    symbols: &mut SymbolTable,
    scope: &mut ScopeStack,
) -> SemaResult<ResolveOutput> {
    scope.enter_function_scope();
    let mut resolutions = Resolutions::new();
    let fn_symbol_id = scope.lookup(&func.name, func.name_span)?;
    let (params, ..) = match symbols.get(fn_symbol_id).kind.clone() {
        SymbolKind::Function { params, ret_ty } => (params, ret_ty),
        _ => todo!(),
    };
    let params_in_order = resolve_params(&params, scope, symbols)?;

    resolve_stmts(&func.body, scope, symbols, &mut resolutions)?;
    let symbol_table = symbols.clone();
    scope.exit_scope();
    Ok(ResolveOutput {
        symbols: symbol_table,
        resolutions,
        params_in_order,
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

        ast::StmtKind::VarDecl {
            name,
            name_span,
            ty,
            ty_span,
            init,
        } => {
            if let Some(e) = init {
                resolve_expr(e, scope, resolutions)?;
            }
            let sym_id = symbols.insert(Symbol {
                name: name.clone(),
                name_span: *name_span,
                ty: ty.clone(),
                type_span: *ty_span,
                kind: SymbolKind::Local,
            });
            scope.declare(name, sym_id, *name_span)?;
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

            scope.enter_block_scope();
            resolve_stmts(then_body, scope, symbols, resolutions)?;
            scope.exit_scope();

            if let Some(else_stmts) = else_body {
                scope.enter_block_scope();
                resolve_stmts(else_stmts, scope, symbols, resolutions)?;
                scope.exit_scope();
            }
            Ok(())
        }
        ast::StmtKind::While { cond, body } => {
            resolve_expr(cond, scope, resolutions)?;

            scope.enter_block_scope();
            resolve_stmts(body, scope, symbols, resolutions)?;
            scope.exit_scope();

            Ok(())
        }
        ast::StmtKind::ExprStmt(expr) => {
            resolve_expr(expr, scope, resolutions)?;
            Ok(())
        }

        ast::StmtKind::Pause(expr) => {
            resolve_expr(expr, scope, resolutions)?;
            Ok(())
        }
        ast::StmtKind::Break => Ok(()),
    }
}

fn resolve_expr(
    expr: &ast::Expr,
    scope: &ScopeStack,
    resolutions: &mut Resolutions,
) -> SemaResult<()> {
    match &expr.kind {
        ast::ExprKind::IntLit(_) | ast::ExprKind::BoolLit(_) | ast::ExprKind::StringLit(_) => {
            Ok(())
        }

        ast::ExprKind::Var(name) => {
            let sym_id = scope.lookup(name, expr.span)?;
            resolutions.insert(expr.id, sym_id);
            Ok(())
        }

        ast::ExprKind::BinOp { lhs, rhs, .. } => {
            resolve_expr(lhs, scope, resolutions)?;
            resolve_expr(rhs, scope, resolutions)
        }

        ast::ExprKind::Unary { expr: inner, .. } => resolve_expr(inner, scope, resolutions),

        ast::ExprKind::Call {
            callee,
            callee_span,
            args,
        } => {
            let sym_id = scope.lookup(callee, *callee_span)?;
            resolutions.insert(expr.id, sym_id);
            for arg in args {
                resolve_expr(arg, scope, resolutions)?;
            }
            Ok(())
        }
        ast::ExprKind::SysCall { args } => {
            for arg in &args[2..] {
                resolve_expr(arg, scope, resolutions)?;
            }
            Ok(())
        }
    }
}
