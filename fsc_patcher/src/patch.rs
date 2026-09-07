use crate::{ExternalSymbolTable, PatchFailure};
use fsc_assembler::binary::{CODE_SECTION_FILE_OFFSET, FscriptBinary};
use fsc_parse::ast::{FuncDecl, FuncDef, Item, Script};

#[allow(dead_code)]
struct ClassifiedFunction<'a> {
    function: &'a FuncDef,
    patch_kind: FunctionPatchKind,
}

#[allow(dead_code)]
enum FunctionPatchKind {
    Replacement {
        original_address: u32,
        entry_offset: u32,
    },
    Custom,
}

#[allow(dead_code)]
struct ExternalFunction<'a> {
    declaration: &'a FuncDecl,
    address: u32,
    entry_offset: u32,
}

fn classify_functions<'a>(
    script: &'a Script,
    symbols: &ExternalSymbolTable,
    code_len: usize,
) -> Result<Vec<ClassifiedFunction<'a>>, PatchFailure> {
    script
        .items
        .iter()
        .filter_map(|item| match item {
            Item::FuncDef(function) => Some(function),
            Item::FuncDecl(_) => None,
        })
        .map(|function| {
            let patch_kind = match symbols.get_function(&function.header.name) {
                // current behavior sets all functions that are listed in the original_symbols as
                // replacement targets
                Some(original_address) => {
                    let entry_offset = function_entry_offset(
                        &function.header.name,
                        original_address,
                        symbols.base_address(),
                        code_len,
                    )?;
                    FunctionPatchKind::Replacement {
                        original_address,
                        entry_offset,
                    }
                }
                None => FunctionPatchKind::Custom,
            };

            Ok(ClassifiedFunction {
                function,
                patch_kind,
            })
        })
        .collect()
}

fn function_entry_offset(
    function_name: &str,
    address: u32,
    base_address: u32,
    code_len: usize,
) -> Result<u32, PatchFailure> {
    let invalid_address = || PatchFailure::InvalidFunctionAddress {
        function_name: function_name.to_owned(),
        address,
    };
    // it is absolute addresses like in the ghidra addon; TODO: define as explicit behavior
    let file_offset = address
        .checked_sub(base_address)
        .ok_or_else(invalid_address)?;
    let entry_offset = file_offset
        .checked_sub(CODE_SECTION_FILE_OFFSET)
        .ok_or_else(invalid_address)?;
    let entry_index = entry_offset as usize;

    if !entry_offset.is_multiple_of(4)
        || entry_index.checked_add(4).is_none_or(|end| end > code_len)
    {
        return Err(invalid_address());
    }

    Ok(entry_offset)
}

fn validate_external_declarations<'a>(
    script: &'a Script,
    symbols: &ExternalSymbolTable,
    code_len: usize,
) -> Result<Vec<ExternalFunction<'a>>, PatchFailure> {
    script
        .items
        .iter()
        .filter_map(|item| match item {
            Item::FuncDef(_) => None,
            Item::FuncDecl(declaration) => Some(declaration),
        })
        .map(|declaration| {
            let address = symbols
                .get_function(&declaration.header.name)
                .ok_or_else(|| PatchFailure::MissingExternalSymbol {
                    function_name: declaration.header.name.clone(),
                })?;
            let entry_offset = function_entry_offset(
                &declaration.header.name,
                address,
                symbols.base_address(),
                code_len,
            )?;
            Ok(ExternalFunction {
                declaration,
                address,
                entry_offset,
            })
        })
        .collect()
}

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
    let binary = FscriptBinary::deserialize(request.original_binary)?;
    let script = fsc_parse::parse(request.patch_source)
        .map_err(|error| PatchFailure::InvalidPatchSource(error.into_diagnostics()))?;
    let _functions = classify_functions(&script, request.symbols, binary.code_len())?;
    let _hir = fsc_sema::analyze(&script)
        .map_err(|error| PatchFailure::InvalidPatchSource(vec![error.into()]))?;
    let _external_functions =
        validate_external_declarations(&script, request.symbols, binary.code_len())?;
    Err(PatchFailure::NotImplemented)
}
