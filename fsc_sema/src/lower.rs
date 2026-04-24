use crate::error::SemaResult;
use crate::frame::{FrameLayout, StackSlot};
use crate::hir;
use crate::infer::infer_expr;
use crate::resolve::ResolveOutput;
use crate::symbol::{SymbolId, SymbolKind};
use fsc_parse::ast::{self, Expr, FuncDef, Stmt};
use std::collections::HashMap;

// TODO: short-circuit
struct Layout {
    slots: HashMap<SymbolId, StackSlot>,
    local_count: i16,
}

impl Layout {
    fn new() -> Self {
        Self {
            slots: HashMap::new(),
            local_count: 0,
        }
    }

    fn slot_for(&mut self, sym_id: SymbolId, kind: &SymbolKind) -> StackSlot {
        if let Some(&slot) = self.slots.get(&sym_id) {
            return slot;
        }
        let slot = match kind {
            SymbolKind::Param { index } => StackSlot(*index as i16),
            SymbolKind::Local => {
                self.local_count += 1;
                StackSlot(-self.local_count)
            }
            SymbolKind::Function { .. } => todo!(),
        };
        self.slots.insert(sym_id, slot);
        slot
    }
}

pub fn lower_fn(func: &FuncDef, resolved: &ResolveOutput) -> SemaResult<hir::FuncDef> {
    let mut layout = Layout::new();
    let param_count = func.params.len() as i16;

    for &sym_id in &resolved.params_in_order {
        let symbol = resolved.symbols.get(sym_id);
        layout.slot_for(sym_id, &symbol.kind);
    }

    let body = lower_stmts(&func.body, resolved, &mut layout)?;

    let frame = FrameLayout::new(param_count, layout.local_count);

    Ok(hir::FuncDef {
        name: func.name.clone(),
        exported: func.exported,
        ret_ty: lower_ty(&func.ret_ty),
        frame,
        body,
    })
}

fn lower_stmts(
    stmts: &[Stmt],
    resolved: &ResolveOutput,
    layout: &mut Layout,
) -> SemaResult<Vec<hir::Stmt>> {
    stmts
        .iter()
        .map(|s| lower_stmt(s, resolved, layout))
        .collect()
}

fn lower_stmt(stmt: &Stmt, resolved: &ResolveOutput, layout: &mut Layout) -> SemaResult<hir::Stmt> {
    match &stmt.kind {
        ast::StmtKind::Return(expr) => match expr {
            Some(e) => Ok(hir::Stmt::Return(lower_expr(e, resolved, layout)?)),
            None => Ok(hir::Stmt::ReturnVoid),
        },
        ast::StmtKind::Break => Ok(hir::Stmt::Break),

        ast::StmtKind::VarDecl { name, ty, init } => {
            let hir_init = init
                .as_ref()
                .map(|e| lower_expr(e, resolved, layout))
                .transpose()?;
            // TODO: rethink statement NOdeId resolution
            let sym_id = resolved.resolutions.symbol(stmt.id);
            let sym = resolved.symbols.get(sym_id);
            let slot = layout.slot_for(sym_id, &sym.kind);

            Ok(hir::Stmt::VarDecl {
                name: name.clone(),
                slot,
                ty: lower_ty(ty),
                init: hir_init,
            })
        }

        ast::StmtKind::Assign { target, expr } => {
            let sym_id = resolved.resolutions.symbol(target.id);
            let sym = resolved.symbols.get(sym_id);
            let slot = layout.slot_for(sym_id, &sym.kind);

            Ok(hir::Stmt::Assign {
                slot,
                value: lower_expr(expr, resolved, layout)?,
            })
        }

        ast::StmtKind::If {
            cond,
            then_body,
            else_body,
        } => Ok(hir::Stmt::If {
            cond: lower_expr(cond, resolved, layout)?,
            then_body: lower_stmts(then_body, resolved, layout)?,
            else_body: else_body
                .as_ref()
                .map(|b| lower_stmts(b, resolved, layout))
                .transpose()?,
        }),

        ast::StmtKind::While { cond, body } => Ok(hir::Stmt::While {
            cond: lower_expr(cond, resolved, layout)?,
            body: lower_stmts(body, resolved, layout)?,
        }),
        ast::StmtKind::ExprStmt(expr) => {
            Ok(hir::Stmt::ExprStmt(lower_expr(expr, resolved, layout)?))
        }
        ast::StmtKind::Pause(expr) => Ok(hir::Stmt::Pause(lower_expr(expr, resolved, layout)?)),
    }
}

fn lower_expr(expr: &Expr, resolved: &ResolveOutput, layout: &mut Layout) -> SemaResult<hir::Expr> {
    match &expr.kind {
        ast::ExprKind::IntLit(v) => Ok(hir::Expr::IntLit {
            value: *v,
            ty: hir::Ty::Int,
        }),
        ast::ExprKind::StringLit(v) => Ok(hir::Expr::StrLit {
            value: v.clone(),
            ty: hir::Ty::Int,
        }),

        ast::ExprKind::BoolLit(v) => Ok(hir::Expr::BoolLit {
            value: *v,
            ty: hir::Ty::Bool,
        }),

        ast::ExprKind::Var(_name) => {
            let sym_id = resolved.resolutions.symbol(expr.id);
            let sym = resolved.symbols.get(sym_id);
            let slot = layout.slot_for(sym_id, &sym.kind);
            Ok(hir::Expr::Var {
                name: sym.name.clone(),
                slot,
                ty: lower_ty(&sym.ty),
            })
        }

        ast::ExprKind::BinOp { op, lhs, rhs } => {
            let ty = infer_expr(expr, resolved)?;
            Ok(hir::Expr::BinOp {
                op: lower_binop(op),
                lhs: Box::new(lower_expr(lhs, resolved, layout)?),
                rhs: Box::new(lower_expr(rhs, resolved, layout)?),
                ty: lower_ty(&ty),
            })
        }

        ast::ExprKind::Unary { op, expr: inner } => {
            let ty = infer_expr(expr, resolved)?;
            Ok(hir::Expr::Unary {
                op: lower_unaryop(op),
                expr: Box::new(lower_expr(inner, resolved, layout)?),
                ty: lower_ty(&ty),
            })
        }

        ast::ExprKind::Call { callee, args } => {
            let ty = infer_expr(expr, resolved)?;

            Ok(hir::Expr::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|x| lower_expr(x, resolved, layout))
                    .collect::<Result<_, _>>()?,
                ty: lower_ty(&ty),
            })
        }
        ast::ExprKind::SysCall { args } => {
            let (page, func) = extract_syscall(args)?;
            let lowered_args: Vec<_> = args[2..]
                .iter()
                .map(|a| lower_expr(a, resolved, layout))
                .collect::<Result<_, _>>()?;

            let subtype = lowered_args.len() as u8;

            Ok(hir::Expr::SysCall {
                page,
                func,
                subtype,
                args: lowered_args,
                ty: hir::Ty::Int,
            })
        }
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

fn lower_ty(ty: &ast::Ty) -> hir::Ty {
    match ty {
        ast::Ty::Int => hir::Ty::Int,
        ast::Ty::Void => hir::Ty::Void,
        ast::Ty::Bool => hir::Ty::Bool,
        ast::Ty::Str => hir::Ty::Str,
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
