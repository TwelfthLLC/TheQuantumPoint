use crate::graph_files::{
    self, create_graph_file, ensure_default_graphs_layout, list_graph_files, load_graph_at,
    read_entry_graph, save_graph_at, set_entry_graph, GraphFileError, GraphFileInfo,
};
use graph_model::{
    decode_registry_meta, decode_registry_meta_legacy_json, registry_meta_to_bytes, Project,
    RegistryMeta, REGISTRY_META_FILE,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub updated_at: u64,
    pub node_count: usize,
    /// Loyiha fayllari joylashgan papka (mutlaq yo‘l)
    pub folder: PathBuf,
}

#[derive(Clone)]
pub struct ProjectStore {
    workspace: PathBuf,
    root: PathBuf,
}

impl ProjectStore {
    pub fn new(workspace_root: &Path) -> Self {
        Self {
            workspace: workspace_root.to_path_buf(),
            root: workspace_root.join(".nocode").join("projects"),
        }
    }

    pub fn ensure(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.root)
    }

    pub fn project_build_dir(folder: &Path) -> PathBuf {
        folder.join(".nocode").join("build").join("rust")
    }

    pub fn list(&self) -> Result<Vec<ProjectMeta>, std::io::Error> {
        self.ensure()?;
        let mut items = Vec::new();
        let entries = match fs::read_dir(&self.root) {
            Ok(e) => e,
            Err(_) => return Ok(items),
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            if let Ok(meta) = self.meta_for_registry_dir(&entry.path(), &id) {
                items.push(meta);
            }
        }
        items.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
        Ok(items)
    }

    pub fn get(&self, id: &str) -> Result<Project, ProjectStoreError> {
        let folder = self.resolve_project_folder(id)?;
        let entry = read_entry_graph(&folder);
        self.load_graph(id, &entry)
    }

    pub fn load_graph(&self, id: &str, relative_path: &str) -> Result<Project, ProjectStoreError> {
        let folder = self.resolve_project_folder(id)?;
        load_graph_at(&folder, relative_path).map_err(graph_file_err)
    }

    pub fn save_graph(
        &self,
        id: &str,
        relative_path: &str,
        project: &Project,
    ) -> Result<ProjectMeta, ProjectStoreError> {
        let folder = self.resolve_project_folder(id)?;
        save_graph_at(&folder, relative_path, project).map_err(graph_file_err)?;
        let entry = read_entry_graph(&folder);
        let _ = set_entry_graph(&folder, &entry);
        self.meta_for_folder(&folder, id)
    }

    pub fn list_graph_files(&self, id: &str) -> Result<Vec<GraphFileInfo>, ProjectStoreError> {
        let folder = self.resolve_project_folder(id)?;
        let entry = read_entry_graph(&folder);
        list_graph_files(&folder, &entry).map_err(ProjectStoreError::Io)
    }

    pub fn entry_graph_path(&self, id: &str) -> Result<String, ProjectStoreError> {
        let folder = self.resolve_project_folder(id)?;
        Ok(read_entry_graph(&folder))
    }

    pub fn create_graph_file(
        &self,
        id: &str,
        name: &str,
        layer: graph_model::GraphLayer,
    ) -> Result<String, ProjectStoreError> {
        let folder = self.resolve_project_folder(id)?;
        let rel = create_graph_file(&folder, name, layer).map_err(graph_file_err)?;
        let entry = read_entry_graph(&folder);
        let _ = set_entry_graph(&folder, &entry);
        Ok(rel)
    }

    pub fn set_entry_graph(&self, id: &str, relative_path: &str) -> Result<(), ProjectStoreError> {
        let folder = self.resolve_project_folder(id)?;
        set_entry_graph(&folder, relative_path).map_err(graph_file_err)
    }

    pub fn folder_for(&self, id: &str) -> Result<PathBuf, ProjectStoreError> {
        self.resolve_project_folder(id)
    }

    /// Yangi loyiha: foydalanuvchi papkasida maxsus fayllar + registry yozuvi
    pub fn create_in_folder(
        &self,
        name: &str,
        folder: &Path,
        project: Project,
    ) -> Result<ProjectMeta, ProjectStoreError> {
        self.ensure().map_err(ProjectStoreError::Io)?;
        let folder = self.normalize_folder(folder)?;
        let folder = resolve_project_directory(&folder, name);
        if folder.exists() && graph_files::is_project_root(&folder) {
            return Err(ProjectStoreError::FolderExists(folder));
        }
        fs::create_dir_all(&folder).map_err(ProjectStoreError::Io)?;

        let id = self.unique_id(name)?;
        self.write_project_scaffold(&folder, &id, name, &project)?;
        self.write_registry_meta(&id, name, &folder)?;
        self.meta_for_folder(&folder, &id)
    }

    /// Eski API: papka berilmasa workspace ostida yaratiladi
    pub fn create(&self, name: &str, project: Project) -> Result<ProjectMeta, ProjectStoreError> {
        let id = self.unique_id(name)?;
        let folder = self.workspace.join("projects").join(&id);
        self.create_in_folder(name, &folder, project)
    }

    pub fn save(&self, id: &str, project: &Project) -> Result<ProjectMeta, ProjectStoreError> {
        let entry = self.entry_graph_path(id)?;
        self.save_graph(id, &entry, project)
    }

    pub fn delete(&self, id: &str) -> Result<(), ProjectStoreError> {
        let registry_dir = self.registry_dir(id);
        if !registry_dir.exists() {
            return Err(ProjectStoreError::NotFound(id.to_string()));
        }
        let folder = self.resolve_project_folder(id)?;
        let registry_canon = registry_dir
            .canonicalize()
            .unwrap_or_else(|_| registry_dir.clone());
        let folder_canon = folder.canonicalize().unwrap_or_else(|_| folder.clone());

        fs::remove_dir_all(&registry_dir).map_err(ProjectStoreError::Io)?;

        // Launcher ro‘yxati registryda; asl fayllar odatda tashqarida (masalan Documents/…).
        if folder_canon != registry_canon && folder_canon.exists() {
            fs::remove_dir_all(&folder_canon).map_err(ProjectStoreError::Io)?;
        }
        Ok(())
    }

    pub fn default_empty(name: &str, layer: graph_model::GraphLayer) -> Project {
        Project {
            name: name.to_string(),
            layer,
            nodes: vec![graph_model::Node {
                id: "start".to_string(),
                kind: graph_model::NODE_START.to_string(),
                position: graph_model::Position { x: 120.0, y: 200.0 },
                data: Default::default(),
            }],
            edges: vec![],
            subgraphs: vec![],
        }
    }

    pub fn seed_if_empty(&self, workspace: &Path) -> Result<(), ProjectStoreError> {
        if !self.list().map_err(ProjectStoreError::Io)?.is_empty() {
            return Ok(());
        }
        let hello = workspace.join("examples/hello-rust/graphs/main.qp");
        if hello.exists() {
            let raw = fs::read(hello).map_err(ProjectStoreError::Io)?;
            let mut project = Project::from_qp_bytes(&raw).map_err(ProjectStoreError::GraphFile)?;
            project.name = "Namuna: Hello Rust".to_string();
            let folder = self.root.join("namuna-hello-rust");
            let _ = self.create_in_folder("Namuna: Hello Rust", &folder, project)?;
        }
        Ok(())
    }

    fn write_project_scaffold(
        &self,
        folder: &Path,
        id: &str,
        name: &str,
        project: &Project,
    ) -> Result<(), ProjectStoreError> {
        graph_files::ensure_project_directories(folder).map_err(graph_file_err)?;
        ensure_default_graphs_layout(folder, id, name, project).map_err(graph_file_err)?;

        let readme = format!(
            "# {name}\n\nQuantum Point vizual loyiha.\n\n## Fayllar\n\n- `graphs/*.qp` — binar graf (`QPGR` + postcard)\n- `graphs/main.qp` — asosiy graf (Run shu fayldan)\n- `quantum-point.qp` — binar manifest (`QPRJ` + postcard)\n- `.nocode/build/` — build natijalari\n"
        );
        fs::write(folder.join("README.md"), readme).map_err(ProjectStoreError::Io)?;

        let gitignore = "/.nocode/\n/target/\n*.rs.bk\n";
        fs::write(folder.join(".gitignore"), gitignore).map_err(ProjectStoreError::Io)?;

        let build_dir = graph_files::build_rust_directory(folder);
        fs::write(build_dir.join(".gitkeep"), "").map_err(ProjectStoreError::Io)?;

        Ok(())
    }

    fn write_registry_meta(
        &self,
        id: &str,
        name: &str,
        folder: &Path,
    ) -> Result<(), ProjectStoreError> {
        let registry = self.registry_dir(id);
        fs::create_dir_all(&registry).map_err(ProjectStoreError::Io)?;
        let folder_str = folder
            .canonicalize()
            .unwrap_or_else(|_| folder.to_path_buf())
            .to_string_lossy()
            .to_string();
        let meta = RegistryMeta {
            id: id.to_string(),
            name: name.to_string(),
            updated_at: unix_now(),
            folder: Some(folder_str),
        };
        let bytes = registry_meta_to_bytes(&meta).map_err(ProjectStoreError::GraphFile)?;
        fs::write(registry.join(REGISTRY_META_FILE), bytes).map_err(ProjectStoreError::Io)?;
        let _ = fs::remove_file(registry.join("meta.json"));
        Ok(())
    }

    fn load_registry_meta(
        &self,
        registry_dir: &Path,
        id: &str,
    ) -> Result<Option<RegistryMeta>, ProjectStoreError> {
        let qp_path = registry_dir.join(REGISTRY_META_FILE);
        if qp_path.is_file() {
            let raw = fs::read(&qp_path).map_err(ProjectStoreError::Io)?;
            return decode_registry_meta(&raw)
                .map(Some)
                .map_err(ProjectStoreError::GraphFile);
        }
        let json_path = registry_dir.join("meta.json");
        if json_path.is_file() {
            let raw = fs::read(&json_path).map_err(ProjectStoreError::Io)?;
            let meta =
                decode_registry_meta_legacy_json(&raw).map_err(ProjectStoreError::GraphFile)?;
            let bytes = registry_meta_to_bytes(&meta).map_err(ProjectStoreError::GraphFile)?;
            fs::write(&qp_path, bytes).map_err(ProjectStoreError::Io)?;
            let _ = fs::remove_file(json_path);
            return Ok(Some(meta));
        }
        let _ = id;
        Ok(None)
    }

    fn meta_for_registry_dir(
        &self,
        registry_dir: &Path,
        id: &str,
    ) -> Result<ProjectMeta, std::io::Error> {
        let folder = self.read_folder_from_registry(registry_dir, id)?;
        self.meta_for_folder(&folder, id)
            .map_err(|e| std::io::Error::other(e.to_string()))
    }

    fn meta_for_folder(&self, folder: &Path, id: &str) -> Result<ProjectMeta, ProjectStoreError> {
        let registry_dir = self.registry_dir(id);
        let (name, updated_at) = if let Some(m) = self.load_registry_meta(&registry_dir, id)? {
            (m.name, m.updated_at)
        } else if graph_files::manifest_path(folder).is_file() {
            let m = graph_files::read_manifest(folder).map_err(graph_file_err)?;
            (m.name, 0)
        } else {
            (id.to_string(), 0)
        };
        let entry = read_entry_graph(folder);
        let node_count = load_graph_at(folder, &entry)
            .ok()
            .map(|p| p.nodes.len())
            .unwrap_or(0);
        Ok(ProjectMeta {
            id: id.to_string(),
            name,
            updated_at,
            node_count,
            folder: folder.to_path_buf(),
        })
    }

    fn read_folder_from_registry(
        &self,
        registry_dir: &Path,
        id: &str,
    ) -> Result<PathBuf, std::io::Error> {
        match self.load_registry_meta(registry_dir, id) {
            Ok(Some(m)) => {
                if let Some(f) = m.folder {
                    return Ok(PathBuf::from(f));
                }
            }
            Ok(None) => {}
            Err(e) => return Err(std::io::Error::other(e.to_string())),
        }
        Ok(registry_dir.to_path_buf())
    }

    fn resolve_project_folder(&self, id: &str) -> Result<PathBuf, ProjectStoreError> {
        let registry_dir = self.registry_dir(id);
        if !registry_dir.exists() {
            return Err(ProjectStoreError::NotFound(id.to_string()));
        }
        self.read_folder_from_registry(&registry_dir, id)
            .map_err(ProjectStoreError::Io)
    }

    fn registry_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    fn normalize_folder(&self, folder: &Path) -> Result<PathBuf, ProjectStoreError> {
        let path = if folder.is_absolute() {
            folder.to_path_buf()
        } else {
            self.workspace.join(folder)
        };
        let s = path.to_string_lossy();
        if s.contains("..") {
            return Err(ProjectStoreError::InvalidPath(s.to_string()));
        }
        Ok(path)
    }

    fn unique_id(&self, name: &str) -> Result<String, ProjectStoreError> {
        let base = slugify(name);
        let base = if base.is_empty() {
            "loyiha".to_string()
        } else {
            base
        };
        let mut candidate = base.clone();
        let mut n = 2;
        while self.registry_dir(&candidate).exists() {
            candidate = format!("{base}-{n}");
            n += 1;
        }
        Ok(candidate)
    }
}

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

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn appends_project_folder_under_parent() {
        let parent = Path::new("C:/Users/Me/Documents");
        let got = resolve_project_directory(parent, "Yangi loyiha");
        assert_eq!(got, parent.join("Yangi loyiha"));
    }

    #[test]
    fn keeps_path_when_already_named() {
        let root = Path::new("C:/Users/Me/Documents/Yangi loyiha");
        let got = resolve_project_directory(root, "Yangi loyiha");
        assert_eq!(got, root);
    }
}

fn slugify(name: &str) -> String {
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

fn graph_file_err(e: GraphFileError) -> ProjectStoreError {
    match e {
        GraphFileError::Io(err) => ProjectStoreError::Io(err),
        GraphFileError::NotFound(s) => ProjectStoreError::NotFound(s),
        GraphFileError::InvalidPath(s) => ProjectStoreError::InvalidPath(s),
        GraphFileError::Parse(e) => ProjectStoreError::GraphFile(e),
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectStoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph file: {0}")]
    GraphFile(#[from] graph_model::GraphFileParseError),
    #[error("project not found: {0}")]
    NotFound(String),
    #[error("invalid project id: {0}")]
    InvalidId(String),
    #[error("invalid folder path: {0}")]
    InvalidPath(String),
    #[error("folder already contains a Quantum Point project: {path}", path = .0.display())]
    FolderExists(PathBuf),
}
