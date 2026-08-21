use crate::{ExternalSymbolTable, PatchFailure};
use fsc_assembler::binary::FscriptBinary;

pub struct PatchRequest<'a> {
    pub patch_source: &'a str,
    pub original_binary: &'a [u8],
    // keeping symbols as typed input so it's independent of the used file format, so we're not
    // too strongly bound to one specific format
    pub symbols: &'a ExternalSymbolTable,
}

impl<'a> PatchRequest<'a> {
    pub const fn new(
        patch_source: &'a str,
        original_binary: &'a [u8],
        symbols: &'a ExternalSymbolTable,
    ) -> Self {
        Self {
            patch_source,
            original_binary,
            symbols,
        }
    }
}

pub struct PatchArtifact {
    binary: Vec<u8>,
    symbols: ExternalSymbolTable,
}

impl PatchArtifact {
    pub fn binary(&self) -> &[u8] {
        &self.binary
    }

    pub const fn symbols(&self) -> &ExternalSymbolTable {
        &self.symbols
    }

    pub fn into_parts(self) -> (Vec<u8>, ExternalSymbolTable) {
        (self.binary, self.symbols)
    }
}

pub fn patch(request: PatchRequest<'_>) -> Result<PatchArtifact, PatchFailure> {
    let _binary = FscriptBinary::deserialize(request.original_binary)?;
    let _script = fsc_parse::parse(request.patch_source)
        .map_err(|error| PatchFailure::InvalidPatchSource(error.into_diagnostics()))?;
    Err(PatchFailure::NotImplemented)
}
