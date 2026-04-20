mod label_ctx;

use crate::error::{CodegenError, CodegenResult};
use crate::lower::label_ctx::LabelCtx;
use fsc_assembler::Assembler;
use fsc_sema::frame::FrameLayout;
use fsc_sema::hir::{BinOp, Expr, FuncDef, Stmt, Ty, UnaryOp};

pub fn lower_func(func: &FuncDef, asm: &mut Assembler) -> CodegenResult<()> {
    let frame = &func.frame;
    let mut label_ctx = LabelCtx::new();

    asm.define_function(&func.name, func.exported)
        .map_err(Into::<CodegenError>::into)?;

    if frame.local_count() > 0 {
        asm.emit_grow_stack(frame.local_count());
    }

    for stmt in &func.body {
        lower_stmt(stmt, frame, &mut label_ctx, asm)?;
    }
    Ok(())
}

pub fn lower_stmt(
    stmt: &Stmt,
    frame: &FrameLayout,
    label_ctx: &mut LabelCtx,
    asm: &mut Assembler,
) -> CodegenResult<()> {
    match stmt {
        Stmt::Return(expr) => Ok(lower_return(expr, frame, label_ctx, asm)?),
        Stmt::ReturnVoid => {
            asm.emit_ret(frame.frame_size());
            Ok(())
        }
        Stmt::Assign { slot, value: expr } => {
            lower_expr(expr, label_ctx, asm)?;
            if let Expr::SysCall { .. } | Expr::Call { .. } = expr {
                asm.emit_push_result();
            }
            asm.emit_store_arg(slot.0);
            Ok(())
        }
        Stmt::VarDecl {
            name: _name,
            slot,
            ty: _ty,
            init,
        } => {
            if let Some(expr) = init {
                lower_expr(expr, label_ctx, asm)?;
                if let Some(Expr::SysCall { .. } | Expr::Call { .. }) = init {
                    asm.emit_push_result();
                }
                asm.emit_store_arg(slot.0);
            }
            Ok(())
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            lower_expr(cond, label_ctx, asm)?;
            match else_body {
                None => {
                    let end = label_ctx.fresh_label("if_end");
                    asm.emit_jz(&end)?;
                    for s in then_body {
                        lower_stmt(s, frame, label_ctx, asm)?;
                    }
                    asm.define_label(&end)?;
                }
                Some(else_stmts) => {
                    let else_lbl = label_ctx.fresh_label("else");
                    let end = label_ctx.fresh_label("if_end");
                    asm.emit_jz(&else_lbl)?;
                    for s in then_body {
                        lower_stmt(s, frame, label_ctx, asm)?;
                    }
                    asm.emit_jmp(&end)?;
                    asm.define_label(&else_lbl)?;
                    for s in else_stmts {
                        lower_stmt(s, frame, label_ctx, asm)?;
                    }
                    asm.define_label(&end)?;
                }
            }
            Ok(())
        }
        Stmt::ExprStmt(expr) => {
            match expr {
                Expr::Call { callee, args, .. } => {
                    lower_call(callee, args, label_ctx, asm)?;
                }
                _ => lower_expr(expr, label_ctx, asm)?,
            }
            Ok(())
        }
        Stmt::While { cond, body } => {
            lower_while(cond, body, frame, label_ctx, asm)?;
            Ok(())
        }
    }
}

fn lower_while(
    cond: &Expr,
    body: &[Stmt],
    frame: &FrameLayout,
    label_ctx: &mut LabelCtx,
    asm: &mut Assembler,
) -> CodegenResult<()> {
    let start = label_ctx.fresh_label("while_start");
    let end = label_ctx.fresh_label("while_end");

    asm.define_label(&start)?;

    lower_expr(cond, label_ctx, asm)?;
    asm.emit_jz(&end)?;

    for s in body {
        lower_stmt(s, frame, label_ctx, asm)?;
    }

    asm.emit_jmp(&start)?;
    asm.define_label(&end)?;

    Ok(())
}
fn lower_return(
    expr: &Expr,
    frame: &FrameLayout,
    label_ctx: &mut LabelCtx,
    asm: &mut Assembler,
) -> CodegenResult<()> {
    lower_expr(expr, label_ctx, asm)?;
    asm.emit_retv(frame.frame_size());
    Ok(())
}

pub fn lower_expr(expr: &Expr, label_ctx: &mut LabelCtx, asm: &mut Assembler) -> CodegenResult<()> {
    match expr {
        Expr::IntLit { value, .. } => {
            emit_int_lit(*value, asm);
        }
        Expr::BoolLit { value, .. } => {
            if *value {
                asm.emit_push(1);
            } else {
                asm.emit_push(0);
            }
        }
        Expr::StrLit { value, .. } => {
            asm.emit_lstr(value)?;
        }

        Expr::Var {
            name: _name,
            slot,
            ty: _ty,
        } => {
            asm.emit_load_arg(slot.0);
        }

        Expr::Unary {
            op,
            expr: expression,
            ty: _ty,
        } => {
            // TODO: optimize push literal directly
            lower_expr(expression, label_ctx, asm)?;
            emit_unary(op, asm);
        }
        Expr::BinOp { op, lhs, rhs, .. } if matches!(op, BinOp::And | BinOp::Or) => match op {
            BinOp::And => emit_and_short_circuit(lhs, rhs, label_ctx, asm)?,
            BinOp::Or => emit_or_short_circuit(lhs, rhs, label_ctx, asm)?,
            _ => unreachable!(),
        },
        Expr::BinOp {
            op,
            lhs,
            rhs,
            ty: _,
        } => {
            //TODO: for Eq == 0 optimization to eq0
            lower_expr(lhs, label_ctx, asm)?;
            lower_expr(rhs, label_ctx, asm)?;

            emit_binop(op, lhs.ty(), asm);
            // TODO: original scripts are saving in arg and load again; check if this is really
            // everytime necessary
        }
        Expr::Call { callee, args, .. } => {
            lower_call(callee, args, label_ctx, asm)?;
        }
        Expr::SysCall {
            page,
            func,
            subtype,
            args,
            ..
        } => {
            for arg in args.iter().rev() {
                lower_expr(arg, label_ctx, asm)?;
            }
            asm.emit_syscall(*subtype, *page, *func);
        }
    }
    Ok(())
}

fn lower_call(
    callee: &str,
    args: &[Expr],
    label_ctx: &mut LabelCtx,
    asm: &mut Assembler,
) -> CodegenResult<()> {
    // TODO: rev() used because of calling convention; check how to make this more explicit
    for arg in args.iter().rev() {
        lower_expr(arg, label_ctx, asm)?;
    }
    asm.emit_call(callee)?;
    Ok(())
}
fn emit_and_short_circuit(
    lhs: &Expr,
    rhs: &Expr,
    ctx: &mut LabelCtx,
    asm: &mut Assembler,
) -> CodegenResult<()> {
    let false_label = ctx.fresh_label("and_false");
    let end_label = ctx.fresh_label("and_end");

    lower_expr(lhs, ctx, asm)?;
    asm.emit_jz(&false_label)?;

    lower_expr(rhs, ctx, asm)?;
    asm.emit_jz(&false_label)?;

    asm.emit_push(1);
    asm.emit_jmp(&end_label)?;

    asm.define_label(&false_label)?;
    asm.emit_push(0);

    asm.define_label(&end_label)?;
    Ok(())
}

fn emit_or_short_circuit(
    lhs: &Expr,
    rhs: &Expr,
    ctx: &mut LabelCtx,
    asm: &mut Assembler,
) -> CodegenResult<()> {
    let true_label = ctx.fresh_label("or_true");
    let end_label = ctx.fresh_label("or_end");

    lower_expr(lhs, ctx, asm)?;
    asm.emit_jnz(&true_label)?;

    lower_expr(rhs, ctx, asm)?;
    asm.emit_jnz(&true_label)?;

    asm.emit_push(0);
    asm.emit_jmp(&end_label)?;

    asm.define_label(&true_label)?;
    asm.emit_push(1);

    asm.define_label(&end_label)?;
    Ok(())
}
fn emit_int_lit(value: i32, asm: &mut Assembler) {
    match i16::try_from(value) {
        Ok(small) => asm.emit_push(small),
        Err(_) => asm.emit_push_imm(value.cast_unsigned()),
        //TODO: check whether push_imm makes
        // sense here
    }
}

fn emit_binop(op: &BinOp, operand_ty: &Ty, asm: &mut Assembler) {
    match op {
        BinOp::Add => match operand_ty {
            Ty::Int => asm.emit_add(),
            Ty::Bool => unreachable!("add on bool"),
            Ty::Void => unreachable!("add on void"),
            Ty::Str => unreachable!("add on str"),
        },
        BinOp::Sub => match operand_ty {
            Ty::Int => asm.emit_sub(),
            Ty::Bool => unreachable!("sub on bool"),
            Ty::Void => unreachable!("sub on void"),
            Ty::Str => unreachable!("sub on str"),
        },
        BinOp::Mul => match operand_ty {
            Ty::Int => asm.emit_mul(),
            Ty::Bool => unreachable!("mul on bool"),
            Ty::Void => unreachable!("mul on void"),
            Ty::Str => unreachable!("mul on str"),
        },
        BinOp::Div => match operand_ty {
            Ty::Int => asm.emit_div(),
            Ty::Bool => unreachable!("div on bool"),
            Ty::Void => unreachable!("div on void"),
            Ty::Str => unreachable!("div on str"),
        },

        BinOp::Eq => asm.emit_eq(),
        BinOp::Neq => asm.emit_neq(),
        BinOp::Gt => asm.emit_gt(),
        BinOp::Ge => asm.emit_ge(),
        BinOp::Lt => asm.emit_lt(),
        BinOp::Le => asm.emit_le(),

        BinOp::And | BinOp::Or => unreachable!("should be in short circuit logic"),
    }
}

fn emit_unary(op: &UnaryOp, asm: &mut Assembler) {
    match op {
        UnaryOp::Not => asm.emit_eq0(), // logical not
        UnaryOp::Neg => asm.emit_neg(),
    }
}
