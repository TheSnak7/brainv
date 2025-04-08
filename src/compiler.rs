// Because brainf**k is so simple a single pass compiler is enough

use crate::vm::Op;

pub struct Compiler<'a> {
    program: &'a str,
}

impl<'a> Compiler<'a> {
    pub fn new(program: &'a str) -> Self {
        Self { program }
    }

    pub fn compile(&self) -> Vec<Op> {
        let mut code = vec![];

        self.program.chars().for_each(|c| match c {
            '+' => code.push(Op::Inc),
            '-' => code.push(Op::Dec),
            '>' => code.push(Op::MovR),
            '<' => code.push(Op::MovL),
            '[' => code.push(Op::JmpIfZ),
            ']' => code.push(Op::JmpIfNZ),
            '.' => code.push(Op::Print),
            ',' => code.push(Op::Read),
            _ => {}
        });
        code
    }
}
