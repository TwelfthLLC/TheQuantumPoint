use graph_model::GraphLayer;
use graph_model::Project;
use nocode_core::{
    default_projects_folder, hello_template, resolve_project_directory, ProjectStore,
};
use std::path::Path;

use super::state::{Screen, Template};
use super::NoCodeApp;

impl NoCodeApp {
    pub(crate) fn reload_projects(&mut self) {
        match self.store.list() {
            Ok(list) => {
                self.projects = list;
                self.store_error = None;
            }
            Err(e) => self.store_error = Some(e.to_string()),
        }
    }

    pub(crate) fn open_project(&mut self, id: &str) {
        self.project_id = Some(id.to_string());
        if let Ok(folder) = self.store.folder_for(id) {
            self.project_folder = Some(folder.clone());
            self.build_dir = nocode_core::resolve_build_dir(&folder, self.build_target);
        }
        self.screen = Screen::Studio { id: id.to_string() };
        self.open_project_graphs(id);
        if let Some(p) = self.project.clone() {
            self.sync_props_from_project(&p);
            self.sync_build_target_for_layer();
        }
        self.terminal.push(format!("Ochildi: {id}"));
    }

    pub(crate) fn sync_props_from_project(&mut self, p: &Project) {
        if let Some(id) = &self.editor.focus {
            if let Some(n) = p.node(id) {
                self.load_props(n);
            }
        }
    }

    pub(crate) fn save_current(&mut self) {
        self.save_active_graph();
        self.reload_projects();
    }

    pub(crate) fn create_project(&mut self) {
        let name = self.new_name.trim().to_string();
        let folder = self.new_folder.trim().to_string();
        if name.is_empty() || folder.is_empty() {
            return;
        }
        let project = match self.template {
            Template::Hello => hello_template(&self.workspace, &name)
                .unwrap_or_else(|| ProjectStore::default_empty(&name, GraphLayer::Core)),
            Template::Empty => ProjectStore::default_empty(&name, GraphLayer::Core),
        };
        let folder_path = resolve_project_directory(Path::new(&folder), &name);
        match self.store.create_in_folder(&name, &folder_path, project) {
            Ok(meta) => {
                self.show_create = false;
                self.new_name.clear();
                self.new_folder.clear();
                self.reload_projects();
                self.terminal
                    .push(format!("✓ Loyiha yaratildi: {}", meta.folder.display()));
                self.open_project(&meta.id);
            }
            Err(e) => self.store_error = Some(e.to_string()),
        }
    }

    pub(crate) fn suggest_folder_from_name(&mut self) {
        if self.new_folder.trim().is_empty() && !self.new_name.trim().is_empty() {
            let path = default_projects_folder(&self.workspace, self.new_name.trim());
            self.new_folder = path.to_string_lossy().to_string();
        }
    }
}
