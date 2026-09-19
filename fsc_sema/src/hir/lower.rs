use crate::bind::BoundScript;
use crate::checked::CheckedScript;
use crate::error::{SemaError, SemaResult};
use crate::frame::{FrameLayout, StackSlot};
use crate::hir;
use crate::infer::infer_expr;
use crate::resolve::Resolutions;
use crate::symbol::{ConstValue, Symbol, SymbolId, SymbolKind, SymbolTable};
use fsc_parse::ast::{self, Expr, FuncDef, Stmt};
use std::collections::HashMap;

// TODO: short-circuit
struct Layout {
    slots: HashMap<SymbolId, StackSlot>,
    local_count: i16,
}

#[derive(Clone, Copy)]
enum StackBinding {
    Param { index: u32 },
    Local,
}

impl Layout {
    fn new() -> Self {
        Self {
            slots: HashMap::new(),
            local_count: 0,
        }
    }

    fn slot_for(&mut self, sym_id: SymbolId, binding: StackBinding) -> StackSlot {
        if let Some(&slot) = self.slots.get(&sym_id) {
            return slot;
        }
        let slot = match binding {
            StackBinding::Param { index } => StackSlot(index as i16),
            StackBinding::Local => {
                self.local_count += 1;
                StackSlot(-self.local_count)
            }
        };
        self.slots.insert(sym_id, slot);
        slot
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
    let mut layout = Layout::new();
    let param_count = func.header.params.len() as i16;

    let body = lower_stmts(&func.body, resolutions, symbols, &mut layout)?;

    let frame = FrameLayout::new(param_count, layout.local_count);

    Ok(hir::FuncDef {
        name: func.header.name.clone(),
        exported: func.exported,
        ret_ty: lower_ty(&func.header.ret_ty),
        frame,
        body,
    })
}

fn lower_stmts(
    stmts: &[Stmt],
    resolutions: &Resolutions,
    symbols: &SymbolTable,
    layout: &mut Layout,
) -> SemaResult<Vec<hir::Stmt>> {
    stmts
        .iter()
        .map(|s| lower_stmt(s, resolutions, symbols, layout))
        .collect()
}

fn lower_stmt(
    stmt: &Stmt,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
    layout: &mut Layout,
) -> SemaResult<hir::Stmt> {
    match &stmt.kind {
        ast::StmtKind::Return(expr) => match expr {
            Some(e) => Ok(hir::Stmt::Return(lower_expr(
                e,
                resolutions,
                symbols,
                layout,
            )?)),
            None => Ok(hir::Stmt::ReturnVoid),
        },
        ast::StmtKind::Break => Ok(hir::Stmt::Break),

        ast::StmtKind::VarDecl { name, ty, init, .. } => {
            let hir_init = init
                .as_ref()
                .map(|e| lower_expr(e, resolutions, symbols, layout))
                .transpose()?;
            // TODO: rethink statement NOdeId resolution
            let sym_id = resolutions.symbol(stmt.id);
            let sym = symbols.get(sym_id);
            let binding = stack_binding(sym, stmt.span)?;
            let slot = layout.slot_for(sym_id, binding);

            Ok(hir::Stmt::VarDecl {
                name: name.clone(),
                slot,
                ty: lower_ty(ty),
                init: hir_init,
            })
        }

        ast::StmtKind::Assign { target, expr } => {
            let sym_id = resolutions.symbol(target.id);
            let sym = symbols.get(sym_id);
            let binding = stack_binding(sym, target.span)?;
            let slot = layout.slot_for(sym_id, binding);

            Ok(hir::Stmt::Assign {
                slot,
                value: lower_expr(expr, resolutions, symbols, layout)?,
            })
        }

        ast::StmtKind::If {
            cond,
            then_body,
            else_body,
        } => Ok(hir::Stmt::If {
            cond: lower_expr(cond, resolutions, symbols, layout)?,
            then_body: lower_stmts(then_body, resolutions, symbols, layout)?,
            else_body: else_body
                .as_ref()
                .map(|b| lower_stmts(b, resolutions, symbols, layout))
                .transpose()?,
        }),

        ast::StmtKind::While { cond, body } => Ok(hir::Stmt::While {
            cond: lower_expr(cond, resolutions, symbols, layout)?,
            body: lower_stmts(body, resolutions, symbols, layout)?,
        }),
        ast::StmtKind::ExprStmt(expr) => Ok(hir::Stmt::ExprStmt(lower_expr(
            expr,
            resolutions,
            symbols,
            layout,
        )?)),
        ast::StmtKind::Pause(expr) => Ok(hir::Stmt::Pause(lower_expr(
            expr,
            resolutions,
            symbols,
            layout,
        )?)),
    }
}

fn lower_expr(
    expr: &Expr,
    resolutions: &Resolutions,
    symbols: &SymbolTable,
    layout: &mut Layout,
) -> SemaResult<hir::Expr> {
    match &expr.kind {
        ast::ExprKind::IntLit(v) => Ok(hir::Expr::IntLit {
            value: *v,
            ty: hir::Ty::Int,
        }),
        ast::ExprKind::FloatLit(v) => Ok(hir::Expr::FloatLit {
            value: *v,
            ty: hir::Ty::Float,
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
            let sym_id = resolutions.symbol(expr.id);
            let sym = symbols.get(sym_id);
            match &sym.kind {
                SymbolKind::Const { value } => Ok(lower_const_value(value)),
                SymbolKind::Config => Err(SemaError::MissingConfigValue {
                    name: sym.name.clone(),
                    declaration_span: sym.name_span,
                }),
                SymbolKind::Param { index } => {
                    let slot = layout.slot_for(sym_id, StackBinding::Param { index: *index });
                    Ok(hir::Expr::Var {
                        name: sym.name.clone(),
                        slot,
                        ty: lower_ty(&sym.ty),
                    })
                }
                SymbolKind::Local => {
                    let slot = layout.slot_for(sym_id, StackBinding::Local);
                    Ok(hir::Expr::Var {
                        name: sym.name.clone(),
                        slot,
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
                lhs: Box::new(lower_expr(lhs, resolutions, symbols, layout)?),
                rhs: Box::new(lower_expr(rhs, resolutions, symbols, layout)?),
                ty: lower_ty(&ty),
            })
        }

        ast::ExprKind::Unary { op, expr: inner } => {
            let ty = infer_expr(expr, resolutions, symbols)?;
            Ok(hir::Expr::Unary {
                op: lower_unaryop(op),
                expr: Box::new(lower_expr(inner, resolutions, symbols, layout)?),
                ty: lower_ty(&ty),
            })
        }

        ast::ExprKind::Call { callee, args, .. } => {
            let ty = infer_expr(expr, resolutions, symbols)?;

            Ok(hir::Expr::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|x| lower_expr(x, resolutions, symbols, layout))
                    .collect::<Result<_, _>>()?,
                ty: lower_ty(&ty),
            })
        }
        ast::ExprKind::SysCall { args } => {
            let (page, func) = extract_syscall(args)?;
            let lowered_args: Vec<_> = args[2..]
                .iter()
                .map(|a| lower_expr(a, resolutions, symbols, layout))
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

fn stack_binding(
    symbol: &Symbol,
    reference_span: fsc_diagnostics::Span,
) -> SemaResult<StackBinding> {
    match symbol.kind {
        SymbolKind::Param { index } => Ok(StackBinding::Param { index }),
        SymbolKind::Local => Ok(StackBinding::Local),
        SymbolKind::Const { .. } => Err(SemaError::AssignmentToConstant {
            name: symbol.name.clone(),
            assignment_span: reference_span,
            declaration_span: symbol.name_span,
        }),
        SymbolKind::Config => Err(SemaError::AssignmentToConfig {
            name: symbol.name.clone(),
            assignment_span: reference_span,
            declaration_span: symbol.name_span,
        }),
        SymbolKind::Function { .. } => Err(SemaError::NotAValue {
            name: symbol.name.clone(),
            reference_span,
            declaration_span: symbol.name_span,
        }),
    }
}

fn lower_const_value(value: &ConstValue) -> hir::Expr {
    match value {
        ConstValue::Int(value) => hir::Expr::IntLit {
            value: *value,
            ty: hir::Ty::Int,
        },
        ConstValue::Float(value) => hir::Expr::FloatLit {
            value: *value,
            ty: hir::Ty::Float,
        },
        ConstValue::Bool(value) => hir::Expr::BoolLit {
            value: *value,
            ty: hir::Ty::Bool,
        },
        ConstValue::Str(value) => hir::Expr::StrLit {
            value: value.clone(),
            ty: hir::Ty::Int,
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

fn lower_ty(ty: &ast::Ty) -> hir::Ty {
    match ty {
        ast::Ty::Int => hir::Ty::Int,
        ast::Ty::Float => hir::Ty::Float,
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
