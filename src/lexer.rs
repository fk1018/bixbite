use std::ops::Range;

use crate::diagnostic::{Diagnostic, DiagnosticReport, Pos, Severity, Span};

/// Token kinds recognized by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// `def` keyword.
    Def,
    /// `self` keyword.
    SelfKw,
    /// Lowercase identifier (method/parameter names).
    Ident,
    /// Uppercase identifier (constant/type name).
    Const,
    /// `->` return type arrow.
    Arrow,
    /// `:` type annotation separator.
    Colon,
    /// `,` parameter separator.
    Comma,
    /// `.` method receiver separator.
    Dot,
    /// `(`.
    LParen,
    /// `)`.
    RParen,
    /// `=` default assignment.
    Eq,
    /// `::` constant path separator.
    DoubleColon,
    /// Quoted string literal.
    StringLiteral,
    /// Numeric literal.
    Number,
    /// Newline token (`\n`).
    Newline,
    /// Unrecognized character token.
    Unknown,
    /// End-of-file marker.
    Eof,
}

/// A single token with its lexeme and source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Kind of token recognized.
    pub kind: TokenKind,
    /// Source substring for the token.
    pub lexeme: String,
    /// Inclusive source span for the token.
    pub span: Span,
    /// Byte range for the token within the source text.
    pub byte_range: Range<usize>,
}

/// Stream of tokens with associated source text and diagnostics.
///
/// Invariants:
/// - Tokens are ordered as they appear in the input.
/// - The final token is always `TokenKind::Eof`.
#[derive(Debug, Clone)]
pub struct TokenStream {
    source: String,
    tokens: Vec<Token>,
    file: String,
    diagnostics: DiagnosticReport,
}

/// Produce a sequence of lexical tokens and diagnostics from the given source text.
///
/// The lexer recognizes keywords (`def`, `self`), identifiers, constants (identifiers starting with
/// an uppercase letter), numeric literals, string literals, punctuation (parentheses, colon, comma,
/// dot, equals, arrow, double colon), newlines, and an explicit end-of-file token. It records
/// diagnostics for unterminated string literals and unexpected characters.
///
/// # Examples
///
/// ```
/// use bixbite::lexer::tokenize;
///
/// let src = "def foo = 42\n";
/// let ts = tokenize(src, "example");
/// assert!(!ts.tokens().is_empty());
/// ```
pub fn tokenize(source: &str, file: impl Into<String>) -> TokenStream {
    let mut tokens = Vec::new();
    let mut diagnostics = DiagnosticReport::default();
    let mut index = 0;
    let mut line = 1;
    let mut col = 1;
    let bytes = source.as_bytes();
    let file = file.into();

    let mut push_token =
        |kind: TokenKind, start: usize, end: usize, start_pos: Pos, end_pos: Pos| {
            let lexeme = source.get(start..end).unwrap_or("").to_string();
            tokens.push(Token {
                kind,
                lexeme,
                span: Span::new(start_pos, end_pos),
                byte_range: start..end,
            });
        };

    while index < bytes.len() {
        let ch = bytes[index] as char;

        if ch == '\n' {
            let start_pos = Pos::new(line, col);
            let end_pos = start_pos;
            push_token(TokenKind::Newline, index, index + 1, start_pos, end_pos);
            index += 1;
            line += 1;
            col = 1;
            continue;
        }

        if ch.is_whitespace() {
            index += 1;
            col += 1;
            continue;
        }

        let start_pos = Pos::new(line, col);

        if ch == '-' && bytes.get(index + 1) == Some(&b'>') {
            let end_pos = Pos::new(line, col + 1);
            push_token(TokenKind::Arrow, index, index + 2, start_pos, end_pos);
            index += 2;
            col += 2;
            continue;
        }

        if ch == ':' && bytes.get(index + 1) == Some(&b':') {
            let end_pos = Pos::new(line, col + 1);
            push_token(TokenKind::DoubleColon, index, index + 2, start_pos, end_pos);
            index += 2;
            col += 2;
            continue;
        }

        match ch {
            '(' => {
                push_token(TokenKind::LParen, index, index + 1, start_pos, start_pos);
                index += 1;
                col += 1;
                continue;
            }
            ')' => {
                push_token(TokenKind::RParen, index, index + 1, start_pos, start_pos);
                index += 1;
                col += 1;
                continue;
            }
            ':' => {
                push_token(TokenKind::Colon, index, index + 1, start_pos, start_pos);
                index += 1;
                col += 1;
                continue;
            }
            ',' => {
                push_token(TokenKind::Comma, index, index + 1, start_pos, start_pos);
                index += 1;
                col += 1;
                continue;
            }
            '.' => {
                push_token(TokenKind::Dot, index, index + 1, start_pos, start_pos);
                index += 1;
                col += 1;
                continue;
            }
            '=' => {
                push_token(TokenKind::Eq, index, index + 1, start_pos, start_pos);
                index += 1;
                col += 1;
                continue;
            }
            '"' | '\'' => {
                let quote = ch;
                let mut end = index + 1;
                let mut end_col = col + 1;
                let mut terminated = false;
                while end < bytes.len() {
                    let next = bytes[end] as char;
                    if next == '\n' {
                        break;
                    }
                    end += 1;
                    end_col += 1;
                    if next == quote {
                        terminated = true;
                        break;
                    }
                }
                let end_pos = Pos::new(line, end_col.saturating_sub(1));
                if !terminated {
                    diagnostics.diagnostics.push(Diagnostic {
                        code: "BIX000".to_owned(),
                        severity: Severity::Error,
                        file: file.clone(),
                        message: "Unterminated string literal.".to_owned(),
                        span: Span::new(start_pos, end_pos),
                        suggestion: Some("Add a closing quote.".to_owned()),
                    });
                }
                push_token(TokenKind::StringLiteral, index, end, start_pos, end_pos);
                index = end;
                col = end_col;
                continue;
            }
            _ => {}
        }

        if ch.is_ascii_digit() {
            let mut end = index + 1;
            let mut end_col = col + 1;
            while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                end += 1;
                end_col += 1;
            }
            let end_pos = Pos::new(line, end_col - 1);
            push_token(TokenKind::Number, index, end, start_pos, end_pos);
            index = end;
            col = end_col;
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut end = index + 1;
            let mut end_col = col + 1;
            while end < bytes.len() {
                let next = bytes[end] as char;
                if next.is_ascii_alphanumeric() || next == '_' {
                    end += 1;
                    end_col += 1;
                } else {
                    break;
                }
            }
            if let Some(next) = bytes.get(end).map(|b| *b as char) {
                if matches!(next, '?' | '!' | '=') {
                    end += 1;
                    end_col += 1;
                }
            }
            let end_pos = Pos::new(line, end_col - 1);
            let lexeme = source.get(index..end).unwrap_or("");
            let kind = match lexeme {
                "def" => TokenKind::Def,
                "self" => TokenKind::SelfKw,
                _ => {
                    if lexeme
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_uppercase())
                        .unwrap_or(false)
                    {
                        TokenKind::Const
                    } else {
                        TokenKind::Ident
                    }
                }
            };
            push_token(kind, index, end, start_pos, end_pos);
            index = end;
            col = end_col;
            continue;
        }

        let end_pos = start_pos;
        diagnostics.diagnostics.push(Diagnostic {
            code: "BIX000".to_owned(),
            severity: Severity::Error,
            file: file.clone(),
            message: format!("Unexpected character `{}`.", ch),
            span: Span::new(start_pos, end_pos),
            suggestion: None,
        });
        push_token(TokenKind::Unknown, index, index + 1, start_pos, end_pos);
        index += 1;
        col += 1;
    }

    let eof_pos = Pos::new(line, col);
    tokens.push(Token {
        kind: TokenKind::Eof,
        lexeme: String::new(),
        span: Span::new(eof_pos, eof_pos),
        byte_range: index..index,
    });

    TokenStream {
        source: source.to_owned(),
        tokens,
        file,
        diagnostics,
    }
}

impl TokenStream {
    /// Returns the original source text for this token stream.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the ordered tokens produced by the lexer.
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// The file identifier associated with this token stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use bixbite::lexer::tokenize;
    ///
    /// let ts = tokenize("", "test.bix");
    /// assert_eq!(ts.file(), "test.bix");
    /// ```
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Accesses the diagnostics collected during tokenization.
    ///
    /// # Examples
    ///
    /// ```
    /// use bixbite::lexer::tokenize;
    ///
    /// let ts = tokenize("", "file.rs");
    /// let diagnostics = ts.diagnostics();
    /// ```
    pub fn diagnostics(&self) -> &DiagnosticReport {
        &self.diagnostics
    }

    /// Decomposes the TokenStream into its constituent parts.
    ///
    /// Consumes the stream and yields a tuple with the original source string, the produced tokens,
    /// the file identifier, and the collected diagnostic report.
    ///
    /// # Examples
    ///
    /// ```
    /// use bixbite::lexer::tokenize;
    ///
    /// let ts = tokenize("let a = 1", "main.bix");
    /// let (source, tokens, file, diagnostics) = ts.into_parts();
    /// assert_eq!(source, "let a = 1");
    /// assert_eq!(file, "main.bix");
    /// ```
    pub fn into_parts(self) -> (String, Vec<Token>, String, DiagnosticReport) {
        (self.source, self.tokens, self.file, self.diagnostics)
    }
}
