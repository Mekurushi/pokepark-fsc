use crate::assembler::{Opcode, Relocation, RelocationKind};
use crate::binary::FscriptBinary;
use crate::binary::symbol_table::BinarySymbolTable;
use crate::encoding::{InsnWord, calculate_call_operand, encode_relative_jump};
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

    pub fn from_binary(binary: FscriptBinary) -> AssemblerResult<(String, Self)> {
        let (script_name, code, binary_symbols, binary_strings) = binary.into_parts();
        let mut symbol_table = SymbolTable::new();
        for (name, offset) in binary_symbols.into_entries() {
            symbol_table.define_function(name, offset, true)?;
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

    pub fn function_offset(&self, name: &str) -> Option<u32> {
        self.symbol_table.function_offset(name)
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

    pub fn rename_function(&mut self, current_name: &str, new_name: &str) -> AssemblerResult<()> {
        self.symbol_table.rename_function(current_name, new_name)?;

        for relocation in &mut self.relocations {
            match &mut relocation.kind {
                RelocationKind::Call { symbol } | RelocationKind::Address { symbol }
                    if symbol == current_name =>
                {
                    new_name.clone_into(symbol);
                }
                RelocationKind::Jump { function, .. } if function == current_name => {
                    new_name.clone_into(function);
                }
                RelocationKind::Call { .. }
                | RelocationKind::Address { .. }
                | RelocationKind::Jump { .. }
                | RelocationKind::StringOffset { .. } => {}
            }
        }

        Ok(())
    }

    pub fn make_function_private(&mut self, name: &str) -> AssemblerResult<()> {
        self.symbol_table.make_function_private(name)
    }

    pub fn define_private_function(&mut self, name: &str, offset: u32) -> AssemblerResult<()> {
        self.symbol_table
            .define_function(name.to_owned(), offset, false)
    }

    pub fn redirect(&mut self, entry_offset: u32, target_offset: u32) -> AssemblerResult<()> {
        for offset in [entry_offset, target_offset] {
            if !offset.is_multiple_of(4)
                || offset
                    .checked_add(4)
                    .is_none_or(|end| end as usize > self.code.len())
            {
                return Err(AssemblerError::InvalidCodeOffset(offset));
            }
        }

        let jump = encode_relative_jump(entry_offset, target_offset)?;
        let entry_index = entry_offset as usize;
        self.code[entry_index..entry_index + 4].copy_from_slice(&jump);
        Ok(())
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
                RelocationKind::Call { symbol } | RelocationKind::Address { symbol } => {
                    let target = self.symbol_table.resolve_symbol(symbol)?;
                    let operand = calculate_call_operand(relocation.code_offset, target.offset())?;
                    let operand_bytes = operand.to_be_bytes();
                    self.code[idx] = operand_bytes[0];
                    self.code[idx + 1] = operand_bytes[1];
                }
                RelocationKind::Jump { function, label } => {
                    let target = self.symbol_table.resolve_label(function, label)?;
                    let operand = calculate_call_operand(relocation.code_offset, target.offset())?;
                    let operand_bytes = operand.to_be_bytes();
                    self.code[idx] = operand_bytes[0];
                    self.code[idx + 1] = operand_bytes[1];
                }
                RelocationKind::StringOffset { value } => {
                    let string_offset = self
                        .string_table
                        .lookup(value)
                        .ok_or_else(|| AssemblerError::UndefinedString(value.clone()))?;
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
        let mut exports: Vec<_> = self.symbol_table.exports().collect();
        exports.sort_by(|(left_name, left), (right_name, right)| {
            left.offset()
                .cmp(&right.offset())
                .then_with(|| left_name.cmp(right_name))
        });

        for (name, symbol) in exports {
            table.add(name.to_owned(), symbol.offset());
        }
        table
    }
}
