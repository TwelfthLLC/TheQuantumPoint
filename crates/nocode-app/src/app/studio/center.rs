use egui::{Align, CentralPanel, Context, Layout, RichText, ScrollArea};

use graph_model::GraphLayer;

use crate::app::props::{get_i64, get_str};
use crate::app::state::ViewStudioMode;
use crate::app::NoCodeApp;
use crate::graph::GraphAction;

impl NoCodeApp {
    pub(crate) fn ui_studio_center(&mut self, ctx: &Context, _id: &str) {
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
