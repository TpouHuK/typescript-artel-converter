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

use yew::{prelude::*, html::IntoPropValue};
use web_sys::{EventTarget, InputEvent, HtmlTextAreaElement};
use wasm_bindgen::JsCast;

#[function_component]
fn App() -> Html {
    let text = use_state(|| String::new() );
    let output = use_state(|| String::new() );

    let onclick = {
        let text = text.clone();
        let output = output.clone();
        move |_| {
            output.set(convert_ts(&*text));
        }
    };

    let oninput = {
        let text = text.clone();
            Callback::from(move |e: InputEvent| {
            // When events are created the target is undefined, it's only
            // when dispatched does the target get added.
            let target: Option<EventTarget> = e.target();
            // Events can bubble so this listener might catch events from child
            // elements which are not of type HtmlInputElement
            let input = target.and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok());

            if let Some(input) = input {
                text.set(input.value());
            }
        })
    };


    let code = r#"
        function double(x: number): number {
            return x * 2;
        }
    "#;

    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading typescript grammar");
    let parsed = parser.parse(code, None).unwrap();
    let root = parsed.root_node();
    assert!(!root.has_error());

    html! {
        <div>
            <textarea rows=25 cols=100 oninput={oninput}
                /* oninput */
            ></textarea>
            <button {onclick}>{ "Convert" }</button>
            <div style="white-space:pre-wrap">{ &*output }</div>
        </div>
    }
}

fn main() {

    yew::Renderer::<App>::new().render();
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
