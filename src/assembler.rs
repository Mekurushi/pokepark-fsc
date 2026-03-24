use crate::ast::{Instruction, Program, Statement};
use crate::binary::string_table::BinaryStringTable;
use crate::binary::symbol_table::BinarySymbolTable;
use crate::binary::FscriptBinary;
use crate::encoding::{calculate_call_operand, encode, instruction_length};
use crate::error::AssemblerResult;
use crate::symbol_table::{Scope, SymbolResolver, SymbolTable};

// opcodes
#[repr(u8)]
pub enum Opcode {
    Call = 0x3,
    Ret = 0x06,
    GrowStack = 0x07,
    Jmp = 0x8,
LoadArg = 0x0b,
    Push = 0x10,
    Alu = 0x14,
}
#[repr(u16)]
pub enum AluOp {
    Add = 0,
    Sub = 1,
}

#[repr(u8)]
pub enum JmpOp {
    Jmp = 0,
    Jnz = 1,
}


#[repr(u8)]
pub enum RetOp {
    Ret = 0,
    Retv = 1,
}

pub struct Assembler;

impl Assembler {
    pub fn assemble(program: &Program) -> AssemblerResult<Assembly> {
        let symbol_table = Self::build_symbol_table(program)?;
        let code = Self::emit(program, &symbol_table)?;
        Ok(Assembly { code, symbol_table })
    }

    fn build_symbol_table(program: &Program) -> AssemblerResult<SymbolTable> {
        let mut lc = 0u32;
        let mut symbol_table = SymbolTable::new();
        for function in &program.functions {
            let scope = if function.private {
                Scope::Private
            } else {
                Scope::Export
            };

            symbol_table.define(function.name.clone(), lc, scope)?;
            for stmt in &function.body {
                match stmt {
                    Statement::Label(name) => {
                        symbol_table.define_local(&function.name, name.clone(), lc);
                    }
                    Statement::Instruction(ins) => lc += instruction_length(ins),
                }
            }
        }
        Ok(symbol_table)
    }
    fn emit(program: &Program, symbol_table: &SymbolTable) -> AssemblerResult<Vec<u8>> {
        let mut out = Vec::new();
        for function in &program.functions {
            let resolver = SymbolResolver::new(symbol_table, &function.name);
            for stmt in &function.body {
                if let Statement::Instruction(ins) = stmt {
                    let offset = out.len() as u32;
                    let word = Self::emit_instruction(ins, offset,&resolver)?;
                    out.extend_from_slice(&word.to_be_bytes());
                }
            }
        }

        Ok(out)
    }
    fn emit_instruction(
        instruction: &Instruction,
        offset: u32,
        resolver: &SymbolResolver,
    ) -> AssemblerResult<u32> {
        let word = match instruction {
            Instruction::GrowStack(n) => encode(*n, 0, Opcode::GrowStack as u8),

            Instruction::LoadArg(n) => encode(*n, 0, Opcode::LoadArg as u8),

            Instruction::Add => encode(AluOp::Add as i16, 0, Opcode::Alu as u8),
            Instruction::Sub => encode(AluOp::Sub as i16, 0, Opcode::Alu as u8),
            Instruction::Push(n) => encode(*n, 0, Opcode::Push as u8),
            Instruction::Call(symbol) => {
                let target = resolver.resolve_global(symbol)?;
                let operand = calculate_call_operand(offset, target.offset)?;
                encode(operand, 0, Opcode::Call as u8)
            }
            Instruction::Jmp(label) => {
                let target = resolver.resolve_local(label)?;
                let operand = calculate_call_operand(offset, target.offset)?;
                encode(operand, JmpOp::Jmp as u8, Opcode::Jmp as u8)
            }

            Instruction::Retv(n) => {
                // Ghidra visualizes as neagtive but actual operand is positive TODO: define
                // explicit
                let operand = n.unsigned_abs();
                encode(operand as i16, RetOp::Retv as u8, Opcode::Ret as u8)
            }
            Instruction::Ret(n) => {
                let operand = n.unsigned_abs();
                encode(operand as i16, RetOp::Ret as u8, Opcode::Ret as u8)
            }
        };
        Ok(word)
    }
}

pub struct Assembly {
    pub code: Vec<u8>,
    pub symbol_table: SymbolTable,
}

impl Assembly {
    pub fn into_binary(self, script_name: String) -> AssemblerResult<FscriptBinary> {
        let mut symbol_table = BinarySymbolTable::new();
        for symbol in self.symbol_table.exports() {
            symbol_table.add(symbol.name.clone(), symbol.offset);
        }

        Ok(FscriptBinary::new(
            script_name,
            self.code.clone(),
            symbol_table,
            BinaryStringTable::new(), // TODO: string fill logic replacement
        ))
    }
}
