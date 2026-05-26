//! Bridge domain emitter — I/O routes and a minimal HTTP listener (no external deps).

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
    let mut route_paths: Vec<(String, String)> = Vec::new();

    for node in &project.nodes {
        match node.kind.as_str() {
            NODE_API_ROUTE => {
                let path = data_get_str(&node.data, "path").unwrap_or_else(|| "/".into());
                let method = data_get_str(&node.data, "method").unwrap_or_else(|| "GET".into());
                writeln!(manifest, "route {} {} {}", node.id, method, path).unwrap();
                writeln!(routes, "        // {} {}", method, path).unwrap();
                route_paths.push((method, path));
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

    let run_stub = bridge_http_server(&route_paths);

    Ok(BridgeOutput {
        routes_rs: routes,
        manifest,
        run_stub,
    })
}

/// Minimal HTTP/1.1 listener on `127.0.0.1:8787` (stdlib only).
pub fn bridge_http_server(routes: &[(String, String)]) -> String {
    let mut s = String::from(
        r#"// Bridge HTTP server — Quantum Point (stdlib)
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn run_bridge_server() {
    let routes: &[(&str, &str)] = &[
"#,
    );
    for (method, path) in routes {
        writeln!(
            s,
            "        ({:?}, {:?}),",
            method.to_ascii_uppercase(),
            path
        )
        .unwrap();
    }
    if routes.is_empty() {
        s.push_str("        (\"GET\", \"/\"),\n");
    }
    s.push_str(
        r#"    ];
    let addr = "127.0.0.1:8787";
    let listener = TcpListener::bind(addr).expect("bind bridge port");
    println!("Bridge listening on http://{addr}/");
    for conn in listener.incoming().flatten() {
        if let Err(e) = handle_connection(conn, routes) {
            eprintln!("bridge connection error: {e}");
        }
    }
}

fn handle_connection(mut stream: TcpStream, routes: &[(&str, &str)]) -> std::io::Result<()> {
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let (status_line, body) =
        if let Some((_, route_path)) = routes.iter().find(|(m, p)| *m == method && *p == path) {
            (
                "200 OK",
                format!("{{\"ok\":true,\"route\":\"{route_path}\"}}"),
            )
        } else {
            ("404 Not Found", "{\"ok\":false,\"error\":\"not found\"}".to_string())
        };
    let response = format!(
        "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}
"#,
    );
    s
}

pub fn bridge_run_stub() -> String {
    bridge_http_server(&[("GET".into(), "/".into())])
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeEmitError {
    #[error("not a Bridge layer graph")]
    WrongLayer(graph_model::GraphLayer),
}
