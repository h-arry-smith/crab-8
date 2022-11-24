// Reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

// TODO: This implementation supports the original 64x32 pixel display but does
//       not support other common display modes.

const PIXEL_COUNT: usize = 64 * 32;

// 2.4 - Display

pub struct Display {
    // The original implementation of the Chip-8 language used a 64x32-pixel
    // monochrome display with this format:
    // (0,  0)    (63,  0)
    // (0, 31)    (63, 31)
    pub memory: [bool; PIXEL_COUNT],
}

impl Display {
    pub fn new() -> Self {
        Self {
            memory: [false; PIXEL_COUNT],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, pixel: bool) -> bool {
        // Sprites are XORed onto the existing screen.
        let current = self.get(x, y);

        if current ^ pixel {
            self.memory[self.to_index(x, y)] = true;
            // Pixel was not erased, so return false
            false
        } else {
            self.memory[self.to_index(x, y)] = false;
            // If the pixel was erased (On -> Off) then return true
            true
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.memory[self.to_index(x, y)]
    }

    pub fn clear(&mut self) {
        self.memory = [false; PIXEL_COUNT];
    }

    fn to_index(&self, x: usize, y: usize) -> usize {
        y * 64 + x
    }

    pub fn dump_to_stdout(&self) {
        for line in self.memory.chunks(64) {
            for pixel in line {
                if *pixel {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
}

// Chip-8 draws graphics on screen through the use of sprites.
// A sprite is a group of bytes which are a binary representation of the desired
// picture.
pub struct Sprite<'a> {
    bytes: &'a [u8],
}

impl<'a> Sprite<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        // Chip-8 sprites may be up to 15 bytes
        assert!(bytes.len() <= 15);

        Self { bytes }
    }

    pub fn draw(&self, x: usize, y: usize, display: &mut Display) -> Collision {
        // If the sprite is positioned so part of it is outside the coordinates
        // of the display, it wraps around to the opposite side of the screen.
        let mut dx = x % 64;
        let mut dy = y % 32;

        let mut collision = Collision::False;

        for byte in self.bytes.iter() {
            // A sprite is a group of bytes which are a binary representation of
            // the desired picture.
            let pixels = self.to_pixels(*byte);

            for pixel in pixels {
                let collide = display.set(dx, dy, pixel);

                // Sprites are XORed onto the existing screen. If this causes
                // any pixels to be erased, VF is set to 1, otherwise it is set
                // to 0
                if collide {
                    collision = Collision::True;
                }

                dx += 1;
                dx %= 64;
            }

            dx = x % 64;
            dy += 1;
            dy %= 32;
        }

        collision
    }

    fn to_pixels(&self, byte: u8) -> [bool; 8] {
        let mut byte = byte;
        let mut pixels = [false; 8];
        for i in 0..8 {
            if byte.leading_ones() > 0 {
                pixels[i] = true;
            }
            byte = byte.rotate_left(1);
        }

        pixels
    }
}

#[derive(PartialEq)]
pub enum Collision {
    True,
    False,
}
