use std::{fmt, io::Read};

use crate::{compiler::Compiler, io::{MemoryIO, IO}};

#[derive(Debug, Clone, Copy)]
pub enum Op {
    #[allow(unused)]
    Nop,
    Inc,
    Dec,
    MovR,
    MovL,
    JmpIfZ,
    JmpIfNZ,
    Print,
    Read,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Nop => write!(f, "Nop"),
            Op::Inc => write!(f, "Inc"),
            Op::Dec => write!(f, "Dec"),
            Op::MovR => write!(f, "MovR"),
            Op::MovL => write!(f, "MovL"),
            Op::JmpIfZ => write!(f, "JmpIfZ"),
            Op::JmpIfNZ => write!(f, "JmpIfNZ"),
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
        loop {
            let instruction = self.program[self.pc];
            match instruction {
                Op::Inc => self.tape[self.tp] = self.tape[self.tp].wrapping_add(1),
                Op::Dec => self.tape[self.tp] = self.tape[self.tp].wrapping_sub(1),
                Op::MovR => self.tp += 1,
                Op::MovL => {
                    if self.tp > 0 {
                        self.tp -= 1
                    } else {
                        panic!(
                            "Vm exceeded available tape at instruction: {} at index: {}",
                            instruction, self.pc
                        );
                    }
                }
                Op::Print => self.io.write_byte(self.tape[self.tp]),
                Op::Read => {
                    self.tape[self.tp] = self.io.read_byte();
                }
                Op::JmpIfZ => {
                    let cell_val = self.tape[self.tp];
                    if cell_val == 0 {
                        let mut depth = 1;
                        while depth > 0 {
                            self.pc += 1;
                            if self.pc >= self.program.len() {
                                panic!("Unmatched [ at position: {}", self.pc);
                            }
                            match self.program[self.pc] {
                                Op::JmpIfZ => depth += 1,
                                Op::JmpIfNZ => depth -= 1,
                                _ => {}
                            }
                        }
                    }
                }
                Op::JmpIfNZ => {
                    let cell_val = self.tape[self.tp];
                    if cell_val != 0 {
                        let mut depth = 1;
                        while depth > 0 {
                            if self.pc == 0 {
                                panic!("Unmatched ] at position: {}", self.pc);
                            }
                            self.pc -= 1;
                            match self.program[self.pc] {
                                Op::JmpIfZ => depth -= 1,
                                Op::JmpIfNZ => depth += 1,
                                _ => {}
                            }
                        }
                        self.pc -= 1;
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
