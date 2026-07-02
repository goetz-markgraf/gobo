//! Editor core modules split by responsibility so state, input, rendering,
//! and supporting behaviors stay easy to test independently.

pub mod buffer;
pub mod cursor;
pub mod history;
pub mod input;
pub mod render;
pub mod search;
pub mod status;
