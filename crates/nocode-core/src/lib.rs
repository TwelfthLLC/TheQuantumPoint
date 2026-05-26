pub mod compile_ctx;
pub mod graph_files;
pub mod pipeline;
pub mod project_tree;
pub mod projects;
pub mod sandbox;
pub mod target;

pub use compile_ctx::compile_context_for_root;

pub use compiler::CompileCache;
pub use graph_files::{build_rust_directory, ensure_project_directories, is_project_root};
pub use graph_files::{
    create_graph_file, ensure_default_graphs_layout, list_graph_files, load_graph_at,
    manifest_path, read_entry_graph, save_graph_at, set_entry_graph, GraphFileError, GraphFileInfo,
    DEFAULT_ENTRY, MANIFEST_FILE, QP_VERSION,
};
pub use graph_model::{
    GraphLayer, ProjectManifest, GRAPH_FILE_EXTENSION, GRAPH_MAGIC, PROJECT_MAGIC,
    PROJECT_MANIFEST_FILE, QP_FILE_VERSION,
};
pub use pipeline::{
    build_domain_artifacts, build_project, check_project, compile_project, compile_project_cached,
    resolve_build_dir, run_project, write_domain_outputs, write_rust, BuildOutput,
    BuildProjectParams, CheckOutput, DomainArtifacts, PipelineError,
};
pub use project_tree::{scan_project_browser, ContentItem, ContentItemKind, ContentSection};
pub use projects::{
    default_projects_folder, folder_name_from_project, hello_template, resolve_project_directory,
    user_documents_dir, ProjectMeta, ProjectStore, ProjectStoreError,
};
pub use target::{project_build_dir_for, BuildTarget};
