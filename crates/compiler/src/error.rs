use crate::safe_expr::ParseConditionError;
use graph_model::{GraphError, Project};
use qp_domain::Domain;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error("compile: {0}")]
    Message(String),
    #[error("condition: {0}")]
    Condition(#[from] ParseConditionError),
    #[error("domain {domain:?} cannot be lowered to native exec yet (use Core entry graph)")]
    WrongDomain { domain: Domain },
    #[error("node '{id}' ({kind}): {detail}")]
    UnsupportedNode {
        id: String,
        kind: String,
        detail: String,
    },
    #[error("subgraph '{path}': {message}")]
    SubGraphLoad { path: String, message: String },
}

/// Loads a subgraph `.qp` by project-relative path.
pub type SubGraphLoader<'a> = Box<dyn Fn(&str) -> Result<Project, CompileError> + 'a>;

/// Optional project-root context for loading `subgraphs[]` modules.
pub struct CompileContext<'a> {
    pub project_root: &'a Path,
    pub load_subgraph: SubGraphLoader<'a>,
}

impl CompileContext<'_> {
    pub fn noop(_root: &Path) -> CompileContext<'_> {
        CompileContext {
            project_root: Path::new("."),
            load_subgraph: Box::new(|path| {
                Err(CompileError::SubGraphLoad {
                    path: path.to_string(),
                    message: "subgraph loader not configured".into(),
                })
            }),
        }
    }
}
