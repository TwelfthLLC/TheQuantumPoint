use crate::ir_map::domain_action_to_ir;
use crate::lower::lower_labeled_chain_optional;
use crate::node_data::require_string;
use crate::values::action_value_from_condition;
use crate::{CompileContext, CompileError};
use graph_model::{data_get_str, Project, NODE_ENUM, NODE_FUNCTION, NODE_STRUCT};
use ir::{EnumDef, FunctionDef, StructDef};
use qp_domain::Domain;

pub(crate) struct CollectedDeclarations {
    pub functions: Vec<FunctionDef>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
}

pub(crate) fn collect_declarations(
    project: &Project,
    ctx: Option<&CompileContext<'_>>,
) -> Result<CollectedDeclarations, CompileError> {
    let mut functions = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();

    for node in &project.nodes {
        if qp_domain::domain_for_kind(&node.kind) != Domain::Core {
            continue;
        }
        match node.kind.as_str() {
            NODE_FUNCTION => {
                let name = require_string(node, "name").map_err(CompileError::from)?;
                let params = parse_csv_idents(data_get_str(&node.data, "params").as_deref());
                let body: Vec<_> = lower_labeled_chain_optional(project, &node.id, "body", ctx)?
                    .unwrap_or_default();
                functions.push(FunctionDef {
                    name,
                    params,
                    body: body.into_iter().map(domain_action_to_ir).collect(),
                });
            }
            NODE_STRUCT => {
                let name = require_string(node, "name").map_err(CompileError::from)?;
                let fields = parse_csv_idents(data_get_str(&node.data, "fields").as_deref());
                structs.push(StructDef { name, fields });
            }
            NODE_ENUM => {
                let name = require_string(node, "name").map_err(CompileError::from)?;
                let variants = parse_csv_idents(data_get_str(&node.data, "variants").as_deref());
                enums.push(EnumDef { name, variants });
            }
            _ => {}
        }
    }

    Ok(CollectedDeclarations {
        functions,
        structs,
        enums,
    })
}

pub(crate) fn parse_csv_exprs(
    s: Option<&str>,
) -> Result<Vec<qp_domain::ActionValue>, CompileError> {
    let Some(s) = s.filter(|t| !t.trim().is_empty()) else {
        return Ok(Vec::new());
    };
    s.split(',')
        .map(|part| action_value_from_condition(part.trim()))
        .collect()
}

pub(crate) fn parse_csv_idents(s: Option<&str>) -> Vec<String> {
    let Some(s) = s.filter(|t| !t.trim().is_empty()) else {
        return Vec::new();
    };
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}
