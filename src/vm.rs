use std::{fmt, io::Read};

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

pub struct Vm {
    program: Vec<Op>,
    // NO WRAP AROUND -> abort on move past end of tape
    tape: Vec<u8>,
    // Program counter
    pc: usize,
    // Tape Pointer
    tp: usize,
}

impl Vm {
    pub fn new(program: Vec<Op>) -> Self {
        Self {
            program,
            tape: vec![0; 1024],
            pc: 0,
            tp: 0,
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
                        panic!("Vm exceeded available tape at instruction: {} at index: {}", instruction, self.pc);
                    }
                }
                Op::Print => print!("{}", self.tape[self.tp] as char),
                Op::Read => {
                    let mut input = [0u8; 1];
                    std::io::stdin().read_exact(&mut input).unwrap();
                    self.tape[self.tp] = input[0];
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
}
