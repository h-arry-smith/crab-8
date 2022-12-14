use clap::Parser;
use sdl2::{audio::AudioSpecDesired, event::Event, keyboard::Keycode};
use std::{thread, time::Duration};

use flake_8::{
    audio::SquareWave,
    chip8::{Chip8, Error},
    cli::Cli,
    keymap::KeyMap,
    render::Renderer,
};

fn main() {
    let mut cpu = Chip8::new();

    let args = Cli::parse();

    cpu.load_rom(&args.path, args.eti_mode);
    cpu.set_debug_output(args.debug);

    let mut renderer = Renderer::new(64, 32, 16);
    renderer.set_colors(args.fg, args.bg);

    let desired_audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = renderer
        .audio_subsystem
        .open_playback(None, &desired_audio_spec, |spec| SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        })
        .unwrap();

    let mut keymap = KeyMap::new();

    'running: loop {
        for event in renderer.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    keymap.add_key(key);
                }
                _ => {}
            }
        }

        match cpu.step(&keymap) {
            Ok(_) => {}
            Err(err) => match err {
                Error::UnrecognisedInstruction(high, low) => {
                    eprintln!("Unrecognised Instruction: {:02X} {:02X}", high, low);
                    break 'running;
                }
            },
        }

        if cpu.sound_on() {
            device.resume();
        } else {
            device.pause();
        }

        renderer.render(&cpu.display);

        keymap.clear();

        thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }

    if args.debug {
        cpu.dump_to_stdout();
    }
}
