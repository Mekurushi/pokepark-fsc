use crate::binary::b40string::encode_b40;
use crate::error::{AssemblerError, AssemblerResult};
const SYMBOL_ENTRY_SIZE: u32 = 12;

pub struct BinarySymbolTable {
    entries: Vec<SymbolEntry>,
}

pub struct SymbolEntry {
    pub name: String, // plain string, encoded to B40 string on serialization
    pub offset: u32,  // file_offset; modified in serialization
}

impl SymbolEntry {
    pub fn serialize(&self, code_ptr: u32) -> AssemblerResult<[u8; 12]> {
        let mut buf = [0u8; 12];
        buf[0x00..0x08].copy_from_slice(&encode_b40(&self.name)?);
        // TODO: replace hardcoded HEADER_SIZE with other method to get file offset
        buf[0x08..0x0c].copy_from_slice(&((self.offset + code_ptr) / 4).to_be_bytes());

        Ok(buf)
    }
}

impl Default for BinarySymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl BinarySymbolTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String, offset: u32) {
        self.entries.push(SymbolEntry { name, offset });
    }

    pub fn serialize(&self, code_ptr: u32) -> AssemblerResult<Vec<u8>> {
        let mut buf = Vec::new();
        for entry in &self.entries {
            buf.extend_from_slice(&entry.serialize(code_ptr)?);
        }
        Ok(buf)
    }

    pub fn byte_len(&self) -> AssemblerResult<u32> {
        Ok(u32::try_from(self.entries.len())
            .map_err(|_err| AssemblerError::SectionTooLarge("Symbol_Table"))?
            * SYMBOL_ENTRY_SIZE)
    }
}
