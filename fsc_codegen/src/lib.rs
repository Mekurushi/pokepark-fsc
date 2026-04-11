use crate::lower::lower_func;
use fsc_assembler::Assembler;
use fsc_sema::hir::FuncDef;

mod error;
mod lower;

// TODO: official entrypoint and better crate visibility handling

pub fn lower(func: &Vec<FuncDef>, asm: &mut Assembler) {
    for fun in func {
        match lower_func(fun, asm) {
            Ok(()) => (),
            Err(e) => todo!(),
        }
    }
}
