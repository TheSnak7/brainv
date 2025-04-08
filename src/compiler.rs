// Because brainf**k is so simple a single pass compiler is enough

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
            '+' => code.push(Op::Add),
            '-' => code.push(Op::Sub),
            '>' => code.push(Op::Mr),
            '<' => code.push(Op::Ml),
            '[' => code.push(Op::Lb),
            ']' => code.push(Op::Rb),
            '.' => code.push(Op::Print),
            _ => {}
        });
        code
    }
}

#[derive(Debug)]
pub enum Op {
    #[allow(unused)]
    Nop,
    Add,
    Sub,
    Mr,
    Ml,
    Rb,
    Lb,
    Print,
}
