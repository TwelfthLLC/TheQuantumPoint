use graph_model::{
    data_get_i64, data_get_str, data_set_str, DataValue, Node, NODE_API_QUERY, NODE_API_ROUTE,
    NODE_ASSIGN, NODE_AWAIT, NODE_CALL, NODE_CONST, NODE_DB_READ, NODE_EMIT_UI, NODE_ENUM,
    NODE_EXPR, NODE_FOR, NODE_FOREACH, NODE_FUNCTION, NODE_IF, NODE_IMPORT, NODE_LIST, NODE_LOG,
    NODE_RETURN, NODE_STRUCT, NODE_SUBGRAPH, NODE_SWITCH, NODE_THROW, NODE_UI_BUTTON,
    NODE_UI_EVENT, NODE_UI_INPUT, NODE_UI_LABEL, NODE_UI_PAGE, NODE_WHILE,
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
        self.props_for_from = data_get_i64(&n.data, "from").unwrap_or(0).to_string();
        self.props_for_to = data_get_i64(&n.data, "to").unwrap_or(0).to_string();
        self.props_expression = get_str(n, "expression");
        self.props_variable = get_str(n, "variable");
        self.props_case1 = get_str(n, "case1");
        self.props_case2 = get_str(n, "case2");
        self.props_cases = get_str(n, "cases");
        self.props_for_var = get_str(n, "var");
        self.props_collection = get_str(n, "collection");
        self.props_item_var = get_str(n, "item_var");
        self.props_params = get_str(n, "params");
        self.props_args = get_str(n, "args");
        self.props_fields = get_str(n, "fields");
        self.props_variants = get_str(n, "variants");
        self.props_items = get_str(n, "items");
        self.props_into = get_str(n, "into");
        self.props_label = match n.kind.as_str() {
            NODE_UI_LABEL => get_str(n, "text"),
            NODE_UI_INPUT => get_str(n, "placeholder"),
            NODE_UI_EVENT => get_str(n, "event"),
            NODE_API_QUERY => get_str(n, "url"),
            NODE_DB_READ => get_str(n, "table"),
            NODE_SUBGRAPH | NODE_IMPORT => get_str(n, "module"),
            NODE_EMIT_UI => get_str(n, "signal"),
            NODE_THROW => get_str(n, "message"),
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
            NODE_IF | NODE_WHILE => set_str(n, "condition", &self.props_condition),
            NODE_FOR => {
                set_str(n, "var", &self.props_for_var);
                let from: i64 = self.props_for_from.parse().unwrap_or(0);
                let to: i64 = self.props_for_to.parse().unwrap_or(0);
                n.data
                    .insert("from".to_string(), DataValue::typed_i64(from));
                n.data.insert("to".to_string(), DataValue::typed_i64(to));
            }
            NODE_FOREACH => {
                set_str(n, "collection", &self.props_collection);
                set_str(n, "item_var", &self.props_item_var);
            }
            NODE_RETURN => set_str(n, "value", &self.props_message),
            NODE_SWITCH => {
                set_str(n, "variable", &self.props_variable);
                set_str(n, "cases", &self.props_cases);
                set_str(n, "case1", &self.props_case1);
                set_str(n, "case2", &self.props_case2);
            }
            NODE_EXPR => {
                set_str(n, "name", &self.props_assign_name);
                set_str(n, "expression", &self.props_expression);
            }
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
            NODE_SUBGRAPH | NODE_IMPORT => set_str(n, "module", &self.props_label),
            NODE_EMIT_UI => set_str(n, "signal", &self.props_label),
            NODE_FUNCTION => {
                set_str(n, "name", &self.props_assign_name);
                set_str(n, "params", &self.props_params);
            }
            NODE_CALL => {
                set_str(n, "name", &self.props_assign_name);
                set_str(n, "args", &self.props_args);
                set_str(n, "into", &self.props_into);
            }
            NODE_CONST => {
                set_str(n, "name", &self.props_assign_name);
                let v: i64 = self.props_assign_value.parse().unwrap_or(0);
                n.data.insert("value".to_string(), DataValue::typed_i64(v));
            }
            NODE_LIST => {
                set_str(n, "name", &self.props_assign_name);
                set_str(n, "items", &self.props_items);
            }
            NODE_THROW => set_str(n, "message", &self.props_message),
            NODE_AWAIT => set_str(n, "into", &self.props_into),
            NODE_STRUCT => {
                set_str(n, "name", &self.props_assign_name);
                set_str(n, "fields", &self.props_fields);
            }
            NODE_ENUM => {
                set_str(n, "name", &self.props_assign_name);
                set_str(n, "variants", &self.props_variants);
            }
            _ => {}
        }
        self.dirty = true;
        self.graph_store.mark_node_dirty(&sel);
    }
}
