use graph_model::GraphLayer;
use serde::{Deserialize, Serialize};

/// Logical domain — never mix View UI codegen with Core exec lowering in one pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    /// Visual UI graphs (was Surface in early builds; postcard index 0).
    #[default]
    View,
    /// Business logic, control flow, data.
    Core,
    /// External I/O: HTTP, signals, integrations.
    Bridge,
}

impl Domain {
    pub fn label(self) -> &'static str {
        match self {
            Self::View => "View",
            Self::Core => "Core",
            Self::Bridge => "Bridge",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Self::View => "UI",
            Self::Core => "Logic",
            Self::Bridge => "I/O",
        }
    }

    pub fn from_layer(layer: GraphLayer) -> Self {
        match layer {
            GraphLayer::View => Domain::View,
            GraphLayer::Core => Domain::Core,
            GraphLayer::Bridge => Domain::Bridge,
        }
    }

    pub fn to_layer(self) -> GraphLayer {
        match self {
            Domain::View => GraphLayer::View,
            Domain::Core => GraphLayer::Core,
            Domain::Bridge => GraphLayer::Bridge,
        }
    }
}
