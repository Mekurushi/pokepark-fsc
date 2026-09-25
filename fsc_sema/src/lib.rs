use crate::error::SemaResult;
use crate::resolve::{ScopeStack, declare_items};
use crate::symbol::SymbolTable;
use fsc_parse::ast;

mod bind;
mod check;
mod checked;
mod error;
pub mod hir;
mod infer;
pub mod local;
pub mod place;
mod resolve;
mod symbol;
pub mod types;

pub use bind::{
    BindError, BindErrors, BoundScript, ConfigRequirement, ConfigType, ConfigValue, ConfigValues,
    bind_configs,
};
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

pub fn lower(bound: BoundScript) -> SemaResult<hir::Script> {
    hir::lower_script(bound)
}
