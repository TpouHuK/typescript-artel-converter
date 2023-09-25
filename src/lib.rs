mod artel_nodes;
mod convert_tree_walk;
mod dbg_tree_walk;

use tree_sitter_c2rust::Parser;
use convert_tree_walk::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub extern fn convert_ts(code: &str) -> String {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading typescript grammar");

    let parsed = parser.parse(code, None).unwrap();
    let root = parsed.root_node();
    let res = walk_tree(&code, &root);

    create_artel_code(res)
}
