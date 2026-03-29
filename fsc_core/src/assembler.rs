use crate::binary::symbol_table::BinarySymbolTable;
use crate::binary::FscriptBinary;
use crate::encoding::{calculate_call_operand, encode, InsnWord};
use crate::error::{AssemblerError, AssemblerResult};
use crate::string_table::StringTable;
use crate::symbol_table::{Scope, SymbolTable};

// opcodes
#[repr(u8)]
pub enum Opcode {
    SC = 0x1,
    Ctrl = 0x2,
    Call = 0x3,
    Ret = 0x06,
    GrowStack = 0x07,
    Jmp = 0x8,
    JeqImm = 0xa,
    LoadArg = 0x0b,
    ArgAlu = 0x0c,
    ShrinkStack = 0xf,
    Push = 0x10,
    PushImm = 0x11,
    PushResult = 0x12,
    LStr = 0x13,
    Alu = 0x14,
    Falu = 0x15,
    Eq = 0x16,
    Feq = 0x17,
    Shift = 0x18,
    Lea = 0x19,
}

#[repr(u8)]
pub enum CtrlSubtype {
    Delay = 0,
    Exit1 = 1,
    Exit2 = 2,
    DelayLoad = 3,
    DelayNeq0 = 4,
    SetArgMode = 5,
}

#[repr(u16)]
pub enum ShiftOp {
    Sl = 0,
    Srm = 1,
    Sr = 2,
}

#[repr(u16)]
pub enum FaluOp {
    Fadd = 0,
    Fsub = 1,
    Fmul = 2,
    Fdiv = 3,
    Feq0 = 9,
    Fneg = 10,
}

#[repr(u16)]
pub enum EqOp {
    Eq = 0xb,
    Neq = 0xc,
    Lt = 0xd,
    Gt = 0xe,
    Le = 0xf,
    Ge = 0x10,
}

#[repr(u16)]
pub enum FeqOp {
    Feq = 0xb,
    Fneq = 0xc,
    Flt = 0xd,
    Fgt = 0xe,
    Fle = 0xf,
    Fge = 0x10,
}

#[repr(u16)]
pub enum AluOp {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
    Mod = 4,
    And = 5,
    Or = 6,
    Xor = 7,
    Not = 8,
    Eq0 = 9,
    Neg = 10,
}

#[repr(u8)]
pub enum ArgType {
    StoreArg = 0,
    ArgAddi = 1,
    ArgSubi = 2,
}

#[repr(u8)]
pub enum JmpType {
    Jmp = 0,
    Jnz = 1,
    Jz = 2,
    JnzPause = 3,
    JzPause = 4,
    JnzSet = 5,
    JzSet = 6,
    Jeq = 7,
}

#[repr(u8)]
pub enum RetOp {
    Ret = 0,
    Retv = 1,
}

enum RelocationKind {
    Global,
    Local(String),
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
impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
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
        let scope = if private {
            Scope::Private
        } else {
            Scope::Export
        };
        self.symbol_table
            .define(name.to_string(), self.program_counter, scope)?;
        self.state = EmitState::InFunction(name.to_string());
        Ok(())
    }
    pub fn define_label(&mut self, name: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(name.to_string()))?,
        };
        self.symbol_table
            .define_local(function, name.to_string(), self.program_counter);
        Ok(())
    }

    // instruction emission

    // --- SC (0x1) ---
    pub fn emit_syscall(&mut self, subtype: u8, page: u8, func: u16) {
        self.emit(
            InsnWord::new(Opcode::SC as u8)
                .subtype(subtype)
                .syscall_page(page)
                .syscall_func(func)
                .build(),
        );
    }

    // --- Ctrl (0x2) ---

    pub fn emit_delay(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::Ctrl as u8)
                .subtype(CtrlSubtype::Delay as u8)
                .operand(operand)
                .build(),
        );
    }

    pub fn emit_exit_1(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Ctrl as u8)
                .subtype(CtrlSubtype::Exit1 as u8)
                .build(),
        );
    }
    pub fn emit_exit_2(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Ctrl as u8)
                .subtype(CtrlSubtype::Exit2 as u8)
                .build(),
        );
    }

    pub fn emit_delay_load(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Ctrl as u8)
                .subtype(CtrlSubtype::DelayLoad as u8)
                .build(),
        );
    }
    pub fn emit_delay_neq0(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Ctrl as u8)
                .subtype(CtrlSubtype::DelayNeq0 as u8)
                .build(),
        );
    }

    pub fn emit_grow_stack(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::GrowStack as u8));
    }

    pub fn emit_shrink_stack(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::ShrinkStack as u8));
    }
    pub fn emit_load_arg(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::LoadArg as u8));
    }
    pub fn emit_store_arg(&mut self, n: i16) {
        self.emit(encode(n, ArgType::StoreArg as u8, Opcode::ArgAlu as u8));
    }
    pub fn emit_arg_addi(&mut self, n: i16) {
        self.emit(encode(n, ArgType::ArgAddi as u8, Opcode::ArgAlu as u8));
    }
    pub fn emit_arg_subi(&mut self, n: i16) {
        self.emit(encode(n, ArgType::ArgSubi as u8, Opcode::ArgAlu as u8));
    }

    pub fn emit_set_arg_mode(&mut self) {
        self.emit(encode(0, CtrlSubtype::SetArgMode as u8, Opcode::Ctrl as u8));
    }
    pub fn emit_push(&mut self, n: i16) {
        self.emit(encode(n, 0, Opcode::Push as u8));
    }
    pub fn emit_push_imm(&mut self, n: i32) {
        self.emit(encode(0, 0, Opcode::PushImm as u8));
        self.emit(n as u32);
    }
    pub fn emit_push_result(&mut self) {
        self.emit(encode(0, 0, Opcode::PushResult as u8));
    }
    pub fn emit_eq0(&mut self) {
        self.emit(encode(AluOp::Eq0 as i16, 0, Opcode::Alu as u8));
    }
    pub fn emit_eq(&mut self) {
        self.emit(encode(EqOp::Eq as i16, 0, Opcode::Eq as u8));
    }

    pub fn emit_neq(&mut self) {
        self.emit(encode(EqOp::Neq as i16, 0, Opcode::Eq as u8));
    }

    pub fn emit_lt(&mut self) {
        self.emit(encode(EqOp::Lt as i16, 0, Opcode::Eq as u8));
    }

    pub fn emit_gt(&mut self) {
        self.emit(encode(EqOp::Gt as i16, 0, Opcode::Eq as u8));
    }

    pub fn emit_le(&mut self) {
        self.emit(encode(EqOp::Le as i16, 0, Opcode::Eq as u8));
    }

    pub fn emit_ge(&mut self) {
        self.emit(encode(EqOp::Ge as i16, 0, Opcode::Eq as u8));
    }

    pub fn emit_sl(&mut self) {
        self.emit(encode(ShiftOp::Sl as i16, 0, Opcode::Shift as u8));
    }

    pub fn emit_srm(&mut self) {
        self.emit(encode(ShiftOp::Srm as i16, 0, Opcode::Shift as u8));
    }

    pub fn emit_sr(&mut self) {
        self.emit(encode(ShiftOp::Sr as i16, 0, Opcode::Shift as u8));
    }

    pub fn emit_lea(&mut self, symbol: &str) {
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: symbol.to_string(),
            kind: RelocationKind::Global,
        });
        self.emit(encode(0, 0, Opcode::Lea as u8));
    }

    pub fn emit_add(&mut self) {
        self.emit(encode(AluOp::Add as i16, 0, Opcode::Alu as u8));
    }
    pub fn emit_sub(&mut self) {
        self.emit(encode(AluOp::Sub as i16, 0, Opcode::Alu as u8));
    }
    pub fn emit_mul(&mut self) {
        self.emit(encode(AluOp::Mul as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_div(&mut self) {
        self.emit(encode(AluOp::Div as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_mod(&mut self) {
        self.emit(encode(AluOp::Mod as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_and(&mut self) {
        self.emit(encode(AluOp::And as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_or(&mut self) {
        self.emit(encode(AluOp::Or as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_xor(&mut self) {
        self.emit(encode(AluOp::Xor as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_not(&mut self) {
        self.emit(encode(AluOp::Not as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_neg(&mut self) {
        self.emit(encode(AluOp::Neg as i16, 0, Opcode::Alu as u8));
    }

    pub fn emit_fadd(&mut self) {
        self.emit(encode(FaluOp::Fadd as i16, 0, Opcode::Falu as u8));
    }
    pub fn emit_fsub(&mut self) {
        self.emit(encode(FaluOp::Fsub as i16, 0, Opcode::Falu as u8));
    }

    pub fn emit_fmul(&mut self) {
        self.emit(encode(FaluOp::Fmul as i16, 0, Opcode::Falu as u8));
    }

    pub fn emit_fdiv(&mut self) {
        self.emit(encode(FaluOp::Fdiv as i16, 0, Opcode::Falu as u8));
    }

    pub fn emit_feq0(&mut self) {
        self.emit(encode(FaluOp::Feq0 as i16, 0, Opcode::Falu as u8));
    }

    pub fn emit_fneg(&mut self) {
        self.emit(encode(FaluOp::Fneg as i16, 0, Opcode::Falu as u8));
    }

    pub fn emit_feq(&mut self) {
        self.emit(encode(FeqOp::Feq as i16, 0, Opcode::Feq as u8));
    }

    pub fn emit_fneq(&mut self) {
        self.emit(encode(FeqOp::Fneq as i16, 0, Opcode::Feq as u8));
    }
    pub fn emit_flt(&mut self) {
        self.emit(encode(FeqOp::Flt as i16, 0, Opcode::Feq as u8));
    }

    pub fn emit_fgt(&mut self) {
        self.emit(encode(FeqOp::Fgt as i16, 0, Opcode::Feq as u8));
    }
    pub fn emit_fle(&mut self) {
        self.emit(encode(FeqOp::Fle as i16, 0, Opcode::Feq as u8));
    }

    pub fn emit_fge(&mut self) {
        self.emit(encode(FeqOp::Fge as i16, 0, Opcode::Feq as u8));
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
        Ok(())
    }
    pub fn emit_jmp(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::Jmp as u8, Opcode::Jmp as u8));
        Ok(())
    }

    pub fn emit_jz(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::Jz as u8, Opcode::Jmp as u8));
        Ok(())
    }

    pub fn emit_jeq(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::Jeq as u8, Opcode::Jmp as u8));
        Ok(())
    }

    // TODO: integrate jeq_imm cleanly, don't like current impl at all; also get integer handling
    // straight
    pub fn emit_jeq_imm(&mut self, imm: i8, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, imm as u8, Opcode::JeqImm as u8));
        Ok(())
    }

    pub fn emit_jnz(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::Jnz as u8, Opcode::Jmp as u8));
        Ok(())
    }

    pub fn emit_jnz_pause(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::JnzPause as u8, Opcode::Jmp as u8));
        Ok(())
    }

    pub fn emit_jnz_set(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::JnzSet as u8, Opcode::Jmp as u8));
        Ok(())
    }

    pub fn emit_jz_set(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::JzSet as u8, Opcode::Jmp as u8));
        Ok(())
    }

    pub fn emit_jz_pause(&mut self, label: &str) -> AssemblerResult<()> {
        let function = match self.state {
            EmitState::InFunction(ref function) => function,
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string()))?,
        };
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: label.to_string(),
            kind: RelocationKind::Local(function.to_string()),
        });
        self.emit(encode(0, JmpType::JzPause as u8, Opcode::Jmp as u8));
        Ok(())
    }

    pub fn emit_ret(&mut self, n: i16) {
        // Ghidra visualizes as neagtive but actual operand is positive TODO: define
        // explicit
        self.emit(encode(
            n.unsigned_abs() as i16,
            RetOp::Ret as u8,
            Opcode::Ret as u8,
        ));
    }
    pub fn emit_retv(&mut self, n: i16) {
        self.emit(encode(
            n.unsigned_abs() as i16,
            RetOp::Retv as u8,
            Opcode::Ret as u8,
        ));
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
        for relocation in self.relocations.iter_mut() {
            let target = match &relocation.kind {
                RelocationKind::Global => self.symbol_table.resolve_global(&relocation.symbol)?,
                RelocationKind::Local(function) => self
                    .symbol_table
                    .resolve_local(function, &relocation.symbol)?,
            };
            let operand = calculate_call_operand(relocation.code_offset, target.offset)?;
            let operand_bytes = operand.to_be_bytes();
            let idx = relocation.code_offset as usize;
            self.code[idx] = operand_bytes[0];
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

    // --- alu ---

    #[test]
    fn emit_add() {
        let mut asm = assembler_in_function();
        asm.emit_add();

        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x14]);
    }
    #[test]
    fn emit_sub() {
        let mut asm = assembler_in_function();
        asm.emit_sub();

        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x14]);
    }

    #[test]
    fn emit_mul() {
        let mut asm = assembler_in_function();
        asm.emit_mul();

        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x00, 0x14]);
    }

    #[test]
    fn emit_div() {
        let mut asm = assembler_in_function();
        asm.emit_div();

        assert_eq!(last_bytes(&asm), [0x00, 0x03, 0x00, 0x14]);
    }
    #[test]
    fn emit_mod() {
        let mut asm = assembler_in_function();
        asm.emit_mod();

        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x00, 0x14]);
    }

    #[test]
    fn emit_and() {
        let mut asm = assembler_in_function();
        asm.emit_and();

        assert_eq!(last_bytes(&asm), [0x00, 0x05, 0x00, 0x14]);
    }

    #[test]
    fn emit_or() {
        let mut asm = assembler_in_function();
        asm.emit_or();

        assert_eq!(last_bytes(&asm), [0x00, 0x06, 0x00, 0x14]);
    }

    #[test]
    fn emit_xor() {
        let mut asm = assembler_in_function();
        asm.emit_xor();

        assert_eq!(last_bytes(&asm), [0x00, 0x07, 0x00, 0x14]);
    }

    #[test]
    fn emit_not() {
        let mut asm = assembler_in_function();
        asm.emit_not();

        assert_eq!(last_bytes(&asm), [0x00, 0x08, 0x00, 0x14]);
    }

    #[test]
    fn emit_neg() {
        let mut asm = assembler_in_function();
        asm.emit_neg();

        assert_eq!(last_bytes(&asm), [0x00, 0x0a, 0x00, 0x14]);
    }

    // --- falu ---
    #[test]
    fn emit_fadd() {
        let mut asm = assembler_in_function();
        asm.emit_fadd();

        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x15]);
    }

    #[test]
    fn emit_fsub() {
        let mut asm = assembler_in_function();
        asm.emit_fsub();

        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x15]);
    }

    #[test]
    fn emit_fmul() {
        let mut asm = assembler_in_function();
        asm.emit_fmul();

        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x00, 0x15]);
    }

    #[test]
    fn emit_fdiv() {
        let mut asm = assembler_in_function();
        asm.emit_fdiv();

        assert_eq!(last_bytes(&asm), [0x00, 0x03, 0x00, 0x15]);
    }

    #[test]
    fn emit_feq0() {
        let mut asm = assembler_in_function();
        asm.emit_feq0();

        assert_eq!(last_bytes(&asm), [0x00, 0x09, 0x00, 0x15]);
    }

    #[test]
    fn emit_fneg() {
        let mut asm = assembler_in_function();
        asm.emit_fneg();

        assert_eq!(last_bytes(&asm), [0x00, 0x0a, 0x00, 0x15]);
    }

    #[test]
    fn emit_feq() {
        let mut asm = assembler_in_function();
        asm.emit_feq();

        assert_eq!(last_bytes(&asm), [0x00, 0x0b, 0x00, 0x17]);
    }

    #[test]
    fn emit_fneq() {
        let mut asm = assembler_in_function();
        asm.emit_fneq();

        assert_eq!(last_bytes(&asm), [0x00, 0x0c, 0x00, 0x17]);
    }

    #[test]
    fn emit_flt() {
        let mut asm = assembler_in_function();
        asm.emit_flt();

        assert_eq!(last_bytes(&asm), [0x00, 0x0d, 0x00, 0x17]);
    }

    #[test]
    fn emit_fgt() {
        let mut asm = assembler_in_function();
        asm.emit_fgt();

        assert_eq!(last_bytes(&asm), [0x00, 0x0e, 0x00, 0x17]);
    }

    #[test]
    fn emit_fle() {
        let mut asm = assembler_in_function();
        asm.emit_fle();

        assert_eq!(last_bytes(&asm), [0x00, 0x0f, 0x00, 0x17]);
    }

    #[test]
    fn emit_fge() {
        let mut asm = assembler_in_function();
        asm.emit_fge();

        assert_eq!(last_bytes(&asm), [0x00, 0x10, 0x00, 0x17]);
    }

    // --- push ---

    #[test]
    fn emit_push_1_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_push(1);

        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x10]);
    }

    // --- push_imm ---

    #[test]
    fn emit_push_imm_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_push_imm(0x3f808000i32);
        // word 1: opcode=0x11, subtype=0, operand=0
        // word 2: 0x3f808000
        assert_eq!(&asm.code[0..4], &[0x00, 0x00, 0x00, 0x11]);
        assert_eq!(&asm.code[4..8], &[0x3f, 0x80, 0x80, 0x00]);
    }

    // --- syscalls ---
    #[test]
    fn emit_sc2_0x0_0x15() {
        let mut asm = assembler_in_function();
        asm.emit_syscall(2, 0x0, 0x15);
        assert_eq!(last_bytes(&asm), [0x00, 0x15, 0x02, 0x01]);
    }

    #[test]
    fn emit_sc3_0x0_0x15() {
        let mut asm = assembler_in_function();
        asm.emit_syscall(3, 0x0, 0x15);
        assert_eq!(last_bytes(&asm), [0x00, 0x15, 0x03, 0x01]);
    }

    #[test]
    fn emit_sc2_0x0_0x3() {
        let mut asm = assembler_in_function();
        asm.emit_syscall(2, 0x0, 0x3);
        assert_eq!(last_bytes(&asm), [0x00, 0x3, 0x02, 0x01]);
    }

    #[test]
    fn emit_sc1_0x0_0x1() {
        let mut asm = assembler_in_function();
        asm.emit_syscall(1, 0x0, 0x1);
        assert_eq!(last_bytes(&asm), [0x00, 0x1, 0x01, 0x01]);
    }

    #[test]
    fn emit_sc1_0x3_0x1() {
        let mut asm = assembler_in_function();
        asm.emit_syscall(1, 0x3, 0x1);
        assert_eq!(last_bytes(&asm), [0x0c, 0x1, 0x01, 0x01]);
    }

    // --- stack ---
    #[test]
    fn emit_grow_stack() {
        let mut asm = assembler_in_function();
        asm.emit_grow_stack(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x07]);
    }

    #[test]
    fn emit_shrink_stack() {
        let mut asm = assembler_in_function();
        asm.emit_shrink_stack(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x0f]);
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

    #[test]
    fn emit_arg_addi() {
        let mut asm = assembler_in_function();
        asm.emit_arg_addi(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x01, 0x0c]);
    }

    #[test]
    fn emit_arg_subi() {
        let mut asm = assembler_in_function();
        asm.emit_arg_subi(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x02, 0x0c]);
    }

    // --- delay_load ---
    #[test]
    fn emit_delay_load() {
        let mut asm = assembler_in_function();
        asm.emit_delay_load();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x03, 0x02]);
    }

    #[test]
    fn emit_delay_neq0() {
        let mut asm = assembler_in_function();
        asm.emit_delay_neq0();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x04, 0x02]);
    }

    #[test]
    fn emit_exit1() {
        let mut asm = assembler_in_function();
        asm.emit_exit_1();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x01, 0x02]);
    }

    #[test]
    fn emit_exit2() {
        let mut asm = assembler_in_function();
        asm.emit_exit_2();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x02, 0x02]);
    }

    #[test]
    fn emit_set_arg_mode() {
        let mut asm = assembler_in_function();
        asm.emit_set_arg_mode();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x05, 0x02]);
    }

    // --- eq ---
    #[test]
    fn emit_eq0() {
        let mut asm = assembler_in_function();
        asm.emit_eq0();
        assert_eq!(last_bytes(&asm), [0x00, 0x09, 0x00, 0x14]);
    }

    #[test]
    fn emit_eq() {
        let mut asm = assembler_in_function();
        asm.emit_eq();
        assert_eq!(last_bytes(&asm), [0x00, 0x0b, 0x00, 0x16]);
    }

    #[test]
    fn emit_neq() {
        let mut asm = assembler_in_function();
        asm.emit_neq();
        assert_eq!(last_bytes(&asm), [0x00, 0x0c, 0x00, 0x16]);
    }

    #[test]
    fn emit_lt() {
        let mut asm = assembler_in_function();
        asm.emit_lt();
        assert_eq!(last_bytes(&asm), [0x00, 0x0d, 0x00, 0x16]);
    }

    #[test]
    fn emit_gt() {
        let mut asm = assembler_in_function();
        asm.emit_gt();
        assert_eq!(last_bytes(&asm), [0x00, 0x0e, 0x00, 0x16]);
    }

    #[test]
    fn emit_le() {
        let mut asm = assembler_in_function();
        asm.emit_le();
        assert_eq!(last_bytes(&asm), [0x00, 0x0f, 0x00, 0x16]);
    }

    #[test]
    fn emit_ge() {
        let mut asm = assembler_in_function();
        asm.emit_ge();
        assert_eq!(last_bytes(&asm), [0x00, 0x10, 0x00, 0x16]);
    }

    // --- Shift ---
    #[test]
    fn emit_sl() {
        let mut asm = assembler_in_function();
        asm.emit_sl();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x18]);
    }

    #[test]
    fn emit_srm() {
        let mut asm = assembler_in_function();
        asm.emit_srm();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x18]);
    }

    #[test]
    fn emit_sr() {
        let mut asm = assembler_in_function();
        asm.emit_sr();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x00, 0x18]);
    }

    // --- lea ---
    // TODO: clean implementation of lea; rethinking label symbol usage
    #[test]
    fn emit_lea() {
        let mut asm = Assembler::new();
        asm.define_function("target", false).unwrap();
        asm.emit_push(1);
        asm.emit_lea("target");

        let binary = asm.finalize("test".to_string()).unwrap();

        assert_eq!(&binary.code[4..8], &[0xff, 0xfe, 0x00, 0x19]);
    }

    #[test]
    fn emit_lea_resolves_global_forward() {
        let mut asm = Assembler::new();
        asm.define_function("main", false).unwrap();
        asm.emit_lea("data");
        asm.emit_ret(0);

        asm.define_function("data", false).unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(&binary.code[0..4], &[0x00, 0x01, 0x00, 0x19]);
    }

    // --- jmp ---
    #[test]
    fn emit_jmp_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jmp("target").unwrap();
        // [operand: 0x0000][JmpType::Jmp = 0x00][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x08]);
    }

    #[test]
    fn emit_jz_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jz("target").unwrap();
        // [operand: 0x0000][JmpType::Jz = 0x02][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x02, 0x08]);
    }

    #[test]
    fn emit_jnz_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jnz("target").unwrap();
        // [operand: 0x0000][JmpType::Jnz = 0x01][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x01, 0x08]);
    }

    #[test]
    fn emit_jnz_pause_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jnz_pause("target").unwrap();
        // [operand: 0x0000][JmpType::JnzPause = 0x03][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x03, 0x08]);
    }

    #[test]
    fn emit_jnz_set_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jnz_set("target").unwrap();
        // [operand: 0x0000][JmpType::JnzSet = 0x05][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x05, 0x08]);
    }

    #[test]
    fn emit_jz_pause_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jz_pause("target").unwrap();
        // [operand: 0x0000][JmpType::JzPause = 0x04][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x04, 0x08]);
    }

    #[test]
    fn emit_jz_set_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jz_set("target").unwrap();
        // [operand: 0x0000][JmpType::JzSet = 0x06][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x06, 0x08]);
    }

    #[test]
    fn emit_jeq_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jeq("target").unwrap();
        // [operand: 0x0000][JmpType::Jeq = 0x07][Opcode::Jmp = 0x08]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x07, 0x08]);
    }

    #[test]
    fn emit_jeq_imm_bytes() {
        let mut asm = assembler_in_function();
        asm.define_label("target").unwrap();
        asm.emit_jeq_imm(0x2, "target").unwrap();
        // [operand: 0x0000][imm = 0x02][Opcode::JeqImm = 0x0a]
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x02, 0x0a]);
    }

    #[test]
    fn emit_jmp_resolves_correctly() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();

        asm.emit_jmp("loop").unwrap(); // offset 0
        asm.emit_push(1); // offset 4
        asm.define_label("loop").unwrap(); // offset 8

        let binary = asm.finalize("test".to_string()).unwrap();

        assert_eq!(&binary.code[0..4], &[0x00, 0x01, 0x00, 0x08]);
    }
}
