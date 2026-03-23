use bixbite::{lexer, parser, types::TypeRef};

#[test]
fn parses_typed_method_signature() {
    let source = r#"def add(x: Integer, y: Integer) -> Integer
  x + y
end
"#;
    let tokens = lexer::tokenize(source, "src/add.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 1);
    assert!(unit.diagnostics.diagnostics.is_empty());
    let method = &unit.typed_methods[0];
    assert_eq!(method.name, "add");
    assert_eq!(method.params.len(), 2);
    assert_eq!(method.params[0].name, "x");
    assert_eq!(
        method.params[0].type_ref,
        TypeRef::Path(vec!["Integer".to_string()])
    );
    assert_eq!(method.params[0].default, None);
    assert_eq!(method.params[1].name, "y");
    assert_eq!(
        method.params[1].type_ref,
        TypeRef::Path(vec!["Integer".to_string()])
    );
    assert_eq!(
        method.return_type,
        TypeRef::Path(vec!["Integer".to_string()])
    );
}

#[test]
fn reports_missing_param_types_when_return_typed() {
    let source = "def f(x: Integer, y) -> Integer\n";
    let tokens = lexer::tokenize(source, "src/f.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 0);
    assert_eq!(unit.diagnostics.diagnostics.len(), 1);
    let diagnostic = &unit.diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "BIX001");
    assert_eq!(diagnostic.span.start.line, 1);
    assert_eq!(diagnostic.span.start.col, 19);
}

#[test]
fn reports_missing_arrow_for_return_type() {
    let source = "def add(x: Integer)\n";
    let tokens = lexer::tokenize(source, "src/add.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 0);
    assert_eq!(unit.diagnostics.diagnostics.len(), 1);
    let diagnostic = &unit.diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "BIX100");
}

#[test]
fn parses_boolean_type_and_default() {
    let source = "def greet(name: String, loud: Boolean = false) -> String\n";
    let tokens = lexer::tokenize(source, "src/greet.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 1);
    let method = &unit.typed_methods[0];
    assert_eq!(method.params[1].type_ref, TypeRef::Boolean);
    assert_eq!(method.params[1].default.as_deref(), Some("false"));
}
