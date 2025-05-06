#[cfg(all(windows, target_arch = "aarch64"))]
mod tests {
    use brainv::{compiler::Compiler, io::MemoryIO, jit::JIT, runtime::Runtime};

    #[test]
    fn returns_bounds_error_before_later_io() {
        let code = JIT::new(Compiler::new("+.<,.").compile()).compile().unwrap();
        let mut output = vec![];
        let io = Box::new(MemoryIO::new(&mut output, vec![]));
        let mut runtime = Runtime::new(io, code);

        assert_eq!(runtime.run(), Err("Tape pointer out of bounds".into()));
        let tape = runtime.tape();
        assert_eq!(tape[0], 1);
        assert_eq!(output, vec![1]);
    }

    #[test]
    fn rejects_move_past_last_cell() {
        let program = format!("{}+.", ">".repeat(30000));
        let code = JIT::new(Compiler::new(&program).compile()).compile().unwrap();
        let mut output = vec![];
        let io = Box::new(MemoryIO::new(&mut output, vec![]));
        let mut runtime = Runtime::new(io, code);

        assert_eq!(runtime.run(), Err("Tape pointer out of bounds".into()));
        let tape = runtime.tape();
        assert!(tape.iter().all(|cell| *cell == 0));
        assert!(output.is_empty());
    }

    #[test]
    fn accesses_both_ends_of_runtime_tape() {
        let program = format!("{}+.{}+.", ">".repeat(29999), "<".repeat(29999));
        let code = JIT::new(Compiler::new(&program).compile()).compile().unwrap();
        let mut output = vec![];
        let io = Box::new(MemoryIO::new(&mut output, vec![]));
        let mut runtime = Runtime::new(io, code);

        assert_eq!(runtime.run(), Ok(()));
        let tape = runtime.tape();
        assert_eq!(tape[0], 1);
        assert_eq!(tape[29999], 1);
        assert_eq!(output, vec![1, 1]);
    }
}
