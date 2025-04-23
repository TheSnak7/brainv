use std::io::Read;
use std::io::{self, Write};

pub trait IO<'a> {
    fn write_byte(&mut self, c: u8);

    fn read_byte(&mut self) -> u8;

    fn flush(&mut self);
}

pub struct SimpleIO {}

impl SimpleIO {
    pub fn new() -> Self {
        Self {  }
    }
}

impl<'a> IO<'a> for SimpleIO {
    fn write_byte(&mut self, c: u8) {
        print!("{}", c as char);
        io::stdout().flush().unwrap();
    }

    fn read_byte(&mut self) -> u8 {
        io::stdout().flush().unwrap();
        let mut buf = [0u8; 1];
        loop {
            std::io::stdin().read_exact(&mut buf).unwrap();
            if buf[0] != b'\r' {
                return buf[0];
            }
        }
    }

    fn flush(&mut self) {
        io::stdout().flush().unwrap();
    }
}

pub struct BatchedIO {
    buffer: Vec<u8>,
    pos: usize,
}

impl BatchedIO {
    pub fn new(buffer_size: usize) -> Self {
        Self { buffer: vec![0; buffer_size], pos: 0}
    }
}

impl<'a> IO<'a> for BatchedIO {
    fn write_byte(&mut self, c: u8) {
        if self.pos == self.buffer.len() {
            self.flush();
        }
        self.buffer[self.pos] = c;
        self.pos += 1;
    }

    fn read_byte(&mut self) -> u8 {
        self.flush();
        let mut buf = [0u8; 1];
        loop {
            std::io::stdin().read_exact(&mut buf).unwrap();
            if buf[0] != b'\r' {
                return buf[0];
            }
        }
    }

    fn flush(&mut self) {
        if self.pos > 0 {
            print!("{}", String::from_utf8_lossy(&self.buffer[..self.pos]));
            self.pos = 0;
            io::stdout().flush().unwrap();
        }
    }
}

pub struct MemoryIO<'a> {
    output: &'a mut Vec<u8>,
    input: Vec<u8>,
    input_pos: usize,
}

impl<'a> MemoryIO<'a> {
    pub fn new(output_buffer: &'a mut Vec<u8>, input: Vec<u8>) -> Self {
        Self {
            output: output_buffer,
            input,
            input_pos: 0,
        }
    }
}

impl<'a> IO<'a> for MemoryIO<'a> {
    fn write_byte(&mut self, c: u8) {
        self.output.push(c);
    }

    fn read_byte(&mut self) -> u8 {
        // Read until a non-newline (skip CR and LF)
        while self.input_pos < self.input.len() {
            let b = self.input[self.input_pos];
            self.input_pos += 1;
            if b != b'\r' && b != b'\n' {
                return b;
            }
        }
        // No more input: panic with a clear error message
        panic!("MemoryIO::read_byte(): no more input available (EOF)");
    }

    fn flush(&mut self) {
        // No-op for memory IO as everything is already in memory
    }
}