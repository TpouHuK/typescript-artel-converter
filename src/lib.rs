mod artel_nodes;
mod convert_tree_walk;
mod dbg_tree_walk;

use convert_tree_walk::*;
use tree_sitter_c2rust::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

#[allow(improper_ctypes_definitions)] // String is fine for wasm_bindgen
#[wasm_bindgen]
pub extern "C" fn convert_ts(code: &str) -> String {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading typescript grammar");

    let parsed = parser.parse(code, None).unwrap();
    let root = parsed.root_node();
    let res = walk_tree(&code, &root);

    create_artel_code(res)
}
