use super::ast_nodes::*;
use itertools::Itertools;
use std::string::String;
use tree_sitter_c2rust::Node;

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
        "ambient_declaration" => Some(parse_ambient_declaration(source, node)),
        "function_declaration" | "function_signature" => Some(ArtelStatement::FunctionDeclaration(
            parse_function_declaration(source, node),
        )),
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
        "enum_declaration" => Some(ArtelStatement::EnumDeclaration(parse_enum_declaration(
            source, node,
        ))),
        "type_alias_declaration" => Some(ArtelStatement::TypeAliasDeclaration(
            parse_type_alias_declaration(source, node),
        )),
        "export_statement" => {
            if let Some(declaration) = node.child_by_field_name("declaration") {
                Some(ArtelStatement::ExportStatement(
                    parse_statement(source, &declaration).unwrap().into(),
                ))
            } else {
                //eprintln!("Warning: Export from the import ignored ");
                None
            }
        }
        "interface_declaration" => Some(ArtelStatement::InterfaceDeclaration(parse_interface(
            source, node,
        ))),
        "lexical_declaration" | "variable_declaration" => Some(ArtelStatement::LexicalDeclaration(
            parse_lexical_declaration(source, node),
        )),
        "statement_block" => Some(ArtelStatement::StatementBlock(parse_statement_block(
            source, node,
        ))),
        "import_statement" | "if_statement" => None,
        "expression_statement" => {
            if node.child(0).unwrap().kind() == "internal_module" {
                Some(parse_internal_module(source, node))
            } else {
                None
            }
        }
        _ => {
            unimplemented!(
                "{} is unimplemented, {:?}",
                node.kind(),
                node.utf8_text(source.as_bytes()).unwrap()
            )
        }
    };

    output
}

fn parse_internal_module(source: &str, node: &Node) -> ArtelStatement {
    let name = ArtelIdentifier::new(
        node.child_by_field_name("name")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
    );
    let body = node
        .child_by_field_name("body")
        .unwrap()
        .named_children(&mut node.walk())
        .filter_map(|s| parse_statement(source, &s))
        .collect();
    ArtelStatement::InternalModule(ArtelInternalModule::new(name, body))
}

fn parse_lexical_declaration(source: &str, node: &Node) -> ArtelLexicalDeclaration {
    let decl_type = {
        let const_or_let_str = node
            .child_by_field_name("kind")
            .map(|k| k.kind())
            .unwrap_or("var");
        ArtelLexicalDeclarationType::new(const_or_let_str)
    };

    let declarations: Vec<ArtelLexicalDeclarationMember> = node
        .named_children(&mut node.walk())
        .map(|decl| {
            assert!(decl.kind() == "variable_declarator");
            let ident_str = decl
                .child_by_field_name("name")
                .unwrap()
                .utf8_text(&source.as_bytes())
                .unwrap();
            let ident = ArtelIdentifier::new(ident_str);

            let var_type = decl.child_by_field_name("type").map_or(
                Type(vec![PrimaryType::UnsupportedAny("no_type".into())]),
                |t| parse_type(source, &t),
            );

            let value = {
                match decl.child_by_field_name("value") {
                    Some(node) => Some(parse_expression(source, &node)),
                    None => None,
                }
            };
            ArtelLexicalDeclarationMember::new(ident, var_type, value)
        })
        .collect();

    ArtelLexicalDeclaration::new(decl_type, declarations)
}

// TODO cleanup
fn parse_ambient_declaration(source: &str, node: &Node) -> ArtelStatement {
    let mut cursor = node.walk();
    let mut iterator = node.children(&mut cursor);

    let mut is_global = false;
    loop {
        let next = iterator.next().unwrap();
        if next.kind() != "declare" && next.kind() != "global" {
            break;
        }
        if next.kind() == "global" {
            is_global = true;
        }
    }

    if node.named_child(0).unwrap().kind() == "internal_module" {
        let internal_module = node.named_child(0).unwrap();
        let internal_module = parse_internal_module(source, &internal_module);
        return ArtelStatement::AmbientDeclaration(ArtelAmbientDeclaration::new(
            false,
            vec![internal_module],
        ));
    }

    assert!(node.named_child_count() == 1);
    let decl_body = node.named_child(0).unwrap();

    let body = match decl_body.kind() {
            "variable_declaration" => vec![ArtelStatement::LexicalDeclaration(
                parse_lexical_declaration(source, &decl_body),
            )],
            "function_signature" => vec![ArtelStatement::FunctionDeclaration(
                parse_function_declaration(source, &decl_body),
            )],
            "type_alias_declaration" => vec![ArtelStatement::TypeAliasDeclaration(
                parse_type_alias_declaration(source, &decl_body),
            )],
            "class_declaration" => vec![ArtelStatement::ClassDeclaration(parse_class_declaration(
                source, &decl_body, false,
            ))],
            "lexical_declaration" => vec![ArtelStatement::LexicalDeclaration(
                parse_lexical_declaration(source, &decl_body),
            )],
            "statement_block" => {
                parse_statement_block(source, &decl_body)
            }
            _ => unimplemented!("{:?}", decl_body.kind()),
    };

    ArtelStatement::AmbientDeclaration(ArtelAmbientDeclaration::new(is_global, body).into())
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
            "construct_signature" => {
                let method = parse_construct_signature(source, &element);
                vec.push(ArtelInterfaceMember::Method(method))
            }
            "index_signature" | "call_signature" => vec.push(ArtelInterfaceMember::Unsupported(
                element.utf8_text(source.as_bytes()).unwrap().to_owned(),
            )),
            "comment" => { // TODO
            }
            _ => {
                unimplemented!("{}", element.kind())
            }
        }
    }

    ArtelInterfaceDeclaration::new(interface_name_ident, type_parameters, vec)
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

    ArtelTypeAliasDeclaration::new(alias, type_parameters, value)
}

fn parse_type(source: &str, node: &Node) -> Type {
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

    assert!(!vec.is_empty(), "parsed type, and got no type");
    Type(vec)
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

fn parse_type_inner(source: &str, node: &Node, vec: &mut Vec<PrimaryType>) {
    match node.kind() {
        "union_type" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                parse_type_inner(source, &child, vec);
            }
        }

        "type_identifier" => {
            let name = node.utf8_text(source.as_bytes()).unwrap();
            let r#type = PrimaryType::TypeReference(TypeReference::new(
                ArtelIdentifier::new(name),
                Vec::new(),
            ));
            vec.push(r#type);
        }

        "predefined_type" => {
            let predefined_type_str = node.utf8_text(source.as_bytes()).unwrap();
            let r#type = PrimaryType::PredefinedType(predefined_type_str.into());
            vec.push(r#type);
        }

        "literal_type" => {
            let node = node.named_child(0).unwrap();
            let r#type = match node.kind() {
                "string" => {
                    let string_fragment = node
                        .named_child(0)
                        .map(|f| f.utf8_text(source.as_bytes()).unwrap())
                        .unwrap_or("".into());
                    LiteralType::String(string_fragment.to_owned())
                }
                "number" | "unary_expression" => {
                    let number = node.utf8_text(source.as_bytes()).unwrap();
                    LiteralType::Number(number.to_owned())
                }
                "null" => LiteralType::Null,
                "true" => LiteralType::Boolean(true),
                "false" => LiteralType::Boolean(false),
                "undefined" => LiteralType::Undefined,
                _ => {
                    unreachable!("{}", node.kind())
                }
            };
            vec.push(PrimaryType::LiteralType(r#type));
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
            vec.push(PrimaryType::TypeReference(TypeReference::new(
                ArtelIdentifier::new(name),
                vec2,
            )));
        }

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
            vec.push(PrimaryType::FunctionType(
                FunctionDeclaration::new(
                    false,
                    ArtelIdentifier::new("$FUNCTION_TYPE$"),
                    type_parameters,
                    arguments,
                    return_type,
                )
                .into(),
            ));
        }

        "object_type" => vec.push(parse_object_type(source, &node)),

        "array_type" => {
            vec.push(PrimaryType::ArrayType(
                parse_type(source, &node.named_child(0).unwrap()).into(),
            ));
        }

        "readonly_type" => {
            vec.push(PrimaryType::ReadonlyType(
                parse_type(source, &node.named_child(0).unwrap()).into(),
            ));
        }
        "tuple_type" => {
            let tuple: Vec<_> = node
                .named_children(&mut node.walk())
                .map(|n| parse_type(source, &n))
                .collect();
            vec.push(PrimaryType::TupleType(tuple));
        }
        "parenthesized_type" => vec.extend(parse_type(source, &node.named_child(0).unwrap()).0),
        "type_predicate" => {
            let name = node
                .child_by_field_name("name")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_owned();
            let r#type = parse_type(source, &node.child_by_field_name("type").unwrap());
            vec.push(PrimaryType::PredicateType(name, r#type.into()));
        }
        "type_predicate_annotation" => {
            vec.extend(parse_type(source, &node.named_child(0).unwrap()).0);
        }
        "type_query"
        | "template_literal_type"
        | "nested_type_identifier"
        | "constructor_type"
        | "intersection_type"
        | "index_type_query"
        | "lookup_type"
        | "this_type"
        | "conditional_type"
        | "rest_type" => vec.push(PrimaryType::UnsupportedAny(
            node.utf8_text(source.as_bytes()).unwrap().into(),
        )),
        _ => {
            unimplemented!("{} is not implemented: {:?}", node.kind(), node)
        }
    };
}

fn parse_object_type(source: &str, node: &Node) -> PrimaryType {
    let body = node
        .named_children(&mut node.walk())
        .filter(|n| n.kind() != "comment")
        .map(|property_signature| {
            // Anti-support for mapped types
            if property_signature.kind() == "index_signature" {
                return None;
            }

            match property_signature.kind() {
                "method_signature" => Some(ArtelInterfaceMember::Method(
                    parse_function_declaration(source, &property_signature),
                )),
                "property_signature" => {
                    let readonly = property_signature.child(0).unwrap().kind() == "readonly";
                    let name_str = property_signature
                        .child_by_field_name("name")
                        .unwrap()
                        .utf8_text(source.as_bytes())
                        .unwrap();
                    let name_ident = ArtelIdentifier::new(name_str);
                    let r#type = parse_type(
                        source,
                        &property_signature.child_by_field_name("type").unwrap(),
                    );
                    Some(ArtelInterfaceMember::Property(ArtelProperty::new(
                        readonly, name_ident, r#type,
                    )))
                }
                "construct_signature" => Some(ArtelInterfaceMember::Method(
                    parse_construct_signature(source, &property_signature),
                )),
                "index_signature" | "call_signature" => Some(ArtelInterfaceMember::Unsupported(
                    property_signature
                        .utf8_text(source.as_bytes())
                        .unwrap()
                        .to_owned(),
                )),
                _ => unimplemented!("{}", property_signature.kind()),
            }
        })
        .take_while_inclusive(|p| p.is_some())
        .collect_vec();

    // Anti-support for mapped types
    if let Some(None) = body.last() {
        return PrimaryType::UnsupportedAny(node.utf8_text(source.as_bytes()).unwrap().into());
    }

    PrimaryType::ObjectType(ObjectType::new(
        body.into_iter().map(|e| e.unwrap()).collect_vec(),
    ))
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
            "comment" => {} // TODO
            _ => {
                unreachable!("unexpected {} in enum", item.kind())
            }
        }
    }

    EnumDeclaration::new(name_ident, items)
}

fn parse_expression(source: &str, node: &Node) -> String {
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

                // TODO NO TYPE IS INSIDE THIS SHIT
                //
                let mut property_type = definition.child_by_field_name("type").map_or(
                    Type(vec![PrimaryType::UnsupportedAny("no_type".into())]),
                    |n| parse_type(source, &n),
                );
                if is_optional {
                    property_type
                        .0
                        .push(PrimaryType::LiteralType(LiteralType::Undefined));
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
                definitions.push(definition);
            }
            "method_definition" => {
                let definition = parse_method_definition(source, &definition);
                definitions.push(definition);
            }
            "abstract_method_signature" => {
                let method = parse_method_definition(source, &definition);
                definitions.push(method);
            }
            "method_signature" => {
                let method = parse_method_definition(source, &definition);
                definitions.push(method);
            }
            "comment" | "decorator" => {}
            _ => {
                unimplemented!("{} is not implemented in a class body", definition.kind())
            }
        };
    }

    ArtelClassDeclaration::new(
        name_ident,
        extends_clause,
        implements_clause,
        is_abstract,
        type_parameters,
        definitions,
    )
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
    // TODO, iterators
    let mut _is_iterator = false;

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
            "*" => {
                _is_iterator = true;
            }
            x => {
                unimplemented!("{x}");
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
        FunctionDeclaration::new(
            is_async,
            name_ident,
            type_parameters,
            arguments,
            return_type,
        ),
    ))
}

fn parse_construct_signature(source: &str, node: &Node) -> FunctionDeclaration {
    let r#async = node.child(0).unwrap().kind() == "async";
    let name_ident = ArtelIdentifier::new("constructor");

    let type_parameters = 'type_parameters: {
        let Some(parameters_node) = node.child_by_field_name("type_parameters") else { break 'type_parameters Vec::new() };
        parse_type_parameters(source, &parameters_node)
    };

    let parameters = node.child_by_field_name("parameters").unwrap();
    let arguments = parse_formal_arguments(source, &parameters);

    let return_type = 'return_type: {
        let Some(return_type) = node.child_by_field_name("type") else { break 'return_type None };
        Some(parse_type(source, &return_type))
    };

    // let body = node.child_by_field_name("body").unwrap();
    // assert_eq!(body.kind(), "statement_block");
    // let body_str = parse_statement_block(source, &body);
    // no body for now

    FunctionDeclaration::new(r#async, name_ident, type_parameters, arguments, return_type)
}

fn parse_function_declaration(source: &str, node: &Node) -> FunctionDeclaration {
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

    FunctionDeclaration::new(r#async, name_ident, type_parameters, arguments, return_type)
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
            Type(vec![PrimaryType::UnsupportedAny("no_type".into())]),
            |arg_type| parse_type(source, &arg_type),
        );

        let mut default_value = None;
        if param.kind() == "optional_parameter" {
            arg_type
                .0
                .push(PrimaryType::LiteralType(LiteralType::Undefined));
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

fn parse_statement_block(source: &str, node: &Node) -> Vec<ArtelStatement> {
    node.named_children(&mut node.walk())
        .filter_map(|s| parse_statement(source, &s))
        .collect()
}

pub fn create_artel_code(artel_program: ArtelProgram) -> String {
    let flat_export = artel_program.artel_str(0);
    flat_export

    /* let flat_export = artel_program.into_iter().filter_map(|stmt| match stmt {
        ArtelStatement::ExportStatement(stmt) => Some(*stmt),
        comment @ ArtelStatement::Comment(_) => Some(comment),
        _ => None,
    });
    [
        "внешнее {\n",
        &flat_export
            .map(|s| {
                if let ArtelStatement::Comment(_) = s {
                    s.artel_str(2)
                } else {
                    [&s.artel_str(2), "\n"].concat()
                }
            })
            .collect::<Vec<String>>()
            .join("\n"),
        "\n}",
    ]
    .concat()*/
}
