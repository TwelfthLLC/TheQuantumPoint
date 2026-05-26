//! Data-oriented graph storage with dirty tracking for incremental compilation.

mod dirty;
mod store;

pub use dirty::DirtyTracker;
pub use store::{EntityId, GraphSnapshot, GraphStore};
