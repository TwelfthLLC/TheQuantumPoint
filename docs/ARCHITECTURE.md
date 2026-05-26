# Quantum Point — Architecture

**Version:** 0.0.0.2 (alpha) — [VERSION](../VERSION)

## Pipeline

```
.qp (QPGR v2 / QPRJ / QPME + postcard)
    → graph-model::Project (DataValue fields, subgraphs[])
    → qp-graph-store::GraphStore (DOD + DirtyTracker)
    → qp-domain::DomainAction
    → ir::Program (universal IR)          ← Run (language-agnostic)
    → emit-rust | emit-view | emit-bridge ← Build (target selection)
    → sandboxed toolchain (cargo, …)      ← Build (Rust Core)
```

### Studio controls

| Control | Purpose |
|---------|---------|
| **▶ Run** | `check_project` — validation + IR / domain lowering, **no file emit or cargo** |
| **Build** | `build_project` — emit + write; Core/Rust runs `cargo build` |
| **Build & Run** | Core/Rust: `cargo run`; View/Bridge: artifacts only |

`BuildTarget`: `rust` · `view` · `bridge` (must match graph layer).

## Domains

| Domain | Layer | Emitter | Output |
|--------|-------|---------|--------|
| **View** | `GraphLayer::View` | `emit-view` + `qp-view-runtime` | `ui.qpview`, live egui preview |
| **Core** | `GraphLayer::Core` | `emit-rust` | `src/main.rs`, `cargo build/run` |
| **Bridge** | `GraphLayer::Bridge` | `emit-bridge` | `routes.rs`, `bridge_manifest.txt` |

## Crates

| Crate | Role |
|-------|------|
| `graph-model` | `.qp` codec, `DataValue`, `SubGraphRef` |
| `qp-domain` | Domains, `DomainAction`, ports |
| `qp-graph-store` | Graph store, dirty flags, subgraph refs |
| `ir` | `Action`, `ValueExpr` (no raw target code) |
| `compiler` | Lowering + `CompileCache` incremental |
| `emit-rust` | Core → Rust |
| `emit-view` | View → UI spec |
| `emit-bridge` | Bridge → routes manifest |
| `nocode-core` | Projects, pipeline, sandbox |
| `nocode-app` | egui IDE |

## `.qp` format

| Magic | File |
|-------|------|
| `QPGR` v2 | `graphs/*.qp` — nodes, edges, layer, **subgraphs** |
| `QPRJ` | `quantum-point.qp` |
| `QPME` | `.nocode/projects/<id>/meta.qp` |

v1 graph files (without subgraphs) are still readable.

## Node data

`DataValue` — typed enum (inside postcard), not a JSON file.

## Safety

- `If` → `safe_expr` parser only.
- Build dir: `<project>/.nocode/build/` only.
- Profiles: `dev` | `release`.

## Incremental compile

- `GraphStore::take_dirty_compile_set()` — dirty nodes + execution successors.
- `CompileCache::compile_incremental()` — invalidates on dirty, caches last `Program` for Core.

## Scaling path

- Sub-graph modules via `Project.subgraphs` / `GraphStore::add_subgraph`.
- Columnar store ready for archetype expansion in `qp-graph-store`.
