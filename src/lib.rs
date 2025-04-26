// Re-export modules for use in benchmarks and tests
pub mod compiler;
pub mod io;
pub mod vm;
pub mod jit;
pub mod runtime;

// Re-export main components if needed
pub use crate::compiler::*;
pub use crate::vm::*;
pub use crate::io::*;
pub use crate::jit::*;