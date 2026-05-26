//! Binar `.qp` codec (`QPGR` / `QPRJ` / `QPME` + postcard).

mod graph;
mod header;
mod manifest;
mod registry;

#[cfg(test)]
mod tests;

pub use graph::{decode_graph_file, encode_graph_file};
pub use manifest::{decode_project_manifest, encode_project_manifest, ProjectManifest};
pub use registry::{
    decode_registry_meta, decode_registry_meta_legacy_json, encode_registry_meta,
    registry_meta_to_bytes, RegistryMeta,
};

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

pub(crate) use header::{decode_with_header, encode_with_header};
