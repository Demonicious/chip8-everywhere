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

        self.execute(d1, d2, d3, d4);

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

    fn execute(&mut self, d1: u16, d2: u16, d3: u16, d4: u16) {
        match(d1, d2, d3, d4) {
            (0, 0, 0xE, 0) => self.op_00e0(),
            (0, 0, 0xE, 0xE) => self.op_00ee(),

            (_, _, _, _) => unimplemented!("Instruction not found.")
        }
    }

    fn op_00e0(&mut self) {
        self.screen = [false; SCREEN_X * SCREEN_Y];
    }

    fn op_00ee(&mut self) {
        self.pc = self.stack.pop() as u16;
    }

    
}