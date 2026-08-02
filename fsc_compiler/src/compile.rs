use crate::diagnostic::CompileFailure;
use fsc_assembler::Assembler;
use fsc_diagnostics::{Diagnostic, Stage};

#[derive(Debug, Clone, Copy)]
pub struct CompileRequest<'src> {
    pub source: &'src str,
    pub script_name: &'src str,
}

impl<'src> CompileRequest<'src> {
    #[must_use]
    pub const fn new(source: &'src str, script_name: &'src str) -> Self {
        Self {
            source,
            script_name,
        }
    }
}

#[derive(Debug)]
pub struct CompileArtifact {
    bytes: Vec<u8>,
    diagnostics: Vec<Diagnostic>,
}

impl CompileArtifact {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

pub fn compile(request: CompileRequest<'_>) -> Result<CompileArtifact, CompileFailure> {
    let script = fsc_parse::parse(request.source)
        .map_err(|error| CompileFailure::from_diagnostics(error.into_diagnostics()))?;

    let hir = fsc_sema::analyze(&script)
        .map_err(|error| CompileFailure::from_diagnostic(Diagnostic::from(error)))?;

    let mut assembler = Assembler::new();
    fsc_codegen::compile(&hir, &mut assembler).map_err(|error| {
        // TODO: attach source labels when HIR has spans
        CompileFailure::from_diagnostic(Diagnostic::error(Stage::Codegen, error.to_string()))
    })?;

    let binary = assembler
        .finalize(request.script_name.to_owned())
        .map_err(assembly_failure)?;
    let bytes = binary.serialize().map_err(assembly_failure)?;

    Ok(CompileArtifact {
        bytes,
        // TODO: collect successful-stage warnings
        diagnostics: Vec::new(),
    })
}

fn assembly_failure(error: impl std::fmt::Display) -> CompileFailure {
    // TODO: attach source labels on assembly error
    CompileFailure::from_diagnostic(Diagnostic::error(Stage::Assembly, error.to_string()))
}
