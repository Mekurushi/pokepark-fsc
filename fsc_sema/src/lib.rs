use crate::check::check_func;
use fsc_parse::ast::FuncDef;

mod check;
mod error;

pub fn check(func: &Vec<FuncDef>) {
    for fun in func {
        match check_func(fun) {
            Ok(_) => (),
            Err(e) => todo!(),
        }
    }
}
