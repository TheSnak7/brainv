use crate::io::IO;
use std::mem;
use std::ptr;

#[cfg(windows)]
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree, VirtualProtect};
#[cfg(windows)]
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READ, PAGE_READWRITE};
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

struct ExecutableCode {
    ptr: *mut u8,
}

impl ExecutableCode {
    fn new(code: &[u8]) -> Self {
        assert!(!code.is_empty(), "JIT code is empty");
        let code_ptr = unsafe {
            VirtualAlloc(
                ptr::null_mut(), code.len() as SIZE_T,
                MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE,
            )
        };
        if code_ptr.is_null() {
            panic!("VirtualAlloc failed");
        }
        // Take ownership before preparing the memory so failures also release it
        let compiled = Self { ptr: code_ptr as *mut u8 };
        unsafe {
            ptr::copy_nonoverlapping(code.as_ptr(), compiled.ptr, code.len());
            let mut old_protect = 0;
            if VirtualProtect(code_ptr, code.len() as SIZE_T, PAGE_EXECUTE_READ, &mut old_protect) == 0 {
                panic!("VirtualProtect failed");
            }
            if FlushInstructionCache(GetCurrentProcess(), code_ptr, code.len() as SIZE_T) == 0 {
                panic!("FlushInstructionCache failed");
            }
        }
        compiled
    }
}

impl Drop for ExecutableCode {
    fn drop(&mut self) {
        unsafe { VirtualFree(self.ptr as *mut _, 0, MEM_RELEASE); }
    }
}

/// Runtime for executing JIT-compiled Brainfuck code
pub struct Runtime<'a> {
    tape: Vec<u8>,
    io: Box<dyn IO<'a> + 'a>,
    code: ExecutableCode,
}

impl<'a> Runtime<'a> {
    /// Create a new runtime and prepare its executable code
    pub fn new(io: Box<dyn IO<'a> + 'a>, code: Vec<u8>) -> Self {
        let code = ExecutableCode::new(&code);
        Self { tape: vec![0; 30000], io, code }
    }

    /// Run the JIT-compiled function
    pub fn run(&mut self) -> Result<(), String> {
        // Cast the code pointer to the BF JIT function signature:
        // fn(*mut u8, *mut u8, extern "C" fn(*mut u8, u8), extern "C" fn(*mut u8) -> u8, usize) -> u8
        let bf_fn = unsafe {
            mem::transmute::<
                *mut u8,
                extern "C" fn(
                    *mut u8,
                    *mut u8,
                    extern "C" fn(*mut u8, u8),
                    extern "C" fn(*mut u8) -> u8,
                    usize,
                ) -> u8,
            >(self.code.ptr)
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
        let status = bf_fn(tape_ptr, rt_ptr, write_trampoline, read_trampoline, self.tape.len());
        self.io.flush();
        if status != 0 {
            return Err("Tape pointer out of bounds".into());
        }
        Ok(())
    }

    /// Consume the runtime and return the tape contents
    pub fn tape(self) -> Vec<u8> {
        self.tape
    }
}

#[cfg(all(test, windows, target_arch = "aarch64"))]
mod tests {
    use super::*;
    use crate::{compiler::Compiler, io::MemoryIO, jit::JIT};
    use winapi::um::memoryapi::VirtualQuery;
    use winapi::um::winnt::{MEMORY_BASIC_INFORMATION, MEM_FREE};

    fn memory_info(ptr: *mut u8) -> MEMORY_BASIC_INFORMATION {
        unsafe {
            let mut info = mem::zeroed();
            assert_ne!(VirtualQuery(ptr as *const _, &mut info, mem::size_of::<MEMORY_BASIC_INFORMATION>()), 0);
            info
        }
    }

    #[test]
    fn reuses_and_frees_executable_memory() {
        let program = "+.".repeat(200);
        let code = JIT::new(Compiler::new(&program).compile()).compile().unwrap();
        let code_len = code.len();
        let mut output = vec![];
        let io = Box::new(MemoryIO::new(&mut output, vec![]));
        let mut runtime = Runtime::new(io, code);
        let code_ptr = runtime.code.ptr;

        assert_eq!(memory_info(code_ptr).State, MEM_COMMIT);
        assert_eq!(memory_info(code_ptr).Protect, PAGE_EXECUTE_READ);
        assert_eq!(memory_info(unsafe { code_ptr.add(code_len - 1) }).Protect, PAGE_EXECUTE_READ);
        for _ in 0..3 {
            assert_eq!(runtime.run(), Ok(()));
            assert_eq!(runtime.code.ptr, code_ptr);
        }
        let tape = runtime.tape();
        assert_eq!(memory_info(code_ptr).State, MEM_FREE);
        assert_eq!(tape[0], 600u16 as u8);
        assert_eq!(output, (1..=600).map(|n| n as u8).collect::<Vec<_>>());

        let code = JIT::new(Compiler::new("<").compile()).compile().unwrap();
        let mut output = vec![];
        let io = Box::new(MemoryIO::new(&mut output, vec![]));
        let mut runtime = Runtime::new(io, code);
        let code_ptr = runtime.code.ptr;

        for _ in 0..3 {
            assert_eq!(runtime.run(), Err("Tape pointer out of bounds".into()));
            assert_eq!(runtime.code.ptr, code_ptr);
        }
        drop(runtime);
        assert_eq!(memory_info(code_ptr).State, MEM_FREE);
        assert!(output.is_empty());
    }
}
