use sdl2::{event::Event, keyboard::Keycode};
use std::{env, process, thread, time::Duration};

use flake_8::{
    chip8::{Chip8, Error},
    render::Renderer,
};

// TODO: Replace CLI argument parsing with a better library before releasing as
//       any public project.

fn main() {
    let mut cpu = Chip8::new();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: flake-8 [path]");
        process::exit(1);
    }

    let mut dump = false;
    match args.get(2) {
        Some(flag) => {
            if flag == "-d" || flag == "--dump" {
                dump = true;
            }
        }
        None => {}
    }

    let path = args.get(1).expect("A file path must be provided.");

    cpu.load_rom(path);
    cpu.set_debug_output(dump);

    let mut renderer = Renderer::new(64, 32, 16);

    'running: loop {
        match cpu.step() {
            Ok(_) => {}
            Err(err) => match err {
                Error::UnrecognisedInstruction(high, low) => {
                    eprintln!("Unrecognised Instruction: {:02X} {:02X}", high, low);
                    break 'running;
                }
            },
        }

        for event in renderer.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        renderer.render(&cpu.display);

        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    if dump {
        cpu.dump_to_stdout();
    }
}
