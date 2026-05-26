use serde::{Deserialize, Serialize};

use super::{
    decode_with_header, encode_with_header, GraphFileParseError, QP_FILE_VERSION,
    REGISTRY_META_MAGIC,
};

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
