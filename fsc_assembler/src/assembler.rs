use crate::binary::FscriptBinary;
use crate::binary::symbol_table::BinarySymbolTable;
use crate::encoding::{InsnWord, calculate_call_operand};
use crate::error::{AssemblerError, AssemblerResult};
use crate::string_table::StringTable;
use crate::symbol_table::{Scope, SymbolTable};

// opcodes
#[repr(u8)]
pub enum Opcode {
    SC = 0x1,
    Ctrl = 0x2,
    Call = 0x3,
    Return = 0x06,
    GrowStack = 0x07,
    Jump = 0x8,
    JeqImm = 0xa,
    LoadArg = 0x0b,
    ArgMem = 0x0c,
    ShrinkStack = 0xf,
    Push = 0x10,
    PushImm = 0x11,
    PushResult = 0x12,
    LStr = 0x13,
    Alu = 0x14,
    FAlu = 0x15,
    Cmp = 0x16,
    FCmp = 0x17,
    Shift = 0x18,
    Lea = 0x19,
    Load = 0x1A,
    Store = 0x1B,
    Conv = 0x1C,
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

#[repr(u8)]
pub enum ReturnSubtype {
    Ret = 0,
    Retv = 1,
}

#[repr(u8)]
pub enum JumpSubtype {
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
pub enum ArgMemSubtype {
    StoreArg = 0,
    ArgAddi = 1,
    ArgSubi = 2,
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
pub enum CmpOp {
    Eq = 0xb,
    Neq = 0xc,
    Lt = 0xd,
    Gt = 0xe,
    Le = 0xf,
    Ge = 0x10,
}

#[repr(u16)]
pub enum FCmpOp {
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
pub enum StoreSubtype {
    Store = 0,
    Add = 1,
    Sub = 2,
}

#[repr(i16)]
pub enum LoadStoreSize {
    Byte = 1,
    Short = 2,
    Word = 4,
}

#[repr(u8)]
pub enum ConvSubtype {
    ItoF = 0,
    FtoI = 1,
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
    pub fn emit_set_arg_mode(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Ctrl as u8)
                .subtype(CtrlSubtype::SetArgMode as u8)
                .build(),
        );
    }

    // --- Call (0x3) ---

    pub fn emit_call(&mut self, symbol: &str) -> AssemblerResult<()> {
        self.push_relocation(symbol, RelocationKind::Global);
        self.emit(InsnWord::new(Opcode::Call as u8).build());
        Ok(())
    }

    // --- Return (0x6) ---
    pub fn emit_ret(&mut self, n: i16) {
        // TODO: define explicit usage of operand; Ghidra visualizes as neagtive but actual operand is positive
        let operand = n.unsigned_abs().cast_signed();
        self.emit(
            InsnWord::new(Opcode::Return as u8)
                .subtype(ReturnSubtype::Ret as u8)
                .operand(operand)
                .build(),
        );
    }
    pub fn emit_retv(&mut self, n: i16) {
        let operand = n.unsigned_abs().cast_signed();
        self.emit(
            InsnWord::new(Opcode::Return as u8)
                .subtype(ReturnSubtype::Retv as u8)
                .operand(operand)
                .build(),
        );
    }
    // --- grow_stack (0x7) ---

    pub fn emit_grow_stack(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::GrowStack as u8)
                .operand(operand)
                .build(),
        );
    }

    // --- Jump (0x8) ---

    pub fn emit_jmp(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::Jmp, label)
    }
    pub fn emit_jnz(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::Jnz, label)
    }
    pub fn emit_jz(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::Jz, label)
    }

    pub fn emit_jnz_pause(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::JnzPause, label)
    }

    pub fn emit_jz_pause(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::JzPause, label)
    }

    pub fn emit_jnz_set(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::JnzSet, label)
    }

    pub fn emit_jz_set(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::JzSet, label)
    }

    pub fn emit_jeq(&mut self, label: &str) -> AssemblerResult<()> {
        self.emit_jump(JumpSubtype::Jeq, label)
    }

    // TODO: integrate jeq_imm cleanly, don't like current impl at all; also get integer handling
    // straight
    pub fn emit_jeq_imm(&mut self, imm: i8, label: &str) -> AssemblerResult<()> {
        let function_name = self.current_function(label)?;
        self.push_relocation(label, RelocationKind::Local(function_name.clone()));
        self.emit(
            InsnWord::new(Opcode::JeqImm as u8)
                .subtype(imm.cast_unsigned())
                .build(),
        );
        Ok(())
    }

    // --- load_arg (0xb) ---
    pub fn emit_load_arg(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::LoadArg as u8)
                .operand(operand)
                .build(),
        );
    }

    // --- ArgMem (0xc) ---
    pub fn emit_store_arg(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::ArgMem as u8)
                .subtype(ArgMemSubtype::StoreArg as u8)
                .operand(operand)
                .build(),
        );
    }

    pub fn emit_arg_addi(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::ArgMem as u8)
                .subtype(ArgMemSubtype::ArgAddi as u8)
                .operand(operand)
                .build(),
        );
    }
    pub fn emit_arg_subi(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::ArgMem as u8)
                .subtype(ArgMemSubtype::ArgSubi as u8)
                .operand(operand)
                .build(),
        );
    }

    // --- shrink_stack (0xf) ---

    pub fn emit_shrink_stack(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::ShrinkStack as u8)
                .operand(operand)
                .build(),
        );
    }

    // --- push (0x10) ---

    pub fn emit_push(&mut self, operand: i16) {
        self.emit(InsnWord::new(Opcode::Push as u8).operand(operand).build());
    }

    // --- push_imm (0x11) ---
    pub fn emit_push_imm(&mut self, operand: u32) {
        self.emit(InsnWord::new(Opcode::PushImm as u8).build());
        self.emit(operand);
    }
    // --- push_result (0x12) ---
    pub fn emit_push_result(&mut self) {
        self.emit(InsnWord::new(Opcode::PushResult as u8).build());
    }

    // --- lstr (0x13) ---
    pub fn emit_lstr(&mut self, s: &str) -> AssemblerResult<()> {
        let str_offset = self.string_table.intern(s)?;
        self.emit(InsnWord::new(Opcode::LStr as u8).imm(str_offset).build());

        Ok(())
    }

    // --- alu (0x14) ---

    pub fn emit_add(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Add as i16)
                .build(),
        );
    }
    pub fn emit_sub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Sub as i16)
                .build(),
        );
    }
    pub fn emit_mul(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Mul as i16)
                .build(),
        );
    }

    pub fn emit_div(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Div as i16)
                .build(),
        );
    }

    pub fn emit_mod(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Mod as i16)
                .build(),
        );
    }

    pub fn emit_and(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::And as i16)
                .build(),
        );
    }

    pub fn emit_or(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Or as i16)
                .build(),
        );
    }

    pub fn emit_xor(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Xor as i16)
                .build(),
        );
    }

    pub fn emit_not(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Not as i16)
                .build(),
        );
    }

    pub fn emit_eq0(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Eq0 as i16)
                .build(),
        );
    }
    pub fn emit_neg(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Alu as u8)
                .operand(AluOp::Neg as i16)
                .build(),
        );
    }

    // --- FAlu (0x15) ---

    pub fn emit_fadd(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FAlu as u8)
                .operand(FaluOp::Fadd as i16)
                .build(),
        );
    }
    pub fn emit_fsub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FAlu as u8)
                .operand(FaluOp::Fsub as i16)
                .build(),
        );
    }

    pub fn emit_fmul(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FAlu as u8)
                .operand(FaluOp::Fmul as i16)
                .build(),
        );
    }

    pub fn emit_fdiv(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FAlu as u8)
                .operand(FaluOp::Fdiv as i16)
                .build(),
        );
    }

    pub fn emit_feq0(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FAlu as u8)
                .operand(FaluOp::Feq0 as i16)
                .build(),
        );
    }

    pub fn emit_fneg(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FAlu as u8)
                .operand(FaluOp::Fneg as i16)
                .build(),
        );
    }

    // --- Cmp (0x16) ---
    pub fn emit_eq(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Cmp as u8)
                .operand(CmpOp::Eq as i16)
                .build(),
        );
    }
    pub fn emit_neq(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Cmp as u8)
                .operand(CmpOp::Neq as i16)
                .build(),
        );
    }

    pub fn emit_lt(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Cmp as u8)
                .operand(CmpOp::Lt as i16)
                .build(),
        );
    }

    pub fn emit_gt(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Cmp as u8)
                .operand(CmpOp::Gt as i16)
                .build(),
        );
    }

    pub fn emit_le(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Cmp as u8)
                .operand(CmpOp::Le as i16)
                .build(),
        );
    }

    pub fn emit_ge(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Cmp as u8)
                .operand(CmpOp::Ge as i16)
                .build(),
        );
    }

    // --- FCmp (0x17) ---
    pub fn emit_feq(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FCmp as u8)
                .operand(FCmpOp::Feq as i16)
                .build(),
        );
    }

    pub fn emit_fneq(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FCmp as u8)
                .operand(FCmpOp::Fneq as i16)
                .build(),
        );
    }
    pub fn emit_flt(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FCmp as u8)
                .operand(FCmpOp::Flt as i16)
                .build(),
        );
    }

    pub fn emit_fgt(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FCmp as u8)
                .operand(FCmpOp::Fgt as i16)
                .build(),
        );
    }
    pub fn emit_fle(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FCmp as u8)
                .operand(FCmpOp::Fle as i16)
                .build(),
        );
    }

    pub fn emit_fge(&mut self) {
        self.emit(
            InsnWord::new(Opcode::FCmp as u8)
                .operand(FCmpOp::Fge as i16)
                .build(),
        );
    }

    // --- shift (0x18) ---

    pub fn emit_sl(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Shift as u8)
                .operand(ShiftOp::Sl as i16)
                .build(),
        );
    }

    pub fn emit_srm(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Shift as u8)
                .operand(ShiftOp::Srm as i16)
                .build(),
        );
    }

    pub fn emit_sr(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Shift as u8)
                .operand(ShiftOp::Sr as i16)
                .build(),
        );
    }

    // --- lea (0x19) ---

    pub fn emit_lea(&mut self, symbol: &str) {
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: symbol.to_string(),
            kind: RelocationKind::Global,
        });
        self.emit(InsnWord::new(Opcode::Lea as u8).build());
    }

    // --- Load (0x1a) ---

    pub fn emit_lb(&mut self) {
        self.emit(InsnWord::new(Opcode::Load as u8).operand(1).build());
    }
    pub fn emit_ls(&mut self) {
        self.emit(InsnWord::new(Opcode::Load as u8).operand(2).build());
    }
    pub fn emit_lw(&mut self) {
        self.emit(InsnWord::new(Opcode::Load as u8).operand(4).build());
    }
    pub fn emit_lbi(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Load as u8)
                .operand(1)
                .indirect_load(true)
                .build(),
        );
    }
    pub fn emit_lsi(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Load as u8)
                .operand(2)
                .indirect_load(true)
                .build(),
        );
    }
    pub fn emit_lwi(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Load as u8)
                .operand(4)
                .indirect_load(true)
                .build(),
        );
    }

    // --- Store (0x1b) ---

    pub fn emit_sb(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Byte as i16)
                .sop(StoreSubtype::Store as u8)
                .build(),
        );
    }
    pub fn emit_ss(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Short as i16)
                .sop(StoreSubtype::Store as u8)
                .build(),
        );
    }
    pub fn emit_sw(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Word as i16)
                .sop(StoreSubtype::Store as u8)
                .build(),
        );
    }
    pub fn emit_sbi(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Byte as i16)
                .sop(StoreSubtype::Store as u8)
                .indirect_load(true)
                .build(),
        );
    }
    pub fn emit_ssi(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Short as i16)
                .sop(StoreSubtype::Store as u8)
                .indirect_load(true)
                .build(),
        );
    }
    pub fn emit_swi(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Word as i16)
                .sop(StoreSubtype::Store as u8)
                .indirect_load(true)
                .build(),
        );
    }

    pub fn emit_sbadd(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Byte as i16)
                .sop(StoreSubtype::Add as u8)
                .build(),
        );
    }
    pub fn emit_sbiadd(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Byte as i16)
                .sop(StoreSubtype::Add as u8)
                .indirect_load(true)
                .build(),
        );
    }

    pub fn emit_sbsub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Byte as i16)
                .sop(StoreSubtype::Sub as u8)
                .build(),
        );
    }
    pub fn emit_sbisub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Byte as i16)
                .sop(StoreSubtype::Sub as u8)
                .indirect_load(true)
                .build(),
        );
    }

    pub fn emit_ssadd(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Short as i16)
                .sop(StoreSubtype::Add as u8)
                .build(),
        );
    }
    pub fn emit_ssiadd(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Short as i16)
                .sop(StoreSubtype::Add as u8)
                .indirect_load(true)
                .build(),
        );
    }

    pub fn emit_sssub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Short as i16)
                .sop(StoreSubtype::Sub as u8)
                .build(),
        );
    }
    pub fn emit_ssisub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Short as i16)
                .sop(StoreSubtype::Sub as u8)
                .indirect_load(true)
                .build(),
        );
    }

    pub fn emit_swadd(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Word as i16)
                .sop(StoreSubtype::Add as u8)
                .build(),
        );
    }
    pub fn emit_swiadd(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Word as i16)
                .sop(StoreSubtype::Add as u8)
                .indirect_load(true)
                .build(),
        );
    }

    pub fn emit_swsub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Word as i16)
                .sop(StoreSubtype::Sub as u8)
                .build(),
        );
    }
    pub fn emit_swisub(&mut self) {
        self.emit(
            InsnWord::new(Opcode::Store as u8)
                .operand(LoadStoreSize::Word as i16)
                .sop(StoreSubtype::Sub as u8)
                .indirect_load(true)
                .build(),
        );
    }

    // --- Conv (0x1c) ---
    pub fn emit_itof(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::Conv as u8)
                .operand(operand)
                .subtype(ConvSubtype::ItoF as u8)
                .build(),
        );
    }

    pub fn emit_ftoi(&mut self, operand: i16) {
        self.emit(
            InsnWord::new(Opcode::Conv as u8)
                .operand(operand)
                .subtype(ConvSubtype::FtoI as u8)
                .build(),
        );
    }
    // --- helpers ---

    fn emit(&mut self, word: u32) {
        self.code.extend_from_slice(&word.to_be_bytes());
        self.program_counter += 4; // TODO: new Instruction Lenght way
    }

    fn push_relocation(&mut self, symbol: &str, kind: RelocationKind) {
        self.relocations.push(Relocation {
            code_offset: self.program_counter,
            symbol: symbol.to_string(),
            kind,
        });
    }

    fn current_function(&self, label: &str) -> AssemblerResult<&String> {
        match self.state {
            EmitState::InFunction(ref function) => Ok(function),
            EmitState::Idle => Err(AssemblerError::LabelOutsideFunction(label.to_string())),
        }
    }

    fn emit_jump(&mut self, subtype: JumpSubtype, label: &str) -> AssemblerResult<()> {
        let function_name = self.current_function(label)?;
        self.push_relocation(label, RelocationKind::Local(function_name.clone()));
        self.emit(
            InsnWord::new(Opcode::Jump as u8)
                .subtype(subtype as u8)
                .build(),
        );
        Ok(())
    }
    // --- finalize ---
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
        for relocation in &mut self.relocations {
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
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::panic)]
    use crate::assembler::Assembler;
    use crate::binary::FscriptBinary;

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

    fn insn_at(binary: &FscriptBinary, insn_index: usize) -> [u8; 4] {
        let offset = insn_index * 4;
        binary.code[offset..offset + 4].try_into().unwrap()
    }
    // --- SC ---

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

    // --- Ctrl ---
    #[test]
    fn emit_delay() {
        let mut asm = assembler_in_function();
        asm.emit_delay(5);
        assert_eq!(last_bytes(&asm), [0x00, 0x05, 0x00, 0x02]);
    }
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

    // --- call ---
    #[test]
    fn emit_call_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("main", false).unwrap();
        asm.emit_call("sub").unwrap();
        asm.emit_ret(0);

        asm.define_function("sub", true).unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x00, 0x03]);
    }

    #[test]
    fn emit_call_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("fun1", false).unwrap();
        asm.emit_ret(0);

        asm.define_function("fun2", true).unwrap();
        asm.emit_call("fun1").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x00, 0x03]);
    }

    #[test]
    fn emit_call_undefined_symbol_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("main", false).unwrap();
        asm.emit_call("nonexistent").unwrap();
        asm.emit_ret(0);

        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_call_placeholder_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_call("foo").unwrap();
        // operand=0 placeholder
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x03]);
    }

    #[test]
    fn emit_call_self_resolves() {
        let mut asm = Assembler::new();
        asm.define_function("recursive", false).unwrap();
        asm.emit_call("recursive").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0xff, 0xff, 0x00, 0x03]);
    }

    #[test]
    fn emit_call_multiple_resolve_independently() {
        let mut asm = Assembler::new();
        asm.define_function("main", false).unwrap();
        asm.emit_call("a").unwrap();
        asm.emit_call("b").unwrap();
        asm.emit_ret(0);

        asm.define_function("a", true).unwrap();
        asm.emit_ret(0);

        asm.define_function("b", true).unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x02, 0x00, 0x03]);
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0x00, 0x02, 0x00, 0x03]);
    }
    // --- Return ---
    #[test]
    fn emit_ret_without_stack() {
        let mut asm = assembler_in_function();
        asm.emit_ret(0);

        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x6]);
    }
    #[test]
    fn emit_retv_without_stack() {
        let mut asm = assembler_in_function();
        asm.emit_retv(0);

        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x01, 0x06]);
    }
    #[test]
    fn emit_ret_with_stack() {
        let mut asm = assembler_in_function();
        asm.emit_ret(3);

        assert_eq!(last_bytes(&asm), [0x00, 0x03, 0x00, 0x6]);
    }
    #[test]
    fn emit_retv_with_stack() {
        let mut asm = assembler_in_function();
        asm.emit_retv(2);

        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x01, 0x06]);
    }
    // --- grow_stack ---
    #[test]
    fn emit_grow_stack() {
        let mut asm = assembler_in_function();
        asm.emit_grow_stack(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x07]);
    }

    // --- Jump ---
    #[test]
    fn emit_jmp_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jmp("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x00, 0x08]);
    }
    #[test]
    fn emit_jz_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jz("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x02, 0x08]);
    }

    #[test]
    fn emit_jnz_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jnz("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x01, 0x08]);
    }

    #[test]
    fn emit_jnz_pause_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jnz_pause("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x03, 0x08]);
    }

    #[test]
    fn emit_jz_pause_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jz_pause("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x04, 0x08]);
    }

    #[test]
    fn emit_jnz_set_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jnz_set("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x05, 0x08]);
    }

    #[test]
    fn emit_jz_set_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jz_set("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x06, 0x08]);
    }

    #[test]
    fn emit_jeq_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jeq("end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x07, 0x08]);
    }

    #[test]
    fn emit_jeq_imm_resolves_forward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jeq_imm(0x2, "end").unwrap();
        asm.emit_push(1);
        asm.define_label("end").unwrap();
        asm.emit_ret(0);

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 0).as_slice(), &[0x00, 0x01, 0x02, 0x0a]);
    }

    #[test]
    fn emit_jmp_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jmp("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x00, 0x08]);
    }

    #[test]
    fn emit_jnz_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jnz("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x01, 0x08]);
    }

    #[test]
    fn emit_jz_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jz("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x02, 0x08]);
    }

    #[test]
    fn emit_jnz_pause_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jnz_pause("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x03, 0x08]);
    }

    #[test]
    fn emit_jz_pause_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jz_pause("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x04, 0x08]);
    }

    #[test]
    fn emit_jnz_set_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jnz_set("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x05, 0x08]);
    }

    #[test]
    fn emit_jz_set_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jz_set("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x06, 0x08]);
    }

    #[test]
    fn emit_jeq_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jeq("loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x07, 0x08]);
    }

    #[test]
    fn emit_jeq_imm_resolves_backward() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.define_label("loop").unwrap();
        asm.emit_push(1);
        asm.emit_jeq_imm(0x2, "loop").unwrap();

        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1).as_slice(), &[0xff, 0xfe, 0x02, 0x0a]);
    }

    #[test]
    fn emit_jmp_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jmp("end").is_err());
    }

    #[test]
    fn emit_jnz_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jnz("end").is_err());
    }

    #[test]
    fn emit_jz_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jz("end").is_err());
    }

    #[test]
    fn emit_jnz_pause_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jnz_pause("end").is_err());
    }

    #[test]
    fn emit_jz_pause_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jz_pause("end").is_err());
    }

    #[test]
    fn emit_jnz_set_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jnz_set("end").is_err());
    }

    #[test]
    fn emit_jz_set_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jz_set("end").is_err());
    }

    #[test]
    fn emit_jeq_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jeq("end").is_err());
    }

    #[test]
    fn emit_jeq_imm_outside_function_returns_error() {
        let mut asm = Assembler::new();
        assert!(asm.emit_jeq_imm(0x2, "end").is_err());
    }

    #[test]
    fn emit_jmp_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jmp("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jnz_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jnz("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jz_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jz("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jnz_pause_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jnz_pause("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jz_pause_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jz_pause("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jnz_set_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jnz_set("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jz_set_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jz_set("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jeq_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jeq("nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    #[test]
    fn emit_jeq_imm_undefined_label_returns_error() {
        let mut asm = Assembler::new();
        asm.define_function("test", false).unwrap();
        asm.emit_jeq_imm(0x2, "nonexistent").unwrap();
        asm.emit_ret(0);
        assert!(asm.finalize("test".to_string()).is_err());
    }

    // --- load_arg ---
    #[test]
    fn emit_load_arg_with_operand() {
        let mut asm = assembler_in_function();
        asm.emit_load_arg(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x0b]);
    }

    #[test]
    fn emit_load_arg_without_operand() {
        let mut asm = assembler_in_function();
        asm.emit_load_arg(0);
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x0b]);
    }

    // --- ArgMem ---
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

    // --- shrink_stack ---

    #[test]
    fn emit_shrink_stack() {
        let mut asm = assembler_in_function();
        asm.emit_shrink_stack(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x0f]);
    }

    // --- push ---

    #[test]
    fn emit_push_1() {
        let mut asm = assembler_in_function();
        asm.emit_push(1);

        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x10]);
    }

    #[test]
    fn emit_push_negative1() {
        let mut asm = assembler_in_function();
        asm.emit_push(-1);

        assert_eq!(last_bytes(&asm), [0xff, 0xff, 0x00, 0x10]);
    }

    #[test]
    fn emit_push_100() {
        let mut asm = assembler_in_function();
        asm.emit_push(100);

        assert_eq!(last_bytes(&asm), [0x00, 0x64, 0x00, 0x10]);
    }

    // --- push_imm ---

    #[test]
    fn emit_push_imm() {
        let mut asm = assembler_in_function();
        asm.emit_push_imm(0x3f80_8000_u32);
        assert_eq!(&asm.code[0..4], &[0x00, 0x00, 0x00, 0x11]);
        assert_eq!(&asm.code[4..8], &[0x3f, 0x80, 0x80, 0x00]);
    }

    // --- push_result ---
    #[test]
    fn emit_push_result() {
        let mut asm = assembler_in_function();
        asm.emit_push_result();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x12]);
    }

    // --- lstr ---

    #[test]
    fn emit_lstr_first_string() {
        let mut asm = assembler_in_function();
        asm.emit_lstr("hello").unwrap();
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x13]);
    }

    #[test]
    fn emit_lstr_second_string() {
        let mut asm = assembler_in_function();
        asm.emit_lstr("hello").unwrap(); // 5 bytes + null terminator
        asm.emit_lstr("hello2").unwrap();
        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1), [0x00, 0x00, 0x06, 0x13]);
    }

    #[test]
    fn emit_lstr_same_string_returns_same_imm() {
        let mut asm = assembler_in_function();
        asm.emit_lstr("hello").unwrap(); // 5 bytes + null terminator
        asm.emit_lstr("hello").unwrap();
        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 1), [0x00, 0x00, 0x00, 0x13]);
    }

    #[test]
    fn emit_lstr_third_string() {
        let mut asm = assembler_in_function();
        asm.emit_lstr("hello").unwrap(); // 5 bytes + null terminator
        asm.emit_lstr("hello2").unwrap();
        asm.emit_lstr("hello3").unwrap();
        let binary = asm.finalize("test".to_string()).unwrap();
        assert_eq!(insn_at(&binary, 2), [0x00, 0x00, 0x0d, 0x13]);
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
    fn emit_eq0() {
        let mut asm = assembler_in_function();
        asm.emit_eq0();
        assert_eq!(last_bytes(&asm), [0x00, 0x09, 0x00, 0x14]);
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

    // --- Compare ---

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

    // --- FCompare ---

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

    // --- Load ---

    #[test]
    fn emit_lb() {
        let mut asm = assembler_in_function();
        asm.emit_lb();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x1a]);
    }
    #[test]
    fn emit_ls() {
        let mut asm = assembler_in_function();
        asm.emit_ls();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x00, 0x1a]);
    }

    #[test]
    fn emit_lw() {
        let mut asm = assembler_in_function();
        asm.emit_lw();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x00, 0x1a]);
    }

    #[test]
    fn emit_lbi_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_lbi();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x10, 0x1a]);
    }

    #[test]
    fn emit_lsi_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_lsi();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x10, 0x1a]);
    }

    #[test]
    fn emit_lwi_bytes() {
        let mut asm = assembler_in_function();
        asm.emit_lwi();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x10, 0x1a]);
    }

    // --- Store ---

    #[test]
    fn emit_sb() {
        let mut asm = assembler_in_function();
        asm.emit_sb();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x1b]);
    }
    #[test]
    fn emit_ss() {
        let mut asm = assembler_in_function();
        asm.emit_ss();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x00, 0x1b]);
    }

    #[test]
    fn emit_sw() {
        let mut asm = assembler_in_function();
        asm.emit_sw();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x00, 0x1b]);
    }

    #[test]
    fn emit_sbi() {
        let mut asm = assembler_in_function();
        asm.emit_sbi();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x10, 0x1b]);
    }

    #[test]
    fn emit_ssi() {
        let mut asm = assembler_in_function();
        asm.emit_ssi();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x10, 0x1b]);
    }

    #[test]
    fn emit_swi() {
        let mut asm = assembler_in_function();
        asm.emit_swi();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x10, 0x1b]);
    }

    #[test]
    fn emit_sbadd() {
        let mut asm = assembler_in_function();
        asm.emit_sbadd();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x01, 0x1b]);
    }
    #[test]
    fn emit_sbiadd() {
        let mut asm = assembler_in_function();
        asm.emit_sbiadd();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x11, 0x1b]);
    }

    #[test]
    fn emit_sbsub() {
        let mut asm = assembler_in_function();
        asm.emit_sbsub();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x02, 0x1b]);
    }
    #[test]
    fn emit_sbisub() {
        let mut asm = assembler_in_function();
        asm.emit_sbisub();
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x12, 0x1b]);
    }

    #[test]
    fn emit_ssadd() {
        let mut asm = assembler_in_function();
        asm.emit_ssadd();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x01, 0x1b]);
    }
    #[test]
    fn emit_ssiadd() {
        let mut asm = assembler_in_function();
        asm.emit_ssiadd();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x11, 0x1b]);
    }

    #[test]
    fn emit_sssub() {
        let mut asm = assembler_in_function();
        asm.emit_sssub();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x02, 0x1b]);
    }
    #[test]
    fn emit_ssisub() {
        let mut asm = assembler_in_function();
        asm.emit_ssisub();
        assert_eq!(last_bytes(&asm), [0x00, 0x02, 0x12, 0x1b]);
    }

    #[test]
    fn emit_swadd() {
        let mut asm = assembler_in_function();
        asm.emit_swadd();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x01, 0x1b]);
    }
    #[test]
    fn emit_swiadd() {
        let mut asm = assembler_in_function();
        asm.emit_swiadd();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x11, 0x1b]);
    }

    #[test]
    fn emit_swsub() {
        let mut asm = assembler_in_function();
        asm.emit_swsub();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x02, 0x1b]);
    }
    #[test]
    fn emit_swisub() {
        let mut asm = assembler_in_function();
        asm.emit_swisub();
        assert_eq!(last_bytes(&asm), [0x00, 0x04, 0x12, 0x1b]);
    }

    // --- Conv ---

    #[test]
    fn emit_itof_1() {
        let mut asm = assembler_in_function();
        asm.emit_itof(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x00, 0x1c]);
    }

    #[test]
    fn emit_itof_0() {
        let mut asm = assembler_in_function();
        asm.emit_itof(0);
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x00, 0x1c]);
    }

    #[test]
    fn emit_ftoi_1() {
        let mut asm = assembler_in_function();
        asm.emit_ftoi(1);
        assert_eq!(last_bytes(&asm), [0x00, 0x01, 0x01, 0x1c]);
    }

    #[test]
    fn emit_ftoi_0() {
        let mut asm = assembler_in_function();
        asm.emit_ftoi(0);
        assert_eq!(last_bytes(&asm), [0x00, 0x00, 0x01, 0x1c]);
    }
}
