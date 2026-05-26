use compiler::{CompileContext, CompileError};
use graph_model::Project;
use std::path::Path;

pub fn compile_context_for_root(project_root: &Path) -> CompileContext<'_> {
    let root = project_root.to_path_buf();
    CompileContext {
        project_root,
        load_subgraph: Box::new(move |rel_path: &str| load_subgraph_qp(&root, rel_path)),
    }
}

fn load_subgraph_qp(project_root: &Path, rel_path: &str) -> Result<Project, CompileError> {
    let path = project_root.join(rel_path);
    let bytes = std::fs::read(&path).map_err(|e| CompileError::SubGraphLoad {
        path: rel_path.to_string(),
        message: e.to_string(),
    })?;
    Project::from_qp_bytes(&bytes).map_err(|e| CompileError::SubGraphLoad {
        path: rel_path.to_string(),
        message: e.to_string(),
    })
}
