use crate::error::{AssemblerError, AssemblerResult};
use std::collections::HashMap;

pub enum SymbolKind {
    Function { exported: bool },
    Data,
}

pub struct Symbol {
    offset: u32,
    kind: SymbolKind,
}

impl Symbol {
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    pub const fn is_function(&self) -> bool {
        matches!(self.kind, SymbolKind::Function { .. })
    }

    pub const fn is_exported_function(&self) -> bool {
        matches!(self.kind, SymbolKind::Function { exported: true })
    }
}

pub struct Label {
    offset: u32,
}

impl Label {
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}

pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    labels_by_function: HashMap<String, HashMap<String, Label>>,
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
            labels_by_function: HashMap::new(),
        }
    }

    pub fn define_function(
        &mut self,
        name: String,
        offset: u32,
        exported: bool,
    ) -> AssemblerResult<()> {
        self.define_symbol(
            name,
            Symbol {
                offset,
                kind: SymbolKind::Function { exported },
            },
        )
    }

    pub fn define_data(&mut self, name: String, offset: u32) -> AssemblerResult<()> {
        self.define_symbol(
            name,
            Symbol {
                offset,
                kind: SymbolKind::Data,
            },
        )
    }

    fn define_symbol(&mut self, name: String, symbol: Symbol) -> AssemblerResult<()> {
        if self.symbols.contains_key(&name) {
            return Err(AssemblerError::DuplicateSymbol(name));
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }

    pub fn define_label(
        &mut self,
        function: &str,
        name: String,
        offset: u32,
    ) -> AssemblerResult<()> {
        let labels = self
            .labels_by_function
            .entry(function.to_owned())
            .or_default();
        if labels.contains_key(&name) {
            return Err(AssemblerError::DuplicateSymbol(name));
        }
        labels.insert(name, Label { offset });
        Ok(())
    }

    pub fn resolve_function_offset(&self, name: &str) -> AssemblerResult<u32> {
        match self.symbols.get(name) {
            Some(Symbol {
                offset,
                kind: SymbolKind::Function { .. },
            }) => Ok(*offset),
            Some(_) => Err(AssemblerError::InvalidSymbolKind {
                name: name.to_owned(),
                expected: "a function",
            }),
            None => Err(AssemblerError::UndefinedSymbol(name.to_owned())),
        }
    }

    pub fn resolve_data_offset(&self, name: &str) -> AssemblerResult<u32> {
        match self.symbols.get(name) {
            Some(Symbol {
                offset,
                kind: SymbolKind::Data,
            }) => Ok(*offset),
            Some(_) => Err(AssemblerError::InvalidSymbolKind {
                name: name.to_owned(),
                expected: "data",
            }),
            None => Err(AssemblerError::UndefinedSymbol(name.to_owned())),
        }
    }

    pub fn resolve_label(&self, function: &str, name: &str) -> AssemblerResult<&Label> {
        self.labels_by_function
            .get(function)
            .and_then(|labels| labels.get(name))
            .ok_or_else(|| AssemblerError::UndefinedSymbol(name.to_owned()))
    }

    pub fn exports(&self) -> impl Iterator<Item = (&str, &Symbol)> {
        self.symbols
            .iter()
            .filter(|(_, symbol)| symbol.is_exported_function())
            .map(|(name, symbol)| (name.as_str(), symbol))
    }

    pub(crate) fn rebase(&mut self, offset: u32) -> AssemblerResult<()> {
        let symbol_offsets = self.symbols.values().map(|symbol| symbol.offset);
        let label_offsets = self
            .labels_by_function
            .values()
            .flat_map(|labels| labels.values().map(|label| label.offset));
        if symbol_offsets
            .chain(label_offsets)
            .any(|symbol_offset| symbol_offset.checked_add(offset).is_none())
        {
            return Err(AssemblerError::AddressOverflow);
        }

        for symbol in self.symbols.values_mut() {
            symbol.offset += offset;
        }
        for labels in self.labels_by_function.values_mut() {
            for label in labels.values_mut() {
                label.offset += offset;
            }
        }
        Ok(())
    }

    pub(crate) fn merge(&mut self, other: Self) -> AssemblerResult<()> {
        for (name, symbol) in other.symbols {
            if self.symbols.contains_key(&name) {
                return Err(AssemblerError::DuplicateSymbol(name));
            }
            self.symbols.insert(name, symbol);
        }

        for (function, other_labels) in other.labels_by_function {
            let labels = self.labels_by_function.entry(function).or_default();
            for (name, label) in other_labels {
                if labels.contains_key(&name) {
                    return Err(AssemblerError::DuplicateSymbol(name));
                }
                labels.insert(name, label);
            }
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
                .get(current_name)
                .filter(|symbol| symbol.is_function())
                .map(|_| ())
                .ok_or_else(|| AssemblerError::UndefinedSymbol(current_name.to_owned()));
        }
        let Some(function) = self.symbols.get(current_name) else {
            return Err(AssemblerError::UndefinedSymbol(current_name.to_owned()));
        };
        if !function.is_function() {
            return Err(AssemblerError::UndefinedSymbol(current_name.to_owned()));
        }
        if self.symbols.contains_key(new_name) || self.labels_by_function.contains_key(new_name) {
            return Err(AssemblerError::DuplicateSymbol(new_name.to_owned()));
        }

        let function = self
            .symbols
            .remove(current_name)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(current_name.to_owned()))?;
        self.symbols.insert(new_name.to_owned(), function);

        if let Some(labels) = self.labels_by_function.remove(current_name) {
            self.labels_by_function.insert(new_name.to_owned(), labels);
        }
        Ok(())
    }

    pub(crate) fn make_function_private(&mut self, name: &str) -> AssemblerResult<()> {
        let symbol = self
            .symbols
            .get_mut(name)
            .ok_or_else(|| AssemblerError::UndefinedSymbol(name.to_owned()))?;
        match &mut symbol.kind {
            SymbolKind::Function { exported } => *exported = false,
            SymbolKind::Data => {
                return Err(AssemblerError::InvalidSymbolKind {
                    name: name.to_owned(),
                    expected: "a function",
                });
            }
        }
        Ok(())
    }
}
