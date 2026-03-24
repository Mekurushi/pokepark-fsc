use std::collections::HashMap;
use crate::error::{AssemblerError, AssemblerResult};

pub enum Scope {
    Export,           // entry points
    Private,
    Local(String),    // label
}

pub struct Symbol {
    pub name: String,
    pub offset: u32,
    pub scope: Scope,
}

pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
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
        self.symbols.insert(name.clone(), Symbol { name, offset, scope });
        Ok(())
    }

    pub fn define_local(&mut self, function: &str, label: String, offset: u32) {
        let key = format!("{}.{}", function, label);
        self.symbols.insert(key, Symbol {
            name: label,
            offset,
            scope: Scope::Local(function.to_string()),
        });
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn lookup_local(&self, function: &str, label: &str) -> Option<&Symbol> {
        self.symbols.get(&format!("{}.{}", function, label))
    }

    pub fn exports(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values().filter(|s| matches!(s.scope, Scope::Export))
    }
}

pub struct SymbolResolver<'a> {
    symbol_table: &'a SymbolTable,
    function: &'a str,
}

impl<'a> SymbolResolver<'a> {
    pub fn new(symbol_table: &'a SymbolTable, function: &'a str) -> Self {
        Self { symbol_table, function }
    }

    pub fn resolve_global(&self, name: &str) -> AssemblerResult<&Symbol> {
        self.symbol_table
            .lookup(name)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(name.to_string()))
    }

    pub fn resolve_local(&self, label: &str) -> AssemblerResult<&Symbol> {
        self.symbol_table
            .lookup_local(self.function, label)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(label.to_string()))
    }
}