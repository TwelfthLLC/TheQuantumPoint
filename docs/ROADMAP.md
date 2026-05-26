# Quantum Point — Roadmap status

**Product version:** [0.0.0.1](../VERSION) (alpha)

Updated: phases 1–6 core items are implemented.

## ✅ Phase 1 — Catalog trust

- `DB Read` → `DomainAction::DbRead` → `ir::Action::DbRead` → `emit-rust` (mock)
- `graph-model::node_catalog` — `ready` / `beta` / `planned` labels
- `CompileError::UnsupportedNode` + `node_support_hint`

## ✅ Phase 2 — Run preview

- `qp-runtime` — IR interpreter (Print, Assign, Branch, DbRead mock)
- `check_project` — Core preview lines appended to summary

## ✅ Phase 3 — View preview + runtime

- `emit_view::parse_view_spec`
- **`qp-view-runtime`** — egui Page / Label / Button / Input / Event
- Studio: **View runtime** panel + **View Runtime** mode in the top bar

## ✅ Phase 4 — Subgraph

- `project.subgraphs[]` auto-inline (`CompileContext` + project root)
- `subgraph_call` node (`module` field)
- `compiler::compile_with_context`

## ✅ Phase 5 — WASM

- `emit-wasm` crate
- `BuildTarget::Wasm` + `cargo build --target wasm32-unknown-unknown`

## ✅ Phase 6 — Bridge stub

- `bridge_main.rs` + `bridge_run_stub()` written on build

## 🔜 Next work

- Real DB driver / SQL
- View: graph ↔ runtime layout sync (positions), Bridge event wiring
- Bridge: axum HTTP server
- Subgraph UI: wizard to add modules to manifest
- Per-node incremental IR cache
- Expand `docs` + integration tests
