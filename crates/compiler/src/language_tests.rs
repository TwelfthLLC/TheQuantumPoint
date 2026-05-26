use super::*;
use graph_model::{
    data_set_str, Edge, GraphLayer, Node, Position, Project, NODE_FOREACH, NODE_LOG, NODE_START,
    NODE_SWITCH,
};
use ir::Action;

fn wire(project: &mut Project, src: &str, sh: &str, tgt: &str) {
    project.edges.push(Edge {
        id: format!("e-{src}-{sh}-{tgt}"),
        source: src.into(),
        source_handle: sh.into(),
        target: tgt.into(),
        target_handle: "exec".into(),
    });
}

#[test]
fn switch_uses_cases_field() {
    let mut p = Project {
        name: "t".into(),
        layer: GraphLayer::Core,
        nodes: vec![
            Node {
                id: "s".into(),
                kind: NODE_START.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: Default::default(),
            },
            Node {
                id: "sw".into(),
                kind: NODE_SWITCH.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: {
                    let mut d = std::collections::HashMap::new();
                    data_set_str(&mut d, "variable", "x");
                    data_set_str(&mut d, "cases", "a,b");
                    d
                },
            },
            Node {
                id: "l1".into(),
                kind: NODE_LOG.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: {
                    let mut d = std::collections::HashMap::new();
                    data_set_str(&mut d, "message", "arm-a");
                    d
                },
            },
        ],
        edges: vec![],
        subgraphs: vec![],
    };
    wire(&mut p, "s", "exec", "sw");
    wire(&mut p, "sw", "case1", "l1");
    let program = compile(&p).expect("compile");
    let switch = program
        .actions
        .iter()
        .find_map(|a| match a {
            Action::Switch { arms, .. } => Some(arms.clone()),
            _ => None,
        })
        .expect("switch action");
    assert_eq!(switch.len(), 1);
    assert_eq!(switch[0].label, "a");
}

#[test]
fn foreach_lowers_to_ir() {
    let mut p = Project {
        name: "t".into(),
        layer: GraphLayer::Core,
        nodes: vec![
            Node {
                id: "s".into(),
                kind: NODE_START.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: Default::default(),
            },
            Node {
                id: "fe".into(),
                kind: NODE_FOREACH.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: {
                    let mut d = std::collections::HashMap::new();
                    data_set_str(&mut d, "collection", "users");
                    data_set_str(&mut d, "item_var", "row");
                    d
                },
            },
        ],
        edges: vec![],
        subgraphs: vec![],
    };
    wire(&mut p, "s", "exec", "fe");
    let program = compile(&p).expect("compile");
    assert!(program.actions.iter().any(|a| matches!(
        a,
        Action::ForEach {
            collection,
            item_var,
            ..
        } if collection == "users" && item_var == "row"
    )));
}

#[test]
fn function_and_call_lower() {
    use graph_model::{NODE_CALL, NODE_FUNCTION};

    let mut p = Project {
        name: "t".into(),
        layer: GraphLayer::Core,
        nodes: vec![
            Node {
                id: "s".into(),
                kind: NODE_START.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: Default::default(),
            },
            Node {
                id: "fn".into(),
                kind: NODE_FUNCTION.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: {
                    let mut d = std::collections::HashMap::new();
                    data_set_str(&mut d, "name", "add");
                    data_set_str(&mut d, "params", "a,b");
                    d
                },
            },
            Node {
                id: "c".into(),
                kind: NODE_CALL.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: {
                    let mut d = std::collections::HashMap::new();
                    data_set_str(&mut d, "name", "add");
                    data_set_str(&mut d, "args", "1,2");
                    d
                },
            },
            Node {
                id: "lg".into(),
                kind: NODE_LOG.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: {
                    let mut d = std::collections::HashMap::new();
                    data_set_str(&mut d, "message", "done");
                    d
                },
            },
        ],
        edges: vec![],
        subgraphs: vec![],
    };
    wire(&mut p, "fn", "body", "lg");
    wire(&mut p, "s", "exec", "c");
    let program = compile(&p).expect("compile");
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "add");
    assert!(program.actions.iter().any(|a| matches!(
        a,
        Action::Call { name, .. } if name == "add"
    )));
}
