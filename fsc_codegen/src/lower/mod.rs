use crate::error::{CodegenError, CodegenResult};
use crate::frame::FrameLayout;
use fsc_assembler::Assembler;
use fsc_parse::ast::{BinOp, Expr, FuncDef, Stmt, Ty};

pub fn lower_func(func: &FuncDef, asm: &mut Assembler) -> CodegenResult<()> {
    asm.define_function(&func.name, func.exported)
        .map_err(Into::<CodegenError>::into)?;
    let frame = FrameLayout::from_params(&func.params); // TODO: local variable assignment
    if frame.local_count() > 0 {
        asm.emit_grow_stack(frame.local_count());
    }
    for stmt in &func.body {
        lower_stmt(stmt, &frame, asm)?;
    }
    Ok(())
}

pub fn lower_stmt(stmt: &Stmt, frame: &FrameLayout, asm: &mut Assembler) -> CodegenResult<()> {
    match stmt {
        Stmt::Return(expr) => {
            if let Some(e) = expr {
                Ok(lower_return(e, frame, asm)?)
            } else {
                asm.emit_ret(frame.frame_size());
                Ok(())
            }
        }
    }
}
fn lower_return(expr: &Expr, frame: &FrameLayout, asm: &mut Assembler) -> CodegenResult<()> {
    lower_expr(expr, frame, asm)?;
    asm.emit_retv(frame.frame_size());
    Ok(())
}

pub fn lower_expr(expr: &Expr, frame: &FrameLayout, asm: &mut Assembler) -> CodegenResult<()> {
    match expr {
        Expr::IntLit(value) => {
            emit_int_lit(*value, asm);
        }

        Expr::Var(name) => {
            let slot = frame.resolve(name)?;
            asm.emit_load_arg(slot.0);
        }

        Expr::BinOp { op, lhs, rhs } => {
            let ty = Ty::Int; //TODO: hardcoded value
            lower_expr(lhs, frame, asm)?;
            lower_expr(rhs, frame, asm)?;

            emit_binop(op, &ty, asm);
        }
    }
    Ok(())
}
fn emit_int_lit(value: i32, asm: &mut Assembler) {
    match i16::try_from(value) {
        Ok(small) => asm.emit_push(small),
        Err(_) => asm.emit_push_imm(value.cast_unsigned()),
    }
}

fn emit_binop(op: &BinOp, ty: &Ty, asm: &mut Assembler) {
    match ty {
        Ty::Int => emit_int_binop(op, asm),
        Ty::Void => todo!(), //TODO: should probably be better with typed ast
    }
}

fn emit_int_binop(op: &BinOp, asm: &mut Assembler) {
    match op {
        BinOp::Add => asm.emit_add(),
        BinOp::Sub => asm.emit_sub(),
        BinOp::Mul => asm.emit_mul(),
        BinOp::Div => asm.emit_div(),
    }
}
