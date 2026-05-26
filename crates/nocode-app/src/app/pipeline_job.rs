use graph_model::Project;
use nocode_core::{
    build_project, check_project, resolve_build_dir, BuildOutput, BuildProjectParams, BuildTarget,
    CheckOutput,
};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use super::state::PipelineJobKind;
use super::NoCodeApp;

#[derive(Debug)]
pub(crate) enum PipelineJobResult {
    Check(Result<CheckOutput, String>),
    Build(Result<BuildOutput, String>),
}

impl NoCodeApp {
    fn pipeline_project(&self) -> Option<Project> {
        let id = self.project_id.clone()?;
        if let Ok(entry) = self.store.entry_graph_path(&id) {
            if let Ok(p) = self.store.load_graph(&id, &entry) {
                return Some(p);
            }
        }
        self.project.clone()
    }

    fn pipeline_context(&self) -> Option<(Project, PathBuf, PathBuf)> {
        let project = self.pipeline_project()?;
        let project_root = self
            .project_folder
            .clone()
            .unwrap_or_else(|| self.workspace.clone());
        let build_dir = resolve_build_dir(&project_root, self.build_target);
        Some((project, project_root, build_dir))
    }

    pub(crate) fn sync_build_target_for_layer(&mut self) {
        if let Some(p) = &self.project {
            self.build_target = BuildTarget::default_for_layer(p.layer);
            if let Some(folder) = &self.project_folder {
                self.build_dir = resolve_build_dir(folder, self.build_target);
            }
        }
    }

    pub(crate) fn start_check(&mut self) {
        self.save_active_graph();
        let Some((project, project_root, _)) = self.pipeline_context() else {
            return;
        };
        let root = project_root.clone();

        let (tx, rx) = mpsc::channel();
        self.pipeline_rx = Some(rx);
        self.pipeline_job = PipelineJobKind::Check;
        self.terminal.push(format!(
            "▶ Run (umumiy) — {} · {}",
            project.name,
            project.layer.label()
        ));

        let dirty = self.graph_store.take_dirty_compile_set();
        let cache = Arc::new(Mutex::new(self.compile_cache.clone()));
        let cache_bg = Arc::clone(&cache);
        thread::spawn(move || {
            let mut c = cache_bg.lock().expect("compile cache lock");
            let result =
                check_project(&project, &mut c, &dirty, Some(&root)).map_err(|e| e.to_string());
            let _ = tx.send(PipelineJobResult::Check(result));
        });
        self.pipeline_cache_handle = Some(cache);
    }

    pub(crate) fn start_build(&mut self, run_after_build: bool) {
        self.save_active_graph();
        let Some((project, project_root, build_dir)) = self.pipeline_context() else {
            return;
        };

        if !self.build_target.matches_layer(project.layer) {
            self.terminal.push(format!(
                "✗ Build target {} bu graf qatlamiga mos emas",
                self.build_target
            ));
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.pipeline_rx = Some(rx);
        self.pipeline_job = if run_after_build {
            PipelineJobKind::BuildRun
        } else {
            PipelineJobKind::Build
        };

        let label = if run_after_build {
            "Build & Run"
        } else {
            "Build"
        };
        self.terminal.push(format!(
            "▶ {label} — {} · {}",
            self.build_target, project.name
        ));

        let profile = self.profile.clone();
        let target = self.build_target;
        let dirty = self.graph_store.take_dirty_compile_set();
        let cache = Arc::new(Mutex::new(self.compile_cache.clone()));
        let cache_bg = Arc::clone(&cache);
        thread::spawn(move || {
            let mut c = cache_bg.lock().expect("compile cache lock");
            let result = build_project(&mut BuildProjectParams {
                project: &project,
                project_root: &project_root,
                out_dir: &build_dir,
                profile: &profile,
                target,
                cache: &mut c,
                dirty_nodes: &dirty,
                run_after_build,
            })
            .map_err(|e| e.to_string());
            let _ = tx.send(PipelineJobResult::Build(result));
        });
        self.pipeline_cache_handle = Some(cache);
    }

    pub(crate) fn poll_pipeline(&mut self) {
        let Some(rx) = &self.pipeline_rx else {
            return;
        };
        let Ok(msg) = rx.try_recv() else {
            return;
        };
        self.pipeline_rx = None;
        if let Some(handle) = self.pipeline_cache_handle.take() {
            if let Ok(c) = handle.lock() {
                self.compile_cache = c.clone();
            }
        }

        match msg {
            PipelineJobResult::Check(result) => match result {
                Ok(out) => {
                    for line in out.summary.lines() {
                        self.terminal.push(line.to_string());
                    }
                    self.view_preview = out.view_items;
                    self.sync_view_runtime();
                    if !out.preview_lines.is_empty() {
                        self.terminal
                            .push("— IR preview (Run) terminalda —".to_string());
                    }
                    self.pipeline_job = PipelineJobKind::Idle;
                }
                Err(e) => {
                    self.terminal.push(format!("✗ Run: {e}"));
                    self.pipeline_job = PipelineJobKind::Idle;
                }
            },
            PipelineJobResult::Build(result) => match result {
                Ok(out) => {
                    if !out.stderr.is_empty() {
                        self.terminal.extend(out.stderr.lines().map(String::from));
                    }
                    if !out.stdout.is_empty() {
                        self.terminal.extend(out.stdout.lines().map(String::from));
                    }
                    self.generated_main = out.preview_source;
                    self.reload_content_browser();
                    self.terminal.push(if out.success {
                        format!("✓ Build muvaffaqiyat (exit {})", out.exit_code)
                    } else {
                        format!("✗ Build xato (exit {})", out.exit_code)
                    });
                    self.pipeline_job = PipelineJobKind::Idle;
                }
                Err(e) => {
                    self.terminal.push(format!("✗ Build: {e}"));
                    self.pipeline_job = PipelineJobKind::Idle;
                }
            },
        }
    }
}
