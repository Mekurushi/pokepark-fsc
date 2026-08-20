use crate::SymbolTableParseError;
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ExternalSymbolTable {
    symbols: HashMap<String, u32>,
}

impl ExternalSymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: String, address: u32) -> Option<u32> {
        self.symbols.insert(name, address)
    }

    pub fn get(&self, name: &str) -> Option<u32> {
        self.symbols.get(name).copied()
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

pub fn parse_symbol_table(input: &str) -> Result<ExternalSymbolTable, SymbolTableParseError> {
    let symbols = toml::from_str(input).map_err(|source| SymbolTableParseError { source })?;
    Ok(ExternalSymbolTable { symbols })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn parses_hexadecimal_symbols_with_comments_and_whitespace() {
        let symbols = parse_symbol_table(
            r#"
                # Exported functions
                MAIN = 0x50020
                UPDATE = 0x5005c
            "#,
        )
        .unwrap();

        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols.get("MAIN"), Some(0x50020));
        assert_eq!(symbols.get("UPDATE"), Some(0x5005c));
    }

    #[test]
    fn rejects_duplicate_symbols() {
        assert!(parse_symbol_table("MAIN = 0x50020\nMAIN = 0x50024").is_err());
    }

    #[test]
    fn rejects_negative_addresses() {
        assert!(parse_symbol_table("MAIN = -1").is_err());
    }

    #[test]
    fn rejects_addresses_larger_than_u32() {
        assert!(parse_symbol_table("MAIN = 0x100000000").is_err());
    }

    #[test]
    fn rejects_non_integer_addresses() {
        assert!(parse_symbol_table("MAIN = \"0x50020\"").is_err());
    }

    #[test]
    fn rejects_malformed_toml() {
        assert!(parse_symbol_table("MAIN: 0x50020").is_err());
    }
}
