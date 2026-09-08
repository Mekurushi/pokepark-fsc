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

    pub(crate) fn function_offset(&self, name: &str) -> Option<u32> {
        self.symbols
            .get(name)
            .filter(|symbol| matches!(symbol.scope, Scope::Export | Scope::Private))
            .map(|symbol| symbol.offset)
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

    pub(crate) fn rebase(&mut self, offset: u32) -> AssemblerResult<()> {
        if self
            .symbols
            .values()
            .any(|symbol| symbol.offset.checked_add(offset).is_none())
        {
            return Err(AssemblerError::AddressOverflow);
        }

        for symbol in self.symbols.values_mut() {
            symbol.offset += offset;
        }
        Ok(())
    }

    pub(crate) fn merge(&mut self, other: Self) -> AssemblerResult<()> {
        for (key, symbol) in other.symbols {
            if self.symbols.contains_key(&key) {
                return Err(AssemblerError::DuplicateSymbol(symbol.name));
            }
            self.symbols.insert(key, symbol);
        }
        Ok(())
    }

    pub(crate) fn rename_function(
        &mut self,
        current_name: &str,
        new_name: &str,
    ) -> AssemblerResult<()> {
        if current_name == new_name {
            return self
                .symbols
                .contains_key(current_name)
                .then_some(())
                .ok_or_else(|| AssemblerError::UndefinedSymbol(current_name.to_owned()));
        }
        if !self.symbols.contains_key(current_name) {
            return Err(AssemblerError::UndefinedSymbol(current_name.to_owned()));
        }
        if self.symbols.contains_key(new_name) {
            return Err(AssemblerError::DuplicateSymbol(new_name.to_owned()));
        }

        let renamed_locals = self
            .symbols
            .iter()
            .filter_map(|(key, symbol)| match &symbol.scope {
                Scope::Local(function) if function == current_name => {
                    Some((key.clone(), format!("{new_name}.{}", symbol.name)))
                }
                Scope::Export | Scope::Private | Scope::Local(_) => None,
            })
            .collect::<Vec<_>>();
        if let Some((_, duplicate)) = renamed_locals
            .iter()
            .find(|(_, renamed)| self.symbols.contains_key(renamed))
        {
            return Err(AssemblerError::DuplicateSymbol(duplicate.clone()));
        }

        let mut function = self
            .symbols
            .remove(current_name)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(current_name.to_owned()))?;
        new_name.clone_into(&mut function.name);
        self.symbols.insert(new_name.to_owned(), function);

        for (current_key, new_key) in renamed_locals {
            let mut local = self
                .symbols
                .remove(&current_key)
                .ok_or_else(|| AssemblerError::UndefinedSymbol(current_key.clone()))?;
            local.scope = Scope::Local(new_name.to_owned());
            self.symbols.insert(new_key, local);
        }

        Ok(())
    }

    pub(crate) fn make_function_private(&mut self, name: &str) -> AssemblerResult<()> {
        let symbol = self
            .symbols
            .get_mut(name)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(name.to_owned()))?;
        match symbol.scope {
            Scope::Export => symbol.scope = Scope::Private,
            Scope::Private => {}
            Scope::Local(_) => {
                return Err(AssemblerError::UndefinedSymbol(name.to_owned()));
            }
        }
        Ok(())
    }
}
