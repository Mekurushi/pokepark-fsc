use crate::error::{AssemblerError, AssemblerResult};

const INSTRUCTION_LENGTH: i32 = 4;
pub fn encode(operand: i16, subtype: u8, opcode: u8) -> u32 {
    let word: u32 = ((operand as u32) << 16) | ((subtype as u32) << 8) | (opcode as u32);
    word
}

pub fn encode_syscall(page: u8, func: u8, argc: u8, opcode: u8) -> u32 {
    u32::from_be_bytes([page, func, argc, opcode])
}
pub fn instruction_length() -> u32 {
    0x4 // TODO: real insn len logic
}
pub fn calculate_call_operand(current_offset: u32, target_offset: u32) -> AssemblerResult<i16> {
    let branch_offset = target_offset as i32 - (current_offset as i32 + INSTRUCTION_LENGTH);
    i16::try_from(branch_offset / INSTRUCTION_LENGTH)
        .map_err(|_| AssemblerError::OperandOutOfRange(branch_offset))
}

/// Builds a single 32-bit instruction word following the fscript slaspec.
///
/// Token layout:
///   opcode        = (0,7)
///   subtype       = (8,15)
///   sop           = (8,11)
///   indirect_load = (12,12)
///   operand       = (16,31) signed
///   syscall_page  = (26,31)
///   syscall_func  = (16,25)
///   imm           = (8,31)
///   simm          = (8,31) signed
pub struct InsnWord(u32);

impl InsnWord {
    pub fn new(opcode: u8) -> Self {
        Self(opcode as u32)
    }

    /// subtype field — bits (8,15)
    pub fn subtype(mut self, val: u8) -> Self {
        self.0 |= (val as u32) << 8;
        self
    }

    /// sop field — bits (8,11)
    pub fn sop(mut self, val: u8) -> Self {
        self.0 |= ((val & 0xf) as u32) << 8;
        self
    }

    /// indirect_load field — bit (12)
    pub fn indirect_load(mut self, val: bool) -> Self {
        self.0 |= (val as u32) << 12;
        self
    }

    /// operand field — bits (16,31) signed
    pub fn operand(mut self, val: i16) -> Self {
        self.0 |= ((val as u32) & 0xffff) << 16;
        self
    }

    /// syscall_page field — bits (26,31)
    pub fn syscall_page(mut self, val: u8) -> Self {
        self.0 |= ((val & 0x3f) as u32) << 26;
        self
    }
    /// syscall_func field — bits (16,25)
    pub fn syscall_func(mut self, val: u16) -> Self {
        self.0 |= ((val & 0x3ff) as u32) << 16;
        self
    }

    /// imm field — bits (8,31) unsigned
    pub fn imm(mut self, val: u32) -> Self {
        self.0 |= (val & 0x00ff_ffff) << 8;
        self
    }

    /// simm field — bits (8,31) signed
    pub fn simm(mut self, val: i32) -> Self {
        self.0 |= ((val as u32) & 0x00ff_ffff) << 8;
        self
    }

    /// Produces the final encoded 32-bit instruction word
    pub fn build(self) -> u32 {
        self.0
    }
}
