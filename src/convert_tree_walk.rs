use itertools::Itertools;
use tree_sitter::Node;
use super::artel_nodes::*;

/// This functions walks the syntax tree of TypeScript and returns converted nodes to artel.
/// In the future it shoudl return it's own *ArtelProgram* which then should be stringified
/// String for now...
pub fn walk_tree(source: &str, node: &Node, dbg_ident: usize) -> String {
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

pub fn parse_lexical_declaration(source: &str, node: Node, _dbg_ident: usize) -> String {
    let mut cursor = node.walk();

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
