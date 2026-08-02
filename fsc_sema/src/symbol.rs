use fsc_diagnostics::Span;
use fsc_parse::ast::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub name_span: Span,
    pub ty: Ty,
    pub type_span: Span,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamInfo {
    pub name: String,
    pub name_span: Span,
    pub ty: Ty,
    pub type_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Param { index: u32 },
    Local,
    Function { ret_ty: Ty, params: Vec<ParamInfo> },
}
#[derive(Default, Clone)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    pub fn insert(&mut self, symbol: Symbol) -> SymbolId {
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(symbol);
        id
    }

    pub fn get(&self, id: SymbolId) -> &Symbol {
        let symbol = self.symbols.get(id.0 as usize);
        match symbol {
            Some(symbol) => symbol,
            None => todo!("Symbol not found"), //TODO: explicit error
        }
    }
}
