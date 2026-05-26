use egui::{Align, Context, Frame, Layout, RichText, TopBottomPanel};
use graph_model::GraphLayer;
use nocode_core::BuildTarget;

use crate::app::state::{Screen, ViewStudioMode};
use crate::app::NoCodeApp;
use crate::branding::logo_image;

impl NoCodeApp {
    pub(crate) fn ui_studio_toolbar(&mut self, ctx: &Context, id: &str) {
        TopBottomPanel::top("toolbar")
            .frame(Frame::NONE.fill(self.palette.panel).inner_margin(10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(logo_image(28.0));
                    ui.separator();
                    if ui.button("← Launcher").clicked() {
                        if self.dirty {
                            self.save_active_graph();
                        }
                        self.screen = Screen::Launcher;
                        self.reload_projects();
                    }
                    ui.separator();
                    if let Some(p) = &self.project {
                        ui.label(RichText::new(&p.name).strong());
                        let layer_color = match p.layer {
                            GraphLayer::View => self.palette.accent,
                            GraphLayer::Bridge => self.palette.warn,
                            GraphLayer::Core => self.palette.success,
                        };
                        ui.label(
                            RichText::new(format!("{} ({})", p.layer.label(), p.layer.subtitle()))
                                .small()
                                .color(layer_color),
                        );
                    }
                    ui.label(
                        RichText::new(format!("id: {id}"))
                            .small()
                            .color(self.palette.muted),
                    );
                    if let Some(folder) = &self.project_folder {
                        ui.label(
                            RichText::new(folder.display().to_string())
                                .small()
                                .color(self.palette.muted),
                        );
                    }
                    if let Some(err) = &self.store_error {
                        ui.separator();
                        ui.label(RichText::new(err).small().color(self.palette.danger));
                    }
                    ui.separator();
                    ui.label(RichText::new("Graf:").small().color(self.palette.muted));
                    let active_label = self.graph_label();
                    let active_path = self.active_graph.clone();
                    egui::ComboBox::from_id_salt("graph_file")
                        .selected_text(&active_label)
                        .width(140.0)
                        .show_ui(ui, |ui| {
                            for (path, label) in self.graph_files.clone() {
                                let is_entry = self
                                    .store
                                    .entry_graph_path(self.project_id.as_deref().unwrap_or(""))
                                    .map(|e| e == path)
                                    .unwrap_or(false);
                                let text = if is_entry {
                                    format!("★ {label}")
                                } else {
                                    label
                                };
                                if ui.selectable_label(path == active_path, text).clicked() {
                                    self.switch_graph_file(path);
                                }
                            }
                        });
                    if ui
                        .button("+")
                        .on_hover_text("Yangi graf fayli / nod")
                        .clicked()
                    {
                        self.show_new_graph_popup = true;
                    }
                    if self.dirty {
                        ui.colored_label(self.palette.warn, "●");
                    }
                    if self
                        .project
                        .as_ref()
                        .is_some_and(|p| p.layer == GraphLayer::View)
                    {
                        ui.separator();
                        let prev = self.view_studio_mode;
                        ui.scope(|ui| {
                            ui.style_mut().spacing.button_padding = egui::vec2(10.0, 6.0);
                            let graph_sel = self.view_studio_mode == ViewStudioMode::GraphEditor;
                            let run_sel = self.view_studio_mode == ViewStudioMode::ViewRuntime;
                            if ui
                                .add(
                                    egui::Button::new("Graf")
                                        .fill(if graph_sel {
                                            self.palette.accent
                                        } else {
                                            self.palette.surface
                                        })
                                        .stroke(egui::Stroke::NONE),
                                )
                                .clicked()
                            {
                                self.view_studio_mode = ViewStudioMode::GraphEditor;
                            }
                            if ui
                                .add(
                                    egui::Button::new("View Runtime")
                                        .fill(if run_sel {
                                            self.palette.accent
                                        } else {
                                            self.palette.surface
                                        })
                                        .stroke(egui::Stroke::NONE),
                                )
                                .clicked()
                            {
                                self.view_studio_mode = ViewStudioMode::ViewRuntime;
                            }
                        });
                        if prev != ViewStudioMode::ViewRuntime
                            && self.view_studio_mode == ViewStudioMode::ViewRuntime
                        {
                            self.sync_view_runtime();
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let busy = self.pipeline_job.is_busy();
                        let layer = self.project.as_ref().map(|p| p.layer);
                        if let Some(layer) = layer {
                            let targets = BuildTarget::available_for_layer(layer);
                            egui::ComboBox::from_id_salt("build_target")
                                .selected_text(self.build_target.label())
                                .width(130.0)
                                .show_ui(ui, |ui| {
                                    for t in targets {
                                        ui.selectable_value(&mut self.build_target, *t, t.label());
                                    }
                                });
                        }
                        egui::ComboBox::from_id_salt("profile")
                            .selected_text(&self.profile)
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.profile, "dev".to_string(), "dev");
                                ui.selectable_value(
                                    &mut self.profile,
                                    "release".to_string(),
                                    "release",
                                );
                            });
                        if ui
                            .add_enabled(
                                !busy && layer.is_some(),
                                egui::Button::new("Build & Run").fill(self.palette.accent),
                            )
                            .on_hover_text("Emit + toolchain (Core: cargo run)")
                            .clicked()
                        {
                            self.start_build(true);
                        }
                        if ui
                            .add_enabled(!busy && layer.is_some(), egui::Button::new("Build"))
                            .on_hover_text("Emit + cargo build (faqat Rust Core)")
                            .clicked()
                        {
                            self.start_build(false);
                        }
                        if ui
                            .add_enabled(
                                !busy,
                                egui::Button::new("▶ Run").fill(self.palette.success),
                            )
                            .on_hover_text("Universal: graf → IR / domain (tilsiz, cargo yo‘q)")
                            .clicked()
                        {
                            self.start_check();
                        }
                        if ui.button("Saqlash").clicked() {
                            self.save_current();
                        }
                    });
                });
            });
    }
}
