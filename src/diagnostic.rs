use serde::Serialize;

/// Severity level for a diagnostic entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// An error that should fail the command.
    Error,
    /// A warning that does not fail the command.
    Warn,
}

/// One-based line/column position in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pos {
    /// One-based line number.
    pub line: usize,
    /// One-based column number.
    pub col: usize,
}

/// Span between two positions in a source file.
///
/// Invariant: `start` and `end` are inclusive positions in the same file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// Start position (inclusive).
    pub start: Pos,
    /// End position (inclusive).
    pub end: Pos,
}

impl Pos {
    /// Creates a new position from one-based line and column numbers.
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl Span {
    /// Creates a span from start and end positions.
    pub const fn new(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    /// Creates a single-point span at a given position (inclusive).
    pub const fn point(line: usize, col: usize) -> Self {
        let pos = Pos::new(line, col);
        Self {
            start: pos,
            end: pos,
        }
    }
}

/// A single diagnostic emitted during lexing, parsing, or checking.
///
/// Invariants:
/// - `file` is the logical source identifier used for reporting.
/// - `span` describes the inclusive range of the issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// Stable diagnostic code (e.g., `BIX001`).
    pub code: String,
    /// Severity of the diagnostic.
    pub severity: Severity,
    /// File identifier for the diagnostic (path or logical name).
    pub file: String,
    /// Human-readable message.
    pub message: String,
    /// Source span for the diagnostic.
    #[serde(rename = "range")]
    pub span: Span,
    /// Optional fix suggestion for editors or humans.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl Diagnostic {
    /// Creates an error diagnostic without a fix suggestion.
    ///
    /// Callers may attach a suggestion by setting `suggestion` directly when needed.
    pub fn error(
        code: impl Into<String>,
        file: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            code: code.into(),
            severity: Severity::Error,
            file: file.into(),
            message: message.into(),
            span,
            suggestion: None,
        }
    }
}

/// Collection of diagnostics emitted during a compiler phase.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DiagnosticReport {
    /// Ordered list of diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    /// Creates a report containing a single diagnostic.
    pub fn single(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    /// Returns true if the report does not contain any diagnostics.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns true if any diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    /// Appends diagnostics from another report in order.
    pub fn extend(&mut self, other: DiagnosticReport) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Appends a single diagnostic to the report.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Renders diagnostics in the human CLI format.
    pub fn render_human(&self) -> String {
        let mut output = String::new();

        for diagnostic in &self.diagnostics {
            let severity = match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warn => "warn",
            };
            output.push_str(&format!(
                "{}:{}:{}: {} {} ({})\n",
                diagnostic.file,
                diagnostic.span.start.line,
                diagnostic.span.start.col,
                severity,
                diagnostic.message,
                diagnostic.code
            ));
            if let Some(suggestion) = &diagnostic.suggestion {
                output.push_str(&format!("  help: {suggestion}\n"));
            }
        }

        output
    }

    /// Prints diagnostics in human-readable form to stderr.
    ///
    /// This is a lossy formatter intended for CLI output, not machine parsing.
    pub fn print_human_stderr(&self) {
        let rendered = self.render_human();
        if !rendered.is_empty() {
            eprint!("{rendered}");
        }
    }
}
