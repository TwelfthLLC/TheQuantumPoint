use crate::{compile_with_context, CompileContext, CompileError};
use graph_model::Project;
use ir::Program;
/// Incremental compile cache — invalidates exec successors on dirty nodes.
#[derive(Debug, Default, Clone)]
pub struct CompileCache {
    revision: u64,
    program: Option<Program>,
    last_dirty_count: usize,
}

impl CompileCache {
    pub fn clear(&mut self) {
        self.revision = 0;
        self.program = None;
        self.last_dirty_count = 0;
    }

    pub fn invalidate_graph(&mut self, dirty_nodes: &[String], project: &Project) {
        if dirty_nodes.is_empty() && self.program.is_some() {
            return;
        }
        if dirty_nodes.is_empty() {
            self.clear();
            return;
        }
        let mut touched = false;
        for id in dirty_nodes {
            if self.program.is_some() {
                touched = true;
            }
            let succ = crate::exec_successors(project, id);
            if !succ.is_empty() || !dirty_nodes.is_empty() {
                touched = true;
            }
        }
        if touched {
            self.program = None;
            self.revision = self.revision.saturating_add(1);
        }
        self.last_dirty_count = dirty_nodes.len();
    }

    pub fn compile_incremental(
        &mut self,
        project: &Project,
        dirty_nodes: &[String],
        ctx: Option<&CompileContext<'_>>,
    ) -> Result<Program, CompileError> {
        self.invalidate_graph(dirty_nodes, project);
        if let Some(p) = self.program.clone() {
            if dirty_nodes.is_empty() {
                return Ok(p);
            }
        }
        let p = compile_with_context(project, ctx)?;
        self.program = Some(p.clone());
        Ok(p)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}
