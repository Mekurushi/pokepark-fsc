use crate::error::{AssemblerError, AssemblerResult};

const INSTRUCTION_LENGTH: i32 = 4;
pub fn encode(operand: i16, subtype: u8, opcode: u8) -> u32{
    let word: u32 = ((operand as u32) << 16)
        | ((subtype as u32)         <<  8)
        |  (opcode  as u32);
    word
}
pub fn instruction_length() -> u32{
    0x4 // TODO: real insn len logic
}
pub fn calculate_call_operand(current_offset: u32, target_offset: u32) -> AssemblerResult<i16>{
    let branch_offset = target_offset as i32 - (current_offset as i32 + INSTRUCTION_LENGTH);
    i16::try_from(branch_offset / INSTRUCTION_LENGTH)
        .map_err(|_| AssemblerError::OperandOutOfRange(branch_offset))
}