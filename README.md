# Flake-8
A fully featured Chip-8 Emulator written in Rust.

This project was written as a learning & development project in rust, and as such 
the code may not be the most idiomatic rust. PR's with improvements welcome :).

All opcodes, sound, delay and keyboard are implemented.

## Usage

```
Usage: flake-8 [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the chip-8 rom that you want to run

Options:
  -d, --debug     Display debug output when running a chip-8 rom
  -f, --fg <FG>   Set the color in hex (e.g #FF0000) for pixels that are on
  -b, --bg <BG>   Set the color in hex (e.g #00FF00) for pixels that are off
  -e, --eti-mode  Start the emulator in ETI 660 Mode
  -h, --help      Print help information
  -V, --version   Print version information
```

## Keyboard
*Original*

|1|2|3|C|    
|-|-|-|-|
|4|5|6|D|    
|7|8|9|E|    
|A|0|B|F|   

*Keyboard Mapping*

|1|2|3|4|
|-|-|-|-|
|Q|W|E|R|
|A|S|D|F|
|Z|X|C|V|

## Reference Material
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
- https://github.com/kripod/chip8-roms
- https://github.com/corax89/chip8-test-rom
- https://chip.netlify.app/emulator/