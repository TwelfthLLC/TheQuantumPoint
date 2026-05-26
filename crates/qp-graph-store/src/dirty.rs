use std::collections::HashSet;

/// Tracks entities/nodes that need re-lowering or re-validation.
#[derive(Debug, Default, Clone)]
pub struct DirtyTracker {
    nodes: HashSet<String>,
    graph_revision: u64,
    structure_dirty: bool,
}

impl DirtyTracker {
    pub fn mark_node(&mut self, id: &str) {
        self.nodes.insert(id.to_string());
        self.graph_revision = self.graph_revision.saturating_add(1);
    }

    pub fn mark_structure(&mut self) {
        self.structure_dirty = true;
        self.graph_revision = self.graph_revision.saturating_add(1);
    }

    pub fn mark_all(&mut self, node_ids: impl IntoIterator<Item = String>) {
        self.nodes.extend(node_ids);
        self.graph_revision = self.graph_revision.saturating_add(1);
    }

    pub fn is_dirty(&self, id: &str) -> bool {
        self.structure_dirty || self.nodes.contains(id)
    }

    pub fn any_dirty(&self) -> bool {
        self.structure_dirty || !self.nodes.is_empty()
    }

    pub fn structure_dirty(&self) -> bool {
        self.structure_dirty
    }

    pub fn drain_dirty_nodes(&mut self) -> Vec<String> {
        self.structure_dirty = false;
        self.nodes.drain().collect()
    }

    pub fn revision(&self) -> u64 {
        self.graph_revision
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.structure_dirty = false;
    }
}
