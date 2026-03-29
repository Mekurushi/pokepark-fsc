use crate::binary::string_table::BinaryStringTable;
use crate::error::{AssemblerError, AssemblerResult};
use std::collections::HashMap;

pub struct StringTable {
    buffer: Vec<u8>,
    index: HashMap<String, u32>,
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
        let offset = self.buffer.len() as u32;
        i16::try_from(offset).map_err(|_| AssemblerError::OperandOutOfRange(offset as i32))?;
        self.buffer.extend_from_slice(s.as_bytes());
        self.buffer.push(0);
        self.index.insert(s.to_string(), offset);
        Ok(offset)
    }

    pub fn into_binary(self) -> BinaryStringTable {
        BinaryStringTable::new(self.buffer)
    }
}
