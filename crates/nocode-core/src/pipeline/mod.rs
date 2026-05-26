//! Run / Build pipeline: check (IR), emit artifacts, optional cargo.

mod artifacts;
mod build;
mod check;
mod compile;
mod summary;

#[cfg(test)]
mod tests;

pub use artifacts::{build_domain_artifacts, write_domain_outputs, write_rust};
pub use build::{build_project, resolve_build_dir, run_project};
pub use check::check_project;
pub use compile::{compile_project, compile_project_cached};

use emit_bridge::BridgeOutput;
use emit_view::ViewOutput;
use ir::Program;
use thiserror::Error;

use crate::sandbox::SandboxError;
use crate::target::BuildTarget;
use compiler::CompileError;
use qp_domain::Domain;

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
    pub project: &'a graph_model::Project,
    pub project_root: &'a std::path::Path,
    pub out_dir: &'a std::path::Path,
    pub profile: &'a str,
    pub target: BuildTarget,
    pub cache: &'a mut compiler::CompileCache,
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

pub(crate) use artifacts::ensure_target_layer;
