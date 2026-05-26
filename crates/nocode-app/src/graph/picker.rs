use egui::{Pos2, Ui, Vec2};
use graph_model::{
    maturity_label, node_maturity, DataValue, Node, Project, NODE_ASSIGN, NODE_EXPR, NODE_FOR,
    NODE_FOREACH, NODE_IF, NODE_LOG, NODE_RETURN, NODE_SUBGRAPH, NODE_SWITCH, NODE_WHILE,
};
use std::collections::HashMap;

use super::edges::{connect_handles, is_input_port, uuid_simple};
use super::editor::GraphEditor;
use super::types::{catalog_for, GraphAction, HandleKind, NodePickerMenu, NODE_H, NODE_W};
use crate::theme::Palette;

pub fn add_node(project: &mut Project, kind: &str) {
    let n = project.nodes.len() as f32;
    let pos = Pos2::new(120.0 + n * 40.0, 120.0 + n * 30.0);
    add_node_at(project, kind, pos);
}

pub fn add_node_at(project: &mut Project, kind: &str, world: Pos2) -> String {
    let id = format!("{kind}_{}", uuid_simple());
    let mut data = HashMap::new();
    match kind {
        NODE_LOG => {
            data.insert("message".to_string(), DataValue::str("Salom"));
        }
        NODE_ASSIGN => {
            data.insert("name".to_string(), DataValue::str("x"));
            data.insert("value".to_string(), DataValue::typed_i64(0));
        }
        NODE_IF => {
            data.insert("condition".to_string(), DataValue::str("true"));
        }
        graph_model::NODE_UI_PAGE => {
            data.insert("title".to_string(), DataValue::str("Home"));
        }
        graph_model::NODE_UI_BUTTON => {
            data.insert("title".to_string(), DataValue::str("OK"));
        }
        graph_model::NODE_UI_LABEL => {
            data.insert("text".to_string(), DataValue::str("Matn"));
        }
        graph_model::NODE_UI_INPUT => {
            data.insert("placeholder".to_string(), DataValue::str("…"));
        }
        graph_model::NODE_UI_EVENT => {
            data.insert("event".to_string(), DataValue::str("on_click"));
        }
        graph_model::NODE_API_ROUTE => {
            data.insert("path".to_string(), DataValue::str("/api/hello"));
        }
        graph_model::NODE_API_QUERY => {
            data.insert("url".to_string(), DataValue::str("https://api.example.com"));
        }
        graph_model::NODE_DB_READ => {
            data.insert("table".to_string(), DataValue::str("users"));
            data.insert("into".to_string(), DataValue::str("row"));
        }
        NODE_SUBGRAPH => {
            data.insert("module".to_string(), DataValue::str("graphs/auth.qp"));
        }
        NODE_WHILE => {
            data.insert("condition".to_string(), DataValue::str("true"));
        }
        NODE_FOR => {
            data.insert("var".to_string(), DataValue::str("i"));
            data.insert("from".to_string(), DataValue::typed_i64(0));
            data.insert("to".to_string(), DataValue::typed_i64(10));
        }
        NODE_RETURN => {
            data.insert("value".to_string(), DataValue::str(""));
        }
        NODE_SWITCH => {
            data.insert("variable".to_string(), DataValue::str("x"));
            data.insert("cases".to_string(), DataValue::str("1,2"));
            data.insert("case1".to_string(), DataValue::str("1"));
            data.insert("case2".to_string(), DataValue::str("2"));
        }
        NODE_FOREACH => {
            data.insert("collection".to_string(), DataValue::str("users"));
            data.insert("item_var".to_string(), DataValue::str("row"));
        }
        NODE_EXPR => {
            data.insert("name".to_string(), DataValue::str("result"));
            data.insert("expression".to_string(), DataValue::str("a + b"));
        }
        graph_model::NODE_EMIT_UI => {
            data.insert("signal".to_string(), DataValue::str("refresh_ui"));
        }
        _ => {}
    }
    project.nodes.push(Node {
        id: id.clone(),
        kind: kind.to_string(),
        position: graph_model::Position {
            x: (world.x - NODE_W * 0.5) as f64,
            y: (world.y - NODE_H * 0.5) as f64,
        },
        data,
    });
    id
}

fn menu_anchor_screen(anchor: Pos2) -> Pos2 {
    anchor + Vec2::new(20.0, 16.0)
}

impl GraphEditor {
    pub(crate) fn open_picker_at(
        &mut self,
        _ui: &Ui,
        anchor: Pos2,
        world: Pos2,
        pending: Option<(String, HandleKind)>,
    ) {
        self.picker = Some(NodePickerMenu {
            open: true,
            screen_pos: menu_anchor_screen(anchor),
            world_pos: world,
            search: String::new(),
            pending_connect: pending,
        });
        self.picker_just_opened = true;
        self.picker_menu_rect = None;
    }

    pub(crate) fn close_picker(&mut self) {
        self.picker = None;
        self.picker_menu_rect = None;
        self.picker_just_opened = false;
    }

    pub(crate) fn try_close_picker_outside(&mut self, ui: &Ui) {
        if !self.picker.as_ref().is_some_and(|p| p.open) {
            return;
        }
        if !ui.input(|i| i.pointer.primary_pressed()) {
            return;
        }
        let Some(press) = ui.input(|i| i.pointer.interact_pos()) else {
            return;
        };
        if let Some(rect) = self.picker_menu_rect {
            if rect.contains(press) {
                return;
            }
        }
        self.close_picker();
    }

    pub(crate) fn pending_wire(&self) -> Option<(String, HandleKind)> {
        if let Some(c) = &self.connecting {
            return Some(c.clone());
        }
        self.picker
            .as_ref()
            .filter(|p| p.open)
            .and_then(|p| p.pending_connect.clone())
    }

    pub(crate) fn show_node_picker(
        &mut self,
        ui: &Ui,
        project: &mut Project,
        palette: &Palette,
    ) -> Option<GraphAction> {
        let picker = self.picker.as_mut()?;
        if !picker.open {
            return None;
        }
        let screen_pos = picker.screen_pos;
        let world_pos = picker.world_pos;
        let pending = picker.pending_connect.clone();
        let has_start = project
            .nodes
            .iter()
            .any(|n| n.kind == graph_model::NODE_START);
        let search_lower = picker.search.to_lowercase();
        let mut picked: Option<&str> = None;
        let mut close_menu = false;
        let layer_title = format!("{} — {}", project.layer.label(), project.layer.subtitle());
        let title = if pending.is_some() {
            format!("Ulanish ({layer_title})")
        } else {
            format!("Nod qo'shish ({layer_title})")
        };

        let area = egui::Area::new(egui::Id::new("node_picker_menu"))
            .fixed_pos(screen_pos)
            .order(egui::Order::Foreground)
            .constrain(true)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(260.0);
                    ui.label(egui::RichText::new(&title).strong());
                    if pending.is_some() {
                        ui.label(
                            egui::RichText::new(
                                "Sim bo‘sh joyga tushdi — nod tanlang, avtomatik ulanadi",
                            )
                            .small()
                            .color(palette.muted),
                        );
                    }
                    ui.add_space(4.0);
                    let search_id = ui.make_persistent_id("node_picker_search");
                    ui.memory_mut(|m| m.request_focus(search_id));
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Qidiruv").small());
                        ui.add(
                            egui::TextEdit::singleline(&mut picker.search)
                                .id(search_id)
                                .hint_text("log, if, assign…")
                                .desired_width(f32::INFINITY),
                        );
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(220.0)
                        .show(ui, |ui| {
                            for entry in catalog_for(project.layer) {
                                if entry.kind == graph_model::NODE_START
                                    && (has_start || pending.is_some())
                                {
                                    continue;
                                }
                                if !search_lower.is_empty() {
                                    let hay = format!(
                                        "{} {} {}",
                                        entry.title, entry.keywords, entry.kind
                                    )
                                    .to_lowercase();
                                    if !hay.contains(&search_lower) {
                                        continue;
                                    }
                                }
                                let color = (entry.color_fn)(palette);
                                let mat = maturity_label(node_maturity(entry.kind));
                                let btn = egui::Button::new(
                                    egui::RichText::new(format!("  {} [{}]", entry.title, mat))
                                        .color(color),
                                )
                                .fill(palette.surface);
                                if ui.add(btn).clicked() {
                                    picked = Some(entry.kind);
                                }
                                ui.label(
                                    egui::RichText::new(entry.keywords)
                                        .small()
                                        .color(palette.muted),
                                );
                                ui.add_space(4.0);
                            }
                        });
                    ui.separator();
                    if ui.button("Yopish").clicked() {
                        close_menu = true;
                    }
                });
            });
        self.picker_menu_rect = Some(area.response.rect);

        if close_menu {
            self.close_picker();
            return None;
        }

        if let Some(kind) = picked {
            let id = add_node_at(project, kind, world_pos);
            if let Some((anchor_id, anchor_kind)) = pending {
                let other = if is_input_port(anchor_kind) {
                    HandleKind::ExecOut
                } else {
                    HandleKind::ExecIn
                };
                connect_handles(project, &anchor_id, anchor_kind, &id, other);
            }
            self.select_one(id);
            self.close_picker();
            return Some(GraphAction::Changed);
        }
        None
    }
}
