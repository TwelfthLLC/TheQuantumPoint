use crate::declarations;
use crate::node_data::{require_action_value, require_string};
use crate::values::action_value_from_condition;
use crate::{CompileContext, CompileError};
use graph_model::{
    data_get_i64, data_get_str, node_support_hint, GraphError, Project, NODE_ASSIGN, NODE_ASYNC,
    NODE_AWAIT, NODE_BREAK, NODE_CALL, NODE_CONST, NODE_CONTINUE, NODE_DB_READ, NODE_ENUM,
    NODE_EXPR, NODE_FOR, NODE_FOREACH, NODE_FUNCTION, NODE_IF, NODE_IMPORT, NODE_LIST, NODE_LOG,
    NODE_RETURN, NODE_START, NODE_STRUCT, NODE_SUBGRAPH, NODE_SWITCH, NODE_THROW, NODE_TRY,
    NODE_WHILE,
};
use qp_domain::{ActionValue, Domain, DomainAction, SwitchArm as DomainSwitchArm};

pub(crate) fn lower_exec_chain(
    project: &Project,
    start_id: &str,
    ctx: Option<&CompileContext<'_>>,
) -> Result<Vec<DomainAction>, CompileError> {
    let mut actions = Vec::new();
    let mut current = Some(start_id.to_string());

    while let Some(node_id) = current {
        let node = project
            .node(&node_id)
            .ok_or_else(|| CompileError::Message(format!("node {node_id} missing")))?;

        if qp_domain::domain_for_kind(&node.kind) != Domain::Core && node.kind != NODE_START {
            current = next_exec(project, &node_id)?;
            continue;
        }

        match node.kind.as_str() {
            NODE_START => {
                current = next_exec(project, &node_id)?;
            }
            NODE_DB_READ => {
                let table = require_string(node, "table")?;
                let into_var = data_get_str(&node.data, "into").unwrap_or_else(|| "row".into());
                actions.push(DomainAction::DbRead { table, into_var });
                current = next_exec(project, &node_id)?;
            }
            NODE_SUBGRAPH => {
                let module = require_string(node, "module")?;
                let Some(ctx) = ctx else {
                    return Err(CompileError::UnsupportedNode {
                        id: node.id.clone(),
                        kind: node.kind.clone(),
                        detail: "subgraph_call uchun loyiha root kerak (Studio/CLI)".into(),
                    });
                };
                let rel = project
                    .subgraphs
                    .iter()
                    .find(|s| s.id == module || s.path == module)
                    .map(|s| s.path.as_str())
                    .unwrap_or(module.as_str());
                let sub = (ctx.load_subgraph)(rel)?;
                let sub_start = sub
                    .nodes
                    .iter()
                    .find(|n| n.kind == NODE_START)
                    .ok_or(GraphError::NoStart)?;
                let sub_actions = lower_exec_chain(&sub, &sub_start.id, Some(ctx))?;
                actions.push(DomainAction::Module {
                    name: module,
                    actions: sub_actions,
                });
                current = next_exec(project, &node_id)?;
            }
            NODE_LOG => {
                let message = require_string(node, "message")?;
                actions.push(DomainAction::Print { message });
                current = next_exec(project, &node_id)?;
            }
            NODE_ASSIGN => {
                let name = require_string(node, "name")?;
                let value = require_action_value(node, "value")?;
                actions.push(DomainAction::DataStore { name, value });
                current = next_exec(project, &node_id)?;
            }
            NODE_IF => {
                let cond_str = require_string(node, "condition")?;
                let condition = action_value_from_condition(&cond_str)?;

                let then_id = project
                    .outgoing_exec_labeled(&node_id, "true")
                    .or_else(|| project.outgoing_exec_labeled(&node_id, "exec"))
                    .map(|e| e.target.clone());
                let else_id = project
                    .outgoing_exec_labeled(&node_id, "false")
                    .map(|e| e.target.clone());

                let then_body = if let Some(id) = then_id {
                    lower_exec_chain(project, &id, ctx)?
                } else {
                    Vec::new()
                };
                let else_body = if let Some(id) = else_id {
                    lower_exec_chain(project, &id, ctx)?
                } else {
                    Vec::new()
                };

                actions.push(DomainAction::Branch {
                    condition,
                    then_body,
                    else_body,
                });

                current = project
                    .outgoing_exec_labeled(&node_id, "done")
                    .map(|e| e.target.clone());
            }
            NODE_WHILE => {
                let cond_str = require_string(node, "condition")?;
                let condition = action_value_from_condition(&cond_str)?;
                let body = lower_labeled_chain(project, &node_id, "body", ctx)?;
                actions.push(DomainAction::While { condition, body });
                current = project
                    .outgoing_exec_labeled(&node_id, "done")
                    .map(|e| e.target.clone());
            }
            NODE_FOR => {
                let var = require_string(node, "var")?;
                let from =
                    data_get_i64(&node.data, "from").ok_or_else(|| GraphError::InvalidNode {
                        id: node.id.clone(),
                        kind: node.kind.clone(),
                        detail: "missing 'from'".into(),
                    })?;
                let to = data_get_i64(&node.data, "to").ok_or_else(|| GraphError::InvalidNode {
                    id: node.id.clone(),
                    kind: node.kind.clone(),
                    detail: "missing 'to'".into(),
                })?;
                let body = lower_labeled_chain(project, &node_id, "body", ctx)?;
                actions.push(DomainAction::For {
                    var,
                    from,
                    to,
                    body,
                });
                current = project
                    .outgoing_exec_labeled(&node_id, "done")
                    .map(|e| e.target.clone());
            }
            NODE_RETURN => {
                let value = data_get_str(&node.data, "value")
                    .filter(|s| !s.is_empty())
                    .map(|s| action_value_from_condition(s.as_str()))
                    .transpose()?;
                actions.push(DomainAction::Return { value });
                return Ok(actions);
            }
            NODE_SWITCH => {
                let var = require_string(node, "variable")?;
                let discriminant = ActionValue::Ident(var);
                let arms = lower_switch_arms(project, &node_id, node, ctx)?;
                let default_body =
                    lower_labeled_chain(project, &node_id, "default", ctx).unwrap_or_default();
                actions.push(DomainAction::Switch {
                    discriminant,
                    arms,
                    default_body,
                });
                current = project
                    .outgoing_exec_labeled(&node_id, "done")
                    .map(|e| e.target.clone());
            }
            NODE_FOREACH => {
                let collection = require_string(node, "collection")?;
                let item_var =
                    data_get_str(&node.data, "item_var").unwrap_or_else(|| "item".into());
                let body = lower_labeled_chain(project, &node_id, "body", ctx)?;
                actions.push(DomainAction::ForEach {
                    item_var,
                    collection,
                    body,
                });
                current = project
                    .outgoing_exec_labeled(&node_id, "done")
                    .map(|e| e.target.clone());
            }
            NODE_BREAK => {
                actions.push(DomainAction::Break);
                return Ok(actions);
            }
            NODE_CONTINUE => {
                actions.push(DomainAction::Continue);
                return Ok(actions);
            }
            NODE_TRY => {
                let try_body = lower_labeled_chain(project, &node_id, "try", ctx)?;
                let catch_body = lower_labeled_chain(project, &node_id, "catch", ctx)?;
                actions.push(DomainAction::Try {
                    try_body,
                    catch_body,
                });
                current = project
                    .outgoing_exec_labeled(&node_id, "done")
                    .map(|e| e.target.clone());
            }
            NODE_EXPR => {
                let name = require_string(node, "name")?;
                let expr_str = require_string(node, "expression")?;
                let value = action_value_from_condition(&expr_str)?;
                actions.push(DomainAction::Expr { name, value });
                current = next_exec(project, &node_id)?;
            }
            NODE_ASYNC => {
                let body = lower_labeled_chain(project, &node_id, "body", ctx)?;
                actions.push(DomainAction::Async { body });
                current = project
                    .outgoing_exec_labeled(&node_id, "done")
                    .map(|e| e.target.clone());
            }
            NODE_CONST => {
                let name = require_string(node, "name")?;
                let value = require_action_value(node, "value")?;
                actions.push(DomainAction::Const { name, value });
                current = next_exec(project, &node_id)?;
            }
            NODE_LIST => {
                let name = require_string(node, "name")?;
                let items =
                    declarations::parse_csv_exprs(data_get_str(&node.data, "items").as_deref())?;
                actions.push(DomainAction::ListStore { name, items });
                current = next_exec(project, &node_id)?;
            }
            NODE_CALL => {
                let name = require_string(node, "name")?;
                let args =
                    declarations::parse_csv_exprs(data_get_str(&node.data, "args").as_deref())?;
                let into = data_get_str(&node.data, "into").filter(|s| !s.is_empty());
                actions.push(DomainAction::Call { name, args, into });
                current = next_exec(project, &node_id)?;
            }
            NODE_THROW => {
                let message = require_string(node, "message")?;
                actions.push(DomainAction::Throw { message });
                current = next_exec(project, &node_id)?;
            }
            NODE_AWAIT => {
                let binding = data_get_str(&node.data, "into").filter(|s| !s.is_empty());
                actions.push(DomainAction::Await { binding });
                current = next_exec(project, &node_id)?;
            }
            NODE_IMPORT => {
                let module = require_string(node, "module")?;
                let inline = if let Some(ctx) = ctx {
                    let rel = project
                        .subgraphs
                        .iter()
                        .find(|s| s.id == module || s.path == module)
                        .map(|s| s.path.as_str())
                        .unwrap_or(module.as_str());
                    if let Ok(sub) = (ctx.load_subgraph)(rel) {
                        let sub_start = sub
                            .nodes
                            .iter()
                            .find(|n| n.kind == NODE_START)
                            .ok_or(GraphError::NoStart)?;
                        Some(lower_exec_chain(&sub, &sub_start.id, Some(ctx))?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                actions.push(DomainAction::Module {
                    name: module,
                    actions: inline.unwrap_or_default(),
                });
                current = next_exec(project, &node_id)?;
            }
            NODE_FUNCTION | NODE_STRUCT | NODE_ENUM => {
                current = next_exec(project, &node_id)?;
            }
            other => {
                return Err(CompileError::UnsupportedNode {
                    id: node.id.clone(),
                    kind: other.to_string(),
                    detail: node_support_hint(other).to_string(),
                });
            }
        }
    }

    Ok(actions)
}

fn switch_case_labels(node: &graph_model::Node) -> Vec<String> {
    if let Some(cases) = data_get_str(&node.data, "cases") {
        let labels: Vec<String> = cases
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !labels.is_empty() {
            return labels;
        }
    }
    vec![
        data_get_str(&node.data, "case1").unwrap_or_else(|| "1".into()),
        data_get_str(&node.data, "case2").unwrap_or_else(|| "2".into()),
    ]
}

fn lower_switch_arms(
    project: &Project,
    node_id: &str,
    node: &graph_model::Node,
    ctx: Option<&CompileContext<'_>>,
) -> Result<Vec<DomainSwitchArm>, CompileError> {
    let labels = switch_case_labels(node);
    let mut arms = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        let handle = format!("case{}", i + 1);
        if let Some(body) = lower_labeled_chain_optional(project, node_id, &handle, ctx)? {
            arms.push(DomainSwitchArm {
                label: label.clone(),
                body,
            });
        }
    }
    Ok(arms)
}

fn lower_labeled_chain(
    project: &Project,
    node_id: &str,
    label: &str,
    ctx: Option<&CompileContext<'_>>,
) -> Result<Vec<DomainAction>, CompileError> {
    if let Some(edge) = project.outgoing_exec_labeled(node_id, label) {
        lower_exec_chain(project, &edge.target, ctx)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn lower_labeled_chain_optional(
    project: &Project,
    node_id: &str,
    label: &str,
    ctx: Option<&CompileContext<'_>>,
) -> Result<Option<Vec<DomainAction>>, CompileError> {
    if project.outgoing_exec_labeled(node_id, label).is_some() {
        Ok(Some(lower_labeled_chain(project, node_id, label, ctx)?))
    } else {
        Ok(None)
    }
}

fn next_exec(project: &Project, node_id: &str) -> Result<Option<String>, CompileError> {
    let outs = project.outgoing_exec(node_id);
    let exec_out = outs
        .iter()
        .find(|e| e.source_handle.eq_ignore_ascii_case("exec"))
        .or_else(|| outs.first());

    Ok(exec_out.map(|e| e.target.clone()))
}
