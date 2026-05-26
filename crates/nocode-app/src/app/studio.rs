use egui::{
    Align, CentralPanel, Context, Frame, Layout, RichText, ScrollArea, SidePanel, TopBottomPanel,
};
use graph_model::{
    GraphLayer, NODE_API_QUERY, NODE_API_ROUTE, NODE_ASSIGN, NODE_DB_READ, NODE_EMIT_UI, NODE_IF,
    NODE_LOG, NODE_SUBGRAPH, NODE_UI_BUTTON, NODE_UI_EVENT, NODE_UI_INPUT, NODE_UI_LABEL,
    NODE_UI_PAGE,
};

use super::props::{get_i64, get_str};
use super::state::{PipelineJobKind, Screen, ViewStudioMode};
use super::NoCodeApp;
use crate::branding::logo_image;
use crate::graph::{add_node, GraphAction};
use nocode_core::BuildTarget;

impl NoCodeApp {
    pub(crate) fn ui_studio(&mut self, ctx: &Context, id: &str) {
        self.poll_pipeline();

        self.ui_content_browser(ctx);

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

        SidePanel::right("props")
            .resizable(true)
            .default_width(260.0)
            .frame(Frame::NONE.fill(self.palette.panel).inner_margin(12.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("Xususiyatlar").strong());
                ui.add_space(8.0);
                let n_sel = self.editor.selected.len();
                if n_sel > 1 {
                    ui.label(
                        RichText::new(format!("{n_sel} ta nod tanlangan"))
                            .color(self.palette.muted),
                    );
                    ui.label(
                        RichText::new(
                            "Xususiyatlar uchun bitta nod tanlang yoki Shift bilan fokusni almashtiring",
                        )
                        .small()
                        .color(self.palette.muted),
                    );
                } else if let Some(sel) = self.editor.focus.clone() {
                    if let Some(p) = self.project.as_ref() {
                        if let Some(n) = p.node(&sel) {
                            ui.label(
                                RichText::new(format!("{} · {}", n.kind, n.id))
                                    .small()
                                    .color(self.palette.muted),
                            );
                            ui.add_space(8.0);
                            match n.kind.as_str() {
                                NODE_LOG => {
                                    ui.label("Xabar");
                                    if ui.text_edit_multiline(&mut self.props_message).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_IF => {
                                    ui.label("Shart (Rust)");
                                    if ui.text_edit_singleline(&mut self.props_condition).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_ASSIGN => {
                                    ui.label("O‘zgaruvchi");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Qiymat");
                                    if ui.text_edit_singleline(&mut self.props_assign_value).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_DB_READ => {
                                    ui.label("Jadval");
                                    if ui.text_edit_singleline(&mut self.props_label).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_SUBGRAPH => {
                                    ui.label("Modul (.qp yo‘li yoki id)");
                                    if ui.text_edit_singleline(&mut self.props_label).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_UI_PAGE
                                | NODE_UI_BUTTON
                                | NODE_UI_LABEL
                                | NODE_UI_INPUT
                                | NODE_UI_EVENT
                                | NODE_API_ROUTE
                                | NODE_API_QUERY
                                | NODE_EMIT_UI => {
                                    ui.label("Parametr");
                                    if ui.text_edit_singleline(&mut self.props_label).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                _ => {
                                    ui.label(
                                        RichText::new("Start — parametrsiz")
                                            .color(self.palette.muted),
                                    );
                                }
                            }
                        }
                    }
                } else {
                    ui.label(RichText::new("Nod tanlang").color(self.palette.muted));
                }
                ui.add_space(16.0);
                ui.separator();
                if self
                    .project
                    .as_ref()
                    .is_some_and(|p| p.layer == GraphLayer::View)
                    && self.view_studio_mode == ViewStudioMode::ViewRuntime
                {
                    ui.label(
                        RichText::new("View Runtime rejimi — graf uchun yuqorida «Graf»")
                            .small()
                            .color(self.palette.muted),
                    );
                    ui.add_space(8.0);
                }
                ui.checkbox(&mut self.show_generated, "Build preview");
                if self.show_generated && !self.generated_main.is_empty() {
                    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.generated_main.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY),
                        );
                    });
                }
            });

        TopBottomPanel::bottom("terminal")
            .resizable(true)
            .default_height(160.0)
            .frame(
                Frame::NONE
                    .fill(egui::Color32::from_rgb(10, 12, 16))
                    .inner_margin(10.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Terminal").strong().color(self.palette.muted));
                    if self.pipeline_job.is_busy() {
                        ui.spinner();
                        ui.label(match self.pipeline_job {
                            PipelineJobKind::Check => "Run (IR)…",
                            PipelineJobKind::Build => "Build…",
                            PipelineJobKind::BuildRun => "Build & Run…",
                            PipelineJobKind::Idle => "",
                        });
                    }
                });
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for line in &self.terminal {
                        ui.label(
                            RichText::new(line)
                                .family(egui::FontFamily::Monospace)
                                .size(12.0),
                        );
                    }
                });
            });

        if self.show_new_graph_popup {
            egui::Window::new("Yangi graf fayli")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Graf qatlami");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.new_graph_layer,
                            GraphLayer::View,
                            "View (UI)",
                        );
                        ui.selectable_value(
                            &mut self.new_graph_layer,
                            GraphLayer::Core,
                            "Core (Backend)",
                        );
                        ui.selectable_value(
                            &mut self.new_graph_layer,
                            GraphLayer::Bridge,
                            "Bridge (I/O)",
                        );
                    });
                    ui.add_space(6.0);
                    ui.label("Fayl nomi (graphs/ ichida .qp yaratiladi)");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_graph_file_name).hint_text(
                            match self.new_graph_layer {
                                GraphLayer::View => "masalan: home-ui, login",
                                GraphLayer::Bridge => "masalan: api-bridge",
                                GraphLayer::Core => "masalan: api, auth",
                            },
                        ),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!(
                            "Faqat {} nodlari ko‘rinadi",
                            self.new_graph_layer.label()
                        ))
                        .small()
                        .color(self.palette.muted),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Bekor").clicked() {
                            self.show_new_graph_popup = false;
                        }
                        let ok = !self.new_graph_file_name.trim().is_empty();
                        if ui.add_enabled(ok, egui::Button::new("Yaratish")).clicked() {
                            self.create_graph_file();
                        }
                    });
                    ui.separator();
                    ui.label(RichText::new("Tez nod qo‘shish (joriy fayl)").small());
                    ui.horizontal(|ui| {
                        if self
                            .project
                            .as_ref()
                            .is_some_and(|p| p.layer == GraphLayer::Core)
                        {
                            if ui.button("+ Log").clicked() {
                                if let Some(p) = self.project.as_mut() {
                                    add_node(p, NODE_LOG);
                                    self.dirty = true;
                                }
                                self.show_new_graph_popup = false;
                            }
                            if ui.button("+ Assign").clicked() {
                                if let Some(p) = self.project.as_mut() {
                                    add_node(p, NODE_ASSIGN);
                                    self.dirty = true;
                                }
                                self.show_new_graph_popup = false;
                            }
                            if ui.button("+ If").clicked() {
                                if let Some(p) = self.project.as_mut() {
                                    add_node(p, NODE_IF);
                                    self.dirty = true;
                                }
                                self.show_new_graph_popup = false;
                            }
                        } else if self
                            .project
                            .as_ref()
                            .is_some_and(|p| p.layer == GraphLayer::View)
                        {
                            if ui.button("+ Page").clicked() {
                                if let Some(p) = self.project.as_mut() {
                                    add_node(p, NODE_UI_PAGE);
                                    self.dirty = true;
                                }
                                self.show_new_graph_popup = false;
                            }
                            if ui.button("+ Button").clicked() {
                                if let Some(p) = self.project.as_mut() {
                                    add_node(p, NODE_UI_BUTTON);
                                    self.dirty = true;
                                }
                                self.show_new_graph_popup = false;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new("★ Entry qilib belgilash").small())
                            .clicked()
                        {
                            self.set_entry_graph_file();
                            self.show_new_graph_popup = false;
                        }
                    });
                });
        }

        let view_runtime_center = self
            .project
            .as_ref()
            .is_some_and(|p| p.layer == GraphLayer::View)
            && self.view_studio_mode == ViewStudioMode::ViewRuntime;

        CentralPanel::default().show(ctx, |ui| {
            if view_runtime_center {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("View Runtime").heading());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Log tozalash").clicked() {
                            self.view_runtime_state.log.clear();
                        }
                        if ui.button("↻ Yangilash").clicked() {
                            self.sync_view_runtime();
                        }
                    });
                });
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(420.0);
                        self.show_view_runtime_panel(ui);
                    });
                });
                return;
            }

            if self.last_props_node != self.editor.focus {
                self.last_props_node = self.editor.focus.clone();
                if let Some(sel) = &self.editor.focus {
                    if let Some(p) = self.project.as_ref() {
                        if let Some(n) = p.node(sel) {
                            let snap = (
                                get_str(n, "message"),
                                get_str(n, "condition"),
                                get_str(n, "name"),
                                get_i64(n, "value").to_string(),
                            );
                            self.props_message = snap.0;
                            self.props_condition = snap.1;
                            self.props_assign_name = snap.2;
                            self.props_assign_value = snap.3;
                        }
                    }
                }
            }
            if let Some(p) = self.project.as_mut() {
                match self.editor.show(ui, p, &self.palette) {
                    GraphAction::Changed => {
                        self.dirty = true;
                        self.graph_store = qp_graph_store::GraphStore::from_project(p);
                        self.graph_store.mark_structure_dirty();
                        self.sync_view_runtime();
                    }
                    GraphAction::None => {}
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(120.0);
                    ui.heading(RichText::new("Graf yuklanmadi").color(self.palette.muted));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            "Fayl binar `.qp` (QPGR) bo‘lishi kerak — matn/JSON emas. \
                             Terminaldagi xatoni ko‘ring yoki yangi graf yarating.",
                        )
                        .small()
                        .color(self.palette.muted),
                    );
                    if let Some(err) = &self.store_error {
                        ui.add_space(12.0);
                        ui.label(RichText::new(err).small().color(self.palette.danger));
                    }
                    ui.add_space(16.0);
                    if ui.button("Qayta yuklash").clicked() {
                        self.load_active_graph();
                    }
                });
            }
        });
    }
}
