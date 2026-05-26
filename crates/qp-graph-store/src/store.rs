use graph_model::{Edge, GraphLayer, Node, Position, Project, SubGraphRef, NODE_START};
use std::collections::HashMap;

use crate::dirty::DirtyTracker;

pub type EntityId = u32;

/// Columnar-ish storage: ids + side tables (DOD-friendly growth path).
#[derive(Debug, Clone)]
pub struct GraphStore {
    pub name: String,
    pub layer: GraphLayer,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub id_index: HashMap<String, usize>,
    pub dirty: DirtyTracker,
    /// Sub-graph module references (incremental compile units).
    pub subgraphs: Vec<SubGraphRef>,
}

#[derive(Debug, Clone)]
pub struct GraphSnapshot {
    pub name: String,
    pub layer: GraphLayer,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Default for GraphStore {
    fn default() -> Self {
        Self {
            name: String::new(),
            layer: GraphLayer::Core,
            nodes: Vec::new(),
            edges: Vec::new(),
            id_index: HashMap::new(),
            dirty: DirtyTracker::default(),
            subgraphs: Vec::new(),
        }
    }
}

impl GraphStore {
    pub fn from_project(project: &Project) -> Self {
        let mut store = Self {
            name: project.name.clone(),
            layer: project.layer,
            nodes: project.nodes.clone(),
            edges: project.edges.clone(),
            subgraphs: project.subgraphs.clone(),
            ..Default::default()
        };
        store.rebuild_index();
        store.dirty.clear();
        store
    }

    pub fn to_project(&self) -> Project {
        Project {
            name: self.name.clone(),
            layer: self.layer,
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            subgraphs: self.subgraphs.clone(),
        }
    }

    pub fn snapshot(&self) -> GraphSnapshot {
        GraphSnapshot {
            name: self.name.clone(),
            layer: self.layer,
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }

    pub fn rebuild_index(&mut self) {
        self.id_index.clear();
        for (i, n) in self.nodes.iter().enumerate() {
            self.id_index.insert(n.id.clone(), i);
        }
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.id_index.get(id).and_then(|&i| self.nodes.get(i))
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut Node> {
        let idx = *self.id_index.get(id)?;
        self.dirty.mark_node(id);
        self.nodes.get_mut(idx)
    }

    pub fn set_position(&mut self, id: &str, x: f64, y: f64) {
        if let Some(n) = self.node_mut(id) {
            n.position = Position { x, y };
        }
    }

    pub fn mark_node_dirty(&mut self, id: &str) {
        self.dirty.mark_node(id);
    }

    pub fn mark_structure_dirty(&mut self) {
        self.dirty.mark_structure();
    }

    /// Dirty node ids for incremental compile (empty = use cached IR if valid).
    pub fn take_dirty_compile_set(&mut self) -> Vec<String> {
        if !self.dirty.any_dirty() {
            return Vec::new();
        }
        let structure = self.dirty.structure_dirty();
        let ids = self.dirty.drain_dirty_nodes();
        if structure || ids.is_empty() {
            self.nodes.iter().map(|n| n.id.clone()).collect()
        } else {
            ids
        }
    }

    pub fn add_subgraph(&mut self, id: impl Into<String>, path: impl Into<String>) {
        let sg = SubGraphRef::new(id, path);
        if !self.subgraphs.iter().any(|s| s.id == sg.id) {
            self.subgraphs.push(sg);
            self.dirty.mark_structure();
        }
    }

    pub fn ensure_start_node(&mut self) {
        if self.nodes.iter().any(|n| n.kind == NODE_START) {
            return;
        }
        let node = Node {
            id: "start".to_string(),
            kind: NODE_START.to_string(),
            position: Position { x: 120.0, y: 200.0 },
            data: Default::default(),
        };
        self.insert_node(node);
    }

    pub fn insert_node(&mut self, node: Node) {
        let id = node.id.clone();
        if let Some(&idx) = self.id_index.get(&id) {
            self.nodes[idx] = node;
        } else {
            let idx = self.nodes.len();
            self.nodes.push(node);
            self.id_index.insert(id.clone(), idx);
        }
        self.dirty.mark_node(&id);
    }

    pub fn remove_node(&mut self, id: &str) {
        if let Some(idx) = self.id_index.remove(id) {
            self.nodes.swap_remove(idx);
            self.rebuild_index();
            self.edges.retain(|e| e.source != id && e.target != id);
            self.dirty.mark_structure();
        }
    }

    pub fn outgoing_exec<'a>(&'a self, node_id: &str) -> Vec<&'a Edge> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id && graph_model::is_exec_handle(&e.source_handle))
            .collect()
    }

    pub fn incoming_exec<'a>(&'a self, node_id: &str) -> Vec<&'a Edge> {
        self.edges
            .iter()
            .filter(|e| e.target == node_id && graph_model::is_exec_target(&e.target_handle))
            .collect()
    }

    pub fn outgoing_exec_labeled<'a>(&'a self, node_id: &str, label: &str) -> Option<&'a Edge> {
        self.edges.iter().find(|e| {
            e.source == node_id
                && e.source_handle.eq_ignore_ascii_case(label)
                && graph_model::is_exec_handle(&e.source_handle)
        })
    }
}
