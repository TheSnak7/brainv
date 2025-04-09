use std::io::Read;

pub trait IO {
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

impl IO for SimpleIO {
    fn write_byte(&mut self, c: u8) {
        print!("{}", c as char);
    }

    fn read_byte(&mut self) -> u8 {
        let mut input = [0u8; 1];
        std::io::stdin().read_exact(&mut input).unwrap();
        return input[0] as u8;
    }

    fn flush(&mut self) {
        
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

impl IO for BatchedIO {
    fn write_byte(&mut self, c: u8) {
        if self.pos == self.buffer.len() {
            self.flush();
        }
        self.buffer[self.pos] = c;
        self.pos += 1;
    }

    fn read_byte(&mut self) -> u8 {
        self.flush();
        let mut input = [0u8; 1];
        std::io::stdin().read_exact(&mut input).unwrap();
        return input[0] as u8;
    }

    fn flush(&mut self) {
        if self.pos > 0 {
            print!("{}", String::from_utf8_lossy(&self.buffer[..self.pos]));
            self.pos = 0;
        }
    }
}