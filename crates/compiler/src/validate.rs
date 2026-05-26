use crate::node_data::{require_action_value, require_string};
use graph_model::{
    data_get_i64, GraphError, Project, NODE_ASSIGN, NODE_ASYNC, NODE_AWAIT, NODE_BREAK, NODE_CALL,
    NODE_CONST, NODE_CONTINUE, NODE_DB_READ, NODE_ENUM, NODE_EXPR, NODE_FOR, NODE_FOREACH,
    NODE_FUNCTION, NODE_IF, NODE_IMPORT, NODE_LIST, NODE_LOG, NODE_RETURN, NODE_START, NODE_STRUCT,
    NODE_SUBGRAPH, NODE_SWITCH, NODE_THROW, NODE_TRY, NODE_WHILE,
};
use qp_domain::Domain;

pub(crate) fn validate(project: &Project) -> Result<(), GraphError> {
    if project.nodes.is_empty() {
        return Err(GraphError::Empty);
    }

    let starts: Vec<_> = project
        .nodes
        .iter()
        .filter(|n| n.kind == NODE_START)
        .collect();
    if starts.is_empty() {
        return Err(GraphError::NoStart);
    }
    if starts.len() > 1 {
        return Err(GraphError::MultipleStarts(
            starts
                .iter()
                .map(|n| n.id.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    for node in &project.nodes {
        validate_node(node)?;
        if Domain::from_layer(project.layer) == Domain::Core
            && qp_domain::domain_for_kind(&node.kind) != Domain::Core
            && node.kind != NODE_START
        {
            // View/Bridge nodes may exist on canvas but are skipped during Core lowering.
        }
    }

    for edge in &project.edges {
        if project.node(&edge.source).is_none() || project.node(&edge.target).is_none() {
            return Err(GraphError::BrokenEdge {
                id: edge.id.clone(),
            });
        }
    }

    for node in &project.nodes {
        if node.kind == NODE_START {
            continue;
        }
        if matches!(node.kind.as_str(), NODE_FUNCTION | NODE_STRUCT | NODE_ENUM) {
            continue;
        }
        let incoming = project.incoming_exec(&node.id);
        if incoming.is_empty() {
            return Err(GraphError::UnreachableExec {
                id: node.id.clone(),
            });
        }
        if incoming.len() > 1 {
            return Err(GraphError::AmbiguousExec {
                id: node.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_node(node: &graph_model::Node) -> Result<(), GraphError> {
    match node.kind.as_str() {
        NODE_START => Ok(()),
        NODE_LOG => require_string(node, "message").map(|_| ()),
        NODE_ASSIGN => {
            require_string(node, "name")?;
            require_action_value(node, "value").map(|_| ())
        }
        NODE_IF | NODE_WHILE => require_string(node, "condition").map(|_| ()),
        NODE_FOR | NODE_FOREACH => {
            if node.kind == NODE_FOREACH {
                require_string(node, "collection")?;
                return Ok(());
            }
            require_string(node, "var")?;
            if data_get_i64(&node.data, "from").is_none() {
                return Err(GraphError::InvalidNode {
                    id: node.id.clone(),
                    kind: node.kind.clone(),
                    detail: "missing or invalid 'from'".into(),
                });
            }
            if data_get_i64(&node.data, "to").is_none() {
                return Err(GraphError::InvalidNode {
                    id: node.id.clone(),
                    kind: node.kind.clone(),
                    detail: "missing or invalid 'to'".into(),
                });
            }
            Ok(())
        }
        NODE_RETURN => Ok(()),
        NODE_SWITCH => {
            require_string(node, "variable")?;
            Ok(())
        }
        NODE_BREAK | NODE_CONTINUE => Ok(()),
        NODE_TRY => Ok(()),
        NODE_EXPR => {
            require_string(node, "name")?;
            require_string(node, "expression").map(|_| ())
        }
        NODE_ASYNC => Ok(()),
        graph_model::NODE_UI_PAGE
        | graph_model::NODE_UI_BUTTON
        | graph_model::NODE_UI_LABEL
        | graph_model::NODE_UI_INPUT
        | graph_model::NODE_UI_EVENT
        | graph_model::NODE_API_ROUTE
        | graph_model::NODE_API_QUERY
        | graph_model::NODE_EMIT_UI => Ok(()),
        NODE_DB_READ => {
            require_string(node, "table")?;
            Ok(())
        }
        NODE_SUBGRAPH | NODE_IMPORT => {
            require_string(node, "module")?;
            Ok(())
        }
        NODE_CONST => {
            require_string(node, "name")?;
            require_action_value(node, "value").map(|_| ())
        }
        NODE_CALL => require_string(node, "name").map(|_| ()),
        NODE_THROW => require_string(node, "message").map(|_| ()),
        NODE_AWAIT => Ok(()),
        NODE_LIST => {
            require_string(node, "name")?;
            Ok(())
        }
        NODE_FUNCTION => {
            require_string(node, "name")?;
            Ok(())
        }
        NODE_STRUCT => {
            require_string(node, "name")?;
            Ok(())
        }
        NODE_ENUM => {
            require_string(node, "name")?;
            Ok(())
        }
        other => Err(GraphError::UnknownNodeType {
            id: node.id.clone(),
            kind: other.to_string(),
        }),
    }
}
