use crate::{Edge, GraphLayer, Node, SubGraphRef};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Graf fayl (`graphs/*.qp`) magic.
pub const GRAPH_MAGIC: [u8; 4] = *b"QPGR";
/// Loyiha manifesti (`quantum-point.qp`) magic.
pub const PROJECT_MAGIC: [u8; 4] = *b"QPRJ";
/// Launcher registry yozuvi (`.nocode/projects/<id>/meta.qp`) magic.
pub const REGISTRY_META_MAGIC: [u8; 4] = *b"QPME";
/// Binar `.qp` format versiyasi (2: subgraphs maydoni).
pub const QP_FILE_VERSION: u32 = 2;
/// Graf fayl kengaytmasi.
pub const GRAPH_FILE_EXTENSION: &str = "qp";
/// Loyiha manifest fayl nomi.
pub const PROJECT_MANIFEST_FILE: &str = "quantum-point.qp";
/// Launcher registry meta fayl nomi.
pub const REGISTRY_META_FILE: &str = "meta.qp";

const HEADER_LEN: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphPayload {
    name: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    #[serde(default)]
    layer: GraphLayer,
    #[serde(default)]
    subgraphs: Vec<SubGraphRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub id: String,
    pub name: String,
    pub qp_tool_version: String,
    pub entry_graph: String,
    pub graphs: Vec<String>,
}

#[derive(Debug, Error)]
pub enum GraphFileParseError {
    #[error("file too short for Quantum Point header")]
    TooShort,
    #[error("invalid magic (expected {expected:?}, got {found:?})")]
    InvalidMagic { expected: [u8; 4], found: [u8; 4] },
    #[error("unsupported .qp version {found} (max {max})")]
    UnsupportedVersion { found: u32, max: u32 },
    #[error("postcard decode: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn encode_graph_file(
    name: &str,
    layer: GraphLayer,
    nodes: &[Node],
    edges: &[Edge],
    subgraphs: &[SubGraphRef],
) -> Result<Vec<u8>, postcard::Error> {
    let payload = GraphPayload {
        name: name.to_string(),
        nodes: nodes.to_vec(),
        edges: edges.to_vec(),
        layer,
        subgraphs: subgraphs.to_vec(),
    };
    encode_with_header(GRAPH_MAGIC, QP_FILE_VERSION, &payload)
}

/// Decoded graph file contents (v1/v2 payload).
pub type DecodedGraph = (String, GraphLayer, Vec<Node>, Vec<Edge>, Vec<SubGraphRef>);

pub fn decode_graph_file(data: &[u8]) -> Result<DecodedGraph, GraphFileParseError> {
    let payload: GraphPayload = decode_graph_file_payload(data)?;
    Ok((
        payload.name,
        payload.layer,
        payload.nodes,
        payload.edges,
        payload.subgraphs,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphPayloadV1 {
    name: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    #[serde(default)]
    layer: GraphLayer,
}

fn decode_graph_file_payload(data: &[u8]) -> Result<GraphPayload, GraphFileParseError> {
    if data.len() < HEADER_LEN {
        return Err(GraphFileParseError::TooShort);
    }
    let mut found = [0u8; 4];
    found.copy_from_slice(&data[0..4]);
    if found != GRAPH_MAGIC {
        return Err(GraphFileParseError::InvalidMagic {
            expected: GRAPH_MAGIC,
            found,
        });
    }
    let version = u32::from_le_bytes(data[4..8].try_into().expect("header"));
    if version > QP_FILE_VERSION {
        return Err(GraphFileParseError::UnsupportedVersion {
            found: version,
            max: QP_FILE_VERSION,
        });
    }
    let body = &data[HEADER_LEN..];
    if version <= 1 {
        let v1: GraphPayloadV1 = postcard::from_bytes(body)?;
        return Ok(GraphPayload {
            name: v1.name,
            nodes: v1.nodes,
            edges: v1.edges,
            layer: v1.layer,
            subgraphs: Vec::new(),
        });
    }
    postcard::from_bytes(body).map_err(Into::into)
}

pub fn encode_project_manifest(manifest: &ProjectManifest) -> Result<Vec<u8>, postcard::Error> {
    encode_with_header(PROJECT_MAGIC, QP_FILE_VERSION, manifest)
}

pub fn decode_project_manifest(data: &[u8]) -> Result<ProjectManifest, GraphFileParseError> {
    decode_with_header(data, PROJECT_MAGIC)
}

/// Launcher ro‘yxati: `.nocode/projects/<id>/meta.qp`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMeta {
    pub id: String,
    pub name: String,
    pub updated_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
}

pub fn encode_registry_meta(meta: &RegistryMeta) -> Result<Vec<u8>, postcard::Error> {
    encode_with_header(REGISTRY_META_MAGIC, QP_FILE_VERSION, meta)
}

pub fn decode_registry_meta(data: &[u8]) -> Result<RegistryMeta, GraphFileParseError> {
    decode_with_header(data, REGISTRY_META_MAGIC)
}

pub fn registry_meta_to_bytes(meta: &RegistryMeta) -> Result<Vec<u8>, GraphFileParseError> {
    encode_registry_meta(meta).map_err(GraphFileParseError::Postcard)
}

/// Bir martalik migratsiya: eski `meta.json` → keyin `meta.qp` yoziladi va JSON o‘chiriladi.
pub fn decode_registry_meta_legacy_json(data: &[u8]) -> Result<RegistryMeta, GraphFileParseError> {
    let s = std::str::from_utf8(data).map_err(|_| GraphFileParseError::TooShort)?;
    serde_json::from_str(s).map_err(GraphFileParseError::Json)
}

fn encode_with_header<T: Serialize>(
    magic: [u8; 4],
    version: u32,
    payload: &T,
) -> Result<Vec<u8>, postcard::Error> {
    let body = postcard::to_allocvec(payload)?;
    let mut out = Vec::with_capacity(HEADER_LEN + body.len());
    out.extend_from_slice(&magic);
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

fn decode_with_header<T: for<'de> Deserialize<'de>>(
    data: &[u8],
    expected_magic: [u8; 4],
) -> Result<T, GraphFileParseError> {
    if data.len() < HEADER_LEN {
        return Err(GraphFileParseError::TooShort);
    }
    let mut found = [0u8; 4];
    found.copy_from_slice(&data[0..4]);
    if found != expected_magic {
        return Err(GraphFileParseError::InvalidMagic {
            expected: expected_magic,
            found,
        });
    }
    let version = u32::from_le_bytes(data[4..8].try_into().expect("header"));
    if version > QP_FILE_VERSION {
        return Err(GraphFileParseError::UnsupportedVersion {
            found: version,
            max: QP_FILE_VERSION,
        });
    }
    Ok(postcard::from_bytes(&data[HEADER_LEN..])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Position, NODE_START};

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
}
