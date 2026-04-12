use crate::error::{CodegenError, CodegenResult};
use fsc_assembler::Assembler;
use fsc_sema::frame::FrameLayout;
use fsc_sema::hir::{BinOp, Expr, FuncDef, Stmt, Ty, UnaryOp};

pub fn lower_func(func: &FuncDef, asm: &mut Assembler) -> CodegenResult<()> {
    let frame = &func.frame;
    asm.define_function(&func.name, func.exported)
        .map_err(Into::<CodegenError>::into)?;

    if frame.local_count() > 0 {
        asm.emit_grow_stack(frame.local_count());
    }

    for stmt in &func.body {
        lower_stmt(stmt, frame, asm)?;
    }
    Ok(())
}

pub fn lower_stmt(stmt: &Stmt, frame: &FrameLayout, asm: &mut Assembler) -> CodegenResult<()> {
    match stmt {
        Stmt::Return(expr) => Ok(lower_return(expr, frame, asm)?),
        Stmt::ReturnVoid => {
            asm.emit_ret(frame.frame_size());
            Ok(())
        }
        Stmt::Assign { slot, value: expr } => {
            lower_expr(expr, asm)?;
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
                lower_expr(expr, asm)?;
                asm.emit_store_arg(slot.0);
            }
            Ok(())
        }
    }
}
fn lower_return(expr: &Expr, frame: &FrameLayout, asm: &mut Assembler) -> CodegenResult<()> {
    lower_expr(expr, asm)?;
    asm.emit_retv(frame.frame_size());
    Ok(())
}

pub fn lower_expr(expr: &Expr, asm: &mut Assembler) -> CodegenResult<()> {
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
            lower_expr(expression, asm)?;
            emit_unary(op, asm);
        }

        Expr::BinOp {
            op,
            lhs,
            rhs,
            ty: _,
        } => {
            lower_expr(lhs, asm)?;
            lower_expr(rhs, asm)?;

            emit_binop(op, lhs.ty(), asm);
            // TODO: original scripts are saving in arg and load again; check if this is really
            // everytime necessary
        }
    }
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
        },
        BinOp::Sub => match operand_ty {
            Ty::Int => asm.emit_sub(),
            Ty::Bool => unreachable!("sub on bool"),
            Ty::Void => unreachable!("sub on void"),
        },
        BinOp::Mul => match operand_ty {
            Ty::Int => asm.emit_mul(),
            Ty::Bool => unreachable!("mul on bool"),
            Ty::Void => unreachable!("mul on void"),
        },
        BinOp::Div => match operand_ty {
            Ty::Int => asm.emit_div(),
            Ty::Bool => unreachable!("div on bool"),
            Ty::Void => unreachable!("div on void"),
        },

        BinOp::And => asm.emit_and(),
        BinOp::Or => asm.emit_or(),
        BinOp::Eq => asm.emit_eq(),
        BinOp::Neq => asm.emit_neq(),
        BinOp::Gt => asm.emit_gt(),
        BinOp::Ge => asm.emit_ge(),
        BinOp::Lt => asm.emit_lt(),
        BinOp::Le => asm.emit_le(),
    }
}

fn emit_unary(op: &UnaryOp, asm: &mut Assembler) {
    match op {
        UnaryOp::Not => asm.emit_eq0(), // logical not
        UnaryOp::Neg => asm.emit_neg(),
    }
}
