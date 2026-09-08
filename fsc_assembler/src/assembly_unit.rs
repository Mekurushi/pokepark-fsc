use crate::assembler::{Opcode, Relocation, RelocationKind};
use crate::binary::FscriptBinary;
use crate::binary::symbol_table::BinarySymbolTable;
use crate::encoding::{InsnWord, calculate_call_operand};
use crate::error::{AssemblerError, AssemblerResult};
use crate::string_table::StringTable;
use crate::symbol_table::SymbolTable;

pub struct AssemblyUnit {
    pub(crate) code: Vec<u8>,
    pub(crate) symbol_table: SymbolTable,
    pub(crate) string_table: StringTable,
    pub(crate) relocations: Vec<Relocation>,
}

impl AssemblyUnit {
    pub(crate) fn new(
        code: Vec<u8>,
        symbol_table: SymbolTable,
        string_table: StringTable,
        relocations: Vec<Relocation>,
    ) -> Self {
        Self {
            code,
            symbol_table,
            string_table,
            relocations,
        }
    }

    pub fn into_binary(mut self, script_name: String) -> AssemblerResult<FscriptBinary> {
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
            let idx = relocation.code_offset as usize;
            match &relocation.kind {
                RelocationKind::Global => {
                    let target = self.symbol_table.resolve_global(&relocation.symbol)?;
                    let operand = calculate_call_operand(relocation.code_offset, target.offset)?;
                    let operand_bytes = operand.to_be_bytes();
                    self.code[idx] = operand_bytes[0];
                    self.code[idx + 1] = operand_bytes[1];
                }
                RelocationKind::Local(function) => {
                    let target = self
                        .symbol_table
                        .resolve_local(function, &relocation.symbol)?;
                    let operand = calculate_call_operand(relocation.code_offset, target.offset)?;
                    let operand_bytes = operand.to_be_bytes();
                    self.code[idx] = operand_bytes[0];
                    self.code[idx + 1] = operand_bytes[1];
                }
                RelocationKind::String => {
                    let string_offset =
                        self.string_table
                            .lookup(&relocation.symbol)
                            .ok_or_else(|| {
                                AssemblerError::UndefinedString(relocation.symbol.clone())
                            })?;
                    let instruction = InsnWord::new(Opcode::LStr as u8)
                        .imm(string_offset)
                        .build()
                        .to_be_bytes();
                    self.code[idx..idx + 4].copy_from_slice(&instruction);
                }
            }
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
