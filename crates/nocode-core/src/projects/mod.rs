//! Loyiha ro‘yxati va fayl tizimi (registry + tashqi papka).

mod paths;
mod store;

#[cfg(test)]
mod tests;

pub use paths::{
    default_projects_folder, folder_name_from_project, hello_template, resolve_project_directory,
    user_documents_dir,
};
pub use store::ProjectStore;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub updated_at: u64,
    pub node_count: usize,
    /// Loyiha fayllari joylashgan papka (mutlaq yo‘l)
    pub folder: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectStoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph file: {0}")]
    GraphFile(#[from] graph_model::GraphFileParseError),
    #[error("project not found: {0}")]
    NotFound(String),
    #[error("invalid project id: {0}")]
    InvalidId(String),
    #[error("invalid folder path: {0}")]
    InvalidPath(String),
    #[error("folder already contains a Quantum Point project: {path}", path = .0.display())]
    FolderExists(std::path::PathBuf),
}

pub(crate) fn graph_file_err(e: crate::graph_files::GraphFileError) -> ProjectStoreError {
    use crate::graph_files::GraphFileError;
    match e {
        GraphFileError::Io(err) => ProjectStoreError::Io(err),
        GraphFileError::NotFound(s) => ProjectStoreError::NotFound(s),
        GraphFileError::InvalidPath(s) => ProjectStoreError::InvalidPath(s),
        GraphFileError::Parse(e) => ProjectStoreError::GraphFile(e),
    }
}

pub(crate) fn unix_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
