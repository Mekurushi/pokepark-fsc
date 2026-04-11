use crate::error::{CodegenError, CodegenResult};
use fsc_assembler::Assembler;
use fsc_sema::frame::FrameLayout;
use fsc_sema::hir::{BinOp, Expr, FuncDef, Stmt, Ty};

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

        Expr::Var {
            name: _name,
            slot,
            ty: _ty,
        } => {
            asm.emit_load_arg(slot.0);
        }

        Expr::BinOp { op, lhs, rhs, ty } => {
            lower_expr(lhs, asm)?;
            lower_expr(rhs, asm)?;
            emit_binop(op, ty, asm);
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

fn emit_binop(op: &BinOp, ty: &Ty, asm: &mut Assembler) {
    match ty {
        Ty::Int => emit_int_binop(op, asm),
        Ty::Void => {
            unreachable!("BinOp with Ty::Void is unreachable due semantic analyses")
        }
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
