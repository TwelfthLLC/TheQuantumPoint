//! Graph (.qp) → domain actions → universal IR.

mod cache;
mod declarations;
mod error;
mod ir_map;
mod lower;
mod node_data;
mod safe_expr;
mod validate;
mod values;

#[cfg(test)]
mod language_tests;

pub use cache::CompileCache;
pub use error::{CompileContext, CompileError, SubGraphLoader};

use graph_model::{GraphError, Project, NODE_START};
use ir::{actions_need_async, Program};
use qp_domain::{Domain, DomainAction};
use std::collections::HashSet;

/// Graph (.qp) → domain actions → universal IR.
pub fn compile(project: &Project) -> Result<Program, CompileError> {
    compile_with_context(project, None)
}

/// Compile with subgraph loader (from `project.subgraphs` + `subgraph_call` nodes).
pub fn compile_with_context(
    project: &Project,
    ctx: Option<&CompileContext<'_>>,
) -> Result<Program, CompileError> {
    let domain = Domain::from_layer(project.layer);
    if domain != Domain::Core {
        return Err(CompileError::WrongDomain { domain });
    }
    validate::validate(project)?;
    let start = project
        .nodes
        .iter()
        .find(|n| n.kind == NODE_START)
        .ok_or(GraphError::NoStart)?;

    let decls = declarations::collect_declarations(project, ctx)?;
    let declarations::CollectedDeclarations {
        functions,
        structs,
        enums,
    } = decls;

    let mut domain_actions = lower::lower_exec_chain(project, &start.id, ctx)?;

    if let Some(ctx) = ctx {
        for sg in &project.subgraphs {
            let sub = (ctx.load_subgraph)(&sg.path)?;
            let sub_start = sub
                .nodes
                .iter()
                .find(|n| n.kind == NODE_START)
                .ok_or(GraphError::NoStart)?;
            let sub_actions = lower::lower_exec_chain(&sub, &sub_start.id, Some(ctx))?;
            domain_actions.push(DomainAction::Module {
                name: if sg.label.is_empty() {
                    sg.id.clone()
                } else {
                    sg.label.clone()
                },
                actions: sub_actions,
            });
        }
    }

    let actions: Vec<_> = domain_actions
        .into_iter()
        .map(ir_map::domain_action_to_ir)
        .collect();

    let needs_async_runtime =
        actions_need_async(&actions) || functions.iter().any(|f| actions_need_async(&f.body));

    Ok(Program {
        name: project.name.clone(),
        needs_async_runtime,
        functions,
        structs,
        enums,
        actions,
    })
}

/// Incremental compile (dirty exec successors invalidate cache).
pub fn compile_from_dirty(
    project: &Project,
    dirty_nodes: &[String],
) -> Result<Program, CompileError> {
    let mut c = CompileCache::default();
    c.compile_incremental(project, dirty_nodes, None)
}

/// Exec graph successors from `from` (for dirty invalidation).
pub fn exec_successors(project: &Project, from: &str) -> HashSet<String> {
    use std::collections::{HashSet, VecDeque};
    let mut out = HashSet::new();
    let mut q = VecDeque::new();
    q.push_back(from.to_string());
    while let Some(id) = q.pop_front() {
        for e in project.outgoing_exec(&id) {
            if out.insert(e.target.clone()) {
                q.push_back(e.target.clone());
            }
        }
        for label in [
            "true", "false", "done", "body", "case1", "case2", "case3", "case4", "case5", "case6",
            "default", "try", "catch",
        ] {
            if let Some(e) = project.outgoing_exec_labeled(&id, label) {
                if out.insert(e.target.clone()) {
                    q.push_back(e.target.clone());
                }
            }
        }
    }
    out.remove(from);
    out
}
