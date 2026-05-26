use serde::{Deserialize, Serialize};

/// Graf qatlami / domain (postcard: View=0, Core=1, Bridge=2 — View eski Surface bilan mos).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GraphLayer {
    #[default]
    View,
    Core,
    Bridge,
}

impl GraphLayer {
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

    /// Eski UI matni (Surface / Frontend).
    pub fn legacy_surface_label(self) -> &'static str {
        match self {
            Self::View => "Surface",
            Self::Core => "Core",
            Self::Bridge => "Bridge",
        }
    }
}
