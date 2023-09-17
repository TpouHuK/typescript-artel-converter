use super::artel_nodes::*;
use itertools::Itertools;
use std::string::String;
use tree_sitter::Node;

/// This functions walks the syntax tree of TypeScript and returns converted nodes to artel.
/// In the future it shoudl return it's own *ArtelProgram* which then should be stringified
/// String for now...
pub fn walk_tree(source: &str, node: &Node) -> String {
    walk_statements(source, node)
}

pub fn walk_statements(source: &str, node: &Node) -> String {
    let mut cursor = node.walk();

    let mut output = String::new();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "lexical_declaration" {
            output.push_str(&parse_lexical_declaration(source, child));
            output.push_str("\n");
        }
        if child.kind() == "expression_statement" {
            // In the expression statement theres actual expression
            output.push_str(&parse_expression(source, child.child(0).unwrap()));
            output.push_str("\n");
        }
        if child.kind() == "if_statement" {
            output.push_str(&parse_if_statement(source, child));
            todo!();
        }
        if child.kind() == "unary_expression" {
            output.push_str(&parse_unary_expression(source, child));
            output.push_str("\n")
        }
    }
    output
}

fn parse_if_statement(source: &str, child: Node) -> String {
    todo!()
}

fn parse_expression(source: &str, node: Node) -> String {
    match node.kind() {
        "number" => node.utf8_text(&source.as_bytes()).unwrap().to_owned(),
        "string" => node.utf8_text(&source.as_bytes()).unwrap().to_owned(),
        "call_expression" => node.utf8_text(&source.as_bytes()).unwrap().to_owned(),
        "binary_expression" => parse_binary_expression(source, node),
        "unary_expression" => parse_unary_expression(source, node),
        _ => todo!("ну надо сделать ещё палучаицца"),
    }
}

fn parse_binary_expression(source: &str, node: Node) -> String {
    let mut cursor = node.walk();
    let left_node = node
        .children_by_field_name("left", &mut cursor)
        .next()
        .unwrap();
    let sign = node
        .children_by_field_name("operator", &mut cursor)
        .next()
        .unwrap();
    let right_node = node
        .children_by_field_name("right", &mut cursor)
        .next()
        .unwrap();

    // (may be troubles with sign)
    format!(
        "{left} {sign} {right}",
        left = parse_expression(source, left_node),
        sign = sign.utf8_text(&source.as_bytes()).unwrap(),
        right = parse_expression(source, right_node)
    )
}

fn parse_unary_expression(source: &str, node: Node) -> String {
    let mut cursor = node.walk();

    let operator = node
        .children_by_field_name("operator", &mut cursor)
        .next()
        .unwrap();
    let argument = node
        .children_by_field_name("argument", &mut cursor)
        .next()
        .unwrap();

    format!(
        "{operator_str}{expr}",
        operator_str = operator.utf8_text(&source.as_bytes()).unwrap(),
        expr = parse_expression(source, argument)
    )
}

fn parse_lexical_declaration(source: &str, node: Node) -> String {
    let mut cursor = node.walk();

    let decl_type = {
        let const_or_let_str = node
            .children_by_field_name("kind", &mut cursor)
            .nth(0)
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
                    .child(1)
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
                    Some(node) => Some(parse_expression(source, node)),
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
        format!(
            "{decl_type} {stmnt}",
            decl_type = decl_type.to_alstr(),
            stmnt = al_declarations[0]
        )
    } else {
        #[allow(unstable_name_collisions)] // for interspese_with
        let body: String = al_declarations
            .into_iter()
            .map(|mut s| {
                s.insert_str(0, "    ");
                s
            })
            .intersperse_with(|| String::from("\n"))
            .collect();
        format!(
            "{decl_type} {{\n{body}\n}}",
            decl_type = decl_type.to_alstr(),
        )
    }
}
