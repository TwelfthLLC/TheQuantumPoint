use serde::{Deserialize, Serialize};

use super::{
    decode_with_header, encode_with_header, GraphFileParseError, PROJECT_MAGIC, QP_FILE_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub id: String,
    pub name: String,
    pub qp_tool_version: String,
    pub entry_graph: String,
    pub graphs: Vec<String>,
}

pub fn encode_project_manifest(manifest: &ProjectManifest) -> Result<Vec<u8>, postcard::Error> {
    encode_with_header(PROJECT_MAGIC, QP_FILE_VERSION, manifest)
}

pub fn decode_project_manifest(data: &[u8]) -> Result<ProjectManifest, GraphFileParseError> {
    decode_with_header(data, PROJECT_MAGIC)
}
