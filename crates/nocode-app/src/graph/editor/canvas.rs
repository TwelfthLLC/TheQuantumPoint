use egui::{Pos2, Rect, Sense, Stroke, Ui};
use graph_model::Project;

use super::super::edges::{
    self, connect_handles, disconnect_edges_at_handle, edge_color, edge_label, handle_color,
    hit_handle,
};
use super::super::nodes::paint_node;
use super::super::types::{GraphAction, Marquee, NodeAction, MARQUEE_DRAG_THRESHOLD};
use super::GraphEditor;
use crate::theme::Palette;

impl GraphEditor {
    pub fn show(&mut self, ui: &mut Ui, project: &mut Project, palette: &Palette) -> GraphAction {
        let mut action = GraphAction::None;
        // Chap tortish nodga qoldiriladi; kanvas faqat bosish + oâ€˜ng pan
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        if self.fit_on_next_frame {
            self.fit_view_to_graph(project, rect);
            self.fit_on_next_frame = false;
        }

        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            let remove: Vec<_> = self
                .selected
                .iter()
                .filter(|id| *id != "start")
                .cloned()
                .collect();
            if !remove.is_empty() {
                for id in &remove {
                    project.nodes.retain(|n| n.id != *id);
                    project.edges.retain(|e| e.source != *id && e.target != *id);
                    self.selected.remove(id);
                }
                if self.focus.as_ref().is_some_and(|f| remove.contains(f)) {
                    self.focus = self.selected.iter().next().cloned();
                }
                action = GraphAction::Changed;
            }
        }

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, palette.bg);

        // grid
        let step = 32.0 * self.zoom;
        if step > 8.0 {
            let mut x = (rect.left() + self.pan.x) % step;
            while x < rect.right() {
                painter.line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    Stroke::new(1.0, palette.border.gamma_multiply(0.35)),
                );
                x += step;
            }
            let mut y = (rect.top() + self.pan.y) % step;
            while y < rect.bottom() {
                painter.line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(1.0, palette.border.gamma_multiply(0.35)),
                );
                y += step;
            }
        }

        let pointer = ui.input(|i| i.pointer.interact_pos());
        let hover = ui.input(|i| {
            i.pointer
                .latest_pos()
                .or(i.pointer.hover_pos())
                .or(i.pointer.interact_pos())
        });

        // Chap: tanlash / marquee. Oâ€˜ng bosib turib tortish: pan; qisqa oâ€˜ng bosish: menyu
        let shift = ui.input(|i| i.modifiers.shift);

        if response.secondary_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if !self.pointer_on_node(project, pos) {
                    let world = self.screen_to_world(pos);
                    self.open_picker_at(ui, pos, world, None);
                }
            }
        }
        if ui.input(|i| i.pointer.secondary_down()) {
            let delta = ui.input(|i| i.pointer.delta());
            if delta.length_sq() > 1.0 {
                self.close_picker();
                self.marquee = None;
                self.marquee_armed = None;
                if hover.is_some_and(|p| rect.contains(p))
                    || pointer.is_some_and(|p| rect.contains(p))
                {
                    self.pan += delta;
                }
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close_picker();
        }

        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 && rect.contains(ui.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO))
        {
            self.zoom = (self.zoom * (1.0 + scroll * 0.002)).clamp(0.35, 2.5);
        }

        let mut node_actions = Vec::new();

        for node in project.nodes.clone() {
            let nr = self.node_rect(&node);
            if !rect.intersects(nr) {
                continue;
            }
            let na = paint_node(
                ui,
                &painter,
                self,
                &node,
                nr,
                self.selected.contains(&node.id),
                palette,
            );
            node_actions.push((node.id.clone(), na));
        }

        for (id, na) in node_actions {
            match na {
                NodeAction::Select { additive } => {
                    if additive {
                        self.toggle_select(id);
                    } else {
                        self.select_one(id);
                    }
                    self.marquee = None;
                    self.marquee_armed = None;
                }
                NodeAction::DragStart { offset } => {
                    if !self.selected.contains(&id) {
                        self.select_one(id.clone());
                    }
                    self.drag_node = Some(id.clone());
                    self.drag_offset = offset;
                    self.drag_group = self.selected.iter().cloned().collect();
                    self.marquee = None;
                    self.marquee_armed = None;
                    if let Some(n) = project.node(&id) {
                        self.drag_anchor_start = (n.position.x, n.position.y);
                    }
                    self.drag_starts = self
                        .drag_group
                        .iter()
                        .filter_map(|gid| {
                            project
                                .node(gid)
                                .map(|n| (gid.clone(), n.position.x, n.position.y))
                        })
                        .collect();
                }
                NodeAction::HandleClick(kind) => {
                    self.connecting = Some((id, kind));
                }
                NodeAction::HandleDisconnect(kind) => {
                    if disconnect_edges_at_handle(project, &id, kind) {
                        action = GraphAction::Changed;
                    }
                    self.connecting = None;
                    self.close_picker();
                }
                NodeAction::None => {}
            }
        }

        if self.drag_node.is_some() || self.connecting.is_some() {
            self.marquee = None;
            self.marquee_armed = None;
        } else if ui.input(|i| i.pointer.primary_pressed()) {
            if let Some(pos) = pointer.or(hover) {
                if !self.pointer_on_node(project, pos) {
                    self.marquee_armed = Some(pos);
                }
            }
        } else if ui.input(|i| i.pointer.primary_down()) {
            if let (Some(start), Some(pos)) = (self.marquee_armed, hover) {
                if self.marquee.is_none() && pos.distance(start) >= MARQUEE_DRAG_THRESHOLD {
                    self.marquee = Some(Marquee {
                        start,
                        current: pos,
                    });
                    self.marquee_armed = None;
                }
            }
        }

        if let Some(m) = &mut self.marquee {
            if self.drag_node.is_none() {
                if let Some(pos) = hover {
                    m.current = pos;
                }
                let sel = Rect::from_two_pos(m.start, m.current);
                painter.rect_filled(sel, 0.0, palette.accent.gamma_multiply(0.12));
                painter.rect_stroke(
                    sel,
                    0.0,
                    Stroke::new(1.5, palette.accent.gamma_multiply(0.85)),
                    egui::StrokeKind::Outside,
                );
            }
        }

        if ui.input(|i| i.pointer.primary_released()) {
            if self.drag_node.is_none() && self.marquee.is_some() {
                self.finish_marquee(project, shift);
            }
            self.marquee_armed = None;
        }

        if response.clicked() {
            if let Some(pos) = pointer.or(hover) {
                if !self.pointer_on_node(project, pos)
                    && self.marquee.is_none()
                    && self.marquee_armed.is_none()
                    && !shift
                {
                    self.clear_selection();
                }
            }
        }

        if let Some(drag_id) = self.drag_node.clone() {
            if let Some(pos) = pointer {
                let w = self.screen_to_world(pos);
                let wx = (w.x - self.drag_offset.x) as f64;
                let wy = (w.y - self.drag_offset.y) as f64;
                let dx = wx - self.drag_anchor_start.0;
                let dy = wy - self.drag_anchor_start.1;
                for (gid, sx, sy) in &self.drag_starts {
                    if let Some(node) = project.nodes.iter_mut().find(|n| n.id == *gid) {
                        node.position.x = sx + dx;
                        node.position.y = sy + dy;
                    }
                }
                let _ = drag_id;
                action = GraphAction::Changed;
            }
            if ui.input(|i| i.pointer.primary_released()) {
                self.drag_node = None;
                self.drag_group.clear();
                self.drag_starts.clear();
            }
        }

        if let Some((from_id, from_kind)) = self.connecting.clone() {
            if ui.input(|i| i.pointer.any_released()) {
                let mut linked = false;
                if let Some(pos) = pointer {
                    for node in &project.nodes {
                        if let Some((target_id, target_kind)) =
                            hit_handle(self, node, pos, &from_id, from_kind)
                        {
                            if connect_handles(
                                project,
                                &from_id,
                                from_kind,
                                &target_id,
                                target_kind,
                            ) {
                                linked = true;
                                action = GraphAction::Changed;
                            }
                            break;
                        }
                    }
                    if !linked {
                        let world = self.screen_to_world(pos);
                        self.open_picker_at(ui, pos, world, Some((from_id, from_kind)));
                    }
                }
                self.connecting = None;
            }
        }

        // Simlar nodlar ustida â€” yaxshi koâ€˜rinadi
        for edge in project.edges.clone() {
            if let (Some(a), Some(b)) = (project.node(&edge.source), project.node(&edge.target)) {
                let from = self.handle_pos_for_edge(a, &edge.source_handle);
                let to = self.handle_pos_for_edge_target(b, &edge.target_handle);
                let color = edge_color(&edge.source_handle, palette);
                let label = edge_label(&edge.source_handle);
                edges::draw_wire(&painter, from, to, color, label, false, self.zoom);
            }
        }

        if let Some((ref nid, kind)) = self.pending_wire() {
            if let Some(node) = project.node(nid) {
                let from = self.handle_pos(node, kind);
                let to = if self
                    .picker
                    .as_ref()
                    .is_some_and(|p| p.open && p.pending_connect.is_some())
                {
                    self.picker
                        .as_ref()
                        .map(|p| p.screen_pos)
                        .or_else(|| ui.input(|i| i.pointer.hover_pos()))
                        .unwrap_or(from)
                } else {
                    hover.or(pointer).unwrap_or(from)
                };
                let color = handle_color(kind, palette);
                edges::draw_wire(&painter, from, to, color, None, true, self.zoom);
            }
        }

        if self.picker.as_ref().is_some_and(|p| p.open) {
            if let Some(a) = self.show_node_picker(ui, project, palette) {
                action = a;
            }
            if self.picker_just_opened {
                self.picker_just_opened = false;
            } else {
                self.try_close_picker_outside(ui);
            }
        }

        self.update_canvas_auto_pan(ui, rect, pointer, hover);

        action
    }
}
