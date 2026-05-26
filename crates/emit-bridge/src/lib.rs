//! Bridge domain emitter — I/O routes and integration stubs.

use graph_model::{data_get_str, Project, NODE_API_QUERY, NODE_API_ROUTE, NODE_EMIT_UI};
use qp_domain::Domain;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct BridgeOutput {
    pub routes_rs: String,
    pub manifest: String,
    pub run_stub: String,
}

pub fn emit_bridge(project: &Project) -> Result<BridgeOutput, BridgeEmitError> {
    if Domain::from_layer(project.layer) != Domain::Bridge {
        return Err(BridgeEmitError::WrongLayer(project.layer));
    }
    let mut routes = String::from("// Bridge routes — Quantum Point\n\n");
    writeln!(routes, "pub struct BridgeRouter;").unwrap();
    writeln!(routes, "impl BridgeRouter {{").unwrap();
    writeln!(routes, "    pub fn register() {{").unwrap();

    let mut manifest = String::from("# Bridge manifest\n");

    for node in &project.nodes {
        match node.kind.as_str() {
            NODE_API_ROUTE => {
                let path = data_get_str(&node.data, "path").unwrap_or_else(|| "/".into());
                writeln!(manifest, "route {} -> {}", node.id, path).unwrap();
                writeln!(routes, "        // route {path}").unwrap();
            }
            NODE_API_QUERY => {
                let url = data_get_str(&node.data, "url").unwrap_or_default();
                writeln!(manifest, "query {} -> {}", node.id, url).unwrap();
            }
            NODE_EMIT_UI => {
                let sig = data_get_str(&node.data, "signal").unwrap_or_default();
                writeln!(manifest, "emit {} -> {}", node.id, sig).unwrap();
            }
            _ => {}
        }
    }

    writeln!(routes, "    }}").unwrap();
    writeln!(routes, "}}").unwrap();

    let run_stub = bridge_run_stub();

    Ok(BridgeOutput {
        routes_rs: routes,
        manifest,
        run_stub,
    })
}

pub fn bridge_run_stub() -> String {
    r#"// Bridge runtime stub — replace with axum/hyper for production HTTP.
pub fn run_bridge_stub() {
    println!("Bridge router registered (stub). Integrate axum for real HTTP.");
}
"#
    .to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeEmitError {
    #[error("not a Bridge layer graph")]
    WrongLayer(graph_model::GraphLayer),
}
