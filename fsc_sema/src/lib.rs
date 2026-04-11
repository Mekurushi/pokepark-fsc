use crate::error::SemaResult;
use fsc_parse::ast::FuncDef;

mod check;
mod error;
pub mod frame;
pub mod hir;
mod infer;
mod lower;
mod resolve;

pub fn analyze(func: &FuncDef) -> SemaResult<hir::FuncDef> {
    let frame = frame::build_frame(func)?;

    let scope = resolve::resolve_func(func)?;

    check::check_func(func, &scope)?;
    lower::lower_func(func, frame, &scope)
}

pub fn check(funcs: &[FuncDef]) -> SemaResult<Vec<hir::FuncDef>> {
    funcs.iter().map(analyze).collect::<SemaResult<Vec<_>>>()
}
