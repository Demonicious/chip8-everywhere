use std::{fs::File, io::Read, vec};
use clap::Parser;
use chip8_cpu::Emulator;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    rom: String
}

fn main() {
    let args = Args::parse();

    let mut chip8 = Emulator::new();

    println!("Attempting to play: {}", args.rom);

    let mut file = File::open(args.rom).expect("No such rom.");
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    chip8.load_rom(&buffer);
    chip8.tick();
}
