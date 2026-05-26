use egui::{Context, RichText};

use graph_model::{GraphLayer, NODE_ASSIGN, NODE_IF, NODE_LOG, NODE_UI_BUTTON, NODE_UI_PAGE};

use crate::app::NoCodeApp;
use crate::graph::add_node;

impl NoCodeApp {
    pub(crate) fn ui_studio_new_graph_popup(&mut self, ctx: &Context, _id: &str) {
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
    }
}
