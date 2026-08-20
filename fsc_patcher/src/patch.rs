use crate::{ExternalSymbolTable, PatchFailure};

pub struct PatchRequest<'a> {
    pub patch_source: &'a str,
    pub original_binary: &'a [u8],
    // keeping symbols as typed input so it's independent of the used file format, so we're not
    // too strongly bound to one specific format
    pub symbols: &'a ExternalSymbolTable,
    pub base_address: u32,
}

impl<'a> PatchRequest<'a> {
    pub const fn new(
        patch_source: &'a str,
        original_binary: &'a [u8],
        symbols: &'a ExternalSymbolTable,
        base_address: u32,
    ) -> Self {
        Self {
            patch_source,
            original_binary,
            symbols,
            base_address,
        }
    }
}

pub fn patch(_request: PatchRequest<'_>) -> Result<Vec<u8>, PatchFailure> {
    Err(PatchFailure::NotImplemented)
}
