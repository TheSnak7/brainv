use crate::vm;


pub struct JIT {
    code: Vec<vm::Op>,
}

impl JIT {
    pub fn new(code: Vec<vm::Op>) -> Self { 
        

        Self { 
            code: code,
        }
    }
}

impl JIT {

    // The final function will be called with the following signature:
    // fn(tape_ptr: *mut u8,
    //    rt_ptr: *mut u8,
    //    write_char: extern "C" fn(rt_ptr: *mut u8, u8),
    //    read_char: extern "C" fn(rt_ptr: *mut u8) -> u8)
    //    -> u8
    // For now the tape is not growable, so we can just pass a pointer to the tape

    // ARM64 only, no support for x86
    pub fn compile(&self) -> Result<Vec<u8>, String> {
        // Ensure we're on ARM64
        assert_eq!(std::env::consts::ARCH, "aarch64");

        // Calling convention:
        //   x0: tape_ptr, x1: rt_ptr, x2: write_fn, x3: read_fn
        // We'll save them in callee-saved registers:
        //   x19 = tape_ptr, x20 = rt_ptr, x21 = write_fn, x22 = read_fn
        let mut code: Vec<u8> = Vec::new();
        // PROLOGUE: push frame pointer & link register
        code.extend(&[0xFD, 0x7B, 0xBF, 0xA9]); // stp x29, x30, [sp, #-16]!
        code.extend(&[0xFD, 0x03, 0x00, 0x91]); // mov x29, sp
        // Save arguments into callee-saved regs via ADD #0 (mov xN, xM)
        code.extend(&[0x13, 0x00, 0x00, 0x91]); // add x19, x0, #0
        code.extend(&[0x34, 0x00, 0x00, 0x91]); // add x20, x1, #0
        code.extend(&[0x55, 0x00, 0x00, 0x91]); // add x21, x2, #0
        code.extend(&[0x76, 0x00, 0x00, 0x91]); // add x22, x3, #0

        // First pass: compute byte offsets for each instruction label
        let mut label_offsets = vec![0usize; self.code.len()];
        {
            let mut offset = code.len();
            for (i, op) in self.code.iter().enumerate() {
                label_offsets[i] = offset;
                offset += match op {
                    vm::Op::Inc(_) | vm::Op::Dec(_) => 12,
                    vm::Op::MovR(_) | vm::Op::MovL(_) => 4,
                    vm::Op::Print | vm::Op::Read => 16,
                    vm::Op::JmpIfZ(_) | vm::Op::JmpIfNZ(_) => 8,
                    vm::Op::Nop => 0,
                };
            }
        }

        // Second pass: emit each opcode sequence
        for (i, op) in self.code.iter().enumerate() {
            match op {
                vm::Op::Nop => {}
                vm::Op::Inc(n) => {
                    // ldrb w4, [x19]
                    code.extend(&0x39401664u32.to_le_bytes());
                    // add w4, w4, #n
                    let instr = 0x11000000u32 | ((*n as u32) << 10) | (4 << 5) | 4;
                    code.extend(&instr.to_le_bytes());
                    // strb w4, [x19]
                    code.extend(&0x39001664u32.to_le_bytes());
                }
                vm::Op::Dec(n) => {
                    code.extend(&0x39401664u32.to_le_bytes());
                    let instr = 0x51000000u32 | ((*n as u32) << 10) | (4 << 5) | 4;
                    code.extend(&instr.to_le_bytes());
                    code.extend(&0x39001664u32.to_le_bytes());
                }
                vm::Op::MovR(n) => {
                    // add x19, x19, #n
                    let instr = 0x91000000u32 | ((*n as u32) << 10) | (19 << 5) | 19;
                    code.extend(&instr.to_le_bytes());
                }
                vm::Op::MovL(n) => {
                    let instr = 0xD1000000u32 | ((*n as u32) << 10) | (19 << 5) | 19;
                    code.extend(&instr.to_le_bytes());
                }
                vm::Op::Print => {
                    // ldrb w4, [x19]
                    code.extend(&0x39401664u32.to_le_bytes());
                    // mov x0, x20 via ADD #0
                    code.extend(&0x91000280u32.to_le_bytes());
                    // mov w1, w4 via ADD #0 (32-bit)
                    code.extend(&0x11000081u32.to_le_bytes());
                    // blr x21 (branch with link to reg X21)
                    code.extend(&[0xA0, 0x02, 0x3F, 0xD6]);
                }
                vm::Op::Read => {
                    // mov x0, x20 via ADD #0
                    code.extend(&0x91000280u32.to_le_bytes());
                    // blr x22 (branch with link to reg X22)
                    code.extend(&[0xC0, 0x02, 0x3F, 0xD6]);
                    // strb w0, [x19]
                    code.extend(&0x390006C4u32.to_le_bytes());
                }
                vm::Op::JmpIfZ(target) => {
                    code.extend(&0x39401664u32.to_le_bytes());
                    // cbz w4, <label>
                    let to = label_offsets[*target as usize] as i64 - (code.len() as i64 + 4);
                    let imm19 = ((to / 4) as i32) & 0x7FFFF;
                    let instr = 0x34000000u32 | ((imm19 as u32) << 5) | 4;
                    code.extend(&instr.to_le_bytes());
                }
                vm::Op::JmpIfNZ(target) => {
                    code.extend(&0x39401664u32.to_le_bytes());
                    let to = label_offsets[*target as usize] as i64 - (code.len() as i64 + 4);
                    let imm19 = ((to / 4) as i32) & 0x7FFFF;
                    let instr = 0x35000000u32 | ((imm19 as u32) << 5) | 4;
                    code.extend(&instr.to_le_bytes());
                }
            }
        }

        // EPILOGUE
        // mov w0, #0
        code.extend(&0x52800000u32.to_le_bytes());
        // ldp x29, x30, [sp], #16
        code.extend(&[0xFD, 0x7B, 0xC1, 0xA8]);
        // ret
        code.extend(&[0xC0, 0x03, 0x5F, 0xD6]);

        /*// DEBUG: map each high-level Op to its code offset
        #[cfg(debug_assertions)]
        {
            println!("JIT instruction map:");
            for (i, op) in self.code.iter().enumerate() {
                let off = label_offsets[i];
                println!("  0x{:04x}: {:?}", off, op);
            }
        }*/
        return Ok(code);
    }

}

