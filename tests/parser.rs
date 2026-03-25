use bixbite::{
    lexer, parser,
    types::{PrimitiveType, TypeRef},
};

#[test]
fn parses_typed_method_signature() {
    let source = r#"def add(x: Int, y: Int) -> Int
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
        TypeRef::Primitive(PrimitiveType::Int)
    );
    assert_eq!(method.params[0].default, None);
    assert_eq!(method.params[1].name, "y");
    assert_eq!(
        method.params[1].type_ref,
        TypeRef::Primitive(PrimitiveType::Int)
    );
    assert_eq!(method.return_type, TypeRef::Primitive(PrimitiveType::Int));
}

#[test]
fn reports_missing_param_types_when_return_typed() {
    let source = "def f(x: Int, y) -> Int\n";
    let tokens = lexer::tokenize(source, "src/f.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 0);
    assert_eq!(unit.diagnostics.diagnostics.len(), 1);
    let diagnostic = &unit.diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "BIX001");
    assert_eq!(diagnostic.span.start.line, 1);
    assert_eq!(diagnostic.span.start.col, 15);
}

#[test]
fn reports_missing_arrow_for_return_type() {
    let source = "def add(x: Int)\n";
    let tokens = lexer::tokenize(source, "src/add.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 0);
    assert_eq!(unit.diagnostics.diagnostics.len(), 1);
    let diagnostic = &unit.diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "BIX100");
}

#[test]
fn parses_boolean_type_and_default() {
    let source = "def greet(name: Str, loud: Bool = false) -> Str\n";
    let tokens = lexer::tokenize(source, "src/greet.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 1);
    let method = &unit.typed_methods[0];
    assert_eq!(
        method.params[0].type_ref,
        TypeRef::Primitive(PrimitiveType::Str)
    );
    assert_eq!(
        method.params[1].type_ref,
        TypeRef::Primitive(PrimitiveType::Bool)
    );
    assert_eq!(method.params[1].default.as_deref(), Some("false"));
    assert_eq!(method.return_type, TypeRef::Primitive(PrimitiveType::Str));
}

#[test]
fn rejects_legacy_builtin_type_names() {
    let source = "def legacy(name: String, loud: Boolean = false) -> Integer\n";
    let tokens = lexer::tokenize(source, "src/legacy.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 0);
    assert_eq!(unit.diagnostics.diagnostics.len(), 3);
    assert!(unit
        .diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == "BIX100"));
    assert!(!unit
        .diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "BIX001"));
    assert_eq!(
        unit.diagnostics.diagnostics[0].suggestion.as_deref(),
        Some("Use `Str` instead.")
    );
    assert_eq!(
        unit.diagnostics.diagnostics[1].suggestion.as_deref(),
        Some("Use `Bool` instead.")
    );
    assert_eq!(
        unit.diagnostics.diagnostics[2].suggestion.as_deref(),
        Some("Use `Int` instead.")
    );
    assert!(!unit
        .diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Expected return type after `->`."));
}

#[test]
fn parses_namespaced_paths_with_legacy_suffixes() {
    let source = "def wrap(value: Foo::String) -> Bar::Integer\n";
    let tokens = lexer::tokenize(source, "src/wrap.bixb");
    let unit = parser::parse(tokens);

    assert_eq!(unit.typed_methods.len(), 1);
    assert!(unit.diagnostics.diagnostics.is_empty());
    let method = &unit.typed_methods[0];
    assert_eq!(
        method.params[0].type_ref,
        TypeRef::Path(vec!["Foo".to_string(), "String".to_string()])
    );
    assert_eq!(
        method.return_type,
        TypeRef::Path(vec!["Bar".to_string(), "Integer".to_string()])
    );
}
