use graph_model::{data_get_str, GraphError};
use qp_domain::ActionValue;

pub(crate) fn require_string(node: &graph_model::Node, key: &str) -> Result<String, GraphError> {
    let err = |detail: &str| GraphError::InvalidNode {
        id: node.id.clone(),
        kind: node.kind.clone(),
        detail: detail.to_string(),
    };
    match data_get_str(&node.data, key) {
        Some(s) if !s.is_empty() => Ok(s),
        Some(_) => Err(err(&format!("'{key}' must not be empty"))),
        None => Err(err(&format!("missing '{key}'"))),
    }
}

pub(crate) fn require_action_value(
    node: &graph_model::Node,
    key: &str,
) -> Result<ActionValue, GraphError> {
    let err = |detail: &str| GraphError::InvalidNode {
        id: node.id.clone(),
        kind: node.kind.clone(),
        detail: detail.to_string(),
    };
    let v = node
        .data
        .get(key)
        .ok_or_else(|| err(&format!("missing '{key}'")))?;
    parse_action_value(v).ok_or_else(|| err(&format!("invalid '{key}'")))
}

pub(crate) fn parse_action_value(v: &graph_model::DataValue) -> Option<ActionValue> {
    match v {
        graph_model::DataValue::Bool(b) => Some(ActionValue::Bool(*b)),
        graph_model::DataValue::I64(n) => Some(ActionValue::I64(*n)),
        graph_model::DataValue::F64(n) => Some(ActionValue::F64(*n)),
        graph_model::DataValue::Str(s) => Some(ActionValue::Str(s.clone())),
        graph_model::DataValue::Typed { ty, value } => match ty.as_str() {
            "bool" => value.as_bool().map(ActionValue::Bool),
            "i64" => value.as_i64().map(ActionValue::I64),
            "f64" => value.as_i64().map(|n| ActionValue::F64(n as f64)),
            "str" => value.as_str().map(|s| ActionValue::Str(s.to_string())),
            "ident" => value.as_str().map(|s| ActionValue::Ident(s.to_string())),
            _ => None,
        },
    }
}
