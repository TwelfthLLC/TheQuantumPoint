//! Node maturity and support hints for IDE + compiler messages.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeMaturity {
    /// Full IR + emit + runtime preview.
    Stable,
    /// IR + emit; behavior may be stubbed.
    Beta,
    /// Listed in picker but not lowered yet.
    Planned,
}

pub fn node_maturity(kind: &str) -> NodeMaturity {
    match kind {
        crate::NODE_START
        | crate::NODE_LOG
        | crate::NODE_ASSIGN
        | crate::NODE_IF
        | crate::NODE_WHILE
        | crate::NODE_FOR
        | crate::NODE_FOREACH
        | crate::NODE_RETURN
        | crate::NODE_BREAK
        | crate::NODE_CONTINUE
        | crate::NODE_EXPR
        | crate::NODE_SWITCH
        | crate::NODE_TRY
        | crate::NODE_ASYNC
        | crate::NODE_DB_READ
        | crate::NODE_FUNCTION
        | crate::NODE_CALL
        | crate::NODE_CONST
        | crate::NODE_THROW
        | crate::NODE_AWAIT
        | crate::NODE_IMPORT
        | crate::NODE_STRUCT
        | crate::NODE_ENUM
        | crate::NODE_LIST => NodeMaturity::Stable,
        crate::NODE_SUBGRAPH => NodeMaturity::Beta,
        crate::NODE_UI_PAGE
        | crate::NODE_UI_BUTTON
        | crate::NODE_UI_LABEL
        | crate::NODE_UI_INPUT
        | crate::NODE_UI_EVENT => NodeMaturity::Beta,
        crate::NODE_API_ROUTE | crate::NODE_API_QUERY | crate::NODE_EMIT_UI => NodeMaturity::Beta,
        _ => NodeMaturity::Planned,
    }
}

pub fn maturity_label(m: NodeMaturity) -> &'static str {
    match m {
        NodeMaturity::Stable => "ready",
        NodeMaturity::Beta => "beta",
        NodeMaturity::Planned => "planned",
    }
}

pub fn node_support_hint(kind: &str) -> &'static str {
    match kind {
        crate::NODE_START => "Entry point only; no codegen.",
        crate::NODE_LOG => "Log → IR print.",
        crate::NODE_ASSIGN => "Assign → IR variable bind.",
        crate::NODE_IF => "If → IR branch (true / false / done).",
        crate::NODE_WHILE => "While → IR loop with body + done ports.",
        crate::NODE_FOR => "For → IR counted loop (from..=to).",
        crate::NODE_FOREACH => "Foreach → iterate mock collection rows.",
        crate::NODE_RETURN => "Return → exit current chain (optional value).",
        crate::NODE_SWITCH => "Switch → match on variable (cases field + caseN ports).",
        crate::NODE_BREAK => "Break → exit innermost loop.",
        crate::NODE_CONTINUE => "Continue → next loop iteration.",
        crate::NODE_TRY => "Try → try/catch (Result-based lowering).",
        crate::NODE_EXPR => "Expr → assign from expression (e.g. a + b).",
        crate::NODE_ASYNC => "Async → block under tokio when emitted.",
        crate::NODE_FUNCTION => "Function → top-level fn (body port, not on main exec chain).",
        crate::NODE_CALL => "Call → invoke a defined function.",
        crate::NODE_CONST => "Const → immutable binding.",
        crate::NODE_THROW => "Throw → error inside try/catch.",
        crate::NODE_AWAIT => "Await → cooperative yield (tokio).",
        crate::NODE_IMPORT => "Import → inline subgraph module.",
        crate::NODE_STRUCT => "Struct → type declaration in emitted Rust.",
        crate::NODE_ENUM => "Enum → type declaration in emitted Rust.",
        crate::NODE_LIST => "List → vec literal binding.",
        crate::NODE_DB_READ => "DB Read → mock table lookup (preview + emit).",
        crate::NODE_SUBGRAPH => "Subgraph → inline module from manifest.",
        crate::NODE_UI_PAGE
        | crate::NODE_UI_BUTTON
        | crate::NODE_UI_LABEL
        | crate::NODE_UI_INPUT
        | crate::NODE_UI_EVENT => "View nodes emit UI spec on View layer.",
        crate::NODE_API_ROUTE | crate::NODE_API_QUERY | crate::NODE_EMIT_UI => {
            "Bridge nodes emit route manifest stubs."
        }
        _ => "This node type is not supported yet.",
    }
}
