use crate::ast::{Function, Instruction, Program};
use crate::error::{CodegenError, CodegenResult, ParseResult};
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

pub struct FsbHeader {
    pub script_name: String,
    pub instructions_ptr: u32,
    pub symbol_table_ptr: u32,
    pub string_table_ptr: u32,
}
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
impl FsbHeader {
    pub fn new(script_name: String, base: u32, code_size: u32) -> Self {
        Self {
            script_name,
            instructions_ptr: base + HEADER_SIZE,
            symbol_table_ptr: base + HEADER_SIZE + code_size, // TODO: real symbol_table
            string_table_ptr: base + HEADER_SIZE + code_size, // TODO: real string_table
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
}

impl Codegen {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    fn emit_insn(&mut self, opcode: u8, subtype: u8, operand: i16) {
        let word: u32 = ((operand as u32) << 16)
            | ((subtype as u32)         <<  8)
            |  (opcode  as u32);
        self.code.extend_from_slice(&word.to_be_bytes());
    }

    pub fn emit_program(&mut self, program: &Program) -> ParseResult<()> {
        for function in &program.functions {
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

    pub fn finalize(&self, header: &FsbHeader) -> CodegenResult<Vec<u8>> {
        let mut out = Vec::with_capacity(32 + self.code.len());
        out.extend_from_slice(&header.serialize()?);
        out.extend_from_slice(&self.code);
        Ok(out)
    }
}