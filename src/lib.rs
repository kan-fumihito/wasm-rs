#![recursion_limit = "1024"]
#![type_length_limit = "2097152"]
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_json;

pub mod binary;
pub mod exec;
pub mod structure;
pub mod validation;
mod wasm_error;
pub use wasm_error::WasmError;
