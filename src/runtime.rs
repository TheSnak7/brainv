use crate::io::IO;
use std::mem;
use std::ptr;

#[cfg(windows)]
use winapi::um::memoryapi::VirtualAlloc;
#[cfg(windows)]
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
#[cfg(windows)]
use winapi::shared::basetsd::SIZE_T;
#[cfg(windows)]
use winapi::um::processthreadsapi::{GetCurrentProcess, FlushInstructionCache};

/// Trampoline to write a byte via the runtime pointer
extern "C" fn write_trampoline(rt_ptr: *mut u8, c: u8) {
    // DEBUG: show the runtime pointer and byte
    //eprintln!("[JIT DEBUG] write_trampoline rt_ptr={:p}, byte=0x{:02x}", rt_ptr, c);
    let rt = rt_ptr as *mut Runtime;
    unsafe { (*rt).io.write_byte(c); }
}

/// Trampoline to read a byte via the runtime pointer
extern "C" fn read_trampoline(rt_ptr: *mut u8) -> u8 {
    // DEBUG: show the runtime pointer for read
    //eprintln!("[JIT DEBUG] read_trampoline rt_ptr={:p}", rt_ptr);
    let rt = rt_ptr as *mut Runtime;
    unsafe { (*rt).io.read_byte() }
}

/// Runtime for executing JIT-compiled Brainfuck code
pub struct Runtime<'a> {
    tape: Vec<u8>,
    io: Box<dyn IO<'a> + 'a>,
    code: Vec<u8>,
}

impl<'a> Runtime<'a> {
    /// Create a new runtime with the given IO and code pointer
    pub fn new(io: Box<dyn IO<'a> + 'a>, code: Vec<u8>) -> Self {
        Self { tape: vec![0; 30000], io, code }
    }

    /// Run the JIT-compiled function
    pub fn run(&mut self) {
        // DEBUG: dump generated JIT code as 32-bit words
        /*println!("Generated JIT code ({} bytes):", self.code.len());
        for (i, chunk) in self.code.chunks(4).enumerate() {
            let bytes = [
                chunk.get(0).copied().unwrap_or(0),
                chunk.get(1).copied().unwrap_or(0),
                chunk.get(2).copied().unwrap_or(0),
                chunk.get(3).copied().unwrap_or(0),
            ];
            let word = u32::from_le_bytes(bytes);
            println!("{:04x}: 0x{:08x}", i * 4, word);
        }*/

        // Allocate an RWX buffer in one go
        let code_ptr = unsafe {
            VirtualAlloc(
                ptr::null_mut(),
                self.code.len() as SIZE_T,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };
        if code_ptr.is_null() {
            panic!("VirtualAlloc failed");
        }
        // Copy the JIT bytes into the new buffer and flush I-cache
        unsafe {
            ptr::copy_nonoverlapping(self.code.as_ptr(), code_ptr as *mut u8, self.code.len());
            // Flush the instruction cache so CPU sees changes
            FlushInstructionCache(GetCurrentProcess(), code_ptr, self.code.len() as SIZE_T);
        }

        // Cast the code pointer to the BF JIT function signature:
        // fn(*mut u8, *mut u8, extern "C" fn(*mut u8, u8), extern "C" fn(*mut u8) -> u8) -> u8
        let bf_fn = unsafe {
            mem::transmute::<
                *mut u8,
                extern "C" fn(
                    *mut u8,
                    *mut u8,
                    extern "C" fn(*mut u8, u8),
                    extern "C" fn(*mut u8) -> u8,
                ) -> u8,
            >(code_ptr as *mut u8)
        };
        // Prepare pointers
        let tape_ptr = self.tape.as_mut_ptr();
        let rt_ptr = self as *mut Runtime as *mut u8;
        // DEBUG: show pointers before JIT call
        /*println!("[JIT RUN] tape_ptr={:p}, rt_ptr={:p}, write_fn={:p}, read_fn={:p}",
            tape_ptr, rt_ptr,
            write_trampoline as *const u8,
            read_trampoline as *const u8
        );*/
        // Call the BF function
        bf_fn(tape_ptr, rt_ptr, write_trampoline, read_trampoline);
    }

    /// Consume the runtime and return the tape contents
    pub fn tape(self) -> Vec<u8> {
        self.tape
    }
}