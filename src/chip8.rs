// Reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

use rand::Rng;
use std::fs;

use crate::display::{Collision, Display, Sprite};

// 2.1 - Memory
// Most Chip-8 programs start at location 0x200 (512), but some begin at
// 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660
// computer.
const NORMAL_START_INDEX: usize = 512;
// const ETI_660_START_INDEX: usize = 1526;

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
        };

        new.load_hexadecimal_display_bytes();
        new
    }

    pub fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).expect("Could not open file.");

        // TODO: Take a CLI flag for the start address to load into memory, for
        //       now we just use the more common 0x200 start address.
        for (index, byte) in bytes.iter().enumerate() {
            self.ram[NORMAL_START_INDEX + index] = *byte;
        }

        // TODO: Set PC start based on CLI flag for start address
        self.pc = NORMAL_START_INDEX;

        eprintln!("bytes loaded: {}", bytes.len());
    }

    pub fn step(&mut self) -> Chip8Result {
        let high_byte = self.high_byte();
        let low_byte = self.low_byte();

        match high(high_byte) {
            0x0 => {
                if *low_byte == 0xE0 {
                    self.pc = self.clear();
                } else if *low_byte == 0xEE {
                    self.pc = self.ret();
                } else {
                    return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte));
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
            0x8 => {
                // TODO: Rest of the 0x8 instructions
                if low(low_byte) == 0 {
                    self.pc = self.set_vx_to_vy();
                } else if low(low_byte) == 2 {
                    self.pc = self.vx_and_vy();
                } else if low(low_byte) == 4 {
                    self.pc = self.add_vx_and_vy();
                } else if low(low_byte) == 6 {
                    self.pc = self.vx_shr();
                } else if low(low_byte) == 0xE {
                    self.pc = self.vx_shl();
                } else {
                    return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte));
                }
            }
            0xA => {
                self.pc = self.load_i();
            }
            0xC => {
                self.pc = self.rand();
            }
            0xD => {
                self.pc = self.draw();
            }
            0xF => {
                // TODO: Rest of the 0xF space instructions
                if *low_byte == 0x1E {
                    self.pc = self.add();
                } else if *low_byte == 0x55 {
                    self.pc = self.store_array();
                } else if *low_byte == 0x65 {
                    self.pc = self.load_array();
                } else {
                    return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte));
                }
            }
            _ => return Err(Error::UnrecognisedInstruction(*high_byte, *low_byte)),
        }

        Ok(())
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

    // 8xy2 - AND Vx, Vy
    fn vx_and_vy(&mut self) -> usize {
        let x = low(self.high_byte());
        let y = high(self.low_byte());
        self.disassemble(format!("AND V{:x}, V{:x}", x, y).as_str());

        // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
        self.registers
            .put(x, self.registers.get(y) & self.registers.get(x));

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

    // Annn - LD I, addr
    fn load_i(&mut self) -> usize {
        self.disassemble(format!("LD I, {}", self.addr()).as_str());

        // The value of register I is set to nnn
        self.registers.i = self.addr();

        self.pc + 2
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

    // Fx65 - LD [I], Vx
    fn store_array(&mut self) -> usize {
        let x = low(self.high_byte());
        let i = self.registers.i;
        self.disassemble(format!("LD [{:x}], V{:x}", x, i).as_str());

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
        println!("i: {:04X}", self.i)
    }
}

pub type Chip8Result = Result<(), Error>;

pub enum Error {
    UnrecognisedInstruction(u8, u8),
}
