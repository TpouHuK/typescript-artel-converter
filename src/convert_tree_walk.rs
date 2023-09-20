use super::artel_nodes::*;
use std::string::String;
use tree_sitter::Node;

/// This functions walks the syntax tree of TypeScript and returns converted nodes to artel.
/// It returns sequence of statements, which is something like AST, and that can be converted into
/// Artel API defitions directly.
pub fn walk_tree(source: &str, node: &Node) -> ArtelProgram {
    let mut statements = Vec::new();

    for child in node.named_children(&mut node.walk()) {
        if let Some(stmt) = parse_statement(source, &child) {
            statements.push(stmt);
        }
    }

    statements
}

pub fn parse_statement(source: &str, node: &Node) -> Option<ArtelStatement> {
    let output = match node.kind() {
        "lexical_declaration" => Some(parse_lexical_declaration(source, node)),
        //"expression_statement" => { parse_expression(source, &node.named_child(0).unwrap())},
        //"if_statement" => parse_if_statement(source, node),
        //"statement_block" => parse_statement_block(source, node),
        "function_declaration" => Some(ArtelStatement::FunctionDeclaration(
            parse_function_declaration(source, node),
        )),
        //"return_statement" => parse_return_statement(source, node),
        "class_declaration" => {
            Some(ArtelStatement::ClassDeclaration(parse_class_declaration(
                source, node, /*is abstract*/ false,
            )))
        }
        "abstract_class_declaration" => {
            Some(ArtelStatement::ClassDeclaration(parse_class_declaration(
                source, node, /*is abstract*/ true,
            )))
        }
        "comment" => Some(ArtelStatement::Comment(
            node.utf8_text(&source.as_bytes()).unwrap().to_owned(),
        )),
        //"while_statement" => parse_while_statement(source, node),
        //"do_statement" => parse_do_statement(source, node),
        //"break_statement" => String::from("прервать цикл"),
        //"continue_statement" => String::from("продолжить цикл"),
        "enum_declaration" => Some(ArtelStatement::EnumDeclaration(parse_enum_declaration(
            source, node,
        ))),
        "type_alias_declaration" => Some(ArtelStatement::TypeAliasDeclaration(
            parse_type_alias_declaration(source, node),
        )),
        "export_statement" => {
            let declaration = node.child_by_field_name("declaration").unwrap();
            Some(ArtelStatement::ExportStatement(
                parse_statement(source, &declaration).unwrap().into(),
            ))
        }
        "interface_declaration" => Some(ArtelStatement::InterfaceDeclaration(parse_interface(
            source, node,
        ))),
        "import_statement" => None,
        _ => {
            unimplemented!("{} is unimplemented", node.kind())
        }
    };

    output
}

fn parse_interface(source: &str, node: &Node) -> ArtelInterfaceDeclaration {
    let interface_name_ident = {
        let name_str = node
            .child_by_field_name("name")
            .expect("No function name")
            .utf8_text(&source.as_bytes())
            .unwrap();
        ArtelIdentifier::new(name_str)
    };

    let type_parameters = 'type_parameters: {
        let Some(parameters_node) = node.child_by_field_name("type_parameters") else { break 'type_parameters Vec::new() };
        parse_type_parameters(source, &parameters_node)
    };

    let mut vec = Vec::new();
    let body = node.child_by_field_name("body").unwrap();
    for element in body.named_children(&mut body.walk()) {
        match element.kind() {
            "property_signature" => {
                let property_name = element
                    .child_by_field_name("name")
                    .expect("No function name")
                    .utf8_text(&source.as_bytes())
                    .unwrap();
                let property_ident = ArtelIdentifier::new(property_name);
                let property_type =
                    parse_type(source, &element.child_by_field_name("type").unwrap());

                // Comments will break this, FIXME 🤬
                let readonly = element.child(0).unwrap().kind() == "readonly";
                vec.push(ArtelInterfaceMember::Property(ArtelProperty::new(
                    readonly,
                    property_ident,
                    property_type,
                )))
            }
            "method_signature" => {
                let method = parse_function_declaration(source, &element);
                vec.push(ArtelInterfaceMember::Method(method))
            }
            "comment" => { // TODO
            }
            _ => {
                unimplemented!("{}", element.kind())
            }
        }
    }

    dbg!(ArtelInterfaceDeclaration::new(
        interface_name_ident,
        type_parameters,
        vec
    ))
}

fn parse_type_alias_declaration(source: &str, node: &Node) -> ArtelTypeAliasDeclaration {
    //  type `name_str``<type_args>` = `value`
    //  for ex:
    //      type NetworkState<T> = NetworkLoadingState | NetworkFailedState | NetworkSuccessState
    //      `name_str` = NetworkState
    //      `type_args` = <T>
    //      `value` = NetworkLoadingState | NetworkFailedState | NetworkSuccessState
    let type_identifier = {
        let name_str = node
            .child_by_field_name("name")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap();
        ArtelIdentifier::new(name_str)
    };

    let type_parameters = 'type_params: {
        let Some(parameters_node) = node.child_by_field_name("type_parameters") else {
            break 'type_params Vec::new();
        };
        parse_type_parameters(source, &parameters_node)
    };

    let alias = type_identifier;
    let value = node.child_by_field_name("value").unwrap();
    let value = parse_type(source, &value);

    dbg!(ArtelTypeAliasDeclaration::new(
        alias,
        type_parameters,
        value
    ))
}

fn parse_type(source: &str, node: &Node) -> ArtelType {
    // what did i just do
    let _temp;
    let node = if node.kind() == "type_annotation" {
        _temp = node.named_child(0).unwrap();
        &_temp
    } else {
        node
    };

    let mut vec = Vec::new();
    parse_type_inner(source, node, &mut vec);

    assert!(!vec.is_empty());
    ArtelType(vec)
}

fn parse_type_parameters(source: &str, parameters_node: &Node) -> Vec<ArtelTypeParameter> {
    let mut vec = Vec::new();
    let mut cursor = parameters_node.walk();
    for type_param in parameters_node.named_children(&mut cursor) {
        let type_param_name = type_param.child_by_field_name("name").unwrap();
        let type_param_name_str = type_param_name.utf8_text(source.as_bytes()).unwrap();
        let param_identfier = ArtelIdentifier::new(type_param_name_str);

        let constraint = 'constraint: {
            let Some(constraint) = type_param.child_by_field_name("constraint") else { break 'constraint None };
            Some(parse_type(source, &constraint.named_child(0).unwrap()))
        };
        let default = 'default: {
            let Some(constraint) = type_param.child_by_field_name("value") else { break 'default None };
            Some(parse_type(source, &constraint.named_child(0).unwrap()))
        };

        vec.push(ArtelTypeParameter::new(
            param_identfier,
            constraint,
            default,
        ));
    }
    vec
}

fn parse_type_inner(source: &str, node: &Node, vec: &mut Vec<ArtelPrimaryType>) {
    match node.kind() {
        "union_type" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                parse_type_inner(source, &child, vec);
            }
        }
        "type_identifier" => {
            let name = node.utf8_text(source.as_bytes()).unwrap();
            let r#type = ArtelPrimaryType::TypeReference(ArtelTypeReference::new(
                ArtelIdentifier::new(name),
                Vec::new(),
            ));
            vec.push(r#type);
        }
        "predefined_type" => {
            let predefined_type_str = node.utf8_text(source.as_bytes()).unwrap();
            let r#type = ArtelPrimaryType::PredefinedType(predefined_type_str.into());
            vec.push(r#type);
        }
        "literal_type" => {
            let node = node.named_child(0).unwrap();
            let r#type = match node.kind() {
                "string" => {
                    let string_fragment = node
                        .named_child(0)
                        .unwrap()
                        .utf8_text(source.as_bytes())
                        .unwrap();
                    ArtelLiteralType::String(string_fragment.to_owned())
                }
                "number" => {
                    let number = node.utf8_text(source.as_bytes()).unwrap();
                    ArtelLiteralType::Number(number.to_owned())
                }
                "null" => ArtelLiteralType::Null,
                "true" => ArtelLiteralType::Boolean(true),
                "false" => ArtelLiteralType::Boolean(false),
                "undefined" => ArtelLiteralType::Undefined,
                _ => {
                    unreachable!()
                }
            };
            vec.push(ArtelPrimaryType::LiteralType(r#type));
        }
        "generic_type" => {
            let name = node
                .child_by_field_name("name")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_owned();
            let generic_params = node.child_by_field_name("type_arguments").unwrap();
            let mut cursor = generic_params.walk();
            let mut vec2 = Vec::new();
            for generic_param in generic_params.named_children(&mut cursor) {
                vec2.push(parse_type(source, &generic_param));
            }
            vec.push(ArtelPrimaryType::TypeReference(ArtelTypeReference::new(
                ArtelIdentifier::new(name),
                vec2,
            )));
        }
        "index_type_query" => vec.push(ArtelPrimaryType::UnsupportedAny),
        "function_type" => {
            let type_parameters = 'type_parameters: {
                let Some(parameters_node) = node.child_by_field_name("type_parameters") else { break 'type_parameters Vec::new() };
                parse_type_parameters(source, &parameters_node)
            };

            let arguments =
                parse_formal_arguments(&source, &node.child_by_field_name("parameters").unwrap());

            let return_type = 'return_type: {
                let Some(return_type) = node.child_by_field_name("return_type") else { break 'return_type None };
                Some(parse_type(source, &return_type))
            };
            vec.push(ArtelPrimaryType::FunctionType(
                ArtelFunctionDeclaration::new(
                    false,
                    ArtelIdentifier::new("$FUNCTION_TYPE$"),
                    type_parameters,
                    arguments,
                    return_type,
                )
                .into(),
            ));
        }
        "array_type" => {
            vec.push(ArtelPrimaryType::ArrayType(
                parse_type(source, &node.named_child(0).unwrap()).into(),
            ));
        }
        _ => {
            unimplemented!("{} is not implemented", node.kind())
        }
    };
}

fn parse_enum_declaration(source: &str, node: &Node) -> EnumDeclaration {
    let name = node.child_by_field_name("name").unwrap();
    let name_str = name.utf8_text(source.as_bytes()).unwrap();
    let name_ident = ArtelIdentifier::new(name_str);

    let body = node.child_by_field_name("body").unwrap();
    let mut cursor = body.walk();

    let mut items = Vec::new();
    for item in body.named_children(&mut cursor) {
        match item.kind() {
            "enum_assignment" => {
                let item_name = item.child_by_field_name("name").unwrap();
                let item_name_str = item_name.utf8_text(source.as_bytes()).unwrap();
                let item_ident = ArtelIdentifier::new(item_name_str);

                let value = item.child_by_field_name("value").unwrap();
                let value_str = value.utf8_text(source.as_bytes()).unwrap().to_owned();

                items.push(EnumItem::new(item_ident, Some(value_str)));
            }
            "property_identifier" => {
                let item_name_str = item.utf8_text(source.as_bytes()).unwrap();
                let item_ident = ArtelIdentifier::new(item_name_str);
                items.push(EnumItem::new(item_ident, None));
            }
            _ => {
                unreachable!("unexpected {} in enum", item.kind())
            }
        }
    }

    EnumDeclaration::new(name_ident, items)
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

fn parse_class_declaration(source: &str, node: &Node, is_abstract: bool) -> ArtelClassDeclaration {
    let name = node.child_by_field_name("name").unwrap();
    let name_ident = ArtelIdentifier::new(name.utf8_text(&source.as_bytes()).unwrap());

    let type_parameters = 'type_parameters: {
        let Some(parameters_node) = node.child_by_field_name("type_parameters") else { break 'type_parameters Vec::new() };
        parse_type_parameters(source, &parameters_node)
    };

    let (extends_clause, implements_clause) = {
        let next = if type_parameters.is_empty() {
            name.next_sibling().unwrap()
        } else {
            node.child_by_field_name("type_parameters")
                .unwrap()
                .next_sibling()
                .unwrap()
        };

        if next.kind() == "class_heritage" {
            let mut extends_clause = None;
            let mut implements_clause = Vec::new();

            for clause in next.named_children(&mut next.walk()) {
                if clause.kind() == "extends_clause" {
                    let type_parameters = 'type_parameters: {
                        let Some(parameters_node) = node.child_by_field_name("type_parameters") else { break 'type_parameters Vec::new() };
                        parse_type_parameters(source, &parameters_node)
                    };
                    extends_clause = Some((
                        ArtelIdentifier::new(
                            clause
                                .child_by_field_name("value")
                                .unwrap()
                                .utf8_text(&source.as_bytes())
                                .unwrap(),
                        ),
                        type_parameters,
                    ));
                }

                if clause.kind() == "implements_clause" {
                    for r#type in clause.named_children(&mut clause.walk()) {
                        implements_clause.push(parse_type(source, &r#type));
                    }
                }
            }

            (extends_clause, implements_clause)
        } else {
            (None, Vec::new())
        }
    };

    let body = node.child_by_field_name("body").unwrap();
    let mut cursor = body.walk();

    let mut definitions = Vec::new();
    for definition in body.named_children(&mut cursor) {
        match definition.kind() {
            "public_field_definition" => {
                let mut modifiers_cursor = definition.walk();
                assert!(modifiers_cursor.goto_first_child());

                let mut access_modifier = ArtelAccessModifier::Default;
                let mut is_static = false;
                let mut is_readonly = false;
                let mut is_abstract = false;

                while modifiers_cursor.field_name() != Some("name") {
                    match modifiers_cursor.node().kind() {
                        "accessibility_modifier" => {
                            access_modifier = ArtelAccessModifier::from(
                                modifiers_cursor
                                    .node()
                                    .utf8_text(source.as_bytes())
                                    .unwrap(),
                            )
                        }
                        "static" => {
                            is_static = true;
                        }
                        "abstract" => {
                            is_abstract = true;
                        }
                        "readonly" => {
                            is_readonly = true;
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                    assert!(modifiers_cursor.goto_next_sibling());
                }

                let is_optional = {
                    modifiers_cursor.goto_next_sibling() && modifiers_cursor.node().kind() == "?"
                };

                let name_str = definition
                    .child_by_field_name("name")
                    .unwrap()
                    .utf8_text(&source.as_bytes())
                    .unwrap();
                let name_ident = ArtelIdentifier::new(name_str);

                let mut property_type =
                    parse_type(source, &definition.child_by_field_name("type").unwrap());

                if is_optional {
                    property_type
                        .0
                        .push(ArtelPrimaryType::LiteralType(ArtelLiteralType::Undefined));
                }

                let art_prop = ArtelProperty::new(is_readonly, name_ident, property_type);
                let modifiers = ClassMemberModifiers::new(
                    access_modifier,
                    if is_abstract {
                        ArtelModifier::Abstract
                    } else if is_static {
                        ArtelModifier::Static
                    } else {
                        ArtelModifier::None
                    },
                );
                let definition = ArtelClassMember::Property((modifiers, art_prop));
                dbg!(&definition);
                definitions.push(definition);
            }
            "method_definition" => {
                let definition = parse_method_definition(source, &definition);
                dbg!(&definition);
                definitions.push(definition);
            }
            "comment" => {}
            _ => {
                unimplemented!("{} is not implemented in a class body", definition.kind())
            }
        };
    }

    dbg!(ArtelClassDeclaration::new(
        name_ident,
        extends_clause,
        implements_clause,
        is_abstract,
        type_parameters,
        definitions,
    ))
}

// TODO arguments/parameters, inconsistent naming
fn parse_method_definition(source: &str, node: &Node) -> ArtelClassMember {
    let mut modifiers_cursor = node.walk();
    assert!(modifiers_cursor.goto_first_child());

    let mut access_modifier = ArtelAccessModifier::Default;
    let mut is_static = false;
    let mut is_async = false;
    let mut is_abstract = false;
    let mut getter_setter = GetterSetter::None;

    while modifiers_cursor.field_name() != Some("name") {
        match modifiers_cursor.node().kind() {
            "accessibility_modifier" => {
                access_modifier = ArtelAccessModifier::from(
                    modifiers_cursor
                        .node()
                        .utf8_text(source.as_bytes())
                        .unwrap(),
                )
            }
            "static" => {
                is_static = true;
            }
            "abstract" => {
                is_abstract = true;
            }
            "async" => {
                is_async = true;
            }
            "get" => {
                getter_setter = GetterSetter::Get;
            }
            "set" => {
                getter_setter = GetterSetter::Set;
            }
            _ => {
                unimplemented!()
            }
        }
        assert!(modifiers_cursor.goto_next_sibling());
    }

    let modifiers = ClassMemberModifiers::new(
        access_modifier,
        if is_abstract {
            ArtelModifier::Abstract
        } else if is_static {
            ArtelModifier::Static
        } else {
            ArtelModifier::None
        },
    );

    let name_str = node
        .child_by_field_name("name")
        .unwrap()
        .utf8_text(&source.as_bytes())
        .unwrap();
    let name_ident = ArtelIdentifier::new(name_str);

    let parameters = node.child_by_field_name("parameters").unwrap();
    let arguments = parse_formal_arguments(source, &parameters);
    let type_parameters = 'type_parameters: {
        let Some(parameters_node) = node.child_by_field_name("type_parameters") else { break 'type_parameters Vec::new() };
        parse_type_parameters(source, &parameters_node)
    };

    let return_type = 'return_type: {
        let Some(return_type) = node.child_by_field_name("return_type") else { break 'return_type None };
        Some(parse_type(source, &return_type))
    };

    ArtelClassMember::Method((
        modifiers,
        getter_setter,
        ArtelFunctionDeclaration::new(
            is_async,
            name_ident,
            type_parameters,
            arguments,
            return_type,
        ),
    ))
}

fn parse_return_statement(source: &str, node: &Node) -> String {
    let Some(expression) = node.named_child(0) else {
        return String::from("возврат");
    };
    let expression_str = parse_expression(source, &expression);
    format!("возврат {expression_str}")
}

fn parse_function_declaration(source: &str, node: &Node) -> ArtelFunctionDeclaration {
    let r#async = node.child(0).unwrap().kind() == "async";
    let name_ident = {
        let name_str = node
            .child_by_field_name("name")
            .expect("No function name")
            .utf8_text(&source.as_bytes())
            .unwrap();
        ArtelIdentifier::new(name_str)
    };

    let type_parameters = 'type_parameters: {
        let Some(parameters_node) = node.child_by_field_name("type_parameters") else { break 'type_parameters Vec::new() };
        parse_type_parameters(source, &parameters_node)
    };

    let parameters = node.child_by_field_name("parameters").unwrap();
    let arguments = parse_formal_arguments(source, &parameters);

    let return_type = 'return_type: {
        let Some(return_type) = node.child_by_field_name("return_type") else { break 'return_type None };
        Some(parse_type(source, &return_type))
    };

    // let body = node.child_by_field_name("body").unwrap();
    // assert_eq!(body.kind(), "statement_block");
    // let body_str = parse_statement_block(source, &body);
    // no body for now

    dbg!(ArtelFunctionDeclaration::new(
        r#async,
        name_ident,
        type_parameters,
        arguments,
        return_type
    ))
}

fn parse_formal_arguments(source: &str, parameters: &Node) -> Vec<ArtelFunctionArgument> {
    let mut arguments = Vec::<ArtelFunctionArgument>::new();
    for param in parameters.named_children(&mut parameters.walk()) {
        let arg_name = {
            let arg_name_str = param
                .child_by_field_name("pattern")
                .unwrap()
                .utf8_text(&source.as_bytes())
                .unwrap();
            ArtelIdentifier::new(arg_name_str)
        };

        let mut arg_type = param.child_by_field_name("type").map_or(
            ArtelType(vec![ArtelPrimaryType::UnsupportedAny]),
            |arg_type| parse_type(source, &arg_type),
        );

        let mut default_value = None;
        if param.kind() == "optional_parameter" {
            arg_type
                .0
                .push(ArtelPrimaryType::LiteralType(ArtelLiteralType::Undefined));
            default_value = Some(ArtelExpression("пусто".to_owned()));
        }

        if let Some(value) = param.child_by_field_name("value") {
            let value_str = value.utf8_text(source.as_bytes()).unwrap().to_owned();
            default_value = Some(ArtelExpression(value_str)); // TODO, expression parsing
        };

        arguments.push(ArtelFunctionArgument::new(
            arg_name,
            arg_type,
            default_value,
        ));
    }
    arguments
}

fn parse_statement_block(source: &str, child: &Node) -> String {
    let mut cursor = child.walk();
    let mut statements = Vec::new();
    for node in child.named_children(&mut cursor) {
        statements.push(parse_statement(source, &node));
    }
    unimplemented!()
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

    unimplemented!()
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

/// Not used
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

fn parse_lexical_declaration(source: &str, node: &Node) -> ArtelStatement {
    let mut cursor = node.walk();

    let decl_type = {
        let const_or_let_str = node
            .child_by_field_name("kind")
            .expect("variable declaration statement was not found")
            .kind();
        ArtelLexicalDeclarationType::new(const_or_let_str)
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
    };
    unimplemented!()
}

pub fn create_artel_code(artel_program: &ArtelProgram) -> String {
    let mut statements = Vec::new();
    for statement in artel_program {
        statements.push(statement.artel_str(4));
    }

    statements.join("\n")
}
