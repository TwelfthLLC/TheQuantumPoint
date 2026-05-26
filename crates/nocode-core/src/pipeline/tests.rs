use graph_model::{data_set_str, Edge, GraphLayer, Node, Position, Project, NODE_LOG, NODE_START};

use crate::pipeline::{check_project, CheckOutput};
use compiler::CompileCache;

fn mini_core() -> Project {
    let mut log = Node {
        id: "log1".into(),
        kind: NODE_LOG.into(),
        position: Position { x: 100.0, y: 0.0 },
        data: Default::default(),
    };
    data_set_str(&mut log.data, "message", "hi");
    Project {
        name: "test".into(),
        nodes: vec![
            Node {
                id: "start".into(),
                kind: NODE_START.into(),
                position: Position { x: 0.0, y: 0.0 },
                data: Default::default(),
            },
            log,
        ],
        edges: vec![Edge {
            id: "e1".into(),
            source: "start".into(),
            target: "log1".into(),
            source_handle: "exec".into(),
            target_handle: "exec".into(),
        }],
        layer: GraphLayer::Core,
        subgraphs: vec![],
    }
}

#[test]
fn check_core_no_cargo() {
    let p = mini_core();
    let mut cache = CompileCache::default();
    let out: CheckOutput = check_project(&p, &mut cache, &[], None).expect("check");
    assert!(out.success);
    assert!(out.program.is_some());
    assert_eq!(out.program.as_ref().unwrap().actions.len(), 1);
}
