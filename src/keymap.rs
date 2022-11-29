// 2.3 - Keyboard
// The computers which originally used the Chip-8 Language had a 16-key
// hexadecimal keypad with the following layout:
//     1 2 3 C
//     4 5 6 D
//     7 8 9 E
//     A 0 B F

// This layout must be mapped into various other configurations to fit the
// keyboards of today's platforms.
//     1 2 3 4
//     Q W E R
//     A S D F
//     Z X C V

use std::collections::HashSet;

use sdl2::keyboard::Keycode;

#[derive(Debug)]
pub struct KeyMap {
    active: HashSet<u8>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            active: HashSet::new(),
        }
    }

    pub fn add_key(&mut self, keycode: Keycode) {
        match Self::to_chip8_key(keycode) {
            Some(key) => {
                self.active.insert(key);
            }
            None => {}
        };
    }

    pub fn remove_key(&mut self, keycode: Keycode) {
        match Self::to_chip8_key(keycode) {
            Some(key) => {
                self.active.remove(&key);
            }
            None => {}
        };
    }

    pub fn is_key_pressed(&self, key: u8) -> bool {
        self.active.contains(&key)
    }

    fn to_chip8_key(keycode: Keycode) -> Option<u8> {
        match keycode {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => return None,
        }
    }
}
