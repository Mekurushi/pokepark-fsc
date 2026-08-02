use fsc_parse::diagnostic as parse_diagnostic;
//TODO: deduplicating diagnostics an having it centralized so usage would be possible for all stages and only displayed here
#[derive(Debug)]
pub struct CompileFailure {
    diagnostics: Vec<CompileDiagnostic>,
}

impl CompileFailure {
    pub(crate) fn from_diagnostic(diagnostic: CompileDiagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[CompileDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn render(&self, source_name: &str, source: &str) -> String {
        render_diagnostics(&self.diagnostics, source_name, source)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileStage {
    Parse,
    Semantic,
    Codegen,
    Assembly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Primary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLabel {
    pub span: SourceSpan,
    pub message: String,
    pub style: LabelStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileDiagnostic {
    stage: CompileStage,
    severity: Severity,
    message: String,
    labels: Vec<SourceLabel>,
}

impl CompileDiagnostic {
    pub(crate) fn error(stage: CompileStage, message: impl Into<String>) -> Self {
        Self {
            stage,
            severity: Severity::Error,
            message: message.into(),
            labels: Vec::new(),
        }
    }

    #[must_use]
    pub const fn stage(&self) -> CompileStage {
        self.stage
    }

    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn labels(&self) -> &[SourceLabel] {
        &self.labels
    }
}

impl From<parse_diagnostic::Diagnostic> for CompileDiagnostic {
    fn from(diagnostic: parse_diagnostic::Diagnostic) -> Self {
        let severity = match diagnostic.severity {
            parse_diagnostic::Severity::Error => Severity::Error,
        };
        let labels = diagnostic
            .labels
            .into_iter()
            .map(|label| SourceLabel {
                span: SourceSpan {
                    start: label.span.start,
                    end: label.span.end,
                },
                message: label.message,
                style: match label.style {
                    parse_diagnostic::LabelStyle::Primary => LabelStyle::Primary,
                },
            })
            .collect();

        Self {
            stage: CompileStage::Parse,
            severity,
            message: diagnostic.message,
            labels,
        }
    }
}

#[must_use]
pub fn render_diagnostics(
    diagnostics: &[CompileDiagnostic],
    source_name: &str,
    source: &str,
) -> String {
    let renderer = DiagnosticRenderer::new(source, source_name);
    diagnostics
        .iter()
        .map(|diagnostic| renderer.render(diagnostic))
        .collect()
}

struct DiagnosticRenderer<'src> {
    source: &'src str,
    source_name: &'src str,
    line_starts: Vec<usize>,
}

impl<'src> DiagnosticRenderer<'src> {
    fn new(source: &'src str, source_name: &'src str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        );
        Self {
            source,
            source_name,
            line_starts,
        }
    }

    fn render(&self, diagnostic: &CompileDiagnostic) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        let severity = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let _ = writeln!(output, "{severity}: {}", diagnostic.message);

        for label in &diagnostic.labels {
            let (line, column) = self.line_col(label.span.start);
            let gutter = digits(line);
            let underline = match label.style {
                LabelStyle::Primary => '^',
            }
            .to_string()
            .repeat(label.span.end.saturating_sub(label.span.start).max(1));

            let _ = writeln!(
                output,
                "{:>width$} {}:{line}:{column}",
                "-->",
                self.source_name,
                width = gutter + 3,
            );
            let _ = writeln!(output, "{:gutter$} |", "");
            let _ = writeln!(output, "{line:gutter$} | {}", self.source_line(line));
            let _ = writeln!(
                output,
                "{:gutter$} | {:column$}{underline} {}",
                "",
                "",
                label.message,
                column = column.saturating_sub(1),
            );
        }

        output
    }

    fn line_col(&self, offset: usize) -> (usize, usize) {
        match self.line_starts.binary_search(&offset) {
            Ok(line_index) => (line_index + 1, 1),
            Err(0) => (1, offset + 1),
            Err(next_line) => {
                let line_start = self.line_starts[next_line - 1];
                (next_line, offset.saturating_sub(line_start) + 1)
            }
        }
    }

    fn source_line(&self, line: usize) -> &str {
        let Some(&start) = self.line_starts.get(line.saturating_sub(1)) else {
            return "";
        };
        let end = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.source.len());
        self.source
            .get(start..end)
            .unwrap_or_default()
            .trim_end_matches(['\r', '\n'])
    }
}

fn digits(number: usize) -> usize {
    number.max(1).ilog10() as usize + 1
}
