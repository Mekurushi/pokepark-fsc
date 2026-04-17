use crate::error::SemaResult;
use crate::resolve::{declare_items, ScopeStack};
use crate::symbol::SymbolTable;
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
    let mut scope = ScopeStack::new();
    let mut symbols = SymbolTable::new();
    declare_items(script, &mut scope, &mut symbols)?;
    let items = script
        .items
        .iter()
        .map(|item| analyze_item(item, &mut symbols, &mut scope))
        .collect::<SemaResult<Vec<_>>>()?;

    Ok(hir::Script { items })
}

fn analyze_func(
    func: &ast::FuncDef,
    symbol_table: &mut SymbolTable,
    scope: &mut ScopeStack,
) -> SemaResult<hir::FuncDef> {
    let resolved = resolve::resolve_fn(func, symbol_table, scope)?;
    check::check_fn(func, &resolved)?;
    lower::lower_fn(func, &resolved)
}

fn analyze_item(
    item: &ast::Item,
    symbol_table: &mut SymbolTable,
    scope: &mut ScopeStack,
) -> SemaResult<hir::Item> {
    match item {
        ast::Item::FuncDef(func) => {
            Ok(hir::Item::FuncDef(analyze_func(func, symbol_table, scope)?))
        }
    }
}
