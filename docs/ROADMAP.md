# Quantum Point — Roadmap status

**Product version:** [0.0.0.2](../VERSION)

Updated: Core language constructs and Bridge HTTP listener are product-ready; View canvas sync remains planned.

## ✅ Phase 1 — Catalog trust

- `DB Read` → mock tables (`users`, `orders`) in IR, emit, and runtime
- `graph-model::node_catalog` — maturity labels
- `CompileError::UnsupportedNode` + hints

## ✅ Phase 2 — Run preview

- `qp-runtime` — full Core control flow including `foreach`, dynamic `switch`, `try`/`async`
- `check_project` — Core preview lines in summary

## ✅ Phase 3 — View preview + runtime

- `emit_view::parse_view_spec`
- **`qp-view-runtime`** — egui Page / Label / Button / Input / Event
- Studio: **View runtime** panel + **View Runtime** mode

## ✅ Phase 4 — Subgraph

- `project.subgraphs[]` auto-inline
- `subgraph_call` node
- `compiler::compile_with_context`

## ✅ Phase 5 — WASM

- `emit-wasm` crate
- `BuildTarget::Wasm` + `wasm32-unknown-unknown`

## ✅ Phase 6 — Bridge HTTP

- `bridge_main.rs` with stdlib TCP listener on `127.0.0.1:8787`
- Route manifest from `api_route` nodes (method + path)

## ✅ Core language (v0.0.0.2)

- **Stable:** `while`, `for`, `foreach`, `return`, `break`, `continue`, `expr`, `switch` (comma `cases` + up to 6 ports), `try` (Result lowering), `async` (tokio when used), `db_read` (mock tables)
- **Beta:** `subgraph_call` (needs project root loader)

## 🔜 Next work

- Real DB driver / SQL
- View: graph ↔ runtime layout sync (positions)
- Bridge: axum integration option + event wiring to Core
- Subgraph UI wizard
- Per-node incremental IR cache improvements
- Full wasm-bindgen / browser host
