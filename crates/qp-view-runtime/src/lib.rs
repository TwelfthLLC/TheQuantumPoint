//! View domain runtime — render View graphs as live egui UI (Studio preview / future player).

use egui::{Color32, RichText, Ui};
use graph_model::{
    data_get_str, Node, Project, NODE_UI_BUTTON, NODE_UI_EVENT, NODE_UI_INPUT, NODE_UI_LABEL,
    NODE_UI_PAGE,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ViewRuntime {
    pub name: String,
    pub widgets: Vec<ViewWidget>,
}

#[derive(Debug, Clone)]
pub struct ViewWidget {
    pub id: String,
    pub kind: ViewWidgetKind,
}

#[derive(Debug, Clone)]
pub enum ViewWidgetKind {
    Page { title: String },
    Label { text: String },
    Button { title: String },
    Input { placeholder: String },
    Event { event: String },
}

#[derive(Debug, Default, Clone)]
pub struct ViewRuntimeState {
    pub inputs: HashMap<String, String>,
    pub log: Vec<String>,
}

impl ViewRuntimeState {
    pub fn push_log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 64 {
            self.log.remove(0);
        }
    }
}

/// Build runtime tree from a View-layer `Project` (canvas order: top→down, left→right).
pub fn build_from_project(project: &Project) -> ViewRuntime {
    let mut nodes: Vec<&Node> = project
        .nodes
        .iter()
        .filter(|n| n.kind != graph_model::NODE_START)
        .collect();
    nodes.sort_by(|a, b| {
        a.position
            .y
            .partial_cmp(&b.position.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.position
                    .x
                    .partial_cmp(&b.position.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let widgets = nodes.into_iter().filter_map(widget_from_node).collect();

    ViewRuntime {
        name: project.name.clone(),
        widgets,
    }
}

fn widget_from_node(node: &Node) -> Option<ViewWidget> {
    let id = node.id.clone();
    let kind = match node.kind.as_str() {
        NODE_UI_PAGE => ViewWidgetKind::Page {
            title: data_get_str(&node.data, "title").unwrap_or_else(|| "Page".into()),
        },
        NODE_UI_LABEL => ViewWidgetKind::Label {
            text: data_get_str(&node.data, "text").unwrap_or_else(|| "Label".into()),
        },
        NODE_UI_BUTTON => ViewWidgetKind::Button {
            title: data_get_str(&node.data, "title").unwrap_or_else(|| "Button".into()),
        },
        NODE_UI_INPUT => ViewWidgetKind::Input {
            placeholder: data_get_str(&node.data, "placeholder").unwrap_or_else(|| "…".into()),
        },
        NODE_UI_EVENT => ViewWidgetKind::Event {
            event: data_get_str(&node.data, "event").unwrap_or_else(|| "on_click".into()),
        },
        _ => return None,
    };
    Some(ViewWidget { id, kind })
}

pub struct ViewRuntimeTheme {
    pub accent: Color32,
    pub muted: Color32,
    pub panel: Color32,
}

impl Default for ViewRuntimeTheme {
    fn default() -> Self {
        Self {
            accent: Color32::from_rgb(96, 165, 250),
            muted: Color32::from_rgb(140, 150, 170),
            panel: Color32::from_rgb(22, 26, 34),
        }
    }
}

/// Draw interactive View UI into `ui`.
pub fn show_runtime(
    ui: &mut Ui,
    runtime: &ViewRuntime,
    state: &mut ViewRuntimeState,
    theme: &ViewRuntimeTheme,
) {
    egui::Frame::new()
        .fill(theme.panel)
        .inner_margin(12.0)
        .corner_radius(8.0)
        .show(ui, |ui| {
            ui.set_min_width(200.0);
            ui.label(RichText::new(&runtime.name).strong().color(theme.accent));
            ui.add_space(6.0);
            if runtime.widgets.is_empty() {
                ui.label(
                    RichText::new("View nodlari yo‘q — Page, Button, Label qo‘shing")
                        .small()
                        .color(theme.muted),
                );
                return;
            }
            ui.separator();
            for w in &runtime.widgets {
                show_widget(ui, w, state, theme);
                ui.add_space(6.0);
            }
            if !state.log.is_empty() {
                ui.separator();
                ui.label(RichText::new("Hodisalar").small().color(theme.muted));
                for line in state.log.iter().rev().take(8) {
                    ui.label(RichText::new(line).small().monospace());
                }
            }
        });
}

fn show_widget(
    ui: &mut Ui,
    w: &ViewWidget,
    state: &mut ViewRuntimeState,
    theme: &ViewRuntimeTheme,
) {
    match &w.kind {
        ViewWidgetKind::Page { title } => {
            ui.heading(title);
            ui.separator();
        }
        ViewWidgetKind::Label { text } => {
            ui.label(text);
        }
        ViewWidgetKind::Button { title } => {
            if ui.button(title).clicked() {
                state.push_log(format!("click: {} ({title})", w.id));
                fire_bound_events(ui, state, w.id.as_str(), theme);
            }
        }
        ViewWidgetKind::Input { placeholder } => {
            let entry = state.inputs.entry(w.id.clone()).or_default();
            ui.horizontal(|ui| {
                ui.label(RichText::new(&w.id).small().color(theme.muted));
                ui.add(
                    egui::TextEdit::singleline(entry)
                        .hint_text(placeholder)
                        .desired_width(f32::INFINITY),
                );
            });
        }
        ViewWidgetKind::Event { event } => {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡").color(theme.accent));
                ui.label(
                    RichText::new(format!("{event} · {}", w.id))
                        .small()
                        .color(theme.muted),
                );
            });
        }
    }
}

/// When a button fires, also log sibling Event nodes (simple binding by id prefix).
fn fire_bound_events(
    ui: &mut Ui,
    state: &mut ViewRuntimeState,
    button_id: &str,
    theme: &ViewRuntimeTheme,
) {
    let _ = ui;
    let _ = theme;
    state.push_log(format!(
        "event hook for {button_id} (Bridge/Core keyin ulanadi)"
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_model::{data_set_str, GraphLayer, Position};

    #[test]
    fn builds_widgets_from_nodes() {
        let mut btn = graph_model::Node {
            id: "btn1".into(),
            kind: NODE_UI_BUTTON.into(),
            position: Position { x: 0.0, y: 10.0 },
            data: Default::default(),
        };
        data_set_str(&mut btn.data, "title", "OK");
        let p = Project {
            name: "ui".into(),
            layer: GraphLayer::View,
            nodes: vec![
                graph_model::Node {
                    id: "start".into(),
                    kind: graph_model::NODE_START.into(),
                    position: Position { x: 0.0, y: 0.0 },
                    data: Default::default(),
                },
                btn,
            ],
            edges: vec![],
            subgraphs: vec![],
        };
        let rt = build_from_project(&p);
        assert_eq!(rt.widgets.len(), 1);
    }
}
