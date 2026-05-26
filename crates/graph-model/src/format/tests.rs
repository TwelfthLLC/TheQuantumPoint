use super::*;
use crate::{GraphLayer, Position, NODE_START};

#[test]
fn graph_roundtrip_binary() {
    let nodes = vec![crate::Node {
        id: "start".into(),
        kind: NODE_START.into(),
        position: Position { x: 0.0, y: 0.0 },
        data: Default::default(),
    }];
    let bytes = encode_graph_file("test", GraphLayer::Core, &nodes, &[], &[]).unwrap();
    assert_eq!(&bytes[0..4], &GRAPH_MAGIC);
    assert_eq!(
        u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        QP_FILE_VERSION
    );
    let (name, layer, decoded, edges, subs) = decode_graph_file(&bytes).unwrap();
    assert!(subs.is_empty());
    assert_eq!(name, "test");
    assert_eq!(layer, GraphLayer::Core);
    assert_eq!(decoded.len(), 1);
    assert!(edges.is_empty());
}

#[test]
fn registry_meta_roundtrip_binary() {
    let m = RegistryMeta {
        id: "test-id".into(),
        name: "Test".into(),
        updated_at: 42,
        folder: Some("C:/proj".into()),
    };
    let bytes = encode_registry_meta(&m).unwrap();
    assert_eq!(&bytes[0..4], &REGISTRY_META_MAGIC);
    let back = decode_registry_meta(&bytes).unwrap();
    assert_eq!(back.id, m.id);
    assert_eq!(back.folder, m.folder);
}

#[test]
fn manifest_roundtrip_binary() {
    let m = ProjectManifest {
        id: "id".into(),
        name: "n".into(),
        qp_tool_version: "0.0.0.2".into(),
        entry_graph: "graphs/main.qp".into(),
        graphs: vec!["graphs/main.qp".into()],
    };
    let bytes = encode_project_manifest(&m).unwrap();
    assert_eq!(&bytes[0..4], &PROJECT_MAGIC);
    let back = decode_project_manifest(&bytes).unwrap();
    assert_eq!(back.entry_graph, m.entry_graph);
}

/// Workspace `examples/` papkalariga binar namunalar yozadi.
#[test]
fn write_workspace_example_binaries() {
    use crate::{
        DataValue, Edge, GraphLayer, Node, Position, Project, NODE_ASSIGN, NODE_IF, NODE_LOG,
        NODE_START,
    };
    use std::path::PathBuf;

    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

    let hello = Project {
        name: "hello-rust".into(),
        layer: GraphLayer::Core,
        nodes: vec![
            Node {
                id: "start".into(),
                kind: NODE_START.into(),
                position: Position { x: 80.0, y: 200.0 },
                data: Default::default(),
            },
            Node {
                id: "log1".into(),
                kind: NODE_LOG.into(),
                position: Position { x: 280.0, y: 200.0 },
                data: [(
                    "message".to_string(),
                    DataValue::str("Salom — Quantum Point → Rust!"),
                )]
                .into_iter()
                .collect(),
            },
            Node {
                id: "assign1".into(),
                kind: NODE_ASSIGN.into(),
                position: Position { x: 480.0, y: 200.0 },
                data: [
                    ("name".to_string(), DataValue::str("version")),
                    ("value".to_string(), DataValue::typed_i64(1)),
                ]
                .into_iter()
                .collect(),
            },
            Node {
                id: "log2".into(),
                kind: NODE_LOG.into(),
                position: Position { x: 680.0, y: 200.0 },
                data: [(
                    "message".to_string(),
                    DataValue::str("Build tugadi, native Rust ishlayapti."),
                )]
                .into_iter()
                .collect(),
            },
        ],
        edges: vec![
            Edge {
                id: "e1".into(),
                source: "start".into(),
                target: "log1".into(),
                source_handle: "exec".into(),
                target_handle: "exec".into(),
            },
            Edge {
                id: "e2".into(),
                source: "log1".into(),
                target: "assign1".into(),
                source_handle: "exec".into(),
                target_handle: "exec".into(),
            },
            Edge {
                id: "e3".into(),
                source: "assign1".into(),
                target: "log2".into(),
                source_handle: "exec".into(),
                target_handle: "exec".into(),
            },
        ],
        subgraphs: vec![],
    };

    let hello_dir = ws.join("examples/hello-rust");
    std::fs::create_dir_all(hello_dir.join("graphs")).unwrap();
    std::fs::write(
        hello_dir.join("graphs/main.qp"),
        hello.to_qp_bytes().unwrap(),
    )
    .unwrap();
    std::fs::write(
        hello_dir.join("quantum-point.qp"),
        encode_project_manifest(&ProjectManifest {
            id: "hello-rust".into(),
            name: "hello-rust".into(),
            qp_tool_version: "0.0.0.2".into(),
            entry_graph: "graphs/main.qp".into(),
            graphs: vec!["graphs/main.qp".into()],
        })
        .unwrap(),
    )
    .unwrap();

    let branch = Project {
        name: "branch-rust".into(),
        layer: GraphLayer::Core,
        nodes: vec![
            Node {
                id: "start".into(),
                kind: NODE_START.into(),
                position: Position { x: 60.0, y: 220.0 },
                data: Default::default(),
            },
            Node {
                id: "if1".into(),
                kind: NODE_IF.into(),
                position: Position { x: 260.0, y: 220.0 },
                data: [("condition".to_string(), DataValue::str("2 > 1"))]
                    .into_iter()
                    .collect(),
            },
            Node {
                id: "log_true".into(),
                kind: NODE_LOG.into(),
                position: Position { x: 500.0, y: 120.0 },
                data: [("message".to_string(), DataValue::str("Shart: rost (true)"))]
                    .into_iter()
                    .collect(),
            },
            Node {
                id: "log_false".into(),
                kind: NODE_LOG.into(),
                position: Position { x: 500.0, y: 320.0 },
                data: [(
                    "message".to_string(),
                    DataValue::str("Shart: yolg'on (false)"),
                )]
                .into_iter()
                .collect(),
            },
            Node {
                id: "log_done".into(),
                kind: NODE_LOG.into(),
                position: Position { x: 740.0, y: 220.0 },
                data: [("message".to_string(), DataValue::str("Tugadi."))]
                    .into_iter()
                    .collect(),
            },
        ],
        edges: vec![
            Edge {
                id: "e_start_if".into(),
                source: "start".into(),
                target: "if1".into(),
                source_handle: "exec".into(),
                target_handle: "exec".into(),
            },
            Edge {
                id: "e_if_true".into(),
                source: "if1".into(),
                target: "log_true".into(),
                source_handle: "true".into(),
                target_handle: "exec".into(),
            },
            Edge {
                id: "e_if_false".into(),
                source: "if1".into(),
                target: "log_false".into(),
                source_handle: "false".into(),
                target_handle: "exec".into(),
            },
            Edge {
                id: "e_if_done".into(),
                source: "if1".into(),
                target: "log_done".into(),
                source_handle: "done".into(),
                target_handle: "exec".into(),
            },
        ],
        subgraphs: vec![],
    };

    let branch_dir = ws.join("examples/branch-rust");
    std::fs::create_dir_all(branch_dir.join("graphs")).unwrap();
    std::fs::write(
        branch_dir.join("graphs/main.qp"),
        branch.to_qp_bytes().unwrap(),
    )
    .unwrap();
    std::fs::write(
        branch_dir.join("quantum-point.qp"),
        encode_project_manifest(&ProjectManifest {
            id: "branch-rust".into(),
            name: "branch-rust".into(),
            qp_tool_version: "0.0.0.2".into(),
            entry_graph: "graphs/main.qp".into(),
            graphs: vec!["graphs/main.qp".into()],
        })
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn data_value_in_node_roundtrip() {
    use crate::{DataValue, Node, Position, NODE_LOG, NODE_START};
    let nodes = vec![
        Node {
            id: "start".into(),
            kind: NODE_START.into(),
            position: Position { x: 0.0, y: 0.0 },
            data: Default::default(),
        },
        Node {
            id: "log1".into(),
            kind: NODE_LOG.into(),
            position: Position { x: 1.0, y: 0.0 },
            data: [("message".to_string(), DataValue::str("Salom"))].into(),
        },
    ];
    let bytes = encode_graph_file("t", GraphLayer::Core, &nodes, &[], &[]).unwrap();
    decode_graph_file(&bytes).expect("decode with DataValue::Str in map");
}

#[test]
fn typed_data_value_roundtrip() {
    use crate::{DataValue, Node, Position, NODE_ASSIGN};
    let nodes = vec![Node {
        id: "a1".into(),
        kind: NODE_ASSIGN.into(),
        position: Position { x: 0.0, y: 0.0 },
        data: [("value".to_string(), DataValue::typed_i64(1))].into(),
    }];
    let bytes = encode_graph_file("t", GraphLayer::Core, &nodes, &[], &[]).unwrap();
    decode_graph_file(&bytes).expect("decode Typed DataValue");
}

#[test]
fn decode_workspace_hello_qp() {
    use crate::Project;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/hello-rust/graphs/main.qp");
    let raw = std::fs::read(&path).expect("read hello main.qp");
    let p = Project::from_qp_bytes(&raw).expect("decode hello main.qp");
    assert_eq!(p.name, "hello-rust");
    assert!(p.nodes.len() >= 2);
}

#[test]
fn rejects_wrong_magic() {
    let bytes = encode_graph_file("x", GraphLayer::Core, &[], &[], &[]).unwrap();
    let mut bad = bytes;
    bad[0] = b'X';
    assert!(matches!(
        decode_graph_file(&bad),
        Err(GraphFileParseError::InvalidMagic { .. })
    ));
}
