mod cache;
mod safe_expr;

pub use cache::CompileCache;

use graph_model::{
    data_get_i64, data_get_str, node_support_hint, GraphError, Project, NODE_ASSIGN, NODE_ASYNC,
    NODE_BREAK,
    NODE_CONTINUE, NODE_DB_READ, NODE_EXPR, NODE_FOR, NODE_FOREACH, NODE_IF, NODE_LOG,
    NODE_RETURN, NODE_START, NODE_SUBGRAPH, NODE_SWITCH, NODE_TRY, NODE_WHILE,
};
use ir::{Action, BinOp, CmpOp, Program, SwitchArm, ValueExpr, actions_need_async};
use qp_domain::{ActionValue, ArithOp, Domain, DomainAction, LogicOp, SwitchArm as DomainSwitchArm};
use safe_expr::ParseConditionError;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error("compile: {0}")]
    Message(String),
    #[error("condition: {0}")]
    Condition(#[from] ParseConditionError),
    #[error("domain {domain:?} cannot be lowered to native exec yet (use Core entry graph)")]
    WrongDomain { domain: Domain },
    #[error("node '{id}' ({kind}): {detail}")]
    UnsupportedNode {
        id: String,
        kind: String,
        detail: String,
    },
    #[error("subgraph '{path}': {message}")]
    SubGraphLoad { path: String, message: String },
}

/// Loads a subgraph `.qp` by project-relative path.
pub type SubGraphLoader<'a> = Box<dyn Fn(&str) -> Result<Project, CompileError> + 'a>;

/// Optional project-root context for loading `subgraphs[]` modules.
pub struct CompileContext<'a> {
    pub project_root: &'a Path,
    pub load_subgraph: SubGraphLoader<'a>,
}

impl CompileContext<'_> {
    pub fn noop(_root: &Path) -> CompileContext<'_> {
        CompileContext {
            project_root: Path::new("."),
            load_subgraph: Box::new(|path| {
                Err(CompileError::SubGraphLoad {
                    path: path.to_string(),
                    message: "subgraph loader not configured".into(),
                })
            }),
        }
    }
}

/// Graph (.qp) → domain actions → universal IR.
pub fn compile(project: &Project) -> Result<Program, CompileError> {
    compile_with_context(project, None)
}

/// Compile with subgraph loader (from `project.subgraphs` + `subgraph_call` nodes).
pub fn compile_with_context(
    project: &Project,
    ctx: Option<&CompileContext<'_>>,
) -> Result<Program, CompileError> {
    let domain = Domain::from_layer(project.layer);
    if domain != Domain::Core {
        return Err(CompileError::WrongDomain { domain });
    }
    validate(project)?;
    let start = project
        .nodes
        .iter()
        .find(|n| n.kind == NODE_START)
        .ok_or(GraphError::NoStart)?;

    let mut domain_actions = lower_exec_chain(project, &start.id, ctx)?;

    if let Some(ctx) = ctx {
        for sg in &project.subgraphs {
            let sub = (ctx.load_subgraph)(&sg.path)?;
            let sub_start = sub
                .nodes
                .iter()
                .find(|n| n.kind == NODE_START)
                .ok_or(GraphError::NoStart)?;
            let sub_actions = lower_exec_chain(&sub, &sub_start.id, Some(ctx))?;
            domain_actions.push(DomainAction::Module {
                name: if sg.label.is_empty() {
                    sg.id.clone()
                } else {
                    sg.label.clone()
                },
                actions: sub_actions,
            });
        }
    }

    let actions: Vec<Action> = domain_actions
        .into_iter()
        .map(domain_action_to_ir)
        .collect();

    Ok(Program {
        name: project.name.clone(),
        needs_async_runtime: actions_need_async(&actions),
        actions,
    })
}

/// Incremental compile (dirty exec successors invalidate cache).
pub fn compile_from_dirty(
    project: &Project,
    dirty_nodes: &[String],
) -> Result<Program, CompileError> {
    let mut c = CompileCache::default();
    c.compile_incremental(project, dirty_nodes, None)
}

/// Exec graph successors from `from` (for dirty invalidation).
pub fn exec_successors(project: &Project, from: &str) -> HashSet<String> {
    use std::collections::{HashSet, VecDeque};
    let mut out = HashSet::new();
    let mut q = VecDeque::new();
    q.push_back(from.to_string());
    while let Some(id) = q.pop_front() {
        for e in project.outgoing_exec(&id) {
            if out.insert(e.target.clone()) {
                q.push_back(e.target.clone());
            }
        }
        for label in [
            "true", "false", "done", "body", "case1", "case2", "case3", "case4", "case5",
            "case6", "default", "try", "catch",
        ] {
            if let Some(e) = project.outgoing_exec_labeled(&id, label) {
                if out.insert(e.target.clone()) {
                    q.push_back(e.target.clone());
                }
            }
        }
    }
    out.remove(from);
    out
}

pub(crate) fn domain_action_to_ir(action: DomainAction) -> Action {
    match action {
        DomainAction::Print { message } => Action::Print { message },
        DomainAction::DataStore { name, value } => Action::DataStore {
            name,
            value: value_to_ir(value),
        },
        DomainAction::Branch {
            condition,
            then_body,
            else_body,
        } => Action::Branch {
            condition: value_to_ir(condition),
            then_body: then_body.into_iter().map(domain_action_to_ir).collect(),
            else_body: else_body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::DbRead { table, into_var } => Action::DbRead { table, into_var },
        DomainAction::While { condition, body } => Action::While {
            condition: value_to_ir(condition),
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::For {
            var,
            from,
            to,
            body,
        } => Action::For {
            var,
            from,
            to,
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::ForEach {
            item_var,
            collection,
            body,
        } => Action::ForEach {
            item_var,
            collection,
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Return { value } => Action::Return {
            value: value.map(value_to_ir),
        },
        DomainAction::Switch {
            discriminant,
            arms,
            default_body,
        } => Action::Switch {
            discriminant: value_to_ir(discriminant),
            arms: arms
                .into_iter()
                .map(|a| SwitchArm {
                    label: a.label,
                    body: a.body.into_iter().map(domain_action_to_ir).collect(),
                })
                .collect(),
            default_body: default_body
                .into_iter()
                .map(domain_action_to_ir)
                .collect(),
        },
        DomainAction::Break => Action::Break,
        DomainAction::Continue => Action::Continue,
        DomainAction::Try {
            try_body,
            catch_body,
        } => Action::Try {
            try_body: try_body.into_iter().map(domain_action_to_ir).collect(),
            catch_body: catch_body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Expr { name, value } => Action::Expr {
            name,
            value: value_to_ir(value),
        },
        DomainAction::Async { body } => Action::Async {
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Module { name, actions } => Action::Module {
            name,
            actions: actions.into_iter().map(domain_action_to_ir).collect(),
        },
    }
}

fn value_to_ir(v: ActionValue) -> ValueExpr {
    match v {
        ActionValue::Bool(b) => ValueExpr::Bool(b),
        ActionValue::I64(n) => ValueExpr::I64(n),
        ActionValue::F64(n) => ValueExpr::F64(n),
        ActionValue::Str(s) => ValueExpr::Str(s),
        ActionValue::Ident(name) => ValueExpr::Ident(name),
        ActionValue::Cmp { op, left, right } => ValueExpr::Cmp {
            op: match op {
                qp_domain::CmpOp::Eq => CmpOp::Eq,
                qp_domain::CmpOp::Ne => CmpOp::Ne,
                qp_domain::CmpOp::Lt => CmpOp::Lt,
                qp_domain::CmpOp::Le => CmpOp::Le,
                qp_domain::CmpOp::Gt => CmpOp::Gt,
                qp_domain::CmpOp::Ge => CmpOp::Ge,
            },
            left: Box::new(value_to_ir(*left)),
            right: Box::new(value_to_ir(*right)),
        },
        ActionValue::BinOp { op, left, right } => ValueExpr::BinOp {
            op: match op {
                ArithOp::Add => BinOp::Add,
                ArithOp::Sub => BinOp::Sub,
                ArithOp::Mul => BinOp::Mul,
                ArithOp::Div => BinOp::Div,
            },
            left: Box::new(value_to_ir(*left)),
            right: Box::new(value_to_ir(*right)),
        },
        ActionValue::Logic { op, left, right } => ValueExpr::BinOp {
            op: match op {
                LogicOp::And => BinOp::And,
                LogicOp::Or => BinOp::Or,
            },
            left: Box::new(value_to_ir(*left)),
            right: Box::new(value_to_ir(*right)),
        },
        ActionValue::Not(inner) => ValueExpr::Not(Box::new(value_to_ir(*inner))),
    }
}

fn validate(project: &Project) -> Result<(), GraphError> {
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
        NODE_SUBGRAPH => {
            require_string(node, "module")?;
            Ok(())
        }
        other => Err(GraphError::UnknownNodeType {
            id: node.id.clone(),
            kind: other.to_string(),
        }),
    }
}

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
                let from = data_get_i64(&node.data, "from").ok_or_else(|| GraphError::InvalidNode {
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
                let item_var = data_get_str(&node.data, "item_var").unwrap_or_else(|| "item".into());
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

fn action_value_from_condition(s: &str) -> Result<ActionValue, CompileError> {
    let expr = safe_expr::parse_condition(s)?;
    Ok(ir_expr_to_action_value(expr))
}

fn ir_expr_to_action_value(expr: ValueExpr) -> ActionValue {
    match expr {
        ValueExpr::Bool(b) => ActionValue::Bool(b),
        ValueExpr::I64(n) => ActionValue::I64(n),
        ValueExpr::F64(n) => ActionValue::F64(n),
        ValueExpr::Str(s) => ActionValue::Str(s),
        ValueExpr::Ident(name) => ActionValue::Ident(name),
        ValueExpr::Cmp { op, left, right } => ActionValue::Cmp {
            op: match op {
                CmpOp::Eq => qp_domain::CmpOp::Eq,
                CmpOp::Ne => qp_domain::CmpOp::Ne,
                CmpOp::Lt => qp_domain::CmpOp::Lt,
                CmpOp::Le => qp_domain::CmpOp::Le,
                CmpOp::Gt => qp_domain::CmpOp::Gt,
                CmpOp::Ge => qp_domain::CmpOp::Ge,
            },
            left: Box::new(ir_expr_to_action_value(*left)),
            right: Box::new(ir_expr_to_action_value(*right)),
        },
        ValueExpr::BinOp { op, left, right } => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => ActionValue::BinOp {
                op: match op {
                    BinOp::Add => ArithOp::Add,
                    BinOp::Sub => ArithOp::Sub,
                    BinOp::Mul => ArithOp::Mul,
                    BinOp::Div => ArithOp::Div,
                    _ => ArithOp::Add,
                },
                left: Box::new(ir_expr_to_action_value(*left)),
                right: Box::new(ir_expr_to_action_value(*right)),
            },
            BinOp::And | BinOp::Or => ActionValue::Logic {
                op: match op {
                    BinOp::And => LogicOp::And,
                    BinOp::Or => LogicOp::Or,
                    _ => LogicOp::And,
                },
                left: Box::new(ir_expr_to_action_value(*left)),
                right: Box::new(ir_expr_to_action_value(*right)),
            },
        },
        ValueExpr::Not(inner) => ActionValue::Not(Box::new(ir_expr_to_action_value(*inner))),
    }
}

fn require_action_value(node: &graph_model::Node, key: &str) -> Result<ActionValue, GraphError> {
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

fn parse_action_value(v: &graph_model::DataValue) -> Option<ActionValue> {
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

fn lower_labeled_chain_optional(
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

fn require_string(node: &graph_model::Node, key: &str) -> Result<String, GraphError> {
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

#[cfg(test)]
mod language_tests {
    use super::*;
    use graph_model::{
        data_set_str, Edge, GraphLayer, Node, Position, Project, NODE_LOG,
        NODE_START, NODE_SWITCH, NODE_FOREACH,
    };

    fn wire(project: &mut Project, src: &str, sh: &str, tgt: &str) {
        project.edges.push(Edge {
            id: format!("e-{src}-{sh}-{tgt}"),
            source: src.into(),
            source_handle: sh.into(),
            target: tgt.into(),
            target_handle: "exec".into(),
        });
    }

    #[test]
    fn switch_uses_cases_field() {
        let mut p = Project {
            name: "t".into(),
            layer: GraphLayer::Core,
            nodes: vec![
                Node {
                    id: "s".into(),
                    kind: NODE_START.into(),
                    position: Position { x: 0.0, y: 0.0 },
                    data: Default::default(),
                },
                Node {
                    id: "sw".into(),
                    kind: NODE_SWITCH.into(),
                    position: Position { x: 0.0, y: 0.0 },
                    data: {
                        let mut d = std::collections::HashMap::new();
                        data_set_str(&mut d, "variable", "x");
                        data_set_str(&mut d, "cases", "a,b");
                        d
                    },
                },
                Node {
                    id: "l1".into(),
                    kind: NODE_LOG.into(),
                    position: Position { x: 0.0, y: 0.0 },
                    data: {
                        let mut d = std::collections::HashMap::new();
                        data_set_str(&mut d, "message", "arm-a");
                        d
                    },
                },
            ],
            edges: vec![],
            subgraphs: vec![],
        };
        wire(&mut p, "s", "exec", "sw");
        wire(&mut p, "sw", "case1", "l1");
        let program = compile(&p).expect("compile");
        let switch = program
            .actions
            .iter()
            .find_map(|a| match a {
                Action::Switch { arms, .. } => Some(arms.clone()),
                _ => None,
            })
            .expect("switch action");
        assert_eq!(switch.len(), 1);
        assert_eq!(switch[0].label, "a");
    }

    #[test]
    fn foreach_lowers_to_ir() {
        let mut p = Project {
            name: "t".into(),
            layer: GraphLayer::Core,
            nodes: vec![
                Node {
                    id: "s".into(),
                    kind: NODE_START.into(),
                    position: Position { x: 0.0, y: 0.0 },
                    data: Default::default(),
                },
                Node {
                    id: "fe".into(),
                    kind: NODE_FOREACH.into(),
                    position: Position { x: 0.0, y: 0.0 },
                    data: {
                        let mut d = std::collections::HashMap::new();
                        data_set_str(&mut d, "collection", "users");
                        data_set_str(&mut d, "item_var", "row");
                        d
                    },
                },
            ],
            edges: vec![],
            subgraphs: vec![],
        };
        wire(&mut p, "s", "exec", "fe");
        let program = compile(&p).expect("compile");
        assert!(program.actions.iter().any(|a| matches!(
            a,
            Action::ForEach {
                collection,
                item_var,
                ..
            } if collection == "users" && item_var == "row"
        )));
    }
}
