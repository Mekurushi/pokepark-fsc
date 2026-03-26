use crate::binary::symbol_table::BinarySymbolTable;
use crate::binary::FscriptBinary;
use crate::encoding::{calculate_call_operand, encode, encode_syscall};
use crate::error::{AssemblerError, AssemblerResult};
use crate::string_table::StringTable;
use crate::symbol_table::{Scope, SymbolTable};

// opcodes
#[repr(u8)]
pub enum Opcode {
    SC = 0x1,
    Call = 0x3,
    Ret = 0x06,
    GrowStack = 0x07,
    Jmp = 0x8,
    LoadArg = 0x0b,
    StoreArg = 0x0c,
    Push = 0x10,
    PushResult = 0x12,
    LStr = 0x13,
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

enum RelocationKind {
    Global,
    Local(String)
}

struct Relocation {
    code_offset: u32,
    symbol: String,
    kind: RelocationKind,
}

pub struct Assembler {
    code: Vec<u8>,
    symbol_table: SymbolTable,
    string_table: StringTable,
    relocations: Vec<Relocation>,
    program_counter: u32,
    state: EmitState,
}
enum EmitState {
    Idle,
    InFunction(String),
}
impl Assembler {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            symbol_table: SymbolTable::new(),
            string_table: StringTable::new(),
            program_counter: 0,
            relocations: Vec::new(),
           state: EmitState::Idle,
        }
    }
    // symbol definition
    pub fn define_function(&mut self, name: &str, private: bool) -> AssemblerResult<()> {
        let scope = if private { Scope::Private } else { Scope::Export };
        self.symbol_table.define(name.to_string(), self.program_counter, scope)?;
        self.state = EmitState::InFunction(name.to_string());
        Ok(())
    }
    pub fn define_label(&mut self, name: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(name.to_string()))?
        };
        self.symbol_table.define_local(function, name.to_string(), self.program_counter);
        Ok(())
    }

    // instruction emission
    pub fn emit_syscall(&mut self, argc: u8,page: u8, func: u8) {
        self.emit(encode_syscall(page,func,argc,Opcode::SC as u8));
    }

    pub fn emit_grow_stack(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::GrowStack as u8));
    }
    pub fn emit_load_arg(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::LoadArg as u8));
    }
    pub fn emit_store_arg(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::StoreArg as u8));
    }
    pub fn emit_push(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::Push as u8));
        }
    pub fn emit_push_result(&mut self) {
        self.emit(encode(0, 0, Opcode::PushResult as u8));
    }
    pub fn emit_add(&mut self) {
        self.emit(encode(AluOp::Add as i16, 0, Opcode::Alu as u8));

    }
    pub fn emit_sub(&mut self) {
        self.emit(encode(AluOp::Sub as i16, 0, Opcode::Alu as u8));

    }
    pub fn emit_lstr(&mut self, s: &str) -> AssemblerResult<()> {
        let str_offset = self.string_table.intern(s)?;
        self.emit(encode(0, str_offset as u8, Opcode::LStr as u8));
        Ok(())
    }
    pub fn emit_call(&mut self, symbol: &str) -> AssemblerResult<()> {
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: symbol.to_string(),
            kind: RelocationKind::Global,
        });
        self.emit(encode(0, 0, Opcode::Call as u8));
        Ok(()) }
    pub fn emit_jmp(&mut self, label: &str) -> AssemblerResult<()>  {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpOp::Jmp as u8, Opcode::Jmp as u8));
        Ok(()) }
    pub fn emit_ret(&mut self, n: i16) {
        // Ghidra visualizes as neagtive but actual operand is positive TODO: define
        // explicit
        self.emit(encode(n.unsigned_abs() as i16, RetOp::Ret as u8, Opcode::Ret as u8));
    }
    pub fn emit_retv(&mut self, n: i16) {
        self.emit(encode(n.unsigned_abs() as i16, RetOp::Retv as u8, Opcode::Ret as u8));
    }

    fn emit(&mut self, word: u32) {
        self.code.extend_from_slice(&word.to_be_bytes());
        self.program_counter += 4; // TODO: new Instruction Lenght way
    }

    // finalize
    pub fn finalize(mut self, script_name: String) -> AssemblerResult<FscriptBinary> {
        self.state = EmitState::Idle;
        self.apply_relocations()?;
        let binary_symbol_table = self.build_binary_symbol_table();

        Ok(FscriptBinary::new(
            script_name,
            self.code,
            binary_symbol_table,
            self.string_table.into_binary(),
        ))
    }
    fn apply_relocations(&mut self) -> AssemblerResult<()> {
        for relocation in self.relocations.iter_mut(){
            let target = match &relocation.kind {
                RelocationKind::Global => {
                    self.symbol_table.resolve_global(&relocation.symbol)?
                }
                RelocationKind::Local(function) => {
                    self.symbol_table.resolve_local(&function,&relocation.symbol)?
                }
            };
            let operand = calculate_call_operand(
                relocation.code_offset,
                target.offset
            )?;
            let operand_bytes = operand.to_be_bytes();
            let idx = relocation.code_offset as usize;
            self.code[idx]     = operand_bytes[0];
            self.code[idx + 1] = operand_bytes[1];
        }
        Ok(())
    }


    fn build_binary_symbol_table(&self) -> BinarySymbolTable {
        let mut table = BinarySymbolTable::new();
        for symbol in self.symbol_table.exports() {
            table.add(symbol.name.clone(), symbol.offset);
        }
        table
    }
}

#[cfg(test)]
mod emit_tests {
    use crate::assembler::Assembler;

    // --- helpers ---
    fn assembler_in_function() -> Assembler {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm
    }

    fn last_bytes(asm: &Assembler) -> [u8; 4] {
        let code = &asm.code;
        let idx = code.len() - 4;
        code[idx..idx + 4].try_into().unwrap()
    }

    // --- push ---

    #[test]
    fn emit_push_1_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_push(1);

        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x10]);
    }

    // --- syscalls ---
    #[test]
    fn emit_syscall_0x0_0x10_argc1_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_syscall(1, 0x0, 0x10);
        // SC1 0x0:0x10
        assert_eq!(last_bytes(&asm), [0x00, 0x10, 0x01, 0x01]);
    }

    #[test]
    fn emit_syscall_0x0_0x15_argc3_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_syscall(3, 0x0, 0x15);
        // SC3 0x0:0x15
        assert_eq!(last_bytes(&asm), [0x00, 0x15, 0x03, 0x01]);
    }

    // --- push_result ---
    #[test]
    fn emit_push_result() {
        let mut asm = assembler_in_function();
        asm.emit_push_result();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x12]);
    }


    // --- store_arg ---
    #[test]
    fn emit_store_arg() {
        let mut asm = assembler_in_function();
        asm.emit_store_arg(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x0c]);
    }
}