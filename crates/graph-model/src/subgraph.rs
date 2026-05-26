use serde::{Deserialize, Serialize};

/// Compile unit reference — alohida `graphs/*.qp` moduli.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubGraphRef {
    pub id: String,
    /// Masalan `graphs/auth.qp`
    pub path: String,
    #[serde(default)]
    pub label: String,
}

impl SubGraphRef {
    pub fn new(id: impl Into<String>, path: impl Into<String>) -> Self {
        let id = id.into();
        let path = path.into();
        let label = path.rsplit('/').next().unwrap_or(&path).to_string();
        Self { id, path, label }
    }
}
