use crate::graph_files::{list_graph_files, read_entry_graph, GRAPHS_DIR};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentItemKind {
    Folder,
    Graph,
    Config,
    Doc,
    Build,
    Other,
}

#[derive(Debug, Clone)]
pub struct ContentItem {
    pub name: String,
    /// Loyiha root ga nisbatan (`graphs/main.qp`); papkalar uchun bo‘sh
    pub path: String,
    pub kind: ContentItemKind,
    pub is_entry: bool,
    pub openable: bool,
    pub children: Vec<ContentItem>,
}

#[derive(Debug, Clone)]
pub struct ContentSection {
    pub title: String,
    pub items: Vec<ContentItem>,
}

pub fn scan_project_browser(folder: &Path, search: &str) -> Vec<ContentSection> {
    let entry = read_entry_graph(folder);
    let query = search.trim().to_lowercase();

    let mut sections = Vec::new();

    let graph_items = build_graphs_section(folder, &entry, &query);
    if !graph_items.is_empty() {
        sections.push(ContentSection {
            title: "Graf fayllar".to_string(),
            items: graph_items,
        });
    }

    let project_items = build_project_files_section(folder, &query);
    if !project_items.is_empty() {
        sections.push(ContentSection {
            title: "Loyiha".to_string(),
            items: project_items,
        });
    }

    let build_items = build_build_section(folder, &query);
    if !build_items.is_empty() {
        sections.push(ContentSection {
            title: "Build".to_string(),
            items: build_items,
        });
    }

    sections
}

fn build_graphs_section(folder: &Path, entry: &str, query: &str) -> Vec<ContentItem> {
    let Ok(files) = list_graph_files(folder, entry) else {
        return Vec::new();
    };
    let mut items: Vec<ContentItem> = files
        .into_iter()
        .filter(|g| matches_filter(&g.label, &g.path, query))
        .map(|g| ContentItem {
            name: if g.is_entry {
                format!("★ {}", g.label)
            } else {
                g.label.clone()
            },
            path: g.path,
            kind: ContentItemKind::Graph,
            is_entry: g.is_entry,
            openable: true,
            children: Vec::new(),
        })
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    if items.is_empty() && query.is_empty() {
        items.push(ContentItem {
            name: GRAPHS_DIR.to_string(),
            path: String::new(),
            kind: ContentItemKind::Folder,
            is_entry: false,
            openable: false,
            children: Vec::new(),
        });
    }
    items
}

fn build_project_files_section(folder: &Path, query: &str) -> Vec<ContentItem> {
    const FILES: &[(&str, ContentItemKind, bool)] = &[
        ("quantum-point.qp", ContentItemKind::Config, false),
        ("README.md", ContentItemKind::Doc, false),
        (".gitignore", ContentItemKind::Other, false),
    ];
    FILES
        .iter()
        .filter(|(name, _, _)| folder.join(name).is_file())
        .filter(|(name, _, _)| matches_filter(name, name, query))
        .map(|(name, kind, openable)| ContentItem {
            name: (*name).to_string(),
            path: (*name).to_string(),
            kind: *kind,
            is_entry: false,
            openable: *openable,
            children: Vec::new(),
        })
        .collect()
}

fn build_build_section(folder: &Path, query: &str) -> Vec<ContentItem> {
    let rel = ".nocode/build/rust/main.rs";
    let main_rs = folder.join(rel);
    if !main_rs.is_file() {
        return Vec::new();
    }
    if !matches_filter("main.rs", rel, query) {
        return Vec::new();
    }
    vec![ContentItem {
        name: "main.rs".to_string(),
        path: rel.to_string(),
        kind: ContentItemKind::Build,
        is_entry: false,
        openable: true,
        children: Vec::new(),
    }]
}

fn matches_filter(name: &str, path: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    name.to_lowercase().contains(query) || path.to_lowercase().contains(query)
}
