use crate::ast::{Function, Instruction, Program};
use crate::error::{CodegenError, CodegenResult};
use crate::formats::{FscriptHeader, StringTable, SymbolTable, HEADER_SIZE};
use std::collections::{HashMap, HashSet};
//TODO: improve build flow; define explictily when encoding, offset arithmetic ...
//TODO: all Instruction data
//TODO: call support symbol table, 2-pass ...

// opcodes
#[repr(u8)]
pub enum Opcode {
    Call = 0x3,
    Ret      = 0x06,
    GrowStack = 0x07,
    LoadArg   = 0x0b,
    Push = 0x10,
    Alu       = 0x14,
}
#[repr(u16)]
pub enum AluOp {
    Add = 0,
    Sub = 1,
}

#[repr(u8)]
pub enum RetOp {
    Ret  = 0,
    Retv = 1,
}

// Codegen
pub struct Codegen {
    pub code: Vec<u8>,
    pub symbols: HashMap<String, u32>,
    pub exports: HashSet<String>
}


impl Codegen {
    pub fn new() -> Self {
        Self { code: Vec::new(), symbols: HashMap::new(), exports: HashSet::new() }
    }
    fn collect_symbols(&mut self, program: &Program) {
        let mut offset: u32 = 0;
        for function in &program.functions {
            self.symbols.insert(function.name.clone(), offset);
            if !function.private {
                self.exports.insert(function.name.clone());
            }
            offset += (function.body.len() as u32) * 4;
        }
    }
    fn emit_insn(&mut self, operand: i16, subtype: u8, opcode: u8) {
        let word: u32 = ((operand as u32) << 16)
            | ((subtype as u32)         <<  8)
            |  (opcode  as u32);
        self.code.extend_from_slice(&word.to_be_bytes());
    }

    pub fn emit_program(&mut self, program: &Program) -> CodegenResult<()> {
        // 1-pass symbol collection
        self.collect_symbols(program);

        // 2-pass code emission
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

    fn emit_function(&mut self, function: &Function) -> CodegenResult<()> {
        for instruction in &function.body {
            self.emit_instruction(instruction)?;
        }
        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> CodegenResult<()> {
        match instruction {
            Instruction::GrowStack(n) => {
                self.emit_insn(*n, 0, Opcode::GrowStack as u8);
            }

            Instruction::LoadArg(n) => {
                self.emit_insn( *n, 0,Opcode::LoadArg as u8);
            }

            Instruction::Add => {
                self.emit_insn(AluOp::Add as i16, 0, Opcode::Alu as u8);
            }
            Instruction::Sub => {
                self.emit_insn(AluOp::Sub as i16, 0, Opcode::Alu as u8);
            }
            Instruction::Push(n) => {
                self.emit_insn(*n, 0, Opcode::Push as u8);
            }
            Instruction::Call(symbol) => {
                let current_offset = self.code.len() as u32;
                let operand = match self.symbols.get(symbol) {
                    Some(&target) => compute_call_operand(current_offset, target)?,
                    None => {
                        return Err(CodegenError::UndefinedSymbol(symbol.clone()));
                    }
                };
                self.emit_insn(operand, 0, Opcode::Call as u8);
            }

            Instruction::Retv(n) => {
                // Ghidra visualizes as neagtive but actual operand is positive TODO: define
                // explicit
                let operand = n.unsigned_abs();
                self.emit_insn(operand as i16, RetOp::Retv as u8, Opcode::Ret as u8);
            }
            Instruction::Ret(n) => {
                let operand = n.unsigned_abs();
                self.emit_insn(operand as i16, RetOp::Ret as u8, Opcode::Ret as u8);
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

fn compute_call_operand(current_offset: u32, target_offset: u32) -> CodegenResult<i16> {
    let branch_offset = target_offset as i32 - (current_offset as i32 + 4);
    i16::try_from(branch_offset / 4)
        .map_err(|_| CodegenError::OperandOutOfRange(branch_offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Instruction;

    #[test]
    fn test_emit_grow_stack() {
        let mut cg = Codegen::new();
        cg.emit_instruction(&Instruction::GrowStack(1)).unwrap();
        assert_eq!(cg.code, &[0x00, 0x01, 0x00, 0x07]);
    }

    #[test]
    fn test_emit_load_arg() {
        let mut cg = Codegen::new();
        cg.emit_instruction(&Instruction::LoadArg(1)).unwrap();
        assert_eq!(cg.code, &[0x00, 0x01, 0x00, 0x0b]);
    }

    #[test]
    fn test_emit_add() {
        let mut cg = Codegen::new();
        cg.emit_instruction(&Instruction::Add).unwrap();
        assert_eq!(cg.code, &[0x00, 0x00, 0x00, 0x14]);
    }

    #[test]
    fn test_emit_sub() {
        let mut cg = Codegen::new();
        cg.emit_instruction(&Instruction::Sub).unwrap();
        assert_eq!(cg.code, &[0x00, 0x01, 0x00, 0x14]);
    }

    #[test]
    fn test_emit_retv() {
        let mut cg = Codegen::new();
        cg.emit_instruction(&Instruction::Retv(1)).unwrap();
        assert_eq!(cg.code, &[0x00, 0x01, 0x01, 0x06]);
    }
    #[test]
    fn test_emit_ret() {
        let mut cg = Codegen::new();
        cg.emit_instruction(&Instruction::Ret(1)).unwrap();
        assert_eq!(cg.code, &[0x00, 0x01, 0x00, 0x06]);
    }

    #[test]
    fn test_emit_call_undefined_symbol() {
        let mut cg = Codegen::new();
        let _symbol = String::from("invalid");
        let invalid_result =cg.emit_instruction(&Instruction::Call(_symbol));
        assert!(matches!(invalid_result, Err(CodegenError::UndefinedSymbol(_symbol))));

    }
    #[test]
    fn test_emit_call() {
        let mut cg = Codegen::new();
        let symbol = String::from("invalid");
        cg.symbols.insert(symbol.clone(), 1);
        cg.emit_instruction(&Instruction::Call(symbol)).unwrap();
        assert_eq!(cg.code, &[0x00, 0x00, 0x00, 0x03]);

    }
}