use super::{Diagnostic, LabelStyle, Severity};
use std::fmt::Write;

pub struct DiagnosticRenderer<'src> {
    source: &'src str,
    filename: &'src str,
    line_starts: &'src [usize],
}

impl<'src> DiagnosticRenderer<'src> {
    pub fn new(source: &'src str, filename: &'src str, line_starts: &'src [usize]) -> Self {
        Self {
            source,
            filename,
            line_starts,
        }
    }

    pub fn render(&self, diag: &Diagnostic) -> String {
        let mut out = String::new();

        let severity = match diag.severity {
            Severity::Error => "error",
        };
        let _ = writeln!(out, "{severity}: {}", diag.message);

        for label in &diag.labels {
            let (line, col) = self.line_col(label.span.start);
            let gutter = digits(line);
            let underline = match label.style {
                LabelStyle::Primary => '^',
            }
            .to_string()
            .repeat(label.span.end.saturating_sub(label.span.start).max(1));

            let _ = writeln!(
                out,
                "{:>width$} {}:{line}:{col}",
                "-->",
                self.filename,
                width = gutter + 3
            );
            let _ = writeln!(out, "{:gutter$} |", "");
            let _ = writeln!(out, "{line:gutter$} | {}", self.source_line(line));
            let _ = writeln!(
                out,
                "{:gutter$} | {:col$}{underline} {}",
                "",
                "",
                label.message,
                col = col.saturating_sub(1)
            );
        }

        out
    }

    fn line_col(&self, offset: usize) -> (usize, usize) {
        match self.line_starts.binary_search(&offset) {
            Ok(line_idx) => (line_idx + 1, 1),
            Err(0) => (1, offset + 1),
            Err(next_line) => match self.line_starts.get(next_line - 1) {
                Some(&line_start) => (next_line, offset - line_start + 1),
                None => (next_line, 1),
            },
        }
    }

    fn source_line(&self, line: usize) -> &str {
        let Some(&start) = self.line_starts.get(line - 1) else {
            return "";
        };
        let end = match self.line_starts.get(line) {
            Some(&offset) => offset,
            None => self.source.len(),
        };
        match self.source.get(start..end) {
            Some(s) => s.trim_end_matches('\n'),
            None => "",
        }
    }
}

fn digits(n: usize) -> usize {
    n.max(1).ilog10() as usize + 1
}
