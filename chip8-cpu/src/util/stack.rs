const STACK_SIZE: usize = 16;

pub struct Stack {
    pointer: u8,
    stack: [u16; STACK_SIZE]
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            pointer: 0,
            stack: [0; STACK_SIZE]
        }
    }

    pub fn push(&mut self, value: u16) {
        self.stack[self.pointer as usize] = value;
        self.pointer += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.pointer -= 1;
        return self.stack[self.pointer as usize];
    }
}