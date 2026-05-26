mod canvas;

use egui::{Pos2, Rect, Vec2};
use graph_model::{Node, Project};
use std::collections::HashSet;

use super::edges::parse_source_handle;
use super::types::{HandleKind, Marquee, NodePickerMenu, NODE_H, NODE_W};

pub struct GraphEditor {
    pub pan: Vec2,
    pub zoom: f32,
    pub selected: HashSet<String>,
    pub focus: Option<String>,
    pub(crate) drag_node: Option<String>,
    pub(crate) drag_offset: Vec2,
    pub(crate) drag_group: Vec<String>,
    pub(crate) drag_anchor_start: (f64, f64),
    pub(crate) drag_starts: Vec<(String, f64, f64)>,
    pub(crate) connecting: Option<(String, HandleKind)>,
    pub(crate) picker: Option<NodePickerMenu>,
    pub(crate) picker_menu_rect: Option<Rect>,
    pub(crate) picker_just_opened: bool,
    pub(crate) marquee: Option<Marquee>,
    pub(crate) marquee_armed: Option<Pos2>,
    pub(crate) fit_on_next_frame: bool,
}

impl Default for GraphEditor {
    fn default() -> Self {
        Self {
            pan: Vec2::new(80.0, 60.0),
            zoom: 1.0,
            selected: HashSet::new(),
            focus: None,
            drag_node: None,
            drag_offset: Vec2::ZERO,
            drag_group: Vec::new(),
            drag_anchor_start: (0.0, 0.0),
            drag_starts: Vec::new(),
            connecting: None,
            picker: None,
            picker_menu_rect: None,
            picker_just_opened: false,
            marquee: None,
            marquee_armed: None,
            fit_on_next_frame: false,
        }
    }
}

impl GraphEditor {
    pub fn node_screen_size(&self) -> Vec2 {
        Vec2::new(NODE_W * self.zoom, NODE_H * self.zoom)
    }

    pub fn node_rect(&self, node: &Node) -> Rect {
        let p = self.world_to_screen(egui::pos2(node.position.x as f32, node.position.y as f32));
        Rect::from_min_size(p, self.node_screen_size())
    }

    pub fn world_to_screen(&self, p: Pos2) -> Pos2 {
        (p.to_vec2() * self.zoom + self.pan).to_pos2()
    }

    pub fn screen_to_world(&self, p: Pos2) -> Pos2 {
        ((p.to_vec2() - self.pan) / self.zoom).to_pos2()
    }

    pub fn request_fit_view(&mut self) {
        self.fit_on_next_frame = true;
    }

    fn fit_view_to_graph(&mut self, project: &Project, canvas: Rect) {
        if project.nodes.is_empty() {
            return;
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for n in &project.nodes {
            let x = n.position.x as f32;
            let y = n.position.y as f32;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + NODE_W);
            max_y = max_y.max(y + NODE_H);
        }
        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;
        let span = (max_x - min_x).max(max_y - min_y).max(200.0);
        let zoom = (canvas.width().min(canvas.height()) * 0.85 / span).clamp(0.35, 2.5);
        self.zoom = zoom;
        let center = canvas.center();
        self.pan = Vec2::new(center.x - cx * zoom, center.y - cy * zoom);
    }

    pub fn handle_pos(&self, node: &Node, kind: HandleKind) -> Pos2 {
        let r = self.node_rect(node);
        match kind {
            HandleKind::ExecIn => Pos2::new(r.left(), r.center().y),
            HandleKind::ExecOut => Pos2::new(r.right(), r.center().y),
            HandleKind::TrueOut => Pos2::new(r.right(), r.top() + r.height() * 0.3),
            HandleKind::FalseOut => Pos2::new(r.right(), r.top() + r.height() * 0.7),
            HandleKind::BodyOut => Pos2::new(r.right(), r.top() + r.height() * 0.35),
            HandleKind::TryOut => Pos2::new(r.right(), r.top() + r.height() * 0.3),
            HandleKind::CatchOut => Pos2::new(r.right(), r.top() + r.height() * 0.7),
            HandleKind::Case1Out => Pos2::new(r.right(), r.top() + r.height() * 0.2),
            HandleKind::Case2Out => Pos2::new(r.right(), r.top() + r.height() * 0.35),
            HandleKind::Case3Out => Pos2::new(r.right(), r.top() + r.height() * 0.5),
            HandleKind::Case4Out => Pos2::new(r.right(), r.top() + r.height() * 0.65),
            HandleKind::Case5Out => Pos2::new(r.right(), r.top() + r.height() * 0.8),
            HandleKind::Case6Out => Pos2::new(r.right(), r.top() + r.height() * 0.9),
            HandleKind::DefaultOut => Pos2::new(r.right(), r.top() + r.height() * 0.95),
            HandleKind::DoneOut => Pos2::new(r.center().x, r.bottom()),
        }
    }

    pub(crate) fn handle_pos_for_edge(&self, node: &Node, handle: &str) -> Pos2 {
        self.handle_pos(node, parse_source_handle(handle))
    }

    pub(crate) fn handle_pos_for_edge_target(&self, node: &Node, handle: &str) -> Pos2 {
        match handle.to_ascii_lowercase().as_str() {
            h if matches!(h, "true" | "false" | "body" | "default" | "try" | "catch")
                || h.starts_with("case") =>
            {
                self.handle_pos(node, HandleKind::ExecIn)
            }
            _ => self.handle_pos(node, HandleKind::ExecIn),
        }
    }
}
