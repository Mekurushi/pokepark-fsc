use crate::diagnostic::{CompileDiagnostic, CompileFailure, CompileStage};
use fsc_assembler::Assembler;
use fsc_parse::diagnostic as parse_diagnostic;

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
    diagnostics: Vec<CompileDiagnostic>,
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
    pub fn diagnostics(&self) -> &[CompileDiagnostic] {
        &self.diagnostics
    }
}

pub fn compile(request: CompileRequest<'_>) -> Result<CompileArtifact, CompileFailure> {
    let script = fsc_parse::parse(request.source).map_err(|error| {
        let diagnostic = parse_diagnostic::Diagnostic::from(error);
        CompileFailure::from_diagnostic(diagnostic.into())
    })?;

    let hir = fsc_sema::analyze(&script).map_err(|error| {
        CompileFailure::from_diagnostic(CompileDiagnostic::error(
            CompileStage::Semantic,
            error.to_string(),
        ))
    })?;

    let mut assembler = Assembler::new();
    fsc_codegen::compile(&hir, &mut assembler).map_err(|error| {
        CompileFailure::from_diagnostic(CompileDiagnostic::error(
            CompileStage::Codegen,
            error.to_string(),
        ))
    })?;

    let binary = assembler
        .finalize(request.script_name.to_owned())
        .map_err(assembly_failure)?;
    let bytes = binary.serialize().map_err(assembly_failure)?;

    Ok(CompileArtifact {
        bytes,
        diagnostics: Vec::new(),
    })
}

fn assembly_failure(error: impl std::fmt::Display) -> CompileFailure {
    CompileFailure::from_diagnostic(CompileDiagnostic::error(
        CompileStage::Assembly,
        error.to_string(),
    ))
}
