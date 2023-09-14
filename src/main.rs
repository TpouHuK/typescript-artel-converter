use std::str::FromStr;

use tree_sitter::Node;
use tree_sitter::Parser;
use itertools::Itertools;

enum AlProgram {
    LexicalDeclaration(AlLexicalDeclaration),
}

enum AlLexicalDeclType {
    CONST,
    LET,
}

impl AlLexicalDeclType {
    fn new(s: &str) -> Self {
        match s {
            "let" => AlLexicalDeclType::LET,
            "const" => AlLexicalDeclType::CONST,
            // `var` can happen, ignore for now, TODO later
            _ => unreachable!("neither let or const found, maybe var? #TODO"),
        }
    }

    fn to_alstr(&self) -> &str {
        match self {
            AlLexicalDeclType::LET => "пусть",
            AlLexicalDeclType::CONST => "конст",
        }
    }
}

struct AlLexicalDeclaration {
    decl_type: AlLexicalDeclType,
    ident: String,
    var_type: String, // TODO: var_type
    value: String,    // TODO expression
}

struct AlFunctionCall {
    callee: String,         // TODO, can also be expession
    arguments: Vec<String>, // todo: Vec of expression
}

fn read_example(filename: &str) -> String {
    std::fs::read_to_string(format!("./examples/{filename}"))
        .unwrap_or_else(|_| std::fs::read_to_string(format!("./{filename}")).unwrap())
}

fn walk_tree_recursively_dbg(source: &str, node: &Node, ident: usize) {
    let mut cursor = node.walk();
    println!("{:\t<1$} {node:?}", "", ident);
    println!(
        "{:\t<1$} {text:?}",
        "",
        ident,
        text = node.utf8_text(source.as_bytes()).unwrap()
    );
    println!("{:\t<1$} {kind}", "", ident, kind = node.kind());
    println!("{:\t<1$} ^^^^^^^", "", ident);

    if cursor.goto_first_child() {
        loop {
            println!(
                "{:\t<1$} FIELD_NAME:{field_name:?}",
                "",
                ident,
                field_name = cursor.field_name()
            );
            walk_tree_recursively_dbg(source, &cursor.node(), ident + 1);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// This functions walks the syntax tree of TypeScript and returns converted nodes to artel.
/// In the future it shoudl return it's own *ArtelProgram* which then should be stringified
/// String for now...
fn walk_tree(source: &str, node: &Node, dbg_ident: usize) -> String {
    let mut cursor = node.walk();

    let mut output = String::new();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "lexical_declaration" {
            output.push_str(&parse_lexical_declaration(source, child, dbg_ident));
            output.push_str("\n");
        }
    }
    output
}

fn parse_lexical_declaration(source: &str, node: Node, dbg_ident: usize) -> String {
    let mut cursor = node.walk();
    let mut buffer = String::new();

    let decl_type = {
        let const_or_let_str = node
            .children_by_field_name("kind", &mut cursor)
            .next()
            .expect("variable declaration statement was not found")
            .kind();
        AlLexicalDeclType::new(const_or_let_str)
    };

    let mut al_declarations = Vec::<String>::new();

    for declaration in node.children(&mut cursor) {
        if declaration.kind() == "variable_declarator" {
            let mut cursor = declaration.walk();
            let ident = declaration
                .children_by_field_name("name", &mut cursor)
                .next()
                .expect("variable name")
                .utf8_text(&source.as_bytes())
                .unwrap();
            let var_type = {
                // This node will containt `:` and type together, we need only type name
                let var_type = declaration
                    .children_by_field_name("type", &mut cursor)
                    .next()
                    .expect("variable type");
                let var_type_name = var_type
                    .children(&mut cursor)
                    .nth(1)
                    .expect("variable type")
                    .utf8_text(&source.as_bytes())
                    .unwrap();
                var_type_name
            };
            let value = {
                // TODO, parse correct value
                match declaration
                    .children_by_field_name("value", &mut cursor)
                    .next()
                {
                    Some(node) => Some(node.utf8_text(&source.as_bytes()).unwrap()),
                    None => None,
                }
            };

            al_declarations.push(match value {
                Some(value) => format!("{ident}: {var_type} = {value}"),
                None => format!("{ident}: {var_type}"),
            });
        }
    }

    if al_declarations.len() == 1 {
        format!("{decl_type} {stmnt}", decl_type=decl_type.to_alstr(), stmnt = al_declarations[0])
    } else {
        let body: String = al_declarations
            .into_iter()
            .map(|mut s| { s.insert_str(0, "    "); s} )
            .intersperse_with(|| { String::from("\n") })
            .collect();
        format!(
            "{decl_type} {{\n{body}\n}}",
            decl_type = decl_type.to_alstr(),
            )
    }
}

fn convert_ts(code: &str) -> () {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading typescript grammar");

    let parsed = parser.parse(code, None).unwrap();
    let root = parsed.root_node();
    dbg!(root.to_sexp());
    walk_tree_recursively_dbg(&code, &root, 0);

    let res = walk_tree(&code, &root, 0);
    println!("{code}");
    println!("---");
    println!("{res}");

    assert!(!root.has_error());
}

fn main() {
    let code = read_example("multiple_declarations.ts");
    convert_ts(&code);
}
