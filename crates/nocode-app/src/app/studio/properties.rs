use egui::{Context, Frame, RichText, ScrollArea, SidePanel};
use graph_model::{
    GraphLayer, NODE_API_QUERY, NODE_API_ROUTE, NODE_ASSIGN, NODE_AWAIT, NODE_CALL, NODE_CONST,
    NODE_DB_READ, NODE_EMIT_UI, NODE_ENUM, NODE_EXPR, NODE_FOR, NODE_FOREACH, NODE_FUNCTION,
    NODE_IF, NODE_IMPORT, NODE_LIST, NODE_LOG, NODE_RETURN, NODE_STRUCT, NODE_SUBGRAPH,
    NODE_SWITCH, NODE_THROW, NODE_UI_BUTTON, NODE_UI_EVENT, NODE_UI_INPUT, NODE_UI_LABEL,
    NODE_UI_PAGE, NODE_WHILE,
};

use crate::app::state::ViewStudioMode;
use crate::app::NoCodeApp;

impl NoCodeApp {
    pub(crate) fn ui_studio_properties(&mut self, ctx: &Context, _id: &str) {
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
                                NODE_IF | NODE_WHILE => {
                                    ui.label("Condition");
                                    if ui.text_edit_singleline(&mut self.props_condition).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_FOR => {
                                    ui.label("Variable");
                                    if ui.text_edit_singleline(&mut self.props_for_var).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("From");
                                    if ui.text_edit_singleline(&mut self.props_for_from).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("To");
                                    if ui.text_edit_singleline(&mut self.props_for_to).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_RETURN => {
                                    ui.label("Return value (optional)");
                                    if ui.text_edit_singleline(&mut self.props_message).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_FOREACH => {
                                    ui.label("Collection (table)");
                                    if ui.text_edit_singleline(&mut self.props_collection).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Item variable");
                                    if ui.text_edit_singleline(&mut self.props_item_var).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_SWITCH => {
                                    ui.label("Variable");
                                    if ui.text_edit_singleline(&mut self.props_variable).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Cases (comma-separated, maps to case1..caseN ports)");
                                    if ui.text_edit_singleline(&mut self.props_cases).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Case 1 label (legacy)");
                                    if ui.text_edit_singleline(&mut self.props_case1).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Case 2 label (legacy)");
                                    if ui.text_edit_singleline(&mut self.props_case2).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_EXPR => {
                                    ui.label("Target variable");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Expression");
                                    if ui.text_edit_singleline(&mut self.props_expression).changed() {
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
                                NODE_SUBGRAPH | NODE_IMPORT => {
                                    ui.label("Modul (.qp yo‘li yoki id)");
                                    if ui.text_edit_singleline(&mut self.props_label).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_FUNCTION => {
                                    ui.label("Name");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Params (comma-separated)");
                                    if ui.text_edit_singleline(&mut self.props_params).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_CALL => {
                                    ui.label("Function");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Args");
                                    if ui.text_edit_singleline(&mut self.props_args).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Into variable");
                                    if ui.text_edit_singleline(&mut self.props_into).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_CONST => {
                                    ui.label("Name");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Value");
                                    if ui.text_edit_singleline(&mut self.props_assign_value).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_LIST => {
                                    ui.label("Name");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Items (comma-separated)");
                                    if ui.text_edit_singleline(&mut self.props_items).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_THROW => {
                                    ui.label("Message");
                                    if ui.text_edit_singleline(&mut self.props_message).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_AWAIT => {
                                    ui.label("Into (optional)");
                                    if ui.text_edit_singleline(&mut self.props_into).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_STRUCT => {
                                    ui.label("Name");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Fields");
                                    if ui.text_edit_singleline(&mut self.props_fields).changed() {
                                        self.apply_props_to_selected();
                                    }
                                }
                                NODE_ENUM => {
                                    ui.label("Name");
                                    if ui.text_edit_singleline(&mut self.props_assign_name).changed() {
                                        self.apply_props_to_selected();
                                    }
                                    ui.label("Variants");
                                    if ui.text_edit_singleline(&mut self.props_variants).changed() {
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
    }
}
