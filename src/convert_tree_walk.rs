use super::artel_nodes::*;
use itertools::Itertools;
use std::string::String;
use tree_sitter::Node;

/// This functions walks the syntax tree of TypeScript and returns converted nodes to artel.
/// In the future it shoudl return it's own *ArtelProgram* which then should be stringified
/// String for now...
pub fn walk_tree(source: &str, node: &Node) -> String {
    let mut cursor = node.walk();

    let mut statements = Vec::new();
    for child in node.named_children(&mut cursor) {
        statements.push(parse_statement(source, &child));
    }
    statements.join("\n").to_owned()
}

pub fn parse_statement(source: &str, node: &Node) -> String {
    let mut output = match node.kind() {
        "lexical_declaration" => parse_lexical_declaration(source, node),
        "expression_statement" => parse_expression(source, &node.named_child(0).unwrap()),
        "if_statement" => parse_if_statement(source, node),
        "statement_block" => parse_statement_block(source, node),
        "function_declaration" => parse_function_declaration(source, node),
        "return_statement" => parse_return_statement(source, node),
        "class_declaration" => parse_class_declaration(source, node),
        "comment" => node.utf8_text(&source.as_bytes()).unwrap().to_owned(),
        "while_statement" => parse_while_statement(source, node),
        "do_statement" => parse_do_statement(source, node),
        "break_statement" => String::from("прервать цикл"),
        "continue_statement" => String::from("продолжить цикл"),
        "enum_declaration" => parse_enum_declaration(source, node),
        "type_alias_declaration" => parse_type_alias_declaration(source, node),
        _ => {
            unimplemented!("{} is unimplemented", node.kind())
        }
    };

    output
}

fn parse_type_alias_declaration(source: &str, node: &Node) -> String {
    let name_str = node
        .child_by_field_name("type_identifier")
        .unwrap()
        .utf8_text(source.as_bytes())
        .unwrap();

    todo!()
}

fn parse_enum_declaration(source: &str, node: &Node) -> String {
    let name = node.child_by_field_name("name").unwrap();
    let name_str = name.utf8_text(source.as_bytes()).unwrap();
    let body = node.child_by_field_name("body").unwrap();
    let mut cursor = body.walk();

    let mut items = Vec::new();
    for item in body.named_children(&mut cursor) {
        match item.kind() {
            "enum_assignment" => {
                let item_name = item.child_by_field_name("name").unwrap();
                let item_name_str = item_name.utf8_text(source.as_bytes()).unwrap();

                let value = item.child_by_field_name("value").unwrap();
                let value_str = value.utf8_text(source.as_bytes()).unwrap();

                items.push(format!("{item_name_str} = {value_str}"));
            }
            "property_identifier" => {
                let item_name_str = item.utf8_text(source.as_bytes()).unwrap();
                items.push(item_name_str.to_owned());
            }
            _ => {
                unreachable!("unexpected {} in enum", item.kind())
            }
        }
    }
    let body_str = items.join("\n");
    format!("тип {name_str} = вариант {{\n{body_str}\n}}")
}

fn parse_do_statement(source: &str, node: &Node) -> String {
    let condition = node.child_by_field_name("condition").unwrap();
    // Condition is parenthesized expression
    // But in artel we do not need `()` in if
    // So we extract single expression from parenths
    assert_eq!(condition.named_child_count(), 1);

    let condition = condition.named_child(0).unwrap();
    let body = node.child_by_field_name("body").unwrap();
    let condition_str = parse_expression(source, &condition);
    let body_str = parse_statement_block(source, &body);
    format!("выполнить\n{body_str}\nповтор пока {condition_str}")
}

fn parse_while_statement(source: &str, node: &Node) -> String {
    let condition = node.child_by_field_name("condition").unwrap();
    // Condition is parenthesized expression
    // But in artel we do not need `()` in if
    // So we extract single expression from parenths
    assert_eq!(condition.named_child_count(), 1);
    let condition = condition.named_child(0).unwrap();

    let body = node.child_by_field_name("body").unwrap();
    let condition_str = parse_expression(source, &condition);
    let body_str = parse_statement_block(source, &body);
    format!("пока {condition_str} выполнить\n{body_str}")
}

fn parse_new_expression(source: &str, node: &Node) -> String {
    return node.utf8_text(&source.as_bytes()).unwrap().to_owned();
}

fn parse_class_declaration(source: &str, node: &Node) -> String {
    let name = node.child_by_field_name("name").unwrap();

    let name_str = name.utf8_text(&source.as_bytes()).unwrap();
    let inheritance = {
        let next = name.next_sibling().unwrap();
        let inheritance = if next.kind() == "class_heritage" {
            let mut cursor = next.walk();
            let mut extends_clause = None;
            for clause in next.named_children(&mut cursor) {
                if clause.kind() == "extends_clause" {
                    extends_clause = Some(
                        clause
                            .child_by_field_name("value")
                            .unwrap()
                            .utf8_text(&source.as_bytes())
                            .unwrap(),
                    )
                }
                if clause.kind() == "implements_clause" {
                    unimplemented!("implements clause is not implemented");
                }
            }
            extends_clause
        } else {
            None
        };
        inheritance
    };

    let body = node.child_by_field_name("body").unwrap();
    let mut cursor = body.walk();

    let mut definitions = Vec::new();

    for definition in body.named_children(&mut cursor) {
        let definition_str = match definition.kind() {
            "public_field_definition" => {
                let name_str = definition
                    .child_by_field_name("name")
                    .unwrap()
                    .utf8_text(&source.as_bytes())
                    .unwrap();
                let type_str = definition
                    .child_by_field_name("type")
                    .unwrap()
                    .named_child(0)
                    .unwrap()
                    .utf8_text(&source.as_bytes())
                    .unwrap();
                format!("{name_str}: {type_str}")
            }
            "method_definition" => parse_method_definition(source, &definition),
            _ => {
                unimplemented!("{} is not implemented in a class body", definition.kind())
            }
        };
        definitions.push(definition_str);
        dbg!(definition);
    }
    let body_str = definitions.join("\n\n");

    let inheritance_str =
        inheritance.map_or_else(|| String::from(""), |i| format!("на основе {i} "));

    format!("тип {name_str} = объект {inheritance_str}\n{{\n{body_str}}}")
}

// TODO arguments/parameters, inconsistent naming
fn parse_method_definition(source: &str, node: &Node) -> String {
    let name_str = node
        .child_by_field_name("name")
        .unwrap()
        .utf8_text(&source.as_bytes())
        .unwrap();
    let parameters = node.child_by_field_name("parameters").unwrap();

    let mut cursor = parameters.walk();

    let mut arguments = Vec::new();
    for param in parameters.named_children(&mut cursor) {
        if param.kind() == "optional_parameter" {
            unimplemented!("optional_parameter");
        }
        let arg_name = param
            .child_by_field_name("pattern")
            .unwrap()
            .utf8_text(&source.as_bytes())
            .unwrap();
        let arg_type = param
            .child_by_field_name("type")
            .unwrap()
            .named_child(0)
            .unwrap()
            .utf8_text(&source.as_bytes())
            .unwrap();

        let value = param.child_by_field_name("value");
        let value_str = value.map(|v| parse_expression(&source, &v));

        arguments.push(if let Some(value_str) = value_str {
            format!("{arg_name}: {arg_type} = {value_str}")
        } else {
            format!("{arg_name}: {arg_type}")
        });
    }
    let arguments_str = arguments.join(", ");
    let body = node.child_by_field_name("body").unwrap();
    assert_eq!(body.kind(), "statement_block");
    let body_str = parse_statement_block(source, &body);
    if name_str == "constructor" {
        format!("при создании({arguments_str})\n{body_str}")
    } else {
        format!("операция {name_str}({arguments_str})\n{body_str}")
    }
}

fn parse_return_statement(source: &str, node: &Node) -> String {
    let Some(expression) = node.named_child(0) else {
        return String::from("возврат");
    };
    let expression_str = parse_expression(source, &expression);
    format!("возврат {expression_str}")
}

fn parse_function_declaration(source: &str, node: &Node) -> String {
    let name_str = node
        .child_by_field_name("name")
        .expect("No function name")
        .utf8_text(&source.as_bytes())
        .unwrap();
    let parameters = node.child_by_field_name("parameters").unwrap();

    let mut cursor = parameters.walk();

    let mut arguments = Vec::new();
    for param in parameters.named_children(&mut cursor) {
        if param.kind() == "optional_parameter" {
            unimplemented!("optional_parameter");
        }
        let arg_name = param
            .child_by_field_name("pattern")
            .unwrap()
            .utf8_text(&source.as_bytes())
            .unwrap();
        let arg_type = param
            .child_by_field_name("type")
            .unwrap()
            .named_child(0)
            .unwrap()
            .utf8_text(&source.as_bytes())
            .unwrap();

        let value = param.child_by_field_name("value");
        let value_str = value.map(|v| parse_expression(&source, &v));

        arguments.push(if let Some(value_str) = value_str {
            format!("{arg_name}: {arg_type} = {value_str}")
        } else {
            format!("{arg_name}: {arg_type}")
        });
    }

    let body = node.child_by_field_name("body").unwrap();
    assert_eq!(body.kind(), "statement_block");
    let body_str = parse_statement_block(source, &body);

    let arguments_str = arguments.join(", ");
    format!("операция {name_str}({arguments_str})\n{body_str}")
}

fn parse_statement_block(source: &str, child: &Node) -> String {
    let mut cursor = child.walk();
    let mut statements = Vec::new();
    for node in child.named_children(&mut cursor) {
        statements.push(parse_statement(source, &node));
    }
    statements
        .iter_mut()
        .map(|s| s.insert_str(0, "    "))
        .for_each(drop);
    format!("{{\n{body}\n}}", body = statements.join("\n"))
}

fn parse_if_statement(source: &str, child: &Node) -> String {
    let condition = child.child_by_field_name("condition").unwrap();

    // Condition is a parenthesized expression in javascript
    // but there are no parenthesis in Artel
    let condition_expression_node = condition.named_child(0).unwrap();
    let condition_expression_str = parse_expression(&source, &condition_expression_node);

    let consequence = child.child_by_field_name("consequence").unwrap();
    let consequence_str = parse_statement(source, &consequence);

    let alternative = child.child_by_field_name("alternative");
    let alternative_str = if let Some(alternative) = alternative {
        Some(parse_statement(
            source,
            &alternative.named_child(0).unwrap(),
        ))
    } else {
        None
    };

    if let Some(alternative_str) = alternative_str {
        format!("если {condition_expression_str} тогда\n    {consequence_str}\nиначе\n    {alternative_str}")
    } else {
        format!("если {condition_expression_str} тогда\n    {consequence_str}")
    }
}

fn parse_expression(source: &str, node: &Node) -> String {
    let raw_text = || node.utf8_text(&source.as_bytes()).unwrap().to_owned();
    match node.kind() {
        "number" => raw_text(),
        "string" => raw_text(),
        "call_expression" => raw_text(), // TODO
        "binary_expression" => parse_binary_expression(source, node),
        "unary_expression" => parse_unary_expression(source, node),
        "true" => String::from("Да"),
        "false" => String::from("Нет"),
        "identifier" => raw_text(),
        "assignment_expression" => parse_assignment_expression(source, node),
        "parenthesized_expression" => parse_parenthesized_expression(source, node),
        "member_expression" => raw_text(), // TODO
        "new_expression" => parse_new_expression(source, node),
        "object" => parse_object(source, node),
        _ => todo!("ну надо сделать ещё палучаицца, {}", node.kind()),
    }
}

fn parse_object(source: &str, node: &Node) -> String {
    let mut cursor = node.walk();
    let mut node_pairs = Vec::new();
    for node_pair in node.named_children(&mut cursor) {
        let key = node_pair.child_by_field_name("key").unwrap();
        let key_str = key.utf8_text(source.as_bytes()).unwrap();

        let value = node_pair.child_by_field_name("value").unwrap();
        let value_str = parse_expression(source, &value);
        node_pairs.push(format!("{key_str} = {value_str}"));
    }

    let object_body = node_pairs.join(", ");
    format!("[{object_body}]")
}

fn parse_parenthesized_expression(source: &str, node: &Node) -> String {
    let inner_expr = node.named_child(0).unwrap();
    let inner_expr_str = parse_expression(source, &inner_expr);
    return format!("({inner_expr_str})");
}

/// TODO
fn parse_lvalue(source: &str, node: &Node) -> String {
    return node.utf8_text(&source.as_bytes()).unwrap().to_owned();
}

fn parse_assignment_expression(source: &str, node: &Node) -> String {
    let left = node.child_by_field_name("left").unwrap();
    let left_str = parse_lvalue(source, &left);

    let right = node.child_by_field_name("right").unwrap();
    let right_str = parse_expression(source, &right);
    format!("{left_str} = {right_str}")
}

fn parse_binary_expression(source: &str, node: &Node) -> String {
    let left_node = node.child_by_field_name("left").unwrap();
    let sign = node.child_by_field_name("operator").unwrap();
    let right_node = node.child_by_field_name("right").unwrap();

    // (may be troubles with sign)
    format!(
        "{left} {sign} {right}",
        left = parse_expression(source, &left_node),
        sign = sign.utf8_text(&source.as_bytes()).unwrap(),
        right = parse_expression(source, &right_node)
    )
}

fn parse_unary_expression(source: &str, node: &Node) -> String {
    let operator = node.child_by_field_name("operator").unwrap();
    let argument = node.child_by_field_name("argument").unwrap();
    format!(
        "{operator_str}{expr}",
        operator_str = operator.utf8_text(&source.as_bytes()).unwrap(),
        expr = parse_expression(source, &argument)
    )
}

fn parse_lexical_declaration(source: &str, node: &Node) -> String {
    let mut cursor = node.walk();

    let decl_type = {
        let const_or_let_str = node
            .child_by_field_name("kind")
            .expect("variable declaration statement was not found")
            .kind();
        AlLexicalDeclType::new(const_or_let_str)
    };

    let mut al_declarations = Vec::<String>::new();

    for declaration in node.children(&mut cursor) {
        if declaration.kind() == "variable_declarator" {
            let ident = declaration
                .child_by_field_name("name")
                .expect("variable name")
                .utf8_text(&source.as_bytes())
                .unwrap();
            let var_type = {
                let var_type_name = declaration
                    .child_by_field_name("type")
                    .expect("variable type")
                    .named_child(0)
                    .expect("variable type")
                    .utf8_text(&source.as_bytes())
                    .unwrap();
                var_type_name
            };
            let value = {
                // TODO, parse correct value
                match declaration.child_by_field_name("value") {
                    Some(node) => Some(parse_expression(source, &node)),
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
        al_declarations
            .iter_mut()
            .map(|s| s.insert_str(0, "    "))
            .for_each(drop);

        format!(
            "{decl_type} {{\n{body}\n}}",
            decl_type = decl_type.to_alstr(),
            body = al_declarations.join("\n"),
        )
    }
}
