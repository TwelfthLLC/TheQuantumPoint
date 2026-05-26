use crate::{Edge, GraphLayer, Node, SubGraphRef};
use serde::{Deserialize, Serialize};

use super::{encode_with_header, GraphFileParseError, GRAPH_MAGIC, QP_FILE_VERSION};

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
struct GraphPayloadV1 {
    name: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    #[serde(default)]
    layer: GraphLayer,
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

fn decode_graph_file_payload(data: &[u8]) -> Result<GraphPayload, GraphFileParseError> {
    use super::header::HEADER_LEN;

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
