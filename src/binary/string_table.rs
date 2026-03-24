use crate::error::AssemblerResult;

pub struct BinaryStringTable {
    _entries: Vec<String>,
}



impl BinaryStringTable {
    pub fn new() -> Self {
        Self { _entries: Vec::new() }
    }

    pub fn serialize(&self) -> AssemblerResult<Vec<u8>> {
        let mut buf = Vec::new();
        //TODO: fill with real used strings
        buf.extend_from_slice("dummy".as_bytes());
        Ok(buf)
    }

}