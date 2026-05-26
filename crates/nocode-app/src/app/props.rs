use graph_model::{
    data_get_i64, data_get_str, data_set_str, DataValue, Node, NODE_API_QUERY, NODE_API_ROUTE,
    NODE_ASSIGN, NODE_DB_READ, NODE_EMIT_UI, NODE_IF, NODE_LOG, NODE_SUBGRAPH, NODE_UI_BUTTON,
    NODE_UI_EVENT, NODE_UI_INPUT, NODE_UI_LABEL, NODE_UI_PAGE,
};

use super::NoCodeApp;

pub(crate) fn get_str(n: &Node, key: &str) -> String {
    data_get_str(&n.data, key).unwrap_or_default()
}

pub(crate) fn get_i64(n: &Node, key: &str) -> i64 {
    data_get_i64(&n.data, key).unwrap_or(0)
}

pub(crate) fn set_str(n: &mut Node, key: &str, val: &str) {
    data_set_str(&mut n.data, key, val);
}

impl NoCodeApp {
    pub(crate) fn load_props(&mut self, n: &Node) {
        self.props_message = get_str(n, "message");
        self.props_condition = get_str(n, "condition");
        self.props_assign_name = get_str(n, "name");
        self.props_assign_value = get_i64(n, "value").to_string();
        self.props_label = match n.kind.as_str() {
            NODE_UI_LABEL => get_str(n, "text"),
            NODE_UI_INPUT => get_str(n, "placeholder"),
            NODE_UI_EVENT => get_str(n, "event"),
            NODE_API_QUERY => get_str(n, "url"),
            NODE_DB_READ => get_str(n, "table"),
            NODE_SUBGRAPH => get_str(n, "module"),
            NODE_EMIT_UI => get_str(n, "signal"),
            NODE_UI_PAGE | NODE_UI_BUTTON => get_str(n, "title"),
            NODE_API_ROUTE => get_str(n, "path"),
            _ => String::new(),
        };
    }

    pub(crate) fn apply_props_to_selected(&mut self) {
        let Some(sel) = self.editor.focus.clone() else {
            return;
        };
        let Some(p) = self.project.as_mut() else {
            return;
        };
        let Some(n) = p.nodes.iter_mut().find(|n| n.id == sel) else {
            return;
        };
        match n.kind.as_str() {
            NODE_LOG => set_str(n, "message", &self.props_message),
            NODE_IF => set_str(n, "condition", &self.props_condition),
            NODE_ASSIGN => {
                set_str(n, "name", &self.props_assign_name);
                let v: i64 = self.props_assign_value.parse().unwrap_or(0);
                n.data.insert("value".to_string(), DataValue::typed_i64(v));
            }
            NODE_UI_PAGE | NODE_UI_BUTTON => set_str(n, "title", &self.props_label),
            NODE_UI_LABEL => set_str(n, "text", &self.props_label),
            NODE_UI_INPUT => set_str(n, "placeholder", &self.props_label),
            NODE_UI_EVENT => set_str(n, "event", &self.props_label),
            NODE_API_ROUTE => set_str(n, "path", &self.props_label),
            NODE_API_QUERY => set_str(n, "url", &self.props_label),
            NODE_DB_READ => {
                set_str(n, "table", &self.props_label);
            }
            NODE_SUBGRAPH => set_str(n, "module", &self.props_label),
            NODE_EMIT_UI => set_str(n, "signal", &self.props_label),
            _ => {}
        }
        self.dirty = true;
        self.graph_store.mark_node_dirty(&sel);
    }
}
