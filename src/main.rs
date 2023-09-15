mod artel_nodes;
mod convert_tree_walk;
mod dbg_tree_walk;

use tree_sitter::Parser;

use convert_tree_walk::*;
use dbg_tree_walk::walk_tree_recursively_dbg;

fn read_example(filename: &str) -> String {
    std::fs::read_to_string(format!("./examples/{filename}"))
        .unwrap_or_else(|_| std::fs::read_to_string(format!("./{filename}")).unwrap())
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

    let res = walk_tree(&code, &root);

    println!("{code}");
    println!("---");
    println!("{res}");

    assert!(!root.has_error());
}

fn main() {
    let code = read_example("hello_world.ts");
    convert_ts(&code);
}
