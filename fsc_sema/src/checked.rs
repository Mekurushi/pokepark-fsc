use crate::resolve::Resolutions;
use crate::symbol::SymbolTable;
use fsc_parse::ast;

pub struct CheckedScript {
    pub(crate) script: ast::Script,
    pub(crate) symbols: SymbolTable,
    pub(crate) functions: Vec<CheckedFunction>,
}

impl CheckedScript {
    #[must_use]
    pub const fn ast(&self) -> &ast::Script {
        &self.script
    }
}

pub struct CheckedFunction {
    pub(crate) function: ast::FuncDef,
    pub(crate) resolutions: Resolutions,
}
