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

        let mut last_instruction = Op::Nop;

        //let mut left_bracket_stack = vec![];

        for i in 0..(self.program.len()) {
            let char = self.program.as_bytes().get(i).expect("Program index oob while compiling").clone() as char;
            match char {
                '+' => {
                    if let Op::Inc(num) = last_instruction {
                        last_instruction = Op::Inc(num + 1);
                    } else {
                        code.push(last_instruction);

                        last_instruction = Op::Inc(1);
                    }
                },
                '-' => {
                    if let Op::Dec(num) = last_instruction {
                        last_instruction = Op::Dec(num + 1);
                    } else {
                        code.push(last_instruction);

                        last_instruction = Op::Dec(1);
                    }
                },
                '>' => {
                    if let Op::MovR(num) = last_instruction {
                        last_instruction = Op::MovR(num + 1);
                    } else {
                        code.push(last_instruction);

                        last_instruction = Op::MovR(1);
                    }
                },
                '<' => {
                    if let Op::MovL(num) = last_instruction {
                        last_instruction = Op::MovL(num + 1);
                    } else {
                        code.push(last_instruction);

                        last_instruction = Op::MovL(1);
                    }
                },
                '[' => {
                    code.push(last_instruction);
                    last_instruction = Op::JmpIfZ(0);
                },
                ']' => {
                    code.push(last_instruction);
                    last_instruction = Op::JmpIfNZ(0);
                },
                '.' => {
                    code.push(last_instruction);
                    last_instruction = Op::Print;
                },
                ',' => {
                    code.push(last_instruction);
                    last_instruction = Op::Read;
                },
                _ => {}
            }

            
        }
        code.push(last_instruction);

        code
    }
}
