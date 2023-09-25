mod artel_nodes;
mod convert_tree_walk;
mod dbg_tree_walk;

use std::env;
use tree_sitter_c2rust::Parser;

use convert_tree_walk::*;
use dbg_tree_walk::walk_tree_recursively_dbg;

fn read_example(filename: &str) -> String {
    std::fs::read_to_string(format!("./examples/{filename}"))
        .unwrap_or_else(|_| std::fs::read_to_string(format!("./{filename}")).unwrap())
}

fn read_file(filename: &str) -> String {
    std::fs::read_to_string(filename).unwrap()
}

fn convert_ts_debug(code: &str) -> () {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading typescript grammar");

    let parsed = parser.parse(code, None).unwrap();
    let root = parsed.root_node();
    walk_tree_recursively_dbg(&code, &root, 0);

    let res = walk_tree(&code, &root);
    dbg!(&res);

    println!("{code}");
    println!("---");
    println!("{}", create_artel_code(res));
}

fn convert_ts(code: &str) -> String {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading typescript grammar");

    let parsed = parser.parse(code, None).unwrap();
    let root = parsed.root_node();
    let res = walk_tree(&code, &root);

    create_artel_code(res)
}

fn main() {
    let mut file_name = env::args().nth(1).expect("Filename as the first argument");

    if !file_name.ends_with(".ts") {
        file_name.push_str(".ts");
    }

    let code = read_example(&file_name);

    if let Some("--dbg") = env::args().nth(2).as_deref() {
        convert_ts_debug(&code);
    } else {
        let res = convert_ts(&code);
        println!("{}", res);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rstest::rstest;

    #[rstest]
    fn test_ts_file(
        #[values(
            "add.ts",
            "optional_param.ts",
            "function_arg_pattern.ts",
            "class.ts",
            "interface.ts",
            "enum.ts",
            "type_union.ts",
            "type_builtin.ts",
            "type_alias.ts",
            "type_undefined.ts",
            "type_generic.ts",
            "type_default_generic.ts",
            "keyof.ts",
            "export.ts",
            "object.ts",
            "object_simple_method.ts",
            "object_simple_prop.ts",
            "array_type.ts"
        )]
        path: &str,
    ) {
        let path = path;
        let text = read_example(path);
        convert_ts_debug(&text);
    }
}
