use vn_core::script::lexer::tokenize;
use vn_core::script::parser::parse_scene;

fn main() {
    let script = r#"
scene start:
    show gabriel happy
    gabriel "Hello!"
    show crownley serious
    "hello"
    crownley "Hello"
    choice:
        "Yes":
            gabriel "character deep test"
            jump yes_scene
        "No":
            "narration deep test 2"
            jump no_scene
"#;

    let tokens = tokenize(script);

    for t in &tokens {
        println!("{:?} (Indent: {})", t.kind, t.indent);
    }

    let (node, _) = parse_scene(&tokens, 0);
    println!("{:#?}", node);
}
