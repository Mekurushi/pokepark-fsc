mod compile;
mod diagnostic;

pub use compile::{CompileArtifact, CompileRequest, compile};
pub use diagnostic::{
    CompileDiagnostic, CompileFailure, CompileStage, LabelStyle, Severity, SourceLabel, SourceSpan,
    render_diagnostics,
};
