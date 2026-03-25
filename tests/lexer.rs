use bixbite::{lexer, parser};

#[test]
fn reports_unterminated_string_literal() {
    let source = "def greet(name: Str = \"oops) -> Str\n";
    let tokens = lexer::tokenize(source, "src/greet.bixb");
    let diagnostics = tokens.diagnostics();

    assert_eq!(diagnostics.diagnostics.len(), 1);
    let diagnostic = &diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "BIX000");
    assert!(diagnostic.message.contains("Unterminated string literal"));
    assert_eq!(diagnostic.span.start.line, 1);
}

/// Verifies that an unknown character detected by the lexer is reported and that the diagnostic is propagated to the parser.
///
/// This test tokenizes a source containing an invalid character (`$`), asserts the lexer produced a single diagnostic,
/// then parses the tokens and asserts the parser's unit contains the same diagnostic code `BIX000`.
///
/// # Examples
///
/// ```
/// let source = "def add(x: Int = $) -> Int\n";
/// let tokens = lexer::tokenize(source, "src/add.bixb");
/// assert_eq!(tokens.diagnostics().diagnostics.len(), 1);
///
/// let unit = parser::parse(tokens);
/// assert_eq!(unit.diagnostics.diagnostics.len(), 1);
/// assert_eq!(unit.diagnostics.diagnostics[0].code, "BIX000");
/// ```
#[test]
fn reports_unknown_character_and_propagates_to_parser() {
    let source = "def add(x: Int = $) -> Int\n";
    let tokens = lexer::tokenize(source, "src/add.bixb");

    assert_eq!(tokens.diagnostics().diagnostics.len(), 1);

    let unit = parser::parse(tokens);
    assert_eq!(unit.diagnostics.diagnostics.len(), 1);
    assert_eq!(unit.diagnostics.diagnostics[0].code, "BIX000");
}

#[test]
fn rejects_non_ascii_input_with_single_diagnostic() {
    let source = "def café() -> Str\n";
    let tokens = lexer::tokenize(source, "src/non_ascii.bixb");
    let diagnostics = tokens.diagnostics();

    assert_eq!(diagnostics.diagnostics.len(), 1);
    let diagnostic = &diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "BIX100");
    assert_eq!(diagnostic.span.start.line, 1);
    assert_eq!(diagnostic.span.start.col, 8);
    assert_eq!(tokens.tokens().len(), 1);
    assert_eq!(tokens.tokens()[0].kind, lexer::TokenKind::Eof);
}

#[test]
fn ignores_non_signature_body_characters() {
    let source = "def add(x: Int, y: Int) -> Int\n  @value = x + y\nend\n";
    let tokens = lexer::tokenize(source, "src/add.bixb");

    assert!(tokens.diagnostics().diagnostics.is_empty());

    let unit = parser::parse(tokens);
    assert!(unit.diagnostics.diagnostics.is_empty());
    assert_eq!(unit.typed_methods.len(), 1);
}
