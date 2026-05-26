use crate::graph_files;
use graph_model::Project;
use std::fs;
use std::path::{Path, PathBuf};

pub fn hello_template(workspace: &Path, name: &str) -> Option<Project> {
    let path = workspace.join("examples/hello-rust/graphs/main.qp");
    let raw = fs::read(path).ok()?;
    let mut project = Project::from_qp_bytes(&raw).ok()?;
    project.name = name.to_string();
    Some(project)
}

/// Foydalanuvchi `Documents` papkasi (Windows/macOS/Linux fallback).
pub fn user_documents_dir() -> PathBuf {
    if let Some(profile) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let docs = PathBuf::from(profile).join("Documents");
        if docs.is_dir() {
            return docs;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Loyiha papkasi nomi (masalan `Yangi loyiha`) — noto‘g‘ri belgilar olib tashlanadi.
pub fn folder_name_from_project(name: &str) -> String {
    const INVALID: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "loyiha".to_string();
    }
    let mut out = String::with_capacity(trimmed.len());
    for c in trimmed.chars() {
        if c == '\0' || INVALID.contains(&c) {
            continue;
        }
        out.push(c);
    }
    let out = out.trim().trim_end_matches('.').to_string();
    if out.is_empty() {
        "loyiha".to_string()
    } else {
        out
    }
}

/// `Documents` + loyiha nomi yoki tanlangan ota-ona + loyiha nomi.
pub fn default_projects_folder(_workspace: &Path, name: &str) -> PathBuf {
    user_documents_dir().join(folder_name_from_project(name))
}

/// Tanlangan yo‘l ota-ona bo‘lsa, ichiga loyiha nomi bilan papka qo‘shiladi.
pub fn resolve_project_directory(parent_or_root: &Path, project_name: &str) -> PathBuf {
    let folder_name = folder_name_from_project(project_name);
    if graph_files::is_project_root(parent_or_root) {
        return parent_or_root.to_path_buf();
    }
    if parent_or_root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case(folder_name.as_str()))
    {
        return parent_or_root.to_path_buf();
    }
    parent_or_root.join(&folder_name)
}

pub(crate) fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in name.chars() {
        let ch = if c.is_ascii_alphanumeric() {
            c.to_ascii_lowercase()
        } else if c == ' ' || c == '-' || c == '_' {
            '-'
        } else {
            continue;
        };
        if ch == '-' {
            if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
        } else {
            out.push(ch);
            last_dash = false;
        }
    }
    out.trim_matches('-').to_string()
}
