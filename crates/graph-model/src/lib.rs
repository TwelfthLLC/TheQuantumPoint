mod data;
mod format;
mod layer;
mod node_catalog;
mod subgraph;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use data::{data_get_i64, data_get_str, data_set_str, DataValue, NodeData};
pub use subgraph::SubGraphRef;

pub use format::{
    decode_graph_file, decode_project_manifest, decode_registry_meta,
    decode_registry_meta_legacy_json, encode_graph_file, encode_project_manifest,
    encode_registry_meta, registry_meta_to_bytes, GraphFileParseError, ProjectManifest,
    RegistryMeta, GRAPH_FILE_EXTENSION, GRAPH_MAGIC, PROJECT_MAGIC, PROJECT_MANIFEST_FILE,
    QP_FILE_VERSION, REGISTRY_META_FILE, REGISTRY_META_MAGIC,
};
pub use layer::GraphLayer;
pub use node_catalog::{maturity_label, node_maturity, node_support_hint, NodeMaturity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub layer: GraphLayer,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub subgraphs: Vec<SubGraphRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub position: Position,
    #[serde(default)]
    pub data: NodeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "sourceHandle", default)]
    pub source_handle: String,
    #[serde(rename = "targetHandle", default)]
    pub target_handle: String,
}

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("project must contain at least one node")]
    Empty,
    #[error("missing start node")]
    NoStart,
    #[error("multiple start nodes: {0}")]
    MultipleStarts(String),
    #[error("unknown node type '{kind}' on node '{id}'")]
    UnknownNodeType { id: String, kind: String },
    #[error("node '{0}' not found")]
    NodeNotFound(String),
    #[error("edge '{id}' references missing node")]
    BrokenEdge { id: String },
    #[error("node '{id}' ({kind}): {detail}")]
    InvalidNode {
        id: String,
        kind: String,
        detail: String,
    },
    #[error("node '{id}' has no incoming exec edge")]
    UnreachableExec { id: String },
    #[error("node '{id}' has multiple incoming exec edges")]
    AmbiguousExec { id: String },
}

impl Project {
    /// Binar `.qp` (`QPGR` + postcard).
    pub fn from_qp_bytes(data: &[u8]) -> Result<Self, GraphFileParseError> {
        let (name, layer, nodes, edges, subgraphs) = decode_graph_file(data)?;
        let mut project = Self {
            name,
            layer,
            nodes,
            edges,
            subgraphs,
        };
        project.ensure_start_node();
        Ok(project)
    }

    /// Bo‘sh grafda ham Start nodi bo‘ladi (sahna bo‘sh qolmasin).
    pub fn ensure_start_node(&mut self) {
        if self.nodes.iter().any(|n| n.kind == NODE_START) {
            return;
        }
        self.nodes.push(Node {
            id: "start".to_string(),
            kind: NODE_START.to_string(),
            position: Position { x: 120.0, y: 200.0 },
            data: Default::default(),
        });
    }

    pub fn to_qp_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        encode_graph_file(
            &self.name,
            self.layer,
            &self.nodes,
            &self.edges,
            &self.subgraphs,
        )
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn outgoing_exec(&self, node_id: &str) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id && is_exec_handle(&e.source_handle))
            .collect()
    }

    pub fn incoming_exec(&self, node_id: &str) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| e.target == node_id && is_exec_target(&e.target_handle))
            .collect()
    }

    pub fn outgoing_exec_labeled(&self, node_id: &str, label: &str) -> Option<&Edge> {
        self.edges.iter().find(|e| {
            e.source == node_id
                && e.source_handle.eq_ignore_ascii_case(label)
                && is_exec_handle(&e.source_handle)
        })
    }
}

pub fn is_exec_handle(handle: &str) -> bool {
    matches!(
        handle.to_ascii_lowercase().as_str(),
        "exec" | "true" | "false" | "body" | "done"
    )
}

pub fn is_exec_target(handle: &str) -> bool {
    matches!(
        handle.to_ascii_lowercase().as_str(),
        "exec" | "true" | "false" | "body"
    )
}

pub const NODE_START: &str = "start";

// Core (Backend)
pub const NODE_LOG: &str = "log";
pub const NODE_ASSIGN: &str = "assign";
pub const NODE_IF: &str = "if";
pub const NODE_API_ROUTE: &str = "api_route";
pub const NODE_API_QUERY: &str = "api_query";
pub const NODE_DB_READ: &str = "db_read";
pub const NODE_SUBGRAPH: &str = "subgraph_call";
pub const NODE_EMIT_UI: &str = "emit_ui";

// Surface (Frontend)
pub const NODE_UI_PAGE: &str = "ui_page";
pub const NODE_UI_BUTTON: &str = "ui_button";
pub const NODE_UI_LABEL: &str = "ui_label";
pub const NODE_UI_INPUT: &str = "ui_input";
pub const NODE_UI_EVENT: &str = "ui_event";
