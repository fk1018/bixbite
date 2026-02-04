use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Pos {
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl Span {
    pub const fn new(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    pub const fn point(line: usize, col: usize) -> Self {
        let pos = Pos::new(line, col);
        Self {
            start: pos,
            end: pos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub file: String,
    pub message: String,
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    pub fn print_human_stderr(&self) {
        for diagnostic in &self.diagnostics {
            let severity = match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warn => "warn",
            };
            eprintln!(
                "{}:{}:{}: {} {} ({})",
                diagnostic.file,
                diagnostic.span.start.line,
                diagnostic.span.start.col,
                severity,
                diagnostic.message,
                diagnostic.code
            );
            if let Some(suggestion) = &diagnostic.suggestion {
                eprintln!("  help: {suggestion}");
            }
        }
    }
}
