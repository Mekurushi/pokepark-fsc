use crate::b40string::encode_b40;
use crate::error::CodegenResult;

// Header
pub const HEADER_SIZE:u32 = 0x20;

pub struct FscriptHeader {
    // plain string, encoded to B40 string on serialization
    pub script_name: String,
    pub instructions_ptr: u32,
    pub symbol_table_ptr: u32,
    pub string_table_ptr: u32,
}


impl FscriptHeader {
    pub fn new(script_name: String, instructions_ptr: u32, symbol_table_ptr: u32,
               string_table_ptr: u32) -> Self {
        Self {
            script_name,
            instructions_ptr,
            symbol_table_ptr,
            string_table_ptr,
        }
    }

    pub fn serialize(&self) -> CodegenResult<[u8; 32]> {
        let mut buf = [0u8; 32];
        buf[0x00..0x04].copy_from_slice(&self.instructions_ptr.to_be_bytes());
        buf[0x04..0x08].copy_from_slice(&self.symbol_table_ptr.to_be_bytes());
        buf[0x08..0x0c].copy_from_slice(&self.string_table_ptr.to_be_bytes()); // unused, mirrors
        buf[0x0c..0x10].copy_from_slice(&self.string_table_ptr.to_be_bytes());
        buf[0x10..0x14].copy_from_slice(&0u32.to_be_bytes());                  // unused2
        buf[0x14..0x1c].copy_from_slice(&encode_b40(&self.script_name)?);
        buf[0x1c..0x20].copy_from_slice(&0x01000000u32.to_be_bytes());         // unused3
        Ok(buf)
    }
}


pub struct SymbolTable {
    entries: Vec<SymbolEntry>,
}
pub struct StringTable {
    _entries: Vec<String>,
}

pub struct SymbolEntry {
    pub name:    String,  // plain string, encoded to B40 string on serialization
    pub offset:  u32, // file_offset; modified in serialization
}
impl SymbolEntry {
    pub fn serialize(&self) -> CodegenResult<[u8; 12]> {
        let mut buf = [0u8; 12];
        buf[0x00..0x08].copy_from_slice(&encode_b40(&self.name)?);
        // TODO: replace hardcoded HEADER_SIZE with other method to get file offset
        buf[0x08..0x0c].copy_from_slice(&((self.offset + HEADER_SIZE) / 4).to_be_bytes());

        Ok(buf)
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, name: String, offset: u32) {
        self.entries.push(SymbolEntry { name, offset });
    }

    pub fn serialize(&self) -> CodegenResult<Vec<u8>> {
        let mut buf = Vec::new();
        for entry in &self.entries {
            buf.extend_from_slice(&entry.serialize()?);
        }
        Ok(buf)
    }
}

impl StringTable {
    pub fn new() -> Self {
        Self { _entries: Vec::new() }
    }

    pub fn serialize(&self) -> CodegenResult<Vec<u8>> {
        let mut buf = Vec::new();
        //TODO: fill with real used strings
        buf.extend_from_slice("dummy".as_bytes());
        Ok(buf)
    }

}