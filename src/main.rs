use std::{fs, path::Path};

use brainv::jit::JIT;
use brainv::runtime::Runtime;
use clap::Parser;
use clap::{ValueEnum, command};

use brainv::compiler::*;
use brainv::io::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    filename: String,

    #[arg(short, long, value_enum, default_value_t = IOMode::Batched)]
    io: IOMode,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum IOMode {
    Batched,
    Simple,
    // For benchmarking only
    OnePrint,
}

fn main() {
    let cli = Cli::parse();

    let program_path = Path::new(&cli.filename);

    let io: Box<dyn IO> = match cli.io {
        IOMode::Simple => Box::new(SimpleIO::new()),
        IOMode::Batched => Box::new(BatchedIO::new(200)),
        IOMode::OnePrint => Box::new(BatchedIO::new(100000)),
    };

    let program_text = fs::read_to_string(program_path).expect("Failed to read the file");

    let compiler = Compiler::new(&program_text);
    let code = compiler.compile();
    let mut jit = JIT::new(code);
    let compiled_code = jit.compile();

    if let Ok(code_vec) = compiled_code {
        let mut runtime = Runtime::new(io, code_vec);
        runtime.run();
        //let tape = runtime.tape();
        //println!("Tape: {:?}", tape);
    }

    

    //vm.run();
    //vm.flush_io();
}
