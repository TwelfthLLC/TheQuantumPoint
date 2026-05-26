use std::path::Path;

use compiler::CompileCache;
use graph_model::Project;
use ir::Program;

use crate::compile_ctx::compile_context_for_root;
use crate::pipeline::PipelineError;

pub fn compile_project(project: &Project) -> Result<Program, PipelineError> {
    Ok(compiler::compile(project)?)
}

pub fn compile_project_cached(
    project: &Project,
    cache: &mut CompileCache,
    dirty_nodes: &[String],
    project_root: Option<&Path>,
) -> Result<Program, PipelineError> {
    let ctx_storage;
    let ctx = if let Some(root) = project_root {
        ctx_storage = compile_context_for_root(root);
        Some(&ctx_storage)
    } else {
        None
    };
    Ok(cache.compile_incremental(project, dirty_nodes, ctx)?)
}
