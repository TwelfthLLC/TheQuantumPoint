//! Quantum Point domain model — Core (logic/DB), View (UI), Bridge (I/O).
//! Visual node kinds map to a single domain; compilation respects domain boundaries.

mod action;
mod domain;
mod ports;

pub use action::{ActionValue, CmpOp, DomainAction, LogicOp};
pub use domain::Domain;
pub use ports::{PortDirection, PortKind, PortSpec};

/// Which domain owns a node kind string.
pub fn domain_for_kind(kind: &str) -> Domain {
    use graph_model::{
        NODE_API_QUERY, NODE_API_ROUTE, NODE_ASSIGN, NODE_DB_READ, NODE_EMIT_UI, NODE_IF, NODE_LOG,
        NODE_START, NODE_SUBGRAPH, NODE_UI_BUTTON, NODE_UI_EVENT, NODE_UI_INPUT, NODE_UI_LABEL,
        NODE_UI_PAGE,
    };
    match kind {
        NODE_START | NODE_LOG | NODE_ASSIGN | NODE_IF | NODE_DB_READ | NODE_SUBGRAPH => {
            Domain::Core
        }
        NODE_UI_PAGE | NODE_UI_BUTTON | NODE_UI_LABEL | NODE_UI_INPUT | NODE_UI_EVENT => {
            Domain::View
        }
        NODE_API_ROUTE | NODE_API_QUERY | NODE_EMIT_UI => Domain::Bridge,
        _ => Domain::Core,
    }
}

/// Standard exec/data ports for IDE (all nodes expose exec in/out where applicable).
pub fn default_ports_for_kind(kind: &str) -> &'static [PortSpec] {
    use graph_model::{NODE_IF, NODE_START};
    use ports::{PORTS_DEFAULT, PORTS_IF, PORTS_START};
    match kind {
        NODE_START => PORTS_START,
        NODE_IF => PORTS_IF,
        _ => PORTS_DEFAULT,
    }
}
