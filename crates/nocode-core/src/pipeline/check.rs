use std::path::Path;

use compiler::CompileCache;
use emit_bridge::emit_bridge;
use emit_view::{emit_view, parse_view_spec};
use graph_model::Project;
use qp_domain::Domain;

use super::compile::compile_project_cached;
use super::summary::format_program_summary;
use super::{CheckOutput, PipelineError};

/// Universal check (Run): validate graph + domain lowering, no files, no cargo.
pub fn check_project(
    project: &Project,
    cache: &mut CompileCache,
    dirty_nodes: &[String],
    project_root: Option<&Path>,
) -> Result<CheckOutput, PipelineError> {
    let domain = Domain::from_layer(project.layer);
    match domain {
        Domain::Core => {
            let program = compile_project_cached(project, cache, dirty_nodes, project_root)?;
            let mut summary = format_program_summary(&program);
            let preview_lines = match qp_runtime::interpret(&program) {
                Ok(p) => {
                    if !p.lines.is_empty() {
                        summary.push_str("\n\n— Run preview (IR) —");
                        for line in &p.lines {
                            summary.push('\n');
                            summary.push_str(line);
                        }
                    }
                    p.lines
                }
                Err(e) => {
                    summary.push_str(&format!("\n(preview: {e})"));
                    Vec::new()
                }
            };
            Ok(CheckOutput {
                success: true,
                domain,
                summary,
                program: Some(program),
                preview_lines,
                view_items: Vec::new(),
            })
        }
        Domain::View => {
            let out = emit_view(project)?;
            let items = parse_view_spec(&out.spec);
            let ui_nodes = project
                .nodes
                .iter()
                .filter(|n| n.kind != graph_model::NODE_START)
                .count();
            let mut summary = format!(
                "✓ View domain tekshirildi\n\
                 • UI nodlar: {ui_nodes}\n\
                 • spec: {} bayt, stub: {} bayt\n\
                 Build → View spec fayllar yoziladi (cargo yo‘q).",
                out.spec.len(),
                out.rust_stub.len()
            );
            if !items.is_empty() {
                summary.push_str("\n\n— View preview —");
                for it in &items {
                    summary.push('\n');
                    summary.push_str(&format!("  [{}] {} — {}", it.kind, it.id, it.title));
                }
            }
            Ok(CheckOutput {
                success: true,
                domain,
                summary,
                program: None,
                preview_lines: Vec::new(),
                view_items: items,
            })
        }
        Domain::Bridge => {
            let out = emit_bridge(project)?;
            let routes = project
                .nodes
                .iter()
                .filter(|n| {
                    n.kind == graph_model::NODE_API_ROUTE || n.kind == graph_model::NODE_API_QUERY
                })
                .count();
            let summary = format!(
                "✓ Bridge domain tekshirildi\n\
                 • route/query nodlar: {routes}\n\
                 • routes.rs: {} bayt\n\
                 Build → Bridge artefaktlar yoziladi (cargo yo‘q).",
                out.routes_rs.len()
            );
            Ok(CheckOutput {
                success: true,
                domain,
                summary,
                program: None,
                preview_lines: Vec::new(),
                view_items: Vec::new(),
            })
        }
    }
}
