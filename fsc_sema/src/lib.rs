use crate::error::SemaResult;
use crate::resolve::{ScopeStack, declare_items};
use crate::symbol::SymbolTable;
use fsc_parse::ast;

mod check;
mod checked;
mod error;
pub mod frame;
pub mod hir;
mod infer;
mod resolve;
mod symbol;

pub use checked::{CheckedFunction, CheckedScript};

pub fn check(script: ast::Script) -> SemaResult<CheckedScript> {
    let mut scope = ScopeStack::new();
    let mut symbols = SymbolTable::new();
    declare_items(&script, &mut scope, &mut symbols)?;

    let mut functions = Vec::new();
    for item in &script.items {
        if let ast::Item::FuncDef(function) = item {
            let resolutions = resolve::resolve_fn(function, &mut symbols, &mut scope)?;
            check::check_fn(function, &resolutions, &symbols)?;
            functions.push(CheckedFunction {
                function: function.clone(),
                resolutions,
            });
        }
    }

    Ok(CheckedScript {
        script,
        symbols,
        functions,
    })
}

pub fn lower(checked: CheckedScript) -> SemaResult<hir::Script> {
    hir::lower_script(checked)
}
