use crate::binary::string_table::BinaryStringTable;
use crate::error::{AssemblerError, AssemblerResult};
use std::collections::HashMap;
const MAX_STRING_TABLE_SIZE: u32 = u16::MAX as u32;

pub struct StringTable {
    buffer: Vec<u8>,
    index: HashMap<String, u32>,
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}

impl StringTable {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> AssemblerResult<u32> {
        if let Some(&offset) = self.index.get(s) {
            return Ok(offset);
        }
        let offset =
            u32::try_from(self.buffer.len()).map_err(|_foo| AssemblerError::StringTableFull)?;

        if offset > MAX_STRING_TABLE_SIZE {
            return Err(AssemblerError::StringTableFull);
        }
        self.buffer.extend_from_slice(s.as_bytes());
        self.buffer.push(0);
        self.index.insert(s.to_string(), offset);
        Ok(offset)
    }

    pub fn into_binary(self) -> BinaryStringTable {
        BinaryStringTable::new(self.buffer)
    }
}
