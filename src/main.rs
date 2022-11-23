use std::{env, process};

use flake_8::chip8::Chip8;

// TODO: Replace CLI argument parsing with a better library before releasing as
//       any public project.

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

    let mut dump = false;
    match args.get(2) {
        Some(flag) => {
            if flag == "-d" || flag == "--dump" {
                dump = true;
            }
        }
        None => {}
    }
    if dump {
        cpu.dump_to_stdout();
    }
}
