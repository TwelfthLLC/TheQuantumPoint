//! Build/run sandbox — path confinement and validated cargo invocation.

use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("path traversal blocked: {0}")]
    PathTraversal(String),
    #[error("build directory must stay under project root")]
    BuildDirOutsideProject,
    #[error("invalid profile: {0}")]
    InvalidProfile(String),
}

/// Build output must live under `project_root/.nocode/build/`.
pub fn validate_build_dir(project_root: &Path, build_dir: &Path) -> Result<PathBuf, SandboxError> {
    let project_abs = to_absolute(project_root)?;
    let build_abs = to_absolute(build_dir)?;

    let allowed = project_abs.join(".nocode").join("build");
    let expected_rust = allowed.join("rust");

    let under_project = build_abs.starts_with(&project_abs);
    let under_build = build_abs.starts_with(&allowed)
        || build_abs == expected_rust
        || build_abs.starts_with(&expected_rust);

    if !under_project || !under_build {
        return Err(SandboxError::BuildDirOutsideProject);
    }

    if let Some(parent) = expected_rust.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(&expected_rust)?;

    Ok(expected_rust)
}

pub fn validate_profile(profile: &str) -> Result<(), SandboxError> {
    match profile {
        "dev" | "release" => Ok(()),
        other => Err(SandboxError::InvalidProfile(other.to_string())),
    }
}

/// Reject `..` in relative paths used by the emitter pipeline.
pub fn validate_relative_path(rel: &str) -> Result<(), SandboxError> {
    let p = Path::new(rel);
    for c in p.components() {
        if matches!(c, Component::ParentDir) {
            return Err(SandboxError::PathTraversal(rel.to_string()));
        }
    }
    Ok(())
}

fn to_absolute(path: &Path) -> Result<PathBuf, std::io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
