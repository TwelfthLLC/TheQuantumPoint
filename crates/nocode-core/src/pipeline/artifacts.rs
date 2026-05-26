use std::path::Path;

use compiler::CompileCache;
use emit_bridge::emit_bridge;
use emit_rust::emit;
use emit_view::emit_view;
use graph_model::Project;
use ir::Program;
use qp_domain::Domain;

use super::compile::compile_project_cached;
use super::{DomainArtifacts, PipelineError};
use crate::target::BuildTarget;

pub fn build_domain_artifacts(
    project: &Project,
    cache: &mut CompileCache,
    dirty_nodes: &[String],
    project_root: Option<&Path>,
) -> Result<DomainArtifacts, PipelineError> {
    match Domain::from_layer(project.layer) {
        Domain::Core => {
            let program = compile_project_cached(project, cache, dirty_nodes, project_root)?;
            let generated = emit(&program);
            Ok(DomainArtifacts::Core {
                program,
                main_rs: generated.main_rs,
            })
        }
        Domain::View => Ok(DomainArtifacts::View(emit_view(project)?)),
        Domain::Bridge => Ok(DomainArtifacts::Bridge(emit_bridge(project)?)),
    }
}

pub fn write_rust(out_dir: &Path, program: &Program) -> Result<String, PipelineError> {
    let generated = emit(program);
    let main_rs = generated.main_rs.clone();
    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(out_dir.join("Cargo.toml"), generated.cargo_toml)?;
    std::fs::write(src_dir.join("main.rs"), generated.main_rs)?;
    Ok(main_rs)
}

pub fn write_domain_outputs(
    out_dir: &Path,
    artifacts: &DomainArtifacts,
) -> Result<(), PipelineError> {
    std::fs::create_dir_all(out_dir)?;
    match artifacts {
        DomainArtifacts::Core {
            program,
            main_rs: _,
        } => {
            write_rust(out_dir, program)?;
        }
        DomainArtifacts::View(v) => {
            std::fs::write(out_dir.join("ui.qpview"), &v.spec)?;
            std::fs::write(out_dir.join("view_stub.rs"), &v.rust_stub)?;
        }
        DomainArtifacts::Bridge(b) => {
            std::fs::write(out_dir.join("bridge_manifest.txt"), &b.manifest)?;
            std::fs::write(out_dir.join("routes.rs"), &b.routes_rs)?;
            std::fs::write(out_dir.join("bridge_main.rs"), &b.run_stub)?;
        }
    }
    Ok(())
}

pub(crate) fn ensure_target_layer(
    project: &Project,
    target: BuildTarget,
) -> Result<(), PipelineError> {
    let got = Domain::from_layer(project.layer);
    let need = target.required_domain();
    if got != need {
        return Err(PipelineError::TargetLayerMismatch { target, need, got });
    }
    Ok(())
}
