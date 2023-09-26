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

fn parse_return_statement(source: &str, node: &Node) -> String {
    let Some(expression) = node.named_child(0) else {
        return String::from("возврат");
    };
    let expression_str = parse_expression(source, &expression);
    format!("возврат {expression_str}")
}
