use crate::compile_ctx::compile_context_for_root;
use crate::sandbox::{validate_build_dir, validate_profile, SandboxError};
use crate::target::{project_build_dir_for, BuildTarget};
use compiler::{CompileCache, CompileError};
use emit_bridge::{emit_bridge, BridgeOutput};
use emit_rust::emit;
use emit_view::{emit_view, parse_view_spec, ViewOutput};
use emit_wasm::write_wasm;
use graph_model::Project;
use ir::{Action, Program};
use qp_domain::Domain;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("compile: {0}")]
    Compile(#[from] CompileError),
    #[error("view: {0}")]
    View(#[from] emit_view::ViewEmitError),
    #[error("bridge: {0}")]
    Bridge(#[from] emit_bridge::BridgeEmitError),
    #[error("sandbox: {0}")]
    Sandbox(#[from] SandboxError),
    #[error("build target {target} requires {need:?} layer, got {got:?}")]
    TargetLayerMismatch {
        target: BuildTarget,
        need: Domain,
        got: Domain,
    },
    #[error("{0}")]
    Message(String),
}

/// Universal **Run** result: graph semantics / IR only, no target toolchain.
#[derive(Debug, Clone)]
pub struct CheckOutput {
    pub success: bool,
    pub domain: Domain,
    pub summary: String,
    /// Core: lowered universal IR (cached in `CompileCache`).
    pub program: Option<Program>,
    /// Core IR interpreter lines (Run preview).
    pub preview_lines: Vec<String>,
    /// View layer spec items for Studio preview.
    pub view_items: Vec<emit_view::ViewSpecItem>,
}

/// Parameters for [`build_project`].
pub struct BuildProjectParams<'a> {
    pub project: &'a Project,
    pub project_root: &'a Path,
    pub out_dir: &'a Path,
    pub profile: &'a str,
    pub target: BuildTarget,
    pub cache: &'a mut CompileCache,
    pub dirty_nodes: &'a [String],
    pub run_after_build: bool,
}

/// **Build** result: emit + optional native toolchain (e.g. cargo).
#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub preview_source: String,
    pub artifact_dir: String,
}

#[derive(Debug, Clone)]
pub enum DomainArtifacts {
    Core { program: Program, main_rs: String },
    View(ViewOutput),
    Bridge(BridgeOutput),
}

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

fn ensure_target_layer(project: &Project, target: BuildTarget) -> Result<(), PipelineError> {
    let got = Domain::from_layer(project.layer);
    let need = target.required_domain();
    if got != need {
        return Err(PipelineError::TargetLayerMismatch { target, need, got });
    }
    Ok(())
}

/// **Build**: emit artifacts for `target`, then run toolchain when applicable (Rust → cargo).
pub fn build_project(params: &mut BuildProjectParams<'_>) -> Result<BuildOutput, PipelineError> {
    let BuildProjectParams {
        project,
        project_root,
        out_dir,
        profile,
        target,
        cache,
        dirty_nodes,
        run_after_build,
    } = params;

    ensure_target_layer(project, *target)?;
    validate_profile(profile)?;
    let out_dir = validate_build_dir(project_root, out_dir)?;

    let artifacts = build_domain_artifacts(project, cache, dirty_nodes, Some(project_root))?;
    write_domain_outputs(&out_dir, &artifacts)?;

    let preview_source = match &artifacts {
        DomainArtifacts::Core { main_rs, .. } => main_rs.clone(),
        DomainArtifacts::View(v) => v.rust_stub.clone(),
        DomainArtifacts::Bridge(b) => b.routes_rs.clone(),
    };

    let artifact_dir = out_dir.display().to_string();

    if *target == BuildTarget::Wasm {
        if let DomainArtifacts::Core { program, .. } = &artifacts {
            write_wasm(&out_dir, program)?;
        }
        let mut log = String::new();
        let build = sandbox_cargo_command(&out_dir, profile)
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--message-format=short")
            .output()?;
        append_output(&mut log, "=== cargo build wasm32 ===", &build);
        let success = build.status.success();
        return Ok(BuildOutput {
            success,
            stdout: if success {
                format!("✓ WASM artefakt → {}", artifact_dir)
            } else {
                "✗ wasm32-unknown-unknown target o‘rnatilmagan bo‘lishi mumkin (rustup target add wasm32-unknown-unknown)"
                    .to_string()
            },
            stderr: log,
            exit_code: build.status.code().unwrap_or(if success { 0 } else { 1 }),
            preview_source,
            artifact_dir,
        });
    }

    if *target != BuildTarget::Rust {
        return Ok(BuildOutput {
            success: true,
            stdout: format!(
                "✓ {} → {}\n  (til toolchain: faqat Rust; bu target fayl yozadi)",
                target.label(),
                artifact_dir
            ),
            stderr: String::new(),
            exit_code: 0,
            preview_source,
            artifact_dir,
        });
    }

    let mut log = String::new();

    let build = sandbox_cargo_command(&out_dir, profile)
        .arg("build")
        .arg("--message-format=short")
        .output()?;
    append_output(&mut log, "=== cargo build (sandbox) ===", &build);
    if !build.status.success() {
        return Ok(BuildOutput {
            success: false,
            stdout: String::new(),
            stderr: log,
            exit_code: build.status.code().unwrap_or(1),
            preview_source,
            artifact_dir,
        });
    }

    if !*run_after_build {
        return Ok(BuildOutput {
            success: true,
            stdout: format!("✓ Rust build muvaffaqiyatli → {artifact_dir}"),
            stderr: log,
            exit_code: 0,
            preview_source,
            artifact_dir,
        });
    }

    let run = sandbox_cargo_command(&out_dir, profile)
        .arg("run")
        .output()?;
    append_output(&mut log, "=== cargo run (sandbox) ===", &run);

    let success = run.status.success();
    let mut stdout = String::from_utf8_lossy(&run.stdout).to_string();
    let program_stderr = String::from_utf8_lossy(&run.stderr).to_string();
    if !program_stderr.is_empty() {
        if !stdout.is_empty() {
            stdout.push('\n');
        }
        stdout.push_str(&program_stderr);
    }

    Ok(BuildOutput {
        success,
        stdout,
        stderr: log,
        exit_code: run.status.code().unwrap_or(if success { 0 } else { 1 }),
        preview_source,
        artifact_dir,
    })
}

/// Resolve build output directory from project root + target.
pub fn resolve_build_dir(project_root: &Path, target: BuildTarget) -> std::path::PathBuf {
    project_build_dir_for(project_root, target)
}

/// Deprecated: use `check_project` + `build_project`. Kept for one-shot tooling.
pub fn run_project(
    project: &Project,
    project_root: &Path,
    out_dir: &Path,
    profile: &str,
    cache: &mut CompileCache,
    dirty_nodes: &[String],
) -> Result<BuildOutput, PipelineError> {
    let _ = check_project(project, cache, dirty_nodes, Some(project_root))?;
    build_project(&mut BuildProjectParams {
        project,
        project_root,
        out_dir,
        profile,
        target: BuildTarget::default_for_layer(project.layer),
        cache,
        dirty_nodes,
        run_after_build: true,
    })
}

fn format_program_summary(program: &Program) -> String {
    let mut lines = vec![
        "✓ Core domain → universal IR (tilsiz)".to_string(),
        format!("• dastur: {}", program.name),
        format!("• amallar: {}", program.actions.len()),
    ];
    for (i, action) in program.actions.iter().enumerate() {
        lines.push(format!("  [{i}] {}", action_summary(action)));
    }
    lines.push("Build → Rust (yoki boshqa emit) + cargo.".to_string());
    lines.join("\n")
}

fn action_summary(action: &Action) -> String {
    match action {
        Action::Print { message } => format!("print {message:?}"),
        Action::DataStore { name, value } => format!("let {name} = {value:?}"),
        Action::Branch {
            condition,
            then_body,
            else_body,
        } => format!(
            "if {condition:?} then {} else {}",
            then_body.len(),
            else_body.len()
        ),
        Action::DbRead { table, into_var } => format!("db.read {table} → {into_var}"),
        Action::While { condition, body } => {
            format!("while {condition:?} body {} actions", body.len())
        }
        Action::For { var, from, to, body } => {
            format!("for {var} in {from}..={to} body {} actions", body.len())
        }
        Action::ForEach {
            item_var,
            collection,
            body,
        } => format!(
            "foreach {item_var} in {collection} body {} actions",
            body.len()
        ),
        Action::Return { value } => format!("return {value:?}"),
        Action::Switch { discriminant, arms, default_body } => format!(
            "switch {discriminant:?} {} arms, default {} actions",
            arms.len(),
            default_body.len()
        ),
        Action::Break => "break".into(),
        Action::Continue => "continue".into(),
        Action::Try {
            try_body,
            catch_body,
        } => format!("try {} / catch {} actions", try_body.len(), catch_body.len()),
        Action::Expr { name, value } => format!("expr {name} = {value:?}"),
        Action::Async { body } => format!("async block {} actions", body.len()),
        Action::Module { name, actions } => format!("module {name} ({} actions)", actions.len()),
    }
}

fn sandbox_cargo_command(out_dir: &Path, profile: &str) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(out_dir);
    cmd.env_remove("RUSTFLAGS");
    if profile == "release" {
        cmd.arg("--release");
    }
    cmd
}

fn append_output(log: &mut String, title: &str, output: &std::process::Output) {
    log.push_str(title);
    log.push('\n');
    if !output.stdout.is_empty() {
        log.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        log.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    if !log.ends_with('\n') {
        log.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_model::{
        data_set_str, Edge, GraphLayer, Node, Position, Project, NODE_LOG, NODE_START,
    };

    fn mini_core() -> Project {
        let mut log = Node {
            id: "log1".into(),
            kind: NODE_LOG.into(),
            position: Position { x: 100.0, y: 0.0 },
            data: Default::default(),
        };
        data_set_str(&mut log.data, "message", "hi");
        Project {
            name: "test".into(),
            nodes: vec![
                Node {
                    id: "start".into(),
                    kind: NODE_START.into(),
                    position: Position { x: 0.0, y: 0.0 },
                    data: Default::default(),
                },
                log,
            ],
            edges: vec![Edge {
                id: "e1".into(),
                source: "start".into(),
                target: "log1".into(),
                source_handle: "exec".into(),
                target_handle: "exec".into(),
            }],
            layer: GraphLayer::Core,
            subgraphs: vec![],
        }
    }

    #[test]
    fn check_core_no_cargo() {
        let p = mini_core();
        let mut cache = CompileCache::default();
        let out = check_project(&p, &mut cache, &[], None).expect("check");
        assert!(out.success);
        assert!(out.program.is_some());
        assert_eq!(out.program.as_ref().unwrap().actions.len(), 1);
    }
}
