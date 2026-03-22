use crate::ast::{Function, Instruction, Program};
use crate::error::{CodegenError, CodegenResult, ParseResult};
//TODO: improve build flow; define explictily when encoding, offset arithmetic ...
//TODO: all Instruction data
//TODO: call support symbol table, 2-pass ...

// opcodes
pub const OP_GROW_STACK:  u8 = 0x07;
pub const OP_LOAD_ARG:    u8 = 0x0b;
pub const OP_ALU:         u8 = 0x14;
pub const OP_RETV:        u8 = 0x06;

pub const ALU_ADD: u16 = 0;

// Header
pub const HEADER_SIZE:u32 = 0x20;
const B40_ALPHABET: &[u8; 40] = b" 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-/";

pub struct FscriptHeader {
    // plain string, encoded to B40 string on serialization
    pub script_name: B40String,
    pub instructions_ptr: u32,
    pub symbol_table_ptr: u32,
    pub string_table_ptr: u32,
}

pub type B40String = String;

fn encode_b40_uint(s: &[u8]) -> CodegenResult<u32> {
    let mut result = 0u32;

    for &byte in s.iter().take(6){
        let upper = byte.to_ascii_uppercase();
        let idx = B40_ALPHABET
            .iter()
            .position(|&c| c == upper)
            .ok_or(CodegenError::InvalidB40Char(upper as char))?;
        result = result * 40 + idx as u32;
    }
    for _ in s.len()..6 {
        result *= 40;
    }
    Ok(result)
}
fn encode_b40(name: &str) -> CodegenResult<[u8; 8]> {
    let bytes = name.as_bytes();
    let first  = encode_b40_uint(&bytes[..bytes.len().min(6)])?;
    // just silently truncate longer strings TODO: explicitly define behavior
    let second = encode_b40_uint(if bytes.len() > 6 { &bytes[6..12.min(bytes.len())] } else { &[]
    })?;
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&first.to_be_bytes());
    out[4..8].copy_from_slice(&second.to_be_bytes());
    Ok(out)
}
fn decode_b40_uint(mut val: u32) -> String {
    let mut chars = [0u8; 6];
    for i in (0..6).rev() {
        let idx = (val % 40) as usize;
        chars[i] = B40_ALPHABET[idx];
        val /= 40;
    }
    unsafe { String::from_utf8_unchecked(chars.to_vec()) }
}
pub fn decode_b40(bytes: &[u8; 8]) -> String {
    let a = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let b = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
    (decode_b40_uint(a) + &decode_b40_uint(b)).trim_end().to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b40_encode() {
        let encoded = encode_b40("EVAR01ZN01_N").unwrap();
        assert_eq!(encoded, [0x60, 0x7a, 0xed, 0x2a, 0xdf, 0x64, 0x8c, 0x60]);
    }

    #[test]
    fn test_b40_decode() {
        let decoded = decode_b40(&[0x60, 0x7a, 0xed, 0x2a, 0xdf, 0x64, 0x8c, 0x60]);
        assert_eq!(decoded, "EVAR01ZN01_N");
    }

    #[test]
    fn test_b40_roundtrip() {
        let name = "ADD";
        let encoded = encode_b40(name).unwrap();
        let decoded = decode_b40(&encoded);
        assert_eq!(decoded, name);
    }
    #[test]
    fn test_b40_roundtrip_full_len() {
        let name = "EVAR01ZN01_N";
        let encoded = encode_b40(name).unwrap();
        let decoded = decode_b40(&encoded);
        assert_eq!(decoded, name);
    }

    #[test]
    fn test_b40_invalid_char() {
        let result = encode_b40("INVALID!");
        assert!(matches!(result, Err(CodegenError::InvalidB40Char('!'))));
    }
}

// Codegen
pub struct Codegen {
    pub code: Vec<u8>,
    pub symbol_table: SymbolTable,
    pub string_table: StringTable,
}

pub struct SymbolTable {
    entries: Vec<SymbolEntry>,
}
pub struct StringTable {
    entries: Vec<String>,
}

pub struct SymbolEntry {
    pub name:    B40String,  // plain string, encoded to B40 string on serialization
    pub offset:  u32, // file_offset; modified in serialization
}
impl SymbolEntry {
    pub fn serialize(&self) -> CodegenResult<[u8; 12]> {
        let mut buf = [0u8; 12];
        buf[0x00..0x08].copy_from_slice(&encode_b40(&self.name)?);
        buf[0x08..0x0c].copy_from_slice(&(self.offset / 4).to_be_bytes());

        Ok(buf)
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, name: B40String, offset: u32) {
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
        Self { entries: Vec::new() }
    }

    pub fn serialize(&self) -> CodegenResult<Vec<u8>> {
        let mut buf = Vec::new();
        //TODO: fill with real used strings
        buf.extend_from_slice("dummy".as_bytes());
        Ok(buf)
    }

}

impl Codegen {
    pub fn new() -> Self {
        Self { code: Vec::new(), symbol_table: SymbolTable::new(), string_table: StringTable::new() }
    }

    fn emit_insn(&mut self, opcode: u8, subtype: u8, operand: i16) {
        let word: u32 = ((operand as u32) << 16)
            | ((subtype as u32)         <<  8)
            |  (opcode  as u32);
        self.code.extend_from_slice(&word.to_be_bytes());
    }

    pub fn emit_program(&mut self, program: &Program) -> ParseResult<()> {
        for function in &program.functions {
            if !function.private{
                let offset = self.code.len() as u32;
                //TODO: explicitly handle offset handling for functions; because this is only
                // offset for instruction section
                self.symbol_table.add(function.name.clone(), offset + HEADER_SIZE );
            }
            self.emit_function(function)?;
        }
        Ok(())
    }

    fn emit_function(&mut self, function: &Function) -> ParseResult<()> {
        for instruction in &function.body {
            self.emit_instruction(instruction)?;
        }
        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> ParseResult<()> {
        match instruction {
            Instruction::GrowStack(n) => {
                self.emit_insn(OP_GROW_STACK, 0, *n as i16);
            }

            Instruction::LoadArg(n) => {
                self.emit_insn(OP_LOAD_ARG, 0, *n as i16);
            }

            Instruction::Add => {
                self.emit_insn(OP_ALU, 0, ALU_ADD as i16);
            }

            Instruction::Retv(n) => {
                self.emit_insn(OP_RETV, 1, *n as i16);
            }
            Instruction::Ret(n) => {
                self.emit_insn(OP_RETV, 0, *n as i16);
            }
        }
        Ok(())
    }

    pub fn finalize(&self, script_name: String) -> CodegenResult<Vec<u8>> {
        let base = 0x00000000;
        let code_bytes = &self.code;
        let sym_bytes  = self.symbol_table.serialize()?;
        let string_bytes = self.string_table.serialize()?;
        let instructions_ptr = base + HEADER_SIZE;
        let symbol_table_ptr = instructions_ptr + code_bytes.len() as u32;
        let string_table_ptr = symbol_table_ptr + sym_bytes.len() as u32;

        let header = FscriptHeader::new(
            script_name,
            instructions_ptr,
            symbol_table_ptr,
            string_table_ptr,
        );

        let mut out = Vec::with_capacity(string_table_ptr as usize);
        out.extend_from_slice(&header.serialize()?);
        out.extend_from_slice(&self.code);
        out.extend_from_slice(&sym_bytes);
        out.extend_from_slice(&string_bytes);
        Ok(out)
    }
}