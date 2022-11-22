use std::{env, process};

use flake_8::chip8::Chip8;

fn main() {
    let mut cpu = Chip8::new();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: flake-8 [path]");
        process::exit(1);
    }

    let path = args.get(1).expect("A file path must be provided.");

    cpu.load_rom(path);

    cpu.run();
}
