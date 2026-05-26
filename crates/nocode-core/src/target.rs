use graph_model::GraphLayer;
use qp_domain::Domain;
use std::fmt;

/// Target language / artifact kind — only used at **Build**, not at Run (check).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildTarget {
    /// Core → `emit-rust` + sandbox `cargo build` (+ optional `cargo run`).
    Rust,
    /// View → `ui.qpview` + `view_stub.rs`.
    ViewSpec,
    /// Bridge → `routes.rs` + manifest.
    BridgeRoutes,
    /// Core → `emit-wasm` + `wasm32-unknown-unknown` cargo build.
    Wasm,
}

impl BuildTarget {
    pub const ALL: &'static [BuildTarget] = &[
        BuildTarget::Rust,
        BuildTarget::Wasm,
        BuildTarget::ViewSpec,
        BuildTarget::BridgeRoutes,
    ];

    pub fn id(self) -> &'static str {
        match self {
            BuildTarget::Rust => "rust",
            BuildTarget::Wasm => "wasm",
            BuildTarget::ViewSpec => "view",
            BuildTarget::BridgeRoutes => "bridge",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            BuildTarget::Rust => "Rust (cargo)",
            BuildTarget::Wasm => "WASM (wasm32)",
            BuildTarget::ViewSpec => "View spec",
            BuildTarget::BridgeRoutes => "Bridge routes",
        }
    }

    pub fn required_domain(self) -> Domain {
        match self {
            BuildTarget::Rust | BuildTarget::Wasm => Domain::Core,
            BuildTarget::ViewSpec => Domain::View,
            BuildTarget::BridgeRoutes => Domain::Bridge,
        }
    }

    pub fn default_for_layer(layer: GraphLayer) -> Self {
        match layer {
            GraphLayer::Core => BuildTarget::Rust,
            GraphLayer::View => BuildTarget::ViewSpec,
            GraphLayer::Bridge => BuildTarget::BridgeRoutes,
        }
    }

    pub fn available_for_layer(layer: GraphLayer) -> &'static [BuildTarget] {
        match layer {
            GraphLayer::Core => &[BuildTarget::Rust, BuildTarget::Wasm],
            GraphLayer::View => &[BuildTarget::ViewSpec],
            GraphLayer::Bridge => &[BuildTarget::BridgeRoutes],
        }
    }

    pub fn build_subdir(self) -> &'static str {
        match self {
            BuildTarget::Rust => "rust",
            BuildTarget::Wasm => "wasm",
            BuildTarget::ViewSpec => "view",
            BuildTarget::BridgeRoutes => "bridge",
        }
    }

    pub fn matches_layer(self, layer: GraphLayer) -> bool {
        Domain::from_layer(layer) == self.required_domain()
    }
}

impl fmt::Display for BuildTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

pub fn project_build_dir_for(folder: &std::path::Path, target: BuildTarget) -> std::path::PathBuf {
    folder
        .join(".nocode")
        .join("build")
        .join(target.build_subdir())
}
