use ir::{Action, Program};

use super::interpret;

#[test]
fn print_preview() {
    let p = Program {
        name: "t".into(),
        needs_async_runtime: false,
        functions: vec![],
        structs: vec![],
        enums: vec![],
        actions: vec![Action::Print {
            message: "hi".into(),
        }],
    };
    let out = interpret(&p).unwrap();
    assert_eq!(out.lines, vec!["hi"]);
}
