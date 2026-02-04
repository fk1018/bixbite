use crate::{
    ast::{CompilationUnit, TypedMethod, TypedParam},
    diagnostic::{Diagnostic, DiagnosticReport, Severity, Span},
    lexer::{Token, TokenKind, TokenStream},
    types::TypeRef,
};

/// Parse a token stream and produce a CompilationUnit representing the parsed source.
///
/// # Examples
///
/// ```
/// // Construct a TokenStream from lexer/tokenizer output, then parse it:
/// // let tokens = TokenStream::from_source("def foo(): -> Boolean {}");
/// // let unit = parse(tokens);
/// ```
pub fn parse(tokens: TokenStream) -> CompilationUnit {
    let (source, tokens, file, diagnostics) = tokens.into_parts();
    let mut parser = Parser::new(source, tokens, file, diagnostics);
    parser.parse();
    CompilationUnit::from_source(parser.source, parser.typed_methods, parser.diagnostics)
}

#[derive(Debug, Clone)]
struct ParsedParam {
    name: String,
    name_span: Span,
    type_ref: Option<TypeRef>,
    default: Option<String>,
}

struct Parser {
    source: String,
    tokens: Vec<Token>,
    file: String,
    index: usize,
    typed_methods: Vec<TypedMethod>,
    diagnostics: DiagnosticReport,
}

impl Parser {
    /// Constructs a new Parser for the given source, token stream, file path, and diagnostic report.
    ///
    /// The returned parser is ready to begin parsing at the start of the token stream with an empty
    /// collection of typed methods.
    ///
    /// # Parameters
    ///
    /// - `diagnostics`: a `DiagnosticReport` to collect parsing errors and warnings.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bixbite::diagnostic::DiagnosticReport;
    /// use bixbite::lexer::tokenize;
    /// use bixbite::parser::Parser;
    ///
    /// let source = "def foo() -> Boolean {}".to_string();
    /// let tokens = tokenize(&source, "src/lib.bix");
    /// let file = "src/lib.bix".to_string();
    /// let diagnostics = DiagnosticReport::default();
    /// let parser = Parser::new(source, tokens.into_parts().1, file, diagnostics);
    /// ```
    fn new(
        source: String,
        tokens: Vec<Token>,
        file: String,
        diagnostics: DiagnosticReport,
    ) -> Self {
        Self {
            source,
            tokens,
            file,
            index: 0,
            typed_methods: Vec::new(),
            diagnostics,
        }
    }

    fn parse(&mut self) {
        while !self.is_at_end() {
            if self.matches(TokenKind::Def) {
                self.advance();
                self.parse_def();
            } else {
                self.advance();
            }
        }
    }

    fn parse_def(&mut self) {
        let mut name = String::new();
        let mut method_span = self.peek_span();

        if self.matches(TokenKind::SelfKw) {
            self.advance();
            if self.matches(TokenKind::Dot) {
                self.advance();
            } else {
                self.emit_error(
                    "BIX100",
                    "Expected `.` after `self`.",
                    self.peek_span(),
                    None,
                );
            }
            if let Some(token) = self.consume_ident("Expected method name after `self.`.") {
                method_span = token.span;
                name = format!("self.{}", token.lexeme);
            }
        } else if let Some(token) = self.consume_ident("Expected method name after `def`.") {
            method_span = token.span;
            name = token.lexeme;
        }

        if name.is_empty() {
            self.emit_error("BIX100", "Expected method name.", method_span, None);
            self.skip_to_newline();
            return;
        }

        if !self.matches(TokenKind::LParen) {
            self.emit_error(
                "BIX100",
                "Expected `(` after method name.",
                self.peek_span(),
                None,
            );
            self.skip_to_newline();
            return;
        }
        self.advance();

        let params = self.parse_params();
        if !self.matches(TokenKind::RParen) {
            self.emit_error(
                "BIX100",
                "Expected `)` to close parameter list.",
                self.peek_span(),
                None,
            );
            self.skip_to_newline();
            return;
        }
        self.advance();

        if !self.matches(TokenKind::Arrow) {
            self.emit_error(
                "BIX100",
                "Missing return type signature: expected `-> <Type>`.",
                self.peek_span(),
                None,
            );
            self.skip_to_newline();
            return;
        }
        self.advance();

        let return_type = match self.parse_type() {
            Some(type_ref) => type_ref,
            None => {
                self.emit_error(
                    "BIX100",
                    "Expected return type after `->`.",
                    self.peek_span(),
                    None,
                );
                self.skip_to_newline();
                return;
            }
        };

        let mut has_missing_types = false;
        for param in &params {
            if param.type_ref.is_none() {
                self.diagnostics.diagnostics.push(Diagnostic {
                    code: "BIX001".to_owned(),
                    severity: Severity::Error,
                    file: self.file.clone(),
                    message: "Typed method signature requires all params to be typed.".to_owned(),
                    span: param.name_span,
                    suggestion: Some("Add `: Type` to this parameter.".to_owned()),
                });
                has_missing_types = true;
            }
        }

        if has_missing_types {
            return;
        }

        let typed_params = params
            .into_iter()
            .filter_map(|param| {
                param.type_ref.map(|type_ref| TypedParam {
                    name: param.name,
                    type_ref,
                    default: param.default,
                })
            })
            .collect();

        self.typed_methods.push(TypedMethod {
            name,
            params: typed_params,
            return_type,
        });
    }

    fn parse_params(&mut self) -> Vec<ParsedParam> {
        let mut params = Vec::new();
        while !self.is_at_end()
            && !self.matches(TokenKind::RParen)
            && !self.matches(TokenKind::Newline)
        {
            if self.matches(TokenKind::Comma) {
                self.advance();
                if self.matches(TokenKind::Newline) {
                    break;
                }
                continue;
            }

            let name_token = match self.consume_ident("Expected parameter name.") {
                Some(token) => token,
                None => {
                    self.skip_param();
                    if self.matches(TokenKind::Newline) {
                        break;
                    }
                    continue;
                }
            };

            let mut type_ref = None;
            if self.matches(TokenKind::Colon) {
                self.advance();
                type_ref = self.parse_type();
            }

            let default = if self.matches(TokenKind::Eq) {
                self.advance();
                Some(self.parse_default_value())
            } else {
                None
            };

            params.push(ParsedParam {
                name: name_token.lexeme,
                name_span: name_token.span,
                type_ref,
                default,
            });

            if self.matches(TokenKind::Comma) {
                self.advance();
                if self.matches(TokenKind::Newline) {
                    break;
                }
            }
        }
        params
    }

    fn parse_type(&mut self) -> Option<TypeRef> {
        let first = self.consume_kind(TokenKind::Const, "Expected type name.")?;
        let mut segments = vec![first.lexeme];
        while self.matches(TokenKind::DoubleColon) {
            self.advance();
            match self.consume_kind(TokenKind::Const, "Expected type segment after `::`.") {
                Some(segment) => segments.push(segment.lexeme),
                None => break,
            }
        }
        if segments.len() == 1 && segments[0] == "Boolean" {
            Some(TypeRef::Boolean)
        } else {
            Some(TypeRef::path(segments))
        }
    }

    fn parse_default_value(&mut self) -> String {
        let start_index = self.index;
        while !self.is_at_end()
            && !matches!(
                self.peek_kind(),
                TokenKind::Comma | TokenKind::RParen | TokenKind::Newline
            )
        {
            self.advance();
        }
        if start_index >= self.index {
            return String::new();
        }
        let start = self.tokens[start_index].byte_range.start;
        let end = self.tokens[self.index - 1].byte_range.end;
        self.source.get(start..end).unwrap_or("").trim().to_string()
    }

    fn consume_ident(&mut self, message: &str) -> Option<Token> {
        self.consume_kind(TokenKind::Ident, message)
    }

    fn consume_kind(&mut self, kind: TokenKind, message: &str) -> Option<Token> {
        if self.matches(kind.clone()) {
            Some(self.advance().clone())
        } else {
            self.emit_error("BIX100", message, self.peek_span(), None);
            None
        }
    }

    fn skip_param(&mut self) {
        while !self.is_at_end()
            && !matches!(
                self.peek_kind(),
                TokenKind::Comma | TokenKind::RParen | TokenKind::Newline
            )
        {
            self.advance();
        }
    }

    fn skip_to_newline(&mut self) {
        while !self.is_at_end() && !self.matches(TokenKind::Newline) {
            self.advance();
        }
    }

    fn emit_error(&mut self, code: &str, message: &str, span: Span, suggestion: Option<String>) {
        self.diagnostics.diagnostics.push(Diagnostic {
            code: code.to_owned(),
            severity: Severity::Error,
            file: self.file.clone(),
            message: message.to_owned(),
            span,
            suggestion,
        });
    }

    fn matches(&self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.index)
            .map(|token| token.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.index)
            .map(|token| token.span)
            .unwrap_or_else(|| Span::point(1, 1))
    }

    fn advance(&mut self) -> &Token {
        let token = self.tokens.get(self.index);
        if !self.is_at_end() {
            self.index += 1;
        }
        token.unwrap_or_else(|| self.tokens.last().expect("token list is empty"))
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}
