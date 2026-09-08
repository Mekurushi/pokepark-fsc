use crate::assembler::{Opcode, Relocation, RelocationKind};
use crate::binary::symbol_table::BinarySymbolTable;
use crate::binary::FscriptBinary;
use crate::encoding::{calculate_call_operand, InsnWord};
use crate::error::{AssemblerError, AssemblerResult};
use crate::string_table::StringTable;
use crate::symbol_table::{Scope, SymbolTable};

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

    pub fn from_binary(binary: FscriptBinary) -> AssemblerResult<(String, Self)> {
        let (script_name, code, binary_symbols, binary_strings) = binary.into_parts();
        let mut symbol_table = SymbolTable::new();
        for (name, offset) in binary_symbols.into_entries() {
            symbol_table.define(name, offset, Scope::Export)?;
        }

        Ok((
            script_name,
            Self::new(
                code,
                symbol_table,
                StringTable::from_binary(binary_strings)?,
                Vec::new(),
            ),
        ))
    }

    fn rebase(mut self, offset: u32) -> AssemblerResult<Self> {
        let code_len =
            u32::try_from(self.code.len()).map_err(|_error| AssemblerError::AddressOverflow)?;
        offset
            .checked_add(code_len)
            .ok_or(AssemblerError::AddressOverflow)?;

        if self
            .relocations
            .iter()
            .any(|relocation| relocation.code_offset.checked_add(offset).is_none())
        {
            return Err(AssemblerError::AddressOverflow);
        }

        self.symbol_table.rebase(offset)?;
        for relocation in &mut self.relocations {
            relocation.code_offset += offset;
        }

        Ok(self)
    }

    pub fn merge(mut self, other: Self) -> AssemblerResult<Self> {
        let offset =
            u32::try_from(self.code.len()).map_err(|_error| AssemblerError::AddressOverflow)?;
        let other = other.rebase(offset)?;

        self.symbol_table.merge(other.symbol_table)?;
        self.string_table.merge(&other.string_table)?;
        self.code.extend(other.code);
        self.relocations.extend(other.relocations);

        Ok(self)
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
