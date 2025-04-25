use std::fmt;

use crate::{compiler::Compiler, io::{MemoryIO, IO}};

#[derive(Debug, Clone, Copy)]
pub enum Op {
    #[allow(unused)]
    Nop,
    Inc(u8),
    Dec(u8),
    MovR(u8),
    MovL(u8),
    /// Jump to the matching right brace if the cell is zero
    JmpIfZ(u16),
    /// Jump to the matching left brace if the cell is not zero
    JmpIfNZ(u16),
    Print,
    Read,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Nop => write!(f, "Nop"),
            Op::Inc(num) => write!(f, "Inc by {num}"),
            Op::Dec(num) => write!(f, "Dec by {num}"),
            Op::MovR(num) => write!(f, "MovR by {num}"),
            Op::MovL(num) => write!(f, "MovL by {num}"),
            Op::JmpIfZ(index) => write!(f, "JmpIfZ to {index}"),
            Op::JmpIfNZ(index) => write!(f, "JmpIfNZ to {index}"),
            Op::Print => write!(f, "Print"),
            Op::Read => write!(f, "Read"),
        }
    }
}

pub struct Vm<'a> {
    program: Vec<Op>,
    // NO WRAP AROUND -> abort on move past end of tape
    tape: Vec<u8>,
    // Program counter
    pc: usize,
    // Tape Pointer
    tp: usize,
    io: Box<dyn IO<'a> + 'a>,
}

impl<'a> Vm<'a> {
    pub fn new(io: Box<dyn IO<'a> + 'a>, program: Vec<Op>) -> Self {
        Self {
            program,
            tape: vec![0; 1024],
            pc: 0,
            tp: 0,
            io,
        }
    }

    pub fn run(&mut self) {
        //println!("Running program: {:?}", self.program);

        loop {
            let instruction = self.program[self.pc];
            match instruction {
                Op::Inc(num) => self.tape[self.tp] = self.tape[self.tp].wrapping_add(num),
                Op::Dec(num) => self.tape[self.tp] = self.tape[self.tp].wrapping_sub(num),
                Op::MovR(num) => {
                    let shift = num as usize;
                    self.tp += shift;
                    // grow tape on the right as needed
                    while self.tp >= self.tape.len() {
                        self.tape.push(0);
                    }
                }
                Op::MovL(num) => {
                    let shift = num as usize;
                    if self.tp > 0 {
                        self.tp -= shift;
                    } else {
                        panic!("Tape pointer underflow: attempted to move left {} from position {}", shift, self.tp);
                    }
                }
                Op::Print => self.io.write_byte(self.tape[self.tp]),
                Op::Read => {
                    self.tape[self.tp] = self.io.read_byte();
                }
                Op::JmpIfZ(jmp_index) => {
                    if self.tape[self.tp] == 0 {
                        self.pc = jmp_index as usize;
                    }
                }
                Op::JmpIfNZ(jmp_index) => {
                    if self.tape[self.tp] != 0 {
                        self.pc = jmp_index as usize;
                    }
                }
                Op::Nop => (),
            }
            self.pc += 1;
            if self.pc >= self.program.len() {
                break;
            }
        }
    }


    pub fn flush_io(&mut self) {
        self.io.flush();
    }
}

pub fn bench_run(program: &str, input: Vec<u8>) -> Vec<u8> {
    let code = Compiler::new(program).compile();
    let mut output_buffer = Vec::with_capacity(1024);
    let input_buffer = input;
    
    {
        let io = Box::new(MemoryIO::new(&mut output_buffer, input_buffer));
        let mut vm = Vm::new(io, code);
        vm.run();
    } // vm and io are dropped here, releasing the borrow on output_buffer
    
    output_buffer
}
