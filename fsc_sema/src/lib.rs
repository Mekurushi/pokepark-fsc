use crate::error::SemaResult;
use fsc_parse::ast;

mod check;
mod error;
pub mod frame;
pub mod hir;
mod infer;
mod lower;
mod resolve;
mod symbol;

pub fn analyze(script: &ast::Script) -> SemaResult<hir::Script> {
    let items = script
        .items
        .iter()
        .map(analyze_item)
        .collect::<SemaResult<Vec<_>>>()?;

    Ok(hir::Script { items })
}

pub fn analyze_func(func: &ast::FuncDef) -> SemaResult<hir::FuncDef> {
    let resolved = resolve::resolve_fn(func)?;
    check::check_fn(func, &resolved)?;
    lower::lower_fn(func, &resolved)
}

fn analyze_item(item: &ast::Item) -> SemaResult<hir::Item> {
    match item {
        ast::Item::FuncDef(func) => Ok(hir::Item::FuncDef(analyze_func(func)?)),
    }
}
