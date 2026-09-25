use crate::error::SemaResult;
use crate::resolve::{ScopeStack, declare_items};
use crate::symbol::SymbolTable;
use fsc_parse::ast;

mod bind;
mod checked;
mod error;
pub mod hir;
mod resolve;
mod symbol;
mod type_check;
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

    for item in &script.items {
        if let ast::Item::ConstDecl(constant) = item {
            type_check::check_constant(constant)?;
        }
    }

    let mut functions = Vec::new();
    for item in &script.items {
        if let ast::Item::FuncDef(function) = item {
            let resolved = resolve::resolve_fn(function, &mut symbols, &mut scope)?;
            let expression_types = type_check::check_fn(function, &resolved.resolutions, &symbols)?;
            functions.push(CheckedFunction {
                function: function.clone(),
                symbol: resolved.symbol,
                resolutions: resolved.resolutions,
                expression_types,
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
