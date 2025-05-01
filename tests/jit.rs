#[cfg(all(windows, target_arch = "aarch64"))]
mod tests {
    use std::{arch::asm, mem, ptr};

    use brainv::{compiler::Compiler, jit::JIT, vm::Op};
    use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
    use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
    use winapi::um::processthreadsapi::{FlushInstructionCache, GetCurrentProcess};

    struct Output {
        byte: u8,
        stack_aligned: bool,
        input: u8,
    }

    extern "C" fn write_byte(rt_ptr: *mut u8, byte: u8) {
        let sp: usize;
        unsafe {
            asm!("mov {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
            let output = &mut *(rt_ptr as *mut Output);
            output.byte = byte;
            output.stack_aligned = sp % 16 == 0;
        }
    }

    extern "C" fn read_byte(rt_ptr: *mut u8) -> u8 {
        unsafe { (*(rt_ptr as *mut Output)).input }
    }

    fn run_code(program: Vec<Op>, tape: &mut [u8], input: u8) -> u8 {
        let code = JIT::new(program).compile().unwrap();
        let mut output = Output { byte: 0, stack_aligned: false, input };

        unsafe {
            let code_ptr = VirtualAlloc(
                ptr::null_mut(), code.len(), MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE,
            );
            assert!(!code_ptr.is_null());
            ptr::copy_nonoverlapping(code.as_ptr(), code_ptr as *mut u8, code.len());
            FlushInstructionCache(GetCurrentProcess(), code_ptr, code.len());

            let bf_fn = mem::transmute::<
                *mut u8,
                extern "C" fn(*mut u8, *mut u8, extern "C" fn(*mut u8, u8), extern "C" fn(*mut u8) -> u8) -> u8,
            >(code_ptr as *mut u8);
            bf_fn(tape.as_mut_ptr(), &mut output as *mut Output as *mut u8, write_byte, read_byte);

            assert_ne!(VirtualFree(code_ptr, 0, MEM_RELEASE), 0);
        }

        output.byte
    }

    #[test]
    fn accesses_current_cell() {
        let mut tape = [254, 21, 30, 40, 50, 60, 70, 80];
        let byte = run_code(vec![
            Op::Inc(3), Op::MovR(2), Op::Inc(9), Op::Dec(3), Op::MovL(2), Op::Dec(2), Op::Print,
        ], &mut tape, 0);

        assert_eq!(tape, [255, 21, 36, 40, 50, 60, 70, 80]);
        assert_eq!(byte, 255);
    }

    #[test]
    fn reads_input_into_current_cell() {
        let mut tape = [10, 20, 30, 40, 50, 60, 70, 80];
        let byte = run_code(vec![Op::MovR(2), Op::Read, Op::Print], &mut tape, 201);

        assert_eq!(tape, [10, 20, 201, 40, 50, 60, 70, 80]);
        assert_eq!(byte, 201);
    }

    #[test]
    fn skips_loop_body_when_cell_is_zero() {
        let mut tape = [0u8; 8];
        let byte = run_code(Compiler::new("[>]+.").compile(), &mut tape, 0);

        assert_eq!(tape, [1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(byte, 1);
    }

    #[test]
    fn loops_after_input() {
        let mut tape = [0u8; 8];
        let byte = run_code(Compiler::new(",[->+<]>.").compile(), &mut tape, 7);

        assert_eq!(tape, [0, 7, 0, 0, 0, 0, 0, 0]);
        assert_eq!(byte, 7);
    }

    #[test]
    fn runs_nested_loops() {
        let mut tape = [0u8; 8];
        let byte = run_code(Compiler::new("++[>++[>+<-]<-]>>.").compile(), &mut tape, 0);

        assert_eq!(tape, [0, 0, 4, 0, 0, 0, 0, 0]);
        assert_eq!(byte, 4);
    }

    #[test]
    fn skips_empty_loop() {
        let mut tape = [0u8; 8];
        let byte = run_code(Compiler::new("[]+.").compile(), &mut tape, 0);

        assert_eq!(tape[0], 1);
        assert_eq!(byte, 1);
    }

    #[test]
    fn loop_can_exit_at_end_of_program() {
        let mut tape = [0u8; 8];
        run_code(Compiler::new("+[-]").compile(), &mut tape, 0);

        assert_eq!(tape[0], 0);
        run_code(Compiler::new("[>]").compile(), &mut tape, 0);
        assert_eq!(tape, [0u8; 8]);
    }

    #[test]
    fn rejects_invalid_jump_target() {
        assert!(JIT::new(vec![Op::JmpIfZ(1)]).compile().is_err());
        assert!(JIT::new(vec![Op::JmpIfNZ(1)]).compile().is_err());
    }

    #[test]
    fn rejects_out_of_range_jump() {
        let mut code = vec![Op::Print; 65537];
        code.push(Op::JmpIfNZ(0));

        assert!(JIT::new(code).compile().is_err());
    }

    #[test]
    fn runs_long_instruction_sequences() {
        for count in [255, 256, 257, 4096] {
            let mut tape = vec![0u8; count + 1];
            let program = format!("{}{}{}{}.",
                "+".repeat(257), ">".repeat(count), "-".repeat(257), "<".repeat(count),
            );
            let byte = run_code(Compiler::new(&program).compile(), &mut tape, 0);

            assert_eq!(tape[0], 1);
            assert_eq!(tape[count], 255);
            assert_eq!(byte, 1);
        }
    }

    #[test]
    fn preserves_registers_across_callback() {
        let code = JIT::new(vec![Op::MovR(1), Op::Print]).compile().unwrap();
        let mut tape = [42u8; 32];
        let mut output = Output { byte: 0, stack_aligned: false, input: 0 };
        let changed: usize;

        unsafe {
            let code_ptr = VirtualAlloc(
                ptr::null_mut(), code.len(), MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE,
            );
            assert!(!code_ptr.is_null());
            ptr::copy_nonoverlapping(code.as_ptr(), code_ptr as *mut u8, code.len());
            FlushInstructionCache(GetCurrentProcess(), code_ptr, code.len());

            // Save the caller's registers, then check that the JIT preserves known values.
            asm!(
                "stp x19, x20, [sp, #-32]!",
                "stp x21, x22, [sp, #16]",
                "mov x19, #19",
                "mov x20, #20",
                "mov x21, #21",
                "mov x22, #22",
                "blr x9",
                "cmp x19, #19",
                "cset w0, ne",
                "cmp x20, #20",
                "cset w10, ne",
                "orr w0, w0, w10, lsl #1",
                "cmp x21, #21",
                "cset w10, ne",
                "orr w0, w0, w10, lsl #2",
                "cmp x22, #22",
                "cset w10, ne",
                "orr w0, w0, w10, lsl #3",
                "ldp x21, x22, [sp, #16]",
                "ldp x19, x20, [sp], #32",
                in("x9") code_ptr,
                inlateout("x0") tape.as_mut_ptr() => changed,
                in("x1") &mut output as *mut Output,
                in("x2") write_byte as extern "C" fn(*mut u8, u8),
                in("x3") read_byte as extern "C" fn(*mut u8) -> u8,
                clobber_abi("C"),
            );

            assert_ne!(VirtualFree(code_ptr, 0, MEM_RELEASE), 0);
        }

        assert_eq!(changed, 0, "JIT changed callee-saved registers");
        assert_eq!(output.byte, 42);
        assert!(output.stack_aligned, "Callback stack must be 16-byte aligned");
    }
}
