

mod func;
mod instance;
mod mem;
mod numerics;
mod stack;
mod utils;

pub use func::FuncAddr;
pub use mem::MemAddr;
mod table;
pub use table::TableAddr;
mod global;
pub use global::GlobalAddr;
mod val;
pub use instance::{ExternalVal, ImportObjects, ModuleInst};
pub use val::Val;


