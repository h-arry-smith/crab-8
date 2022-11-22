// Reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

use std::fs::{self, File};

// 2.1 - Memory
// Most Chip-8 programs start at location 0x200 (512), but some begin at
// 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660
// computer.
const NORMAL_START_INDEX: usize = 512;
const ETI_660_START_INDEX: usize = 1526;

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
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            ram: [0; 4096],
            registers: Registers::new(),
            pc: 0,
            sp: 0,
            stack: [0; 16],
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let bytes = fs::read(path).expect("Could not open file.");

        // TODO: Take a CLI flag for the start address to load into memory, for
        //       now we just use the more common 0x200 start address.
        for (index, byte) in bytes.iter().enumerate() {
            self.ram[NORMAL_START_INDEX + index] = *byte;
        }

        eprintln!("bytes loaded: {}", bytes.len());
    }

    pub fn run(&mut self) {
        // TODO: Set PC start based on CLI flag for start address
        self.pc = NORMAL_START_INDEX;

        loop {
            let high_byte = self.high_byte();
            let low_byte = self.low_byte();

            match high(high_byte) {
                0xA => {
                    self.pc = self.load_i();
                }
                _ => {
                    eprintln!("Unrecognised instrution 0x{:x}{:x}", high_byte, low_byte);
                    break;
                }
            }
        }
    }

    // Annn - LD I, addr
    fn load_i(&mut self) -> usize {
        // The value of register I is set to nnn
        self.registers.i = self.addr();

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
}

fn high(byte: &u8) -> u8 {
    let mask = (1 << 4) - 1;
    (byte & mask << 4) >> 4
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
}
