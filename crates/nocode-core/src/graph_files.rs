use graph_model::{
    decode_project_manifest, encode_project_manifest, GraphFileParseError, GraphLayer, Project,
    ProjectManifest, GRAPH_FILE_EXTENSION, PROJECT_MANIFEST_FILE,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Product / manifest tool version (see repo `VERSION`).
pub const QP_VERSION: &str = "0.0.0.2";
pub const GRAPHS_DIR: &str = "graphs";
pub const DEFAULT_ENTRY: &str = "graphs/main.qp";
pub const MANIFEST_FILE: &str = PROJECT_MANIFEST_FILE;

pub use graph_model::{GRAPH_MAGIC, PROJECT_MAGIC, QP_FILE_VERSION};

#[derive(Debug, Clone)]
pub struct GraphFileInfo {
    pub path: String,
    pub label: String,
    pub is_entry: bool,
    pub layer: GraphLayer,
}

pub fn graphs_directory(folder: &Path) -> PathBuf {
    folder.join(GRAPHS_DIR)
}

pub fn build_rust_directory(folder: &Path) -> PathBuf {
    folder.join(".nocode").join("build").join("rust")
}

pub fn manifest_path(folder: &Path) -> PathBuf {
    folder.join(MANIFEST_FILE)
}

pub fn ensure_project_directories(folder: &Path) -> Result<(), GraphFileError> {
    fs::create_dir_all(graphs_directory(folder)).map_err(GraphFileError::Io)?;
    fs::create_dir_all(build_rust_directory(folder)).map_err(GraphFileError::Io)?;
    Ok(())
}

pub fn is_project_root(folder: &Path) -> bool {
    manifest_path(folder).is_file()
        || graphs_directory(folder)
            .join(format!("main.{GRAPH_FILE_EXTENSION}"))
            .is_file()
}

pub fn list_graph_files(folder: &Path, entry: &str) -> Result<Vec<GraphFileInfo>, std::io::Error> {
    let mut out = Vec::new();
    let graphs = graphs_directory(folder);
    if graphs.is_dir() {
        collect_graph_files(&graphs, folder, entry, &mut out)?;
    }
    if out.is_empty() {
        out.push(GraphFileInfo {
            path: DEFAULT_ENTRY.to_string(),
            label: format!("main.{GRAPH_FILE_EXTENSION} · Core"),
            is_entry: true,
            layer: GraphLayer::Core,
        });
    }
    out.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(out)
}

fn collect_graph_files(
    dir: &Path,
    project_root: &Path,
    entry: &str,
    out: &mut Vec<GraphFileInfo>,
) -> Result<(), std::io::Error> {
    for entry_fs in fs::read_dir(dir)? {
        let entry_fs = entry_fs?;
        let path = entry_fs.path();
        if path.extension().and_then(|e| e.to_str()) != Some(GRAPH_FILE_EXTENSION) {
            continue;
        }
        let rel = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let file_label = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| rel.clone());
        let layer = load_graph_at(project_root, &rel)
            .map(|p| p.layer)
            .unwrap_or(GraphLayer::Core);
        let label = format!("{file_label} · {}", layer.label());
        out.push(GraphFileInfo {
            is_entry: rel == entry,
            path: rel,
            label,
            layer,
        });
    }
    Ok(())
}

pub fn load_graph_at(folder: &Path, relative_path: &str) -> Result<Project, GraphFileError> {
    let path = resolve_graph_path(folder, relative_path)?;
    if !path.exists() {
        return Err(GraphFileError::NotFound(relative_path.to_string()));
    }
    let raw = fs::read(&path).map_err(GraphFileError::Io)?;
    Project::from_qp_bytes(&raw).map_err(GraphFileError::Parse)
}

pub fn save_graph_at(
    folder: &Path,
    relative_path: &str,
    project: &Project,
) -> Result<(), GraphFileError> {
    let path = resolve_graph_path(folder, relative_path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(GraphFileError::Io)?;
    }
    let body = project
        .to_qp_bytes()
        .map_err(GraphFileParseError::from)
        .map_err(GraphFileError::Parse)?;
    fs::write(&path, body).map_err(GraphFileError::Io)?;
    Ok(())
}

pub fn create_graph_file(
    folder: &Path,
    name: &str,
    layer: GraphLayer,
) -> Result<String, GraphFileError> {
    let base = slugify_file(name);
    let base = if base.is_empty() {
        "graf".to_string()
    } else {
        base
    };
    let graphs = graphs_directory(folder);
    fs::create_dir_all(&graphs).map_err(GraphFileError::Io)?;

    let mut candidate = format!("{base}.{GRAPH_FILE_EXTENSION}");
    let mut n = 2;
    while graphs.join(&candidate).exists() {
        candidate = format!("{base}-{n}.{GRAPH_FILE_EXTENSION}");
        n += 1;
    }

    let rel = format!("{GRAPHS_DIR}/{candidate}");
    let project = Project {
        name: base.clone(),
        layer,
        nodes: vec![graph_model::Node {
            id: "start".to_string(),
            kind: graph_model::NODE_START.to_string(),
            position: graph_model::Position { x: 120.0, y: 200.0 },
            data: Default::default(),
        }],
        edges: vec![],
        subgraphs: vec![],
    };
    save_graph_at(folder, &rel, &project)?;
    Ok(rel)
}

pub fn ensure_default_graphs_layout(
    folder: &Path,
    id: &str,
    project_name: &str,
    initial: &Project,
) -> Result<String, GraphFileError> {
    ensure_project_directories(folder)?;

    let entry = DEFAULT_ENTRY.to_string();
    let mut main = initial.clone();
    main.name = format!("{project_name} — main");
    save_graph_at(folder, &entry, &main)?;

    sync_manifest_graphs(folder, id, project_name, &entry)?;
    Ok(entry)
}

pub fn read_manifest(folder: &Path) -> Result<ProjectManifest, GraphFileError> {
    let path = manifest_path(folder);
    let raw = fs::read(&path).map_err(GraphFileError::Io)?;
    decode_project_manifest(&raw).map_err(GraphFileError::Parse)
}

pub fn read_entry_graph(folder: &Path) -> String {
    if let Ok(m) = read_manifest(folder) {
        return m.entry_graph.replace('\\', "/");
    }
    DEFAULT_ENTRY.to_string()
}

pub fn set_entry_graph(folder: &Path, entry: &str) -> Result<(), GraphFileError> {
    let path = manifest_path(folder);
    let mut manifest = if path.is_file() {
        read_manifest(folder)?
    } else {
        ProjectManifest {
            id: String::new(),
            name: String::new(),
            qp_tool_version: QP_VERSION.to_string(),
            entry_graph: DEFAULT_ENTRY.to_string(),
            graphs: vec![],
        }
    };
    manifest.entry_graph = entry.replace('\\', "/");
    manifest.graphs = list_graph_files(folder, &manifest.entry_graph)
        .unwrap_or_default()
        .into_iter()
        .map(|g| g.path)
        .collect();
    write_manifest(folder, &manifest)
}

fn write_manifest(folder: &Path, manifest: &ProjectManifest) -> Result<(), GraphFileError> {
    let bytes = encode_project_manifest(manifest).map_err(GraphFileParseError::from)?;
    fs::write(manifest_path(folder), bytes).map_err(GraphFileError::Io)
}

fn sync_manifest_graphs(
    folder: &Path,
    id: &str,
    name: &str,
    entry: &str,
) -> Result<(), GraphFileError> {
    let files: Vec<String> = list_graph_files(folder, entry)
        .unwrap_or_default()
        .into_iter()
        .map(|g| g.path)
        .collect();
    let manifest = ProjectManifest {
        id: id.to_string(),
        name: name.to_string(),
        qp_tool_version: QP_VERSION.to_string(),
        entry_graph: entry.to_string(),
        graphs: files,
    };
    write_manifest(folder, &manifest)
}

fn resolve_graph_path(folder: &Path, relative_path: &str) -> Result<PathBuf, GraphFileError> {
    let norm = relative_path.replace('\\', "/");
    if norm.contains("..") {
        return Err(GraphFileError::InvalidPath(norm.clone()));
    }
    if !norm.ends_with(&format!(".{GRAPH_FILE_EXTENSION}")) {
        return Err(GraphFileError::InvalidPath(format!(
            "graf fayli .{GRAPH_FILE_EXTENSION} bo‘lishi kerak: {norm}"
        )));
    }
    Ok(folder.join(norm))
}

fn slugify_file(name: &str) -> String {
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

#[derive(Debug, thiserror::Error)]
pub enum GraphFileError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph file: {0}")]
    Parse(#[from] graph_model::GraphFileParseError),
    #[error("graph file not found: {0}")]
    NotFound(String),
    #[error("invalid path: {0}")]
    InvalidPath(String),
}
