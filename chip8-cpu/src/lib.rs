pub mod util;

use std::process;

use util::stack::Stack;

const NUM_KEYS: usize = 16;
const NUM_REGS: usize = 16;
const RAM_SIZE: usize = 4096;

const START_ADDRESS: u16 = 0x200;

pub const SCREEN_X: usize = 64;
pub const SCREEN_Y: usize = 32;

const FONTSET_SIZE: usize = 16 * 5; // 16 Characters; 5 bytes each;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emulator {
    // Program Counter; Stack;
    pc: u16,
    stack: Stack,

    // Ram; V-Registers; I-Register
    ram: [u8; RAM_SIZE],
    v: [u8; NUM_REGS],
    i: u16,

    // Delay Timer; Sound Timer;
    dt: u8,
    st: u8,

    // Keyboard
    keys: [bool; NUM_KEYS],

    // Graphics; Array of Pixels in Screen
    screen: [bool; SCREEN_X * SCREEN_Y]
}

impl Emulator {
    pub fn new() -> Emulator {
        let mut emu = Emulator {
            pc: START_ADDRESS,
            stack: Stack::new(),

            ram: [0; RAM_SIZE],
            v: [0; NUM_REGS],
            i: 0,

            keys: [false; NUM_KEYS],

            dt: 0,
            st: 0,

            screen: [false; SCREEN_X * SCREEN_Y]
        };

        // Copy Fontset into the start of RAM;
        emu.ram[0..FONTSET_SIZE].copy_from_slice(&FONTSET);

        return emu;
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        if data.len() > self.ram[(START_ADDRESS as usize)..].len() {
            eprintln!("Rom too Large!");
            process::exit(1);
        }

        self.ram[(START_ADDRESS as usize)..(START_ADDRESS as usize) + data.len()].copy_from_slice(&data);
    }

    pub fn input(&mut self, key_idx: usize, pressed: bool) {
        self.keys[key_idx] = pressed;
    }

    pub fn key_down(&mut self, key_idx: usize) {
        self.input(key_idx, true);
    }

    pub fn key_up(&mut self, key_idx: usize) {
        self.input(key_idx, false);
    }

    pub fn tick(&mut self) -> [bool; SCREEN_X * SCREEN_Y] {
        let instruction = self.fetch();
        let (d1, d2, d3, d4) = self.decode(instruction);

        self.execute(d1, d2, d3, d4, instruction);

        self.screen
    }
    
    pub fn tick_timers(&mut self) -> bool {
        let mut should_beep: bool = false;

        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                should_beep = true;
            }

            self.st -= 1;
        }

        should_beep
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        self.pc += 2;

        // Shift Higher byte to left by 8 bits to free up 8 bits on the lower side and perform a Bitwise OR to populate the lower bits with the lower byte;
        ((higher_byte << 8) | lower_byte) as u16
    }

    fn decode(&mut self, instruction: u16) -> (u16, u16, u16, u16) {
        let d1 = (instruction & 0xF000) >> 12;
        let d2 = (instruction & 0x0F00) >> 8;
        let d3 = (instruction & 0x00F0) >> 4;
        let d4 = instruction & 0x000F;

        return (d1, d2, d3, d4);
    }

    fn execute(&mut self, d1: u16, d2: u16, d3: u16, d4: u16, instruction: u16) {
        match(d1, d2, d3, d4) {

            /* 
             * Use Bitwise Operations to get the values you need for each instructions. For example (instruction & 0x00FF) as u8 to get the last byte.
             * Digit 2 - 4 are used as Vx, Vy for instructions that require them.
             */

            // 00E0 - Clear Screen
            (0, 0, 0xE, 0)   => self.op_00e0(), 

            // 00EE - Return from Subroutine
            (0, 0, 0xE, 0xE) => self.op_00ee(), 

            // 1NNN - Jump to Address.
            (1, _, _, _)     => self.op_1nnn((instruction & 0x0FFF) as u16),             

            // 2NNN - Call subroutine.
            (2, _, _, _)     => self.op_2nnn((instruction & 0x0FFF) as u16),             
            
            // 3XKK - Skip if Vx == KK.
            (3, _, _, _)     => self.op_3xkk(d2 as usize, (instruction & 0x00FF) as u8),  

            // 4XKK - Skip if Vx != KK.
            (4, _, _, _)     => self.op_4xkk(d2 as usize, (instruction & 0x00FF) as u8), 

            // 5XY0 - Skip if Vx == Vy.
            (5, _, _, 0)     => self.op_5xy0(d2 as usize, d3 as usize),

            // 6XKK - Set Vx = kk.
            (6, _, _, _)     => self.op_6xkk(d2 as usize, (instruction & 0x00FF) as u8),

            // 7XKK - Add Vx + kk.
            (7, _, _, _)     => self.op_7xkk(d2 as usize, (instruction & 0x00FF) as u8),

            (_, _, _, _) => unimplemented!("Instruction not found.")

        }
    }

    // Clear Screen
    fn op_00e0(&mut self) {
        self.screen = [false; SCREEN_X * SCREEN_Y];
        println!("Clear");
    }

    // Return from Subroutine
    fn op_00ee(&mut self) {
        self.pc = self.stack.pop() as u16;
    }

    // Jump to Address
    fn op_1nnn(&mut self, address: u16) {
        // Bitwise & to get the trailing 12-bits which is the address.
        self.pc = address;
    }

    // Call Sub-routine
    fn op_2nnn(&mut self, address: u16) {
        self.stack.push(self.pc);
        self.pc = address;
    }

    // Skip if Vx == kk
    fn op_3xkk(&mut self, vx: usize, kk: u8) {
        if self.v[vx] == kk {
            self.pc += 2;
        }
    }

    // Skip if Vx != kk
    fn op_4xkk(&mut self, vx: usize, kk: u8) {
        if self.v[vx] != kk {
            self.pc += 2;
        }
    }

    // Skip if Vx == Vy
    fn op_5xy0(&mut self, vx: usize, vy: usize) {
        if self.v[vx] == self.v[vy] {
            self.pc += 2;
        }
    }

    // Set Vx = kk
    fn op_6xkk(&mut self, vx: usize, kk: u8) {
        self.v[vx] = kk;
    }

    // Wrapping Add Vx = Vx + kk
    fn op_7xkk(&mut self, vx: usize, kk: u8) {
        self.v[vx] = self.v[vx].wrapping_add(kk);
    }

    // Set Vx = Vy
    fn op_8xy0(&mut self, vx: usize, vy: usize) {
        self.v[vx] = self.v[vy];
    }

    // Set Vx = Vx | Vy
    fn op_8xy1(&mut self, vx: usize, vy: usize) {
        self.v[vx] = self.v[vx] | self.v[vy];
    }

    // Set Vx = Vx & Vy
    fn op_8xy2(&mut self, vx: usize, vy: usize) {
        self.v[vx] = self.v[vx] & self.v[vy];
    }

    // Set Vx = Vx ^ Vy
    fn op_8xy3(&mut self, vx: usize, vy: usize) {
        self.v[vx] = self.v[vx] ^ self.v[vy];
    }

    // Overflowing add Vx = Vx + Vy. Set VF to 1 if there's a carry.
    fn op_8xy4(&mut self, vx: usize, vy: usize) {
        let (new_vx, carry) = self.v[vx].overflowing_add(self.v[vy]);

        self.v[0xF] = if carry { 1 } else { 0 };
        self.v[vx] = new_vx;
    }

    // Subtract Vx = Vx - Vy. Set VF to 1 if there's no borrow.
    fn op_8xy5(&mut self, vx: usize, vy: usize) {
        let (new_vx, borrow) = self.v[vx].overflowing_sub(self.v[vy]);

        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[vx] = new_vx;
    }

    // Set Vx = Vx Shift Right 1. Set VF to 1 if Vx LSB is 1.
    fn op_8xy6(&mut self, vx: usize) {
        self.v[0xF] = self.v[vx] & 0b1;
        self.v[vx] >>= 1;
    }

    // Set VF = Vy > Vx; Vx = Vy - Vx
    fn op_8xy7(&mut self, vx: usize, vy: usize) {
        self.v[0xF] = if self.v[vy] > self.v[vx] { 1 } else { 0 };
        self.v[vx] = self.v[vy].wrapping_sub(self.v[vx]);
    }

    // Set Vx = Vx Shift Left 1. Set VF = 1 if Vx MSB is 1.
    fn op_8xye(&mut self, vx: usize) {
        self.v[0xF] = (self.v[vx] >> 7) & 0b1;
        self.v[vx] <<= 1;
    }

    // Skip if Vx != Vy
    fn op_9xy0(&mut self, vx: usize, vy: usize) {
        if self.v[vx] != self.v[vy] {
            self.pc += 2;
        }
    }

    // Set I = nnn
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // Jump to nnn + V0
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0] as u16;
    }

    // Set Vx = random byte & kk;
    fn op_cxkk(&mut self, vx: usize, kk: u8) {
        self.v[vx] = fastrand::u8(..) & kk;
    }

    // Draw ...
    fn op_dxyn(&mut self, vx: usize, vy: usize, n: u8) {

    }

    // Skip if Key in VX is held down.
    fn op_ex9e(&mut self, vx: usize) {
        if self.keys[self.v[vx] as usize] {
            self.pc += 2;
        }
    }

    // Skip if Key in VX is not held down.
    fn op_exa1(&mut self, vx: usize) {
        if !self.keys[self.v[vx] as usize] {
            self.pc += 2;
        }
    }

    // Set Vx = Delay Timer Value
    fn op_fx07(&mut self, vx: usize) {
        self.v[vx] = self.dt;
    }

    // Wait for Keypress and store it in Vx
    fn op_fx0a(&mut self, vx: usize) {
        let mut pressed = false;

        for i in (0..self.keys.len()) {
            if self.keys[i] {
                pressed = true;
                self.v[vx] = i as u8;
            }
        }

        if !pressed {
            self.pc -= 2;
        }
    }

    // Set DT = Vx
    fn op_fx15(&mut self, vx: usize) {
        self.dt = self.v[vx];
    }

    // Set ST = Vx
    fn op_fx18(&mut self, vx: usize) {
        self.st = self.v[vx];
    }

    // Add I = I + Vx
    fn op_fx1e(&mut self, vx: usize) {
        self.i = self.i.wrapping_add(self.v[vx] as u16);
    }

    // Set I = Address of Fontset character in Vx
    fn op_fx29(&mut self, vx: usize) {
        self.i = (self.v[vx] * 5) as u16;
    }

    // Store BCD Vx in Ram at I, I + 1, I + 2 for Hundreds, Tens and Ones.
    fn op_fx33(&mut self, vx: usize) {
        let val_vx = self.v[vx] as f32;

        let hundreds = (val_vx / 100.0).floor() as u8;
        let tens = ((val_vx / 10.0) % 10.0).floor() as u8;
        let ones = (val_vx % 10.0) as u8;

        self.ram[self.i as usize] = hundreds;
        self.ram[self.i as usize + 1] = tens;
        self.ram[self.i as usize + 2] = ones;

    }

    // Copy V0 to Vx to RAM starting at I
    fn op_fx55(&mut self, vx: usize) {
        self.ram[(self.i as usize)..((self.i as usize) + (vx + 1))].copy_from_slice(&self.v[0..(vx + 1)]);
    }

    // Copy into V0 through Vx from RAM starting at I
    fn op_fx65(&mut self, vx: usize) {
        self.v[0..(vx + 1)].copy_from_slice(&self.ram[(self.i as usize)..((self.i as usize) + (vx + 1))]);
    }
    
}