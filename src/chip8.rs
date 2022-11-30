// Reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

use rand::Rng;
use std::{fs, time::SystemTime};

use crate::display::{Collision, Display, Sprite};
use crate::keymap::KeyMap;

// 2.1 - Memory
// Most Chip-8 programs start at location 0x200 (512), but some begin at
// 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660
// computer.
const NORMAL_START_INDEX: usize = 512;
const ETI_660_START_INDEX: usize = 1526;

// 2.2 - Regissters
// Chip-8 also has two special purpose 8-bit registers, for the delay and sound
// timers. When these registers are non-zero, they are automatically decremented
// at a rate of 60Hz.
const CLOCK_CYCLE: f64 = 1.0 / 60.0;

pub struct Chip8 {
    // 2.1 - Memory
    // The Chip-8 language is capable of accessing up to 4KB (4,096 bytes) of
    // RAM, from location 0x000 (0) to 0xFFF (4095).
    ram: [u8; 4096],

    // 2.2 - Registers
    registers: Registers,
    // There are also some "pseudo-registers" which are not accessable from
    // Chip-8 programs.

    // NOTE: While spec asks for 16-bit numbers, we use usize to simplify direct
    //       access of arrays in rust.

    // The program counter (PC) should be 16-bit, and is used to store the
    // currently executing address.
    pc: usize,

    // The stack pointer (SP) can be 8-bit, it is used to point to the topmost
    // level of the stack.
    sp: usize,

    // The stack is an array of 16 16-bit values, used to store the address that
    // the interpreter shoud return to when finished with a subroutine. Chip-8
    // allows for up to 16 levels of nested subroutines.
    stack: [usize; 16],

    pub display: Display,

    debug_output: bool,

    start_time: Option<SystemTime>,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut new = Self {
            ram: [0; 4096],
            registers: Registers::new(),
            pc: 0,
            sp: 0,
            stack: [0; 16],
            display: Display::new(),
            debug_output: false,
            start_time: None,
        };

        new.load_hexadecimal_display_bytes();
        new
    }

    pub fn load_rom(&mut self, path: &str, eti_mode: bool) {
        let bytes = fs::read(path).expect("Could not open file.");

        let mut start_index = NORMAL_START_INDEX;

        if eti_mode {
            start_index = ETI_660_START_INDEX;
        }

        for (index, byte) in bytes.iter().enumerate() {
            self.ram[start_index + index] = *byte;
        }

        self.pc = start_index;

        eprintln!("bytes loaded: {}", bytes.len());
    }

    pub fn step(&mut self, keymap: &KeyMap) -> Chip8Result {
        if self.start_time.is_none() {
            self.start_time = Some(SystemTime::now());
        }

        self.run_timers();

        let high_byte = self.high_byte();
        let low_byte = self.low_byte();

        match high(high_byte) {
            0x0 => {
                if *low_byte == 0xE0 {
                    self.pc = self.clear();
                } else if *low_byte == 0xEE {
                    self.pc = self.ret();
                } else {
                    // 0nnn SYS opcodes are ignored on modern systems
                    self.pc += 2
                }
            }
            0x1 => {
                self.pc = self.jump();
            }
            0x2 => {
                self.pc = self.call();
            }
            0x3 => {
                self.pc = self.skip_eq();
            }
            0x4 => {
                self.pc = self.skip_neq();
            }
            0x5 => {
                self.pc = self.skip_eq_reg();
            }
            0x6 => {
                self.pc = self.load_vx();
            }
            0x7 => {
                self.pc = self.add_vx();
            }
            0x8 => match low(low_byte) {
                0 => {
                    self.pc = self.set_vx_to_vy();
                }
                1 => {
                    self.pc = self.vx_or_vy();
                }
                2 => {
                    self.pc = self.vx_and_vy();
                }
                3 => {
                    self.pc = self.vx_xor_vy();
                }
                4 => {
                    self.pc = self.add_vx_and_vy();
                }
                5 => {
                    self.pc = self.sub_vx_and_vy();
                }
                6 => {
                    self.pc = self.vx_shr();
                }
                7 => {
                    self.pc = self.vx_subn_vy();
                }
                0xE => {
                    self.pc = self.vx_shl();
                }
                _ => {
                    return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte));
                }
            },
            0x9 => {
                self.pc = self.skip_vx_neq_vy();
            }
            0xA => {
                self.pc = self.load_i();
            }
            0xB => {
                self.pc = self.jump_plus_v0();
            }
            0xC => {
                self.pc = self.rand();
            }
            0xD => {
                self.pc = self.draw();
            }
            0xE => {
                match low_byte {
                    0x9E => {
                        self.skip_pressed(keymap);
                    }
                    0xA1 => {
                        self.skip_not_pressed(keymap);
                    }
                    _ => return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte)),
                };
            }
            0xF => match low_byte {
                0x07 => {
                    self.pc = self.set_vx_delay_timer();
                }
                0x0A => {
                    self.pc = self.wait_and_load_key_press(keymap);
                }
                0x15 => {
                    self.pc = self.set_delay_timer();
                }
                0x18 => {
                    self.pc = self.set_sound_timer();
                }
                0x1E => {
                    self.pc = self.add();
                }
                0x29 => {
                    self.pc = self.set_i_to_sprite_vx();
                }
                0x33 => {
                    self.pc = self.store_bcd();
                }
                0x55 => {
                    self.pc = self.store_array();
                }
                0x65 => {
                    self.pc = self.load_array();
                }
                _ => {
                    return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte));
                }
            },
            _ => return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte)),
        }

        Ok(())
    }

    fn run_timers(&mut self) {
        match self.start_time {
            Some(ref time) => {
                if !(time.elapsed().unwrap().as_secs_f64() % CLOCK_CYCLE <= 0.01) {
                    return;
                }

                if self.registers.dt > 0 {
                    self.registers.dt -= 1;
                }
                if self.registers.st > 0 {
                    self.registers.st -= 1;
                }
            }
            None => {}
        }
    }

    pub fn sound_on(&self) -> bool {
        self.registers.st > 0
    }

    // 00E0 - CLS
    fn clear(&mut self) -> usize {
        self.disassemble("CLS");

        // Clear the display.
        self.display.clear();

        self.pc + 2
    }

    // 00EE - RET
    fn ret(&mut self) -> usize {
        self.disassemble("RET");

        // The interpreter sets the program counter to the address at the top of
        // the stack, then subtracts 1 from the stack pointer.

        // NOTE: We do this in reverse order, as our stack pointer always points
        //       the next available space in the stack

        self.sp -= 1;

        // NOTE: We add two here counter-intuitively, as the we want to execute
        //       the next instruction after the return point.
        self.stack[self.sp] + 2
    }

    // 1nnn - JP addr
    fn jump(&mut self) -> usize {
        let addr = self.addr();
        self.disassemble(format!("JP {}", addr).as_str());

        // The interpreter sets the program counter to nnn.
        // As we always return the new program counter, we just return the addr
        addr.into()
    }

    // 2nnn - CALL addr
    fn call(&mut self) -> usize {
        let addr = self.addr();
        self.disassemble(format!("CALL {}", addr).as_str());

        // The interpreter increments the stack pointer, then puts the current
        // PC on the top of the stack. The PC is then set to nnn.

        // NOTE: We do this action in reverse order, so the stack pointer always
        //       points to the next available space on stack

        self.stack[self.sp] = self.pc;
        self.sp += 1;

        // As we always return the new program counter, we just return the addr
        addr.into()
    }

    // 3xkk - SE Vx, byte
    fn skip_eq(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("SE V{:x}, {}", x, self.low_byte()).as_str());

        // The interpreter compares register Vx to kk
        let contents = self.registers.get(x);

        // and if they are equal, increments the program counter by 2.
        if contents == *self.low_byte() {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    // 4xkk - SNE Vx, byte
    fn skip_neq(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("SNE V{:x}, {}", x, self.low_byte()).as_str());

        // The interpreter compares register Vx to kk
        let contents = self.registers.get(x);

        // and if they are not equal, increments the program counter by 2.
        if contents != *self.low_byte() {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    // 5xy0 - SE Vx, Vy
    fn skip_eq_reg(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("SE V{:x}, V{:x}", x, y).as_str());

        // The interpreter compares register Vx to register Vy, and if they are
        // equal, increments the program counter by 2.
        if self.registers.get(x) == self.registers.get(y) {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    // 6xkk - LD Vx, byte
    fn load_vx(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("LD V{:x}, {}", x, self.low_byte()).as_str());

        // The interpreter puts the value kk into register Vx.
        self.registers.put(x, *self.low_byte());

        self.pc + 2
    }

    // 7xkk - ADD Vx, byte
    fn add_vx(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("ADD V{:x}, {}", x, self.low_byte()).as_str());

        // Adds the value kk to the value of register Vx, then stores the result
        // in Vx.
        let (result, _) = self.registers.get(x).overflowing_add(*self.low_byte());
        self.registers.put(x, result);

        self.pc + 2
    }

    // 8xy0 - LD Vx, Vy
    fn set_vx_to_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("LD V{:x}, V{:x}", x, y).as_str());

        // Stores the value of register Vy in register Vx.
        self.registers.put(x, self.registers.get(y));

        self.pc + 2
    }

    // 8xy1 - OR Vx, Vy
    fn vx_or_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("OR V{:x}, V{:x}", x, y).as_str());

        // Performs a bitwise OR on the values of Vx and Vy, then stores the
        // result in Vx.
        self.registers
            .put(x, self.registers.get(y) | self.registers.get(x));

        self.pc + 2
    }

    // 8xy2 - AND Vx, Vy
    fn vx_and_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("AND V{:x}, V{:x}", x, y).as_str());

        // Performs a bitwise AND on the values of Vx and Vy, then stores the
        // result in Vx.
        self.registers
            .put(x, self.registers.get(y) & self.registers.get(x));

        self.pc + 2
    }

    // 8xy3 - XOR Vx, Vy
    fn vx_xor_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("XOR V{:x}, V{:x}", x, y).as_str());

        // Performs a bitwise exclusive OR on the values of Vx and Vy, then
        // stores the result in Vx.
        self.registers
            .put(x, self.registers.get(y) ^ self.registers.get(x));

        self.pc + 2
    }

    // 8xy4 - ADD Vx, Vy
    fn add_vx_and_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("ADD V{:x}, V{:x}", x, y).as_str());

        // The values of Vx and Vy are added together.
        let (result, carry) = self.registers.get(x).overflowing_add(self.registers.get(y));

        // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1,
        if carry {
            self.registers.v_f = 1;
        } else {
            // otherwise 0.
            self.registers.v_f = 0;
        }

        // Only the lowest 8 bits of the result are kept, and stored in Vx.
        self.registers.put(x, result);

        self.pc + 2
    }

    // 8xy5 - SUB Vx, Vy
    fn sub_vx_and_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("SUB V{:x}, V{:x}", x, y).as_str());

        let vx = self.registers.get(x);
        let vy = self.registers.get(y);

        // If Vx > Vy, then VF is set to 1, otherwise 0.
        if vx > vy {
            self.registers.v_f = 1
        } else {
            self.registers.v_f = 0
        }

        // Then Vy is subtracted from Vx, and the results stored in Vx.
        let (result, _) = vx.overflowing_sub(vy);
        self.registers.put(x, result);

        self.pc + 2
    }

    // 8xy6 - SHR Vx {, Vy}
    fn vx_shr(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("SHR V{:x}", x).as_str());

        // If the least-significant bit of Vx is 1, then VF is set to 1,
        // otherwise 0. Then Vx is divided by 2.

        let (result, carry) = self.registers.get(x).overflowing_shr(1);

        self.registers.put(x, result);

        if carry {
            self.registers.v_f = 1
        } else {
            self.registers.v_f = 0
        }

        self.pc + 2
    }

    // 8xy7 - SUBN Vx, Vy
    fn vx_subn_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = low(self.low_byte());
        self.disassemble(format!("SUBN V{:x}, V{:x}", x, y).as_str());

        let vx = self.registers.get(x);
        let vy = self.registers.get(y);

        // If Vy > Vx, then VF is set to 1,
        if vy > vx {
            self.registers.v_f = 1
        } else {
            // otherwise 0.
            self.registers.v_f = 0
        }

        // Then Vx is subtracted from Vy, and the results stored in Vx.
        let (result, _) = vy.overflowing_sub(vx);
        self.registers.put(x, result);

        self.pc + 2
    }

    // 8xyE - SHL Vx {, Vy}
    fn vx_shl(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("SHL V{:x}", x).as_str());

        // If the most-significant bit of Vx is 1, then VF is set to 1,
        // otherwise to 0. Then Vx is multiplied by 2.

        let (result, carry) = self.registers.get(x).overflowing_shl(1);

        self.registers.put(x, result);

        if carry {
            self.registers.v_f = 1
        } else {
            self.registers.v_f = 0
        }

        self.pc + 2
    }

    // 9xy0 - SNE Vx, Vy
    fn skip_vx_neq_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("SNE V{:x}, V{:x}", x, y).as_str());

        // The values of Vx and Vy are compared,
        if self.registers.get(x) != self.registers.get(y) {
            // and if they are not equal, the program counter is increased by 2.
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    // Annn - LD I, addr
    fn load_i(&mut self) -> usize {
        let addr = self.addr();
        self.disassemble(format!("LD I, {:x}", addr).as_str());

        // The value of register I is set to nnn
        self.registers.i = addr;

        self.pc + 2
    }

    // Bnnn - JP V0, addr
    fn jump_plus_v0(&mut self) -> usize {
        let addr = self.addr();
        self.disassemble(format!("JP V0, {:x}", addr).as_str());

        // The program counter is set to nnn plus the value of V0.

        // As we always return the new program counter, we return the sum of
        // addr and v0
        addr as usize + self.registers.v_0 as usize
    }

    // Cxkk - RND Vx, byte
    fn rand(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("RND V{:x}, {:x}", x, self.low_byte()).as_str());

        // The interpreter generates a random number from 0 to 255
        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen();

        // which is then ANDed with the value kk.
        let random_number = random_number & self.low_byte();

        // The results are stored in Vx.
        self.registers.put(x, random_number);

        self.pc + 2
    }

    // Dxyn - DRW Vx, Vy, nibble
    fn draw(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        let n = low(self.low_byte());
        self.disassemble(format!("DRW V{:x}, V{:x}, {}", x, y, n).as_str());

        // The interpreter reads n bytes from memory, starting at the address
        // stored in I.
        let address = self.registers.i;
        let bytes = self
            .ram
            .get(address as usize..(address + n as u16) as usize)
            .expect("Bytes to draw out of range");

        // These bytes are then displayed as sprites on screen at
        // coordinates (Vx, Vy).
        let x = self.registers.get(x);
        let y = self.registers.get(y);
        let sprite = Sprite::new(bytes);

        let collision = sprite.draw(x.into(), y.into(), &mut self.display);

        // If this causes any pixels to be erased, VF is set to 1
        if collision == Collision::True {
            self.registers.v_f = 1;
        } else {
            // otherwise it is set to 0.
            self.registers.v_f = 0;
        }

        self.pc + 2
    }

    // Ex9E - SKP Vx
    fn skip_pressed(&mut self, keymap: &KeyMap) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("SKP V{:x}", x).as_str());

        // Skip next instruction if key with the value of Vx is pressed.
        let vx = self.registers.get(x);

        if keymap.is_key_pressed(vx) {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    // ExA1 - SKNP Vx
    fn skip_not_pressed(&mut self, keymap: &KeyMap) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("SKNP V{:x}", x).as_str());

        // Skip next instruction if key with the value of Vx is not pressed.
        let vx = self.registers.get(x);

        if !keymap.is_key_pressed(vx) {
            self.pc + 4
        } else {
            self.pc + 2
        }
    }

    // Fx07 - LD Vx, DT
    fn set_vx_delay_timer(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("LD V{:x}, DT", x).as_str());

        // The value of DT is placed into Vx.
        self.registers.put(x, self.registers.dt);

        self.pc + 2
    }

    // Fx0A - LD Vx, K
    fn wait_and_load_key_press(&mut self, keymap: &KeyMap) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("LD V{:x}, K", x).as_str());

        match keymap.most_recent_key() {
            Some(key) => {
                //then the value of that key is stored in Vx.
                self.registers.put(x, *key);
                self.pc + 2
            }
            None => {
                // All execution stops until a key is pressed,
                // Rather than setting some state varaible on the cpu, we can
                // leave the program counter where it is and return to the
                // execution and render loop.
                self.pc
            }
        }
    }

    // Fx15 - LD DT, Vx
    fn set_delay_timer(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("LD DT, V{:x}", x).as_str());

        // DT is set equal to the value of Vx.
        self.registers.dt = self.registers.get(x);

        self.pc + 2
    }

    // Fx18 - LD ST, Vx
    fn set_sound_timer(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("SD DT, V{:x}", x).as_str());

        // ST is set equal to the value of Vx.
        self.registers.st = self.registers.get(x);

        self.pc + 2
    }

    // Fx1E - ADD I, Vx
    fn add(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("ADD I, V{:x}", x).as_str());

        // The values of I and Vx are added, and the results are stored in I.
        let (result, _) = self
            .registers
            .i
            .overflowing_add(self.registers.get(x) as u16);

        self.registers.i = result;

        self.pc + 2
    }

    // Fx29 - LD F, Vx
    fn set_i_to_sprite_vx(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("LD F, V{:x}", x).as_str());

        // The value of I is set to the location for the hexadecimal sprite
        // corresponding to the value of Vx.
        let vx = self.registers.get(x);

        // Sprite characters stored every 5 bytes starting at 0
        // NOTE: Specification is unclear on what to do when this value is greater
        //       than 0xF, for now we will do nothing

        if vx <= 0xF {
            self.registers.i = vx as u16 * 5;
        }

        self.pc + 2
    }

    // Fx33 - LD B, Vx
    fn store_bcd(&mut self) -> usize {
        let x = low(self.high_byte());
        self.disassemble(format!("LD B, V{:x}", x).as_str());

        // The interpreter takes the decimal value of Vx, and places the
        let vx = self.registers.get(x);

        // hundreds digit in memory at location in I,
        let i = self.registers.i as usize;
        self.ram[i] = vx / 100;

        // the tens digit at location I+1,
        self.ram[i + 1] = vx / 10 % 10;

        // and the ones digit at location I+2.
        self.ram[i + 2] = vx % 10;

        self.pc + 2
    }

    // Fx55 - LD [I], Vx
    fn store_array(&mut self) -> usize {
        let x = low(self.high_byte());
        let i = self.registers.i;
        self.disassemble(format!("LD [{:x}], V{:x}", i, x).as_str());

        // The interpreter copies the values of registers V0 through Vx into
        // memory, starting at the address in I.

        for n in 0..=x {
            self.ram[i as usize + n as usize] = self.registers.get(n);
        }

        self.pc + 2
    }

    // Fx65 - LD Vx, [I]
    fn load_array(&mut self) -> usize {
        let x = low(self.high_byte());
        let i = self.registers.i;
        self.disassemble(format!("LD V{:x}, [{:x}]", x, i).as_str());

        // The interpreter reads values from memory starting at location I
        // into registers V0 through Vx.
        for n in 0..=x {
            self.registers.put(n, self.ram[i as usize + n as usize]);
        }

        self.pc + 2
    }

    // 3.0 - Chip-8 Instrutions
    // All instructions are 2 bytes long and are stored
    // most-significant-byte first. In memory, the first byte of each
    // instruction should be located at an even addresses.
    fn high_byte(&self) -> &u8 {
        &self.ram[self.pc]
    }

    fn low_byte(&self) -> &u8 {
        &self.ram[self.pc + 1]
    }

    fn addr(&self) -> u16 {
        let mask = (1 << 12) - 1;
        self.instruction() & mask
    }

    fn instruction(&self) -> u16 {
        ((*self.high_byte() as u16) << 8) | *self.low_byte() as u16
    }

    fn load_hexadecimal_display_bytes(&mut self) {
        // Programs may also refer to a group of sprites representing the
        // hexadecimal digits 0 through F. These sprites are 5 bytes long, or
        // 8x5 pixels. The data should be stored in the interpreter area of
        // Chip-8 memory (0x000 to 0x1FF).
        let bytes = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // "0"
            0x20, 0x60, 0x20, 0x20, 0x70, // "1"
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // "2"
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // "3"
            0x90, 0x90, 0xF0, 0x10, 0x10, // "4"
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // "5"
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // "6"
            0xF0, 0x10, 0x20, 0x40, 0x40, // "7"
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // "8"
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // "9"
            0xF0, 0x90, 0xF0, 0x90, 0x90, // "A"
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // "B"
            0xF0, 0x80, 0x80, 0x80, 0xF0, // "C"
            0xE9, 0x90, 0x90, 0x90, 0xE0, // "D"
            0xF0, 0x80, 0xF0, 0x80, 0xF9, // "E"
            0xF0, 0x80, 0xF0, 0x80, 0x80, // "F"
        ];

        for (i, byte) in bytes.iter().enumerate() {
            self.ram[i] = *byte;
        }
    }
    fn disassemble(&self, note: &str) {
        if !self.debug_output {
            return;
        }

        println!(
            "[{}] {:02x}{:02x} - {}",
            self.pc,
            self.high_byte(),
            self.low_byte(),
            note
        );
    }

    pub fn set_debug_output(&mut self, value: bool) {
        self.debug_output = value;
    }

    pub fn dump_to_stdout(&self) {
        println!("=== MEMORY ===");
        for line in self.ram.chunks(64) {
            for instruction in line.chunks(2) {
                print!("{:02X}{:02X} ", instruction[0], instruction[1]);
            }
            print!("\n");
        }

        println!();
        println!("=== REGISTERS ===");
        self.registers.dump_to_stdout();

        println!();
        println!("=== CPU STATE ===");
        println!("pc: {:04X}", self.pc);
        println!("sp: {:04X}", self.sp);
        println!("stack: {:?}", self.stack);

        println!();
        println!("=== SCREEN ===");
        self.display.dump_to_stdout();
    }
}

fn high(byte: &u8) -> u8 {
    let mask = (1 << 4) - 1;
    (byte & mask << 4) >> 4
}

fn low(byte: &u8) -> u8 {
    let mask = (1 << 4) - 1;
    byte & mask
}

// 2.2 - Registers
pub struct Registers {
    // Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx,
    // where x is a hexadecimal digit (0 through F).
    v_0: u8,
    v_1: u8,
    v_2: u8,
    v_3: u8,
    v_4: u8,
    v_5: u8,
    v_6: u8,
    v_7: u8,
    v_8: u8,
    v_9: u8,
    v_a: u8,
    v_b: u8,
    v_c: u8,
    v_d: u8,
    v_e: u8,
    v_f: u8,

    // There is also a 16-bit register called I. This register is generally
    // used to store memory addresses
    i: u16,

    // Chip-8 provides 2 timers, a delay timer and a sound timer.
    // The delay timer is active whenever the delay timer register (DT) is non-
    // zero. This timer does nothing more than subtract 1 from the value of DT
    // at a rate of 60Hz. When DT reaches 0, it deactivates.
    dt: u8,

    // The sound timer is active whenever the sound timer register (ST) is non-
    // zero. This timer also decrements at a rate of 60Hz, however, as long as
    // ST's value is greater than zero, the Chip-8 buzzer will sound. When ST
    // reaches zero, the sound timer deactivates.
    st: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            v_0: 0,
            v_1: 0,
            v_2: 0,
            v_3: 0,
            v_4: 0,
            v_5: 0,
            v_6: 0,
            v_7: 0,
            v_8: 0,
            v_9: 0,
            v_a: 0,
            v_b: 0,
            v_c: 0,
            v_d: 0,
            v_e: 0,
            v_f: 0,
            i: 0,
            dt: 0,
            st: 0,
        }
    }

    pub fn put(&mut self, register: u8, value: u8) {
        match register {
            0x0 => self.v_0 = value,
            0x1 => self.v_1 = value,
            0x2 => self.v_2 = value,
            0x3 => self.v_3 = value,
            0x4 => self.v_4 = value,
            0x5 => self.v_5 = value,
            0x6 => self.v_6 = value,
            0x7 => self.v_7 = value,
            0x8 => self.v_8 = value,
            0x9 => self.v_9 = value,
            0xa => self.v_a = value,
            0xb => self.v_b = value,
            0xc => self.v_c = value,
            0xd => self.v_d = value,
            0xe => self.v_e = value,
            0xf => self.v_f = value,
            _ => panic!(
                "Tried to set a register that doesn't exist v_{:x}",
                register
            ),
        }
    }

    pub fn get(&self, register: u8) -> u8 {
        match register {
            0x0 => self.v_0,
            0x1 => self.v_1,
            0x2 => self.v_2,
            0x3 => self.v_3,
            0x4 => self.v_4,
            0x5 => self.v_5,
            0x6 => self.v_6,
            0x7 => self.v_7,
            0x8 => self.v_8,
            0x9 => self.v_9,
            0xa => self.v_a,
            0xb => self.v_b,
            0xc => self.v_c,
            0xd => self.v_d,
            0xe => self.v_e,
            0xf => self.v_f,
            _ => panic!(
                "Tried to set a register that doesn't exist v_{:x}",
                register
            ),
        }
    }

    pub fn dump_to_stdout(&self) {
        print!("v_0: {:02X} ", self.v_0);
        print!("v_1: {:02X} ", self.v_1);
        print!("v_2: {:02X} ", self.v_2);
        print!("v_3: {:02X} ", self.v_3);
        print!("v_4: {:02X} ", self.v_4);
        print!("v_5: {:02X} ", self.v_5);
        print!("v_6: {:02X} ", self.v_6);
        print!("v_7: {:02X} ", self.v_7);
        print!("v_8: {:02X} ", self.v_8);
        print!("v_9: {:02X} ", self.v_9);
        print!("v_a: {:02X} ", self.v_a);
        print!("v_b: {:02X} ", self.v_b);
        print!("v_c: {:02X} ", self.v_c);
        print!("v_d: {:02X} ", self.v_d);
        print!("v_e: {:02X} ", self.v_e);
        print!("v_f: {:02X} ", self.v_f);
        println!();
        println!("i: {:04X}", self.i);
        println!("dt: {:02X} {:?}", self.dt, self.dt > 0);
        println!("st: {:02X} {:?}", self.st, self.st > 0);
    }
}

pub type Chip8Result = Result<(), Error>;

pub enum Error {
    UnrecognisedInstruction(u8, u8),
}
