use crate::error::{AssemblerError, AssemblerResult};
use std::collections::HashMap;

pub enum Scope {
    Export, // entry points
    Private,
    Local(String), // label
}

pub struct Symbol {
    pub name: String,
    pub offset: u32,
    pub scope: Scope,
}

pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, offset: u32, scope: Scope) -> AssemblerResult<()> {
        if self.symbols.contains_key(&name) {
            return Err(AssemblerError::DuplicateSymbol(name));
        }
        self.symbols.insert(
            name.clone(),
            Symbol {
                name,
                offset,
                scope,
            },
        );
        Ok(())
    }

    pub fn define_local(&mut self, function: &str, label: String, offset: u32) {
        let key = format!("{function}.{label}");
        self.symbols.insert(
            key,
            Symbol {
                name: label,
                offset,
                scope: Scope::Local(function.to_string()),
            },
        );
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn lookup_local(&self, function: &str, label: &str) -> Option<&Symbol> {
        self.symbols.get(&format!("{function}.{label}"))
    }

    pub fn resolve_global(&self, name: &str) -> AssemblerResult<&Symbol> {
        self.lookup(name)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(name.to_string()))
    }

    pub fn resolve_local(&self, function: &str, label: &str) -> AssemblerResult<&Symbol> {
        self.lookup_local(function, label)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(label.to_string()))
    }

    pub fn exports(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols
            .values()
            .filter(|s| matches!(s.scope, Scope::Export))
    }
}
