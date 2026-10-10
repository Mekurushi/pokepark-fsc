pub mod assembler;
pub mod assembly_unit;
pub mod binary;
pub mod encoding;
pub mod error;
pub mod string_table;
pub mod symbol_table;
mod vm_ir;

pub use assembler::Assembler;
pub use assembly_unit::AssemblyUnit;
pub use vm_ir::assemble_program;
