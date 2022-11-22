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

    // The program counter (PC) should be 16-bit, and is used to store the
    // currently executing address.
    pc: u16,

    // The stack pointer (SP) can be 8-bit, it is used to point to the topmost
    // level of the stack.
    sp: u8,

    // The stack is an array of 16 16-bit values, used to store the address that
    // the interpreter shoud return to when finished with a subroutine. Chip-8
    // allows for up to 16 levels of nested subroutines.
    stack: [u16; 16],
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

    pub fn run(&self) {
        todo!()
    }
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
