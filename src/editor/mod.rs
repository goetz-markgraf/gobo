//! Editor core modules split by responsibility so state, input, rendering,
//! and supporting behaviors stay easy to test independently.

pub mod buffer;
pub mod cursor;
pub mod history;
pub mod input;
pub mod render;
pub mod search;
pub mod status;

// Re-export the selection type so callers can use `gobo::editor::Selection`
// alongside the existing cursor/history re-exports (spec 007, data-model §3).
pub use cursor::Selection;
