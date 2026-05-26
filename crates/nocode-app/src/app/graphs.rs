use graph_model::GraphLayer;

use super::state::ViewStudioMode;
use super::NoCodeApp;
use crate::graph::GraphEditor;
use qp_graph_store::GraphStore;

impl NoCodeApp {
    pub(crate) fn reload_graph_files(&mut self) {
        let Some(id) = self.project_id.clone() else {
            return;
        };
        match self.store.list_graph_files(&id) {
            Ok(files) => {
                self.graph_files = files
                    .into_iter()
                    .map(|f| (f.path.clone(), f.label))
                    .collect();
                self.reload_content_browser();
            }
            Err(e) => self.store_error = Some(e.to_string()),
        }
    }

    pub(crate) fn load_active_graph(&mut self) {
        let Some(id) = self.project_id.clone() else {
            return;
        };
        let path = self.active_graph.clone();
        match self.store.load_graph(&id, &path) {
            Ok(p) => {
                let is_view = p.layer == GraphLayer::View;
                self.graph_store = GraphStore::from_project(&p);
                self.compile_cache.clear();
                self.project = Some(p);
                self.editor = GraphEditor::default();
                self.editor.request_fit_view();
                self.editor.selected.clear();
                self.editor.focus = None;
                self.last_props_node = None;
                self.dirty = false;
                self.store_error = None;
                self.sync_build_target_for_layer();
                self.sync_view_runtime();
                if !is_view {
                    self.view_studio_mode = ViewStudioMode::GraphEditor;
                }
            }
            Err(e) => {
                self.project = None;
                let msg = format!(
                    "Graf yuklanmadi ({}): {e}",
                    path.rsplit('/').next().unwrap_or(&path)
                );
                self.store_error = Some(msg.clone());
                self.terminal.push(format!("✗ {msg}"));
            }
        }
    }

    pub(crate) fn switch_graph_file(&mut self, new_path: String) {
        if new_path == self.active_graph {
            return;
        }
        if self.dirty {
            self.save_active_graph();
        }
        self.active_graph = new_path;
        self.load_active_graph();
        self.terminal.push(format!("Fayl: {}", self.graph_label()));
    }

    pub(crate) fn save_active_graph(&mut self) {
        let Some(id) = self.project_id.clone() else {
            return;
        };
        let Some(mut p) = self.project.clone() else {
            return;
        };
        let path = self.active_graph.clone();
        p.name = self.graph_display_name();
        match self.store.save_graph(&id, &path, &p) {
            Ok(_) => {
                self.dirty = false;
                self.terminal
                    .push(format!("✓ Saqlandi: {}", self.graph_label()));
                self.reload_graph_files();
                self.reload_content_browser();
            }
            Err(e) => self.terminal.push(format!("✗ Saqlash: {e}")),
        }
    }

    pub(crate) fn create_graph_file(&mut self) {
        let name = self.new_graph_file_name.trim().to_string();
        if name.is_empty() {
            return;
        }
        let Some(id) = self.project_id.clone() else {
            return;
        };
        if self.dirty {
            self.save_active_graph();
        }
        match self
            .store
            .create_graph_file(&id, &name, self.new_graph_layer)
        {
            Ok(rel) => {
                self.show_new_graph_popup = false;
                self.new_graph_file_name.clear();
                self.new_graph_layer = graph_model::GraphLayer::View;
                self.reload_graph_files();
                self.reload_content_browser();
                self.active_graph = rel.clone();
                self.load_active_graph();
                self.terminal
                    .push(format!("✓ Yangi fayl: {}", self.graph_label()));
            }
            Err(e) => self.store_error = Some(e.to_string()),
        }
    }

    pub(crate) fn set_entry_graph_file(&mut self) {
        let Some(id) = self.project_id.clone() else {
            return;
        };
        let path = self.active_graph.clone();
        match self.store.set_entry_graph(&id, &path) {
            Ok(_) => {
                self.reload_graph_files();
                self.reload_content_browser();
                self.terminal
                    .push(format!("★ Entry graf: {}", self.graph_label()));
            }
            Err(e) => self.store_error = Some(e.to_string()),
        }
    }

    pub(crate) fn graph_label(&self) -> String {
        self.graph_files
            .iter()
            .find(|(p, _)| *p == self.active_graph)
            .map(|(_, l)| l.clone())
            .unwrap_or_else(|| {
                self.active_graph
                    .rsplit('/')
                    .next()
                    .unwrap_or(&self.active_graph)
                    .to_string()
            })
    }

    fn graph_display_name(&self) -> String {
        let label = self.graph_label();
        label.trim_end_matches(".qp").to_string()
    }

    pub(crate) fn open_project_graphs(&mut self, id: &str) {
        self.reload_graph_files();
        if let Ok(entry) = self.store.entry_graph_path(id) {
            self.active_graph = entry;
        }
        self.load_active_graph();
        self.reload_content_browser();
    }
}
