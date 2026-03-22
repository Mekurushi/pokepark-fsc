use crate::ast::{Function, Instruction, Program};
use crate::error::{CodegenResult, ParseResult};
use crate::formats::{FscriptHeader, StringTable, SymbolTable, HEADER_SIZE};
use std::collections::{HashMap, HashSet};
//TODO: improve build flow; define explictily when encoding, offset arithmetic ...
//TODO: all Instruction data
//TODO: call support symbol table, 2-pass ...

// opcodes
pub const OP_GROW_STACK:  u8 = 0x07;
pub const OP_LOAD_ARG:    u8 = 0x0b;
pub const OP_ALU:         u8 = 0x14;
pub const OP_RETV:        u8 = 0x06;

pub const ALU_ADD: u16 = 0;

// Codegen
pub struct Codegen {
    pub code: Vec<u8>,
    pub symbols: HashMap<String, u32>,
    pub exports: HashSet<String>,
}


impl Codegen {
    pub fn new() -> Self {
        Self { code: Vec::new(), symbols: HashMap::new(), exports: HashSet::new() }
    }

    fn emit_insn(&mut self, opcode: u8, subtype: u8, operand: i16) {
        let word: u32 = ((operand as u32) << 16)
            | ((subtype as u32)         <<  8)
            |  (opcode  as u32);
        self.code.extend_from_slice(&word.to_be_bytes());
    }

    pub fn emit_program(&mut self, program: &Program) -> ParseResult<()> {
        for function in &program.functions {
            let offset = self.code.len() as u32;
            self.symbols.insert(function.name.clone(), offset);
            if !function.private {
                self.exports.insert(function.name.clone());
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

    pub fn assemble_fsb(&self, script_name: String) -> CodegenResult<Vec<u8>> {
        let base = 0x00000000;
        let code_bytes = &self.code;
        let mut symbol_table = SymbolTable::new();
        for symbol in &self.exports{
            let offset = match self.symbols.get(symbol){
                Some(offset) => offset,
                None => { continue; }//TODO: explicit Codegen Error
            };
            symbol_table.add(symbol.clone(), *offset)
        }
        let string_table = StringTable::new();

        let sym_bytes  = symbol_table.serialize()?;
        let string_bytes = string_table.serialize()?;

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