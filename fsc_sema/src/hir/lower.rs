use crate::bind::BoundScript;
use crate::checked::CheckedScript;
use crate::error::{SemaError, SemaResult};
use crate::hir;
use crate::infer::infer_expr;
use crate::local::LocalId;
use crate::place::Place;
use crate::resolve::Resolutions;
use crate::symbol::{ConstValue, SymbolId, SymbolKind, SymbolTable};
use crate::types::Ty;
use fsc_parse::ast::{self, Expr, FuncDef, Stmt};
use std::collections::HashMap;

struct HirLocals {
    declarations: Vec<hir::Local>,
    by_symbol: HashMap<SymbolId, LocalId>,
}

//TODO: cleanup with separation of concern
impl HirLocals {
    fn new(resolutions: &Resolutions, symbols: &SymbolTable) -> Self {
        let mut declarations = Vec::new();
        let mut by_symbol = HashMap::new();

        for &symbol_id in resolutions.frame_symbols() {
            let symbol = symbols.get(symbol_id);
            let kind = match symbol.kind {
                SymbolKind::Param { .. } => hir::LocalKind::Parameter,
                SymbolKind::Local => hir::LocalKind::Variable,
                SymbolKind::Const { .. } | SymbolKind::Config | SymbolKind::Function { .. } => {
                    continue;
                }
            };
            let local = LocalId::new(declarations.len() as u32);
            declarations.push(hir::Local {
                id: local,
                ty: Ty::from(&symbol.ty),
                kind,
            });
            by_symbol.insert(symbol_id, local);
        }

        Self {
            declarations,
            by_symbol,
        }
    }

    fn local_for_symbol(
        &self,
        symbol: SymbolId,
        name: &str,
        reference_span: fsc_diagnostics::Span,
    ) -> SemaResult<LocalId> {
        self.by_symbol
            .get(&symbol)
            .copied()
            .ok_or_else(|| SemaError::MissingLocalIdentity {
                name: name.to_owned(),
                reference_span,
            })
    }

    fn into_declarations(self) -> Vec<hir::Local> {
        self.declarations
    }
}

pub(crate) fn lower_script(bound: BoundScript) -> SemaResult<hir::Script> {
    let BoundScript(checked) = bound;
    let CheckedScript {
        functions, symbols, ..
    } = checked;
    let items = functions
        .into_iter()
        .map(|checked| {
            lower_fn(&checked.function, &checked.resolutions, &symbols).map(hir::Item::FuncDef)
        })
        .collect::<SemaResult<_>>()?;

    Ok(hir::Script { items })
}

fn lower_fn(
    func: &FuncDef,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
) -> SemaResult<hir::FuncDef> {
    let locals = HirLocals::new(resolutions, symbols);

    let ret_ty = lower_ty(&func.header.ret_ty);
    let mut body = lower_stmts(&func.body, resolutions, symbols, &locals)?;

    if ret_ty == Ty::Void && stmts_can_fall_through(&body) {
        body.push(hir::Stmt::ReturnVoid);
    }

    Ok(hir::FuncDef {
        name: func.header.name.clone(),
        exported: func.exported,
        ret_ty,
        locals: locals.into_declarations(),
        body,
    })
}

fn stmts_can_fall_through(stmts: &[hir::Stmt]) -> bool {
    stmts.iter().all(stmt_can_fall_through)
}

fn stmt_can_fall_through(stmt: &hir::Stmt) -> bool {
    match stmt {
        hir::Stmt::Return(_) | hir::Stmt::ReturnVoid | hir::Stmt::Break => false,
        hir::Stmt::If {
            then_body,
            else_body: Some(else_body),
            ..
        } => stmts_can_fall_through(then_body) || stmts_can_fall_through(else_body),
        hir::Stmt::If {
            else_body: None, ..
        }
        | hir::Stmt::While { .. }
        | hir::Stmt::VarDecl { .. }
        | hir::Stmt::Assign { .. }
        | hir::Stmt::ExprStmt(_)
        | hir::Stmt::Pause(_) => true,
    }
}

fn lower_stmts(
    stmts: &[Stmt],
    resolutions: &Resolutions,
    symbols: &SymbolTable,
    locals: &HirLocals,
) -> SemaResult<Vec<hir::Stmt>> {
    stmts
        .iter()
        .map(|s| lower_stmt(s, resolutions, symbols, locals))
        .collect()
}

fn lower_stmt(
    stmt: &Stmt,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
    locals: &HirLocals,
) -> SemaResult<hir::Stmt> {
    match &stmt.kind {
        ast::StmtKind::Return(expr) => match expr {
            Some(e) => Ok(hir::Stmt::Return(lower_expr(
                e,
                resolutions,
                symbols,
                locals,
            )?)),
            None => Ok(hir::Stmt::ReturnVoid),
        },
        ast::StmtKind::Break => Ok(hir::Stmt::Break),

        ast::StmtKind::VarDecl { init, .. } => {
            let hir_init = init
                .as_ref()
                .map(|e| lower_expr(e, resolutions, symbols, locals))
                .transpose()?;
            let sym_id = resolutions.symbol(stmt.id);
            let sym = symbols.get(sym_id);
            let local = locals.local_for_symbol(sym_id, &sym.name, stmt.span)?;

            Ok(hir::Stmt::VarDecl {
                local,
                init: hir_init,
            })
        }

        ast::StmtKind::Assign { target, expr } => {
            let sym_id = resolutions.symbol(target.id);
            let sym = symbols.get(sym_id);
            let local = locals.local_for_symbol(sym_id, &sym.name, target.span)?;

            Ok(hir::Stmt::Assign {
                target: Place::new(local),
                value: lower_expr(expr, resolutions, symbols, locals)?,
            })
        }

        ast::StmtKind::If {
            cond,
            then_body,
            else_body,
        } => Ok(hir::Stmt::If {
            cond: lower_expr(cond, resolutions, symbols, locals)?,
            then_body: lower_stmts(then_body, resolutions, symbols, locals)?,
            else_body: else_body
                .as_ref()
                .map(|b| lower_stmts(b, resolutions, symbols, locals))
                .transpose()?,
        }),

        ast::StmtKind::While { cond, body } => Ok(hir::Stmt::While {
            cond: lower_expr(cond, resolutions, symbols, locals)?,
            body: lower_stmts(body, resolutions, symbols, locals)?,
        }),
        ast::StmtKind::ExprStmt(expr) => Ok(hir::Stmt::ExprStmt(lower_expr(
            expr,
            resolutions,
            symbols,
            locals,
        )?)),
        ast::StmtKind::Pause(expr) => Ok(hir::Stmt::Pause(lower_expr(
            expr,
            resolutions,
            symbols,
            locals,
        )?)),
    }
}

fn lower_expr(
    expr: &Expr,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
    locals: &HirLocals,
) -> SemaResult<hir::Expr> {
    match &expr.kind {
        ast::ExprKind::IntLit(v) => Ok(hir::Expr::IntLit {
            value: *v,
            ty: Ty::Int,
        }),
        ast::ExprKind::FloatLit(v) => Ok(hir::Expr::FloatLit {
            value: *v,
            ty: Ty::Float,
        }),
        ast::ExprKind::StringLit(v) => Ok(hir::Expr::StrLit {
            value: v.clone(),
            ty: Ty::Int,
        }),

        ast::ExprKind::BoolLit(v) => Ok(hir::Expr::BoolLit {
            value: *v,
            ty: Ty::Bool,
        }),

        ast::ExprKind::Var(_name) => {
            let sym_id = resolutions.symbol(expr.id);
            let sym = symbols.get(sym_id);
            match &sym.kind {
                SymbolKind::Const { value } => Ok(lower_const_value(value)),
                SymbolKind::Config => Err(SemaError::MissingConfigValue {
                    name: sym.name.clone(),
                    declaration_span: sym.name_span,
                }),
                SymbolKind::Param { .. } | SymbolKind::Local => {
                    let local = locals.local_for_symbol(sym_id, &sym.name, expr.span)?;
                    Ok(hir::Expr::Load {
                        place: Place::new(local),
                        ty: lower_ty(&sym.ty),
                    })
                }
                SymbolKind::Function { .. } => Err(SemaError::NotAValue {
                    name: sym.name.clone(),
                    reference_span: expr.span,
                    declaration_span: sym.name_span,
                }),
            }
        }

        ast::ExprKind::BinOp { op, lhs, rhs } => {
            let ty = infer_expr(expr, resolutions, symbols)?;
            Ok(hir::Expr::BinOp {
                op: lower_binop(op),
                lhs: Box::new(lower_expr(lhs, resolutions, symbols, locals)?),
                rhs: Box::new(lower_expr(rhs, resolutions, symbols, locals)?),
                ty: lower_ty(&ty),
            })
        }

        ast::ExprKind::Unary { op, expr: inner } => {
            let ty = infer_expr(expr, resolutions, symbols)?;
            Ok(hir::Expr::Unary {
                op: lower_unaryop(op),
                expr: Box::new(lower_expr(inner, resolutions, symbols, locals)?),
                ty: lower_ty(&ty),
            })
        }

        ast::ExprKind::Call { callee, args, .. } => {
            let ty = infer_expr(expr, resolutions, symbols)?;

            Ok(hir::Expr::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|x| lower_expr(x, resolutions, symbols, locals))
                    .collect::<Result<_, _>>()?,
                ty: lower_ty(&ty),
            })
        }
        ast::ExprKind::SysCall { args } => {
            let (page, func) = extract_syscall(args)?;
            let lowered_args: Vec<_> = args[2..]
                .iter()
                .map(|a| lower_expr(a, resolutions, symbols, locals))
                .collect::<Result<_, _>>()?;

            let subtype = lowered_args.len() as u8;

            Ok(hir::Expr::SysCall {
                page,
                func,
                subtype,
                args: lowered_args,
                ty: Ty::Int,
            })
        }
    }
}

fn lower_const_value(value: &ConstValue) -> hir::Expr {
    match value {
        ConstValue::Int(value) => hir::Expr::IntLit {
            value: *value,
            ty: Ty::Int,
        },
        ConstValue::Float(value) => hir::Expr::FloatLit {
            value: *value,
            ty: Ty::Float,
        },
        ConstValue::Bool(value) => hir::Expr::BoolLit {
            value: *value,
            ty: Ty::Bool,
        },
        ConstValue::Str(value) => hir::Expr::StrLit {
            value: value.clone(),
            ty: Ty::Int,
        },
    }
}

pub fn extract_syscall(args: &[Expr]) -> SemaResult<(u8, u16)> {
    if args.len() < 3 {
        todo!("Syscall called with too few arguments");
    }

    let page = extract_syscall_u8(&args[0])?;
    let func = extract_syscall_u16(&args[1])?;

    Ok((page, func))
}
fn extract_syscall_u8(expr: &Expr) -> SemaResult<u8> {
    match &expr.kind {
        ast::ExprKind::IntLit(n) => u8::try_from(*n).map_err(|_| todo!("out of range")),
        _ => todo!("unsupported expr kind"),
    }
}

fn extract_syscall_u16(expr: &Expr) -> SemaResult<u16> {
    match &expr.kind {
        ast::ExprKind::IntLit(n) => u16::try_from(*n).map_err(|_| todo!("out of range")),
        _ => todo!("unsupported expr kind"),
    }
}

fn lower_ty(ty: &ast::Ty) -> Ty {
    Ty::from(ty)
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
