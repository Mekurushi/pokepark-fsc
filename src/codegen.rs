use crate::ast::{Function, Instruction, Program};
use crate::error::{ Result};
//TODO: all Instruction data
//TODO: fsb metadata for file; header; symbol table; pointers ...
//TODO: call support symbol table, 2-pass ...
pub const OP_GROW_STACK:  u8 = 0x07;
pub const OP_LOAD_ARG:    u8 = 0x0b;
pub const OP_ALU:         u8 = 0x14;
pub const OP_RETV:        u8 = 0x06;

pub const ALU_ADD: u16 = 0;


pub struct Codegen {
    pub output: Vec<u8>,
}

impl Codegen {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }

    fn emit_insn(&mut self, opcode: u8, subtype: u8, operand: i16) {
        let word: u32 = ((operand as u32) << 16)
            | ((subtype as u32)         <<  8)
            |  (opcode  as u32);
        self.output.extend_from_slice(&word.to_be_bytes());
    }

    pub fn emit_program(&mut self, program: &Program) -> Result<()> {
        for function in &program.functions {
            self.emit_function(function)?;
        }
        Ok(())
    }

    fn emit_function(&mut self, function: &Function) -> Result<()> {
        for instruction in &function.body {
            self.emit_instruction(instruction)?;
        }
        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> Result<()> {
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
        }
        Ok(())
    }
}