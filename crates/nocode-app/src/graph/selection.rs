use egui::Pos2;
use graph_model::Project;
use std::collections::HashSet;

use super::editor::GraphEditor;
use super::nodes::handle_hit_zones;

impl GraphEditor {
    pub(crate) fn select_one(&mut self, id: String) {
        self.selected.clear();
        self.selected.insert(id.clone());
        self.focus = Some(id);
    }

    pub(crate) fn toggle_select(&mut self, id: String) {
        if self.selected.contains(&id) {
            self.selected.remove(&id);
            if self.focus.as_deref() == Some(id.as_str()) {
                self.focus = self.selected.iter().next().cloned();
            }
        } else {
            self.selected.insert(id.clone());
            self.focus = Some(id);
        }
    }

    pub(crate) fn clear_selection(&mut self) {
        self.selected.clear();
        self.focus = None;
    }

    pub(crate) fn pointer_on_node(&self, project: &Project, pos: Pos2) -> bool {
        project.nodes.iter().any(|n| {
            let nr = self.node_rect(n);
            if nr.expand(6.0).contains(pos) {
                return true;
            }
            handle_hit_zones(n, nr, self.zoom)
                .iter()
                .any(|(_, hit)| hit.contains(pos))
        })
    }

    pub(crate) fn finish_marquee(&mut self, project: &Project, additive: bool) {
        let Some(m) = self.marquee.take() else {
            return;
        };
        let sel_rect = egui::Rect::from_two_pos(m.start, m.current);
        let hits: HashSet<String> = project
            .nodes
            .iter()
            .filter(|n| sel_rect.intersects(self.node_rect(n)))
            .map(|n| n.id.clone())
            .collect();
        if hits.is_empty() {
            if !additive {
                self.clear_selection();
            }
            return;
        }
        if additive {
            for id in hits {
                self.selected.insert(id);
            }
            self.focus = self.selected.iter().next().cloned();
        } else {
            self.selected = hits;
            self.focus = self.selected.iter().next().cloned();
        }
    }
}
