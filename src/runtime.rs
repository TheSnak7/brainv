use crate::io::IO;
use std::mem;

/// Trampoline to write a byte via the runtime pointer
extern "C" fn write_trampoline(rt_ptr: *mut u8, c: u8) {
    let rt = rt_ptr as *mut Runtime;
    unsafe { (*rt).io.write_byte(c); }
}

/// Trampoline to read a byte via the runtime pointer
extern "C" fn read_trampoline(rt_ptr: *mut u8) -> u8 {
    let rt = rt_ptr as *mut Runtime;
    unsafe { (*rt).io.read_byte() }
}

/// Runtime for executing JIT-compiled Brainfuck code
pub struct Runtime<'a> {
    tape: Vec<u8>,
    io: Box<dyn IO<'a> + 'a>,
    code_ptr: *const u8,
}

impl<'a> Runtime<'a> {
    /// Create a new runtime with the given IO and code pointer
    pub fn new(io: Box<dyn IO<'a> + 'a>, code_ptr: *const u8) -> Self {
        Self { tape: vec![0; 30000], io, code_ptr }
    }

    /// Run the JIT-compiled function
    pub fn run(&mut self) {
        // Cast the code pointer to the BF JIT function signature:
        // fn(*mut u8, *mut u8, extern "C" fn(*mut u8, u8), extern "C" fn(*mut u8) -> u8) -> u8
        let bf_fn = unsafe {
            mem::transmute::<
                *const u8,
                extern "C" fn(
                    *mut u8,
                    *mut u8,
                    extern "C" fn(*mut u8, u8),
                    extern "C" fn(*mut u8) -> u8,
                ) -> u8,
            >(self.code_ptr)
        };
        // Prepare pointers
        let tape_ptr = self.tape.as_mut_ptr();
        let rt_ptr = self as *mut Runtime as *mut u8;
        // Call the BF function
        bf_fn(tape_ptr, rt_ptr, write_trampoline, read_trampoline);
    }

    /// Consume the runtime and return the tape contents
    pub fn tape(self) -> Vec<u8> {
        self.tape
    }
}