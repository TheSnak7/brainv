// Re-export modules for use in benchmarks and tests
pub mod compiler;
pub mod io;
pub mod vm;

// Re-export main components if needed
pub use crate::compiler::*;
pub use crate::vm::*;
pub use crate::io::*;
