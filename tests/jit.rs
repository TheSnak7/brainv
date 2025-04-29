#[cfg(all(windows, target_arch = "aarch64"))]
mod tests {
    use std::{arch::asm, ptr};

    use brainv::{jit::JIT, vm::Op};
    use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
    use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
    use winapi::um::processthreadsapi::{FlushInstructionCache, GetCurrentProcess};

    struct Output {
        byte: u8,
        stack_aligned: bool,
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

    extern "C" fn read_byte(_: *mut u8) -> u8 {
        0
    }

    #[test]
    fn preserves_registers_across_callback() {
        let code = JIT::new(vec![Op::MovR(1), Op::Print]).compile().unwrap();
        let mut tape = [42u8; 32];
        let mut output = Output { byte: 0, stack_aligned: false };
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
