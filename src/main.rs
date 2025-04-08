use std::{env, fs, path::Path};

mod compiler;
mod vm;
use crate::compiler::*;
use crate::vm::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);

    let program_path = &args[1];

    let program_text =
        fs::read_to_string(Path::new(program_path)).expect("Failed to read the file");

    let compiler = Compiler::new(&program_text);
    let code = compiler.compile();

    let mut vm = Vm::new(code);

    vm.run();
}
