use crate::PatchError;
use fsc_assembler::binary::FscriptBinary;
use fsc_assembler::encoding::encode_relative_jump;

/// First version of patching/replacing functions in an existing binary. The idea is to replace the
/// first instruction with a relative jump to the new function, appended at the end of the code
/// section.
///
/// Advantage: no decompilation necessary; callers that enter at the original function entry are
/// redirected automatically. Simple and quick.
///
/// Limitations:
/// - The replacement must be within the relative-jump range.
/// - The original function body remains in the code section.
pub fn append_and_redirect(
    binary: &mut FscriptBinary,
    entry_offset: u32,
    appended_code: Vec<u8>,
) -> Result<(), PatchError> {
    let target = u32::try_from(binary.code_len()).map_err(|_| PatchError::UnalignedCode)?;
    if !target.is_multiple_of(4) || !appended_code.len().is_multiple_of(4) {
        return Err(PatchError::UnalignedCode);
    }
    if !entry_offset.is_multiple_of(4) || entry_offset.checked_add(4).is_none_or(|end| end > target)
    {
        return Err(PatchError::InvalidEntryOffset(entry_offset));
    }
    let jump = encode_relative_jump(entry_offset, target).map_err(|_error| {
        PatchError::JumpOutOfRange {
            entry: entry_offset,
            target,
        }
    })?;

    binary.replace_code_word(entry_offset, jump);
    binary.append_code(appended_code);
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use fsc_assembler::Assembler;

    #[test]
    fn redirects_a_one_instruction_function_to_appended_code() {
        let mut assembler = Assembler::new();
        assembler.define_function("MAIN", false).unwrap();
        assembler.emit_ret(0);
        let mut binary = assembler.finalize("TEST".to_owned()).unwrap();

        append_and_redirect(&mut binary, 0, vec![0, 0, 0, 6]).unwrap();

        let bytes = binary.serialize().unwrap();
        assert_eq!(&bytes[0x20..0x24], &[0, 0, 0, 8]);
        assert_eq!(&bytes[0x24..0x28], &[0, 0, 0, 6]);
    }
}
