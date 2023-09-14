use tree_sitter::Node;

pub fn walk_tree_recursively_dbg(source: &str, node: &Node, ident: usize) {
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
