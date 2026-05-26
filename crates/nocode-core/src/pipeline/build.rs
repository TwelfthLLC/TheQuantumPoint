use std::path::Path;
use std::process::Command;

use emit_wasm::write_wasm;

use super::artifacts::{build_domain_artifacts, write_domain_outputs};
use super::check::check_project;
use super::{ensure_target_layer, BuildOutput, BuildProjectParams, DomainArtifacts, PipelineError};
use crate::sandbox::{validate_build_dir, validate_profile};
use crate::target::{project_build_dir_for, BuildTarget};

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
    project: &graph_model::Project,
    project_root: &Path,
    out_dir: &Path,
    profile: &str,
    cache: &mut compiler::CompileCache,
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
