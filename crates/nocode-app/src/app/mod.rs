//! Asosiy ilova: launcher, studio, build/run.

mod content_browser;
mod graphs;
mod launcher;
mod pipeline_job;
mod project;
mod props;
mod state;
mod studio;
mod view_runtime;

use egui::Context;
use graph_model::{GraphLayer, Project};
use nocode_core::{BuildTarget, CompileCache, ContentSection, ProjectMeta, ProjectStore};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use crate::graph::GraphEditor;
use crate::theme::{apply, Palette};
use qp_graph_store::GraphStore;
use qp_view_runtime::ViewRuntimeState;

pub struct NoCodeApp {
    pub(crate) palette: Palette,
    pub(crate) workspace: PathBuf,
    pub(crate) build_dir: PathBuf,
    pub(crate) store: ProjectStore,
    pub(crate) screen: state::Screen,
    pub(crate) projects: Vec<ProjectMeta>,
    pub(crate) store_error: Option<String>,

    pub(crate) show_create: bool,
    pub(crate) new_name: String,
    pub(crate) new_folder: String,
    pub(crate) template: state::Template,
    pub(crate) project_folder: Option<PathBuf>,
    pub(crate) delete_confirm: Option<String>,

    pub(crate) project: Option<Project>,
    pub(crate) project_id: Option<String>,
    /// `(graphs/foo.qp, foo.qp)` — loyiha ichidagi graf fayllar
    pub(crate) graph_files: Vec<(String, String)>,
    pub(crate) active_graph: String,
    pub(crate) show_new_graph_popup: bool,
    pub(crate) new_graph_file_name: String,
    pub(crate) new_graph_layer: GraphLayer,
    pub(crate) content_search: String,
    pub(crate) content_sections: Vec<ContentSection>,
    pub editor: GraphEditor,
    /// DOD graph storage + dirty tracking (synced from `project` on load/save).
    pub graph_store: GraphStore,
    pub(crate) compile_cache: CompileCache,
    pub(crate) profile: String,
    pub(crate) build_target: BuildTarget,
    pub(crate) terminal: Vec<String>,
    pub(crate) generated_main: String,
    pub(crate) show_generated: bool,
    pub(crate) view_preview: Vec<emit_view::ViewSpecItem>,
    pub(crate) view_runtime: Option<qp_view_runtime::ViewRuntime>,
    pub(crate) view_runtime_state: qp_view_runtime::ViewRuntimeState,
    pub(crate) view_studio_mode: state::ViewStudioMode,
    pub(crate) dirty: bool,
    pub(crate) props_message: String,
    pub(crate) props_condition: String,
    pub(crate) props_assign_name: String,
    pub(crate) props_assign_value: String,
    pub(crate) props_label: String,
    pub(crate) props_for_var: String,
    pub(crate) props_for_from: String,
    pub(crate) props_for_to: String,
    pub(crate) props_expression: String,
    pub(crate) props_variable: String,
    pub(crate) props_case1: String,
    pub(crate) props_case2: String,
    pub(crate) props_cases: String,
    pub(crate) props_collection: String,
    pub(crate) props_item_var: String,
    pub(crate) last_props_node: Option<String>,

    pub(crate) pipeline_job: state::PipelineJobKind,
    pub(crate) pipeline_rx: Option<Receiver<pipeline_job::PipelineJobResult>>,
    pub(crate) pipeline_cache_handle: Option<std::sync::Arc<std::sync::Mutex<CompileCache>>>,
}

impl NoCodeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply(&cc.egui_ctx, &Palette::default());
        let workspace = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let build_dir = workspace.join(".nocode/build/rust");
        let store = ProjectStore::new(&workspace);
        let _ = store.ensure();
        let _ = store.seed_if_empty(&workspace);

        let mut app = Self {
            palette: Palette::default(),
            workspace,
            build_dir,
            store,
            screen: state::Screen::Launcher,
            projects: vec![],
            store_error: None,
            show_create: false,
            new_name: String::new(),
            new_folder: String::new(),
            template: state::Template::Empty,
            project_folder: None,
            delete_confirm: None,
            project: None,
            project_id: None,
            graph_files: Vec::new(),
            active_graph: nocode_core::DEFAULT_ENTRY.to_string(),
            show_new_graph_popup: false,
            new_graph_file_name: String::new(),
            new_graph_layer: GraphLayer::View,
            content_search: String::new(),
            content_sections: Vec::new(),
            editor: GraphEditor::default(),
            graph_store: GraphStore::default(),
            compile_cache: CompileCache::default(),
            profile: "dev".to_string(),
            terminal: vec!["Quantum Point — Run: universal IR · Build: til tanlash".to_string()],
            build_target: BuildTarget::Rust,
            generated_main: String::new(),
            show_generated: false,
            view_preview: Vec::new(),
            view_runtime: None,
            view_runtime_state: ViewRuntimeState::default(),
            view_studio_mode: state::ViewStudioMode::GraphEditor,
            dirty: false,
            props_message: String::new(),
            props_condition: String::new(),
            props_assign_name: String::new(),
            props_assign_value: String::new(),
            props_label: String::new(),
            props_for_var: String::new(),
            props_for_from: String::new(),
            props_for_to: String::new(),
            props_expression: String::new(),
            props_variable: String::new(),
            props_case1: String::new(),
            props_case2: String::new(),
            props_cases: String::new(),
            props_collection: String::new(),
            props_item_var: String::new(),
            last_props_node: None,
            pipeline_job: state::PipelineJobKind::Idle,
            pipeline_rx: None,
            pipeline_cache_handle: None,
        };
        app.reload_projects();
        app
    }
}

impl eframe::App for NoCodeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply(ctx, &self.palette);
        match &self.screen {
            state::Screen::Launcher => self.ui_launcher(ctx),
            state::Screen::Studio { id } => {
                let id = id.clone();
                self.ui_studio(ctx, &id);
            }
        }
    }
}
