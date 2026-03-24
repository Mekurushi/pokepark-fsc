use crate::binary::header::{FscriptHeader, HEADER_SIZE};
use crate::binary::string_table::BinaryStringTable;
use crate::binary::symbol_table::BinarySymbolTable;
use crate::error::AssemblerResult;

mod header;
pub mod symbol_table;
pub mod string_table;
mod b40string;

pub struct FscriptBinary {
    script_name: String,
    code: Vec<u8>,
    symbol_table: BinarySymbolTable,
    string_table: BinaryStringTable,
}

impl FscriptBinary {
    pub fn new(
        program_name: String,
        code: Vec<u8>,
        symbol_table: BinarySymbolTable,
        string_table: BinaryStringTable,
    ) -> Self {
        Self { script_name: program_name, code, symbol_table, string_table }
    }

    pub fn serialize(&self) -> AssemblerResult<Vec<u8>> {
        let code_ptr         = HEADER_SIZE;
        let symbol_table_ptr = code_ptr + self.code.len() as u32;
        let string_table_ptr = symbol_table_ptr + self.symbol_table.byte_len() as u32;

        let header = FscriptHeader::new(
            self.script_name.clone(),
            code_ptr,
            symbol_table_ptr,
            string_table_ptr,
        );

        let mut out = Vec::new();
        out.extend_from_slice(&header.serialize()?);
        out.extend_from_slice(&self.code);
        out.extend_from_slice(&self.symbol_table.serialize(code_ptr)?);
        out.extend_from_slice(&self.string_table.serialize()?);
        Ok(out)
    }
}