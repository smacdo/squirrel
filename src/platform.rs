//! Functions and structs that model common platform functionality regardless
//! if running in regular std Rust or wasm Rust.
mod fileio;
mod time;

pub use fileio::*;
pub use time::*;
