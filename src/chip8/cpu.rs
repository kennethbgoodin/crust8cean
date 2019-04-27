use rand::Rng;
use minifb::{Key, WindowOptions, Window, KeyRepeat, Scale};
use std::num::Wrapping;
use std::time::{Instant, Duration};
use std::thread;
use std::ops::Sub;

const MEMORY_SIZE: u16 = 4096;
const SCREEN_WIDTH: u16 = 64;
const SCREEN_HEIGHT: u16 = 32;
const SCREEN_SIZE: u16 = SCREEN_WIDTH as u16 * SCREEN_HEIGHT as u16;

// 5 bytes by 16 characters
pub const FONT_LEN: u8 = 5;
pub const FONT_SIZE: u8 = FONT_LEN * 16;

pub const PROGRAM_COUNTER_START_ADDR: u16 = 0x200;

// 60 frames a second
// TODO don't hardcode, configurable
const TICK_DURATION: Duration = Duration::from_millis(
    ((1 as f64 / 120 as f64) * 1000 as f64) as u64);

// sprites
const FONT_SPRITES: [u8; FONT_SIZE as usize] = [
                            // 0
                            0b11110000,
                            0b10010000,
                            0b10010000,
                            0b10010000,
                            0b11110000,
                            // 1
                            0b00100000,
                            0b01100000,
                            0b00100000,
                            0b00100000,
                            0b01110000,
                            // 2
                            0b11110000,
                            0b00010000,
                            0b11110000,
                            0b10000000,
                            0b11110000,
                            // 3
                            0b11110000,
                            0b00010000,
                            0b11110000,
                            0b00010000,
                            0b11110000,
                            // 4
                            0b10100000,
                            0b10100000,
                            0b11100000,
                            0b00100000,
                            0b00100000,
                            // 5
                            0b11110000,
                            0b10000000,
                            0b11110000,
                            0b00010000,
                            0b11110000,
                            // 6
                            0b11110000,
                            0b10000000,
                            0b11110000,
                            0b10010000,
                            0b11110000,
                            // 7
                            0b11110000,
                            0b00010000,
                            0b00100000,
                            0b01000000,
                            0b01000000,
                            // 8
                            0b11110000,
                            0b10010000,
                            0b11110000,
                            0b10010000,
                            0b11110000,
                            // 9
                            0b11110000,
                            0b10010000,
                            0b11110000,
                            0b00010000,
                            0b11110000,
                            // A
                            0b11110000,
                            0b10010000,
                            0b11110000,
                            0b10010000,
                            0b10010000,
                            // B
                            0b11100000,
                            0b10000000,
                            0b11100000,
                            0b10010000,
                            0b11100000,
                            // C
                            0b11110000,
                            0b10000000,
                            0b10000000,
                            0b10000000,
                            0b11110000,
                            // D
                            0b11100000,
                            0b10010000,
                            0b10010000,
                            0b10010000,
                            0b11100000,
                            // E
                            0b11110000,
                            0b10000000,
                            0b11110000,
                            0b10000000,
                            0b11110000,
                            // F
                            0b11110000,
                            0b10000000,
                            0b11110000,
                            0b10000000,
                            0b10000000];

pub struct Cpu {
    // memory
    memory: [u8; MEMORY_SIZE as usize],
    // registers (V0-VF)
    registers: [u8; 16],
    i: usize, // address register

    program_counter: u16,

    // stack
    stack: [u16; 16],
    stack_pointer: u8,

    // timers
    delay_timer: u8,
    sound_timer: u8,

    // screen memory
    screen: [u8; SCREEN_SIZE as usize],
    redraw: bool,

    // input
    keys_pressed: [bool; 16],
    awaiting_keypress: bool,
    awaiting_keypress_register: usize,

    // window
    window: Window,

    //
    last_cycle: Instant,

    // interpreter specific
    paused: bool,
}

impl Cpu {
    pub fn new(program: &Vec<u8>) -> Self {
        let mut cpu = Cpu {
            memory: [0; MEMORY_SIZE as usize],
            registers: [0; 16],
            i: 0,
            program_counter: PROGRAM_COUNTER_START_ADDR,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: [0; SCREEN_SIZE as usize],
            redraw: false,
            keys_pressed: [false; 16],
            awaiting_keypress: false,
            awaiting_keypress_register: 0,

            window: Window::new("crust8cean",
                                SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize,
                                WindowOptions {
                                    borderless: false,
                                    title: true,
                                    resize: false,
                                    scale: Scale::X8,
                                }).unwrap_or_else(|e| {
                                        println!("{}", e);
                                        panic!("{}", e);
            }),
            last_cycle: Instant::now().sub(TICK_DURATION),
            paused: false,
        };

        // init fonts
        for (x, line) in FONT_SPRITES.iter().enumerate() {
            cpu.memory[x] = *line;
        }

        // init program
        let start_addr = cpu.program_counter;
        for (i, val) in program.iter().enumerate() {
            cpu.memory[(start_addr + i as u16) as usize] = *val;
        }

        cpu
    }

    pub fn run(&mut self) {
        if self.paused {
            return;
        }

        // TODO clean up
        let keys_pressed: Option<Vec<Option<i32>>> = self.window.get_keys_pressed(KeyRepeat::Yes)
            .map(|keys| {
                keys.into_iter().map(|key| {
                    let key_pressed: Option<i32> = match key {
                        // chip-8 16 key keypad
                        Key::Key1 => Some(0),
                        Key::Key2 => Some(1),
                        Key::Key3 => Some(2),
                        Key::Key4 => Some(3),
                        Key::Q => Some(4),
                        Key::W => Some(5),
                        Key::E => Some(6),
                        Key::R => Some(7),
                        Key::A => Some(8),
                        Key::S => Some(9),
                        Key::D => Some(10),
                        Key::F => Some(11),
                        Key::Z => Some(12),
                        Key::X => Some(13),
                        Key::C => Some(14),
                        Key::V => Some(15),
                        _ => None
                    };
                    key_pressed
                }).collect::<Vec<Option<i32>>>()
        });

        if keys_pressed.is_some() {
            let keys_pressed = keys_pressed.unwrap();
            for (i, key) in self.keys_pressed.iter_mut().enumerate() {
                let pressed = keys_pressed.contains(&Some(i as i32));
                *key = pressed;
            }
        }

        // wait for key press
        if self.awaiting_keypress {
            for (i, key) in self.keys_pressed.iter_mut().enumerate() {
                if *key {
                    self.awaiting_keypress = false;
                    self.registers[self.awaiting_keypress_register] = i as u8;
                    break;
                }
            }
        } else {
            // run instruction
            self.emulate_cycle();

            // graphics
            if self.redraw {
                self.render_screen();
                self.redraw = false;
            }

            // decrement timers
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }

        self.last_cycle = Instant::now();
        let elapsed = Instant::now().sub(now);
        if elapsed.as_millis() < TICK_DURATION.as_millis() {
            thread::sleep(TICK_DURATION.sub(elapsed));
        }
    }

    pub fn is_window_open(&self) -> bool {
        return self.window.is_open();
    }

    pub fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        let val = self.stack[self.stack_pointer as usize];
        val
    }

    pub fn get_keys_pressed(&self) -> Vec<Key> {
        let mut keys = Vec::new();
        for (i, key) in self.keys_pressed.iter().enumerate() {
            if *key {
                let pressed = match i {
                    0 => Some(Key::Key1),
                    1 => Some(Key::Key2),
                    2 => Some(Key::Key3),
                    3 => Some(Key::Key4),
                    4 => Some(Key::Q),
                    5 => Some(Key::W),
                    6 => Some(Key::E),
                    7 => Some(Key::R),
                    8 => Some(Key::A),
                    9 => Some(Key::S),
                    10 => Some(Key::D),
                    11 => Some(Key::F),
                    12 => Some(Key::Z),
                    13 => Some(Key::X),
                    14 => Some(Key::C),
                    15 => Some(Key::V),
                    _ => None
                };

                if pressed.is_some() {
                    keys.push(pressed.unwrap());
                }
            }
        }

        keys
    }

    pub fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    pub fn get_program_counter(&self) -> u16 {
        self.program_counter
    }

    pub fn get_stack_pointer(&self) -> u8 {
        self.stack_pointer
    }

    pub fn set_register(&mut self, register: usize, value: u8) {
        self.registers[register] = value;
    }

    pub fn get_register(&self, register: usize) -> u8 {
        self.registers[register]
    }

    pub fn get_address_register(&self) -> usize {
        self.i
    }

    pub fn set_address_register(&mut self, value: usize) {
        self.i = value;
    }

    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }

    pub fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer = value;
    }

    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn set_sound_timer(&mut self, value: u8) {
        self.sound_timer = value;
    }

    fn render_screen(&mut self) {
        let mut screen = [0; SCREEN_SIZE as usize];
        for (i, x) in self.screen.iter().enumerate() {
            if *x != 0 {
                screen[i] = 0xFFFF_FFFF;
            }
        }

        self.window.update_with_buffer(&screen).unwrap();
    }

    fn read(&self, address: u16) -> u8 {
        if address > self.memory.len() as u16 {
            panic!("Read at 0x{:x} out of bounds", address);
        }

        self.memory[address as usize]
    }

    fn read_word(&self, address: u16) -> u16 {
        let low = self.read(address);
        let high = self.read(address + 1);

        ((low as u16) << 8) | (high as u16)
    }

    fn read_word_increment_pc(&mut self, address: u16) -> u16 {
        let word = self.read_word(address);
        self.program_counter += 2;
        word
    }

    fn write(&mut self, address: u16, value: u8) {
        if address > self.memory.len() as u16 {
            panic!("write at {:x} out of bounds", address);
        }

        self.memory[address as usize] = value;
    }

    fn set_carry_flag(&mut self, value: u8) {
        self.registers[0x0F] = value;
    }

    // ingenious nibble matching borrowed from https://github.com/starrhorne/chip8-rust/blob/master/src/processor.rs#L120
    fn emulate_cycle(&mut self) {
        let pc = self.program_counter;

        // the full 16 bits of an instruction, including operands
        let opcode = self.read_word(pc);
        self.program_counter += 2;

        // opcodes are stored in the first 2 bits of an instruction, big endian
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8
        );

        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let z = nibbles.3 as usize;
        println!("program counter: {}", pc);
        println!("opcode: {:x}", opcode);
        match nibbles {
            // 0nnn - SYS addr
            // Jump to routine at nnn
            (0x00, 0x00, 0x00, 0x00) => {
                println!("0nnn");
                // not used in interpreters
            },
            // 00E0 - CLS
            // Clear display
            (0x00, 0x00, 0x0E, 0x00) => {
                println!("CLS");
                for x in 0..SCREEN_SIZE {
                    self.screen[x as usize] = 0;
                }
                self.redraw = true;
            },
            // 00EE - RET
            // Return from a subroutine i.e. set pc to top of stack
            (0x00, 0x00, 0x0E, 0x0E) => {
                println!("RET");
                self.program_counter = self.pop();
            },
            // 1nnn - JMP addr
            // Jump to address nnn
            (0x01, _, _, _) => {
                println!("JMP {:x}", nnn);
                self.program_counter = nnn;
            },
            // 2nnn - CALL addr
            // Call subroutine at nnn
            (0x02, _, _, _) => {
                println!("CALL {:x}", nnn);
                let pc = self.program_counter;
                self.push(pc);
                self.program_counter = nnn;
            },
            // 3xkk - SE Vx, byte
            // Skip next instruction if Vx == kk
            (0x03, _, _, _) => {
                println!("SE V{:x}, {:x}", x, kk);
                let x = self.registers[x];
                if x == kk {
                    self.program_counter += 2;
                }
            },
            // 4xkk - SNE Vx, byte
            // Skip next instruction if Vx != kk
            (0x04, _, _, _) => {
                println!("SNE V{:x}, {:x}", x, kk);
                let x = self.registers[x];
                if x != kk {
                    self.program_counter += 2;
                }
            },
            // 5xy0 - SE Vx, Vy
            // Skip next instruction if Vx == Vy
            (0x05, _, _, 0x00) => {
                println!("SE V{:x}, V{:x}", x, y);
                let x = self.registers[x];
                let y = self.registers[y];

                if x == y {
                    self.program_counter += 2;
                }
            },
            // 6xkk - LD Vx, byte
            // Set Vx to value kk
            // Set Vx to value kk
            (0x06, _, _, _) => {
                println!("LD V{:x}, {:x}", x, kk);
                self.registers[x] = kk;
            },
            // 7xkk ADD Vx, byte
            // Set Vx = Vx + kk
            (0x07, _, _, _) => {
                println!("ADD V{:X}, {:x}", x, kk);
                self.registers[x] = (Wrapping(self.registers[x]) + Wrapping(kk)).0;
            },
            // 8xy0 - LD Vx, Vy
            // Set Vx to value of Vy
            (0x08, _, _, 0x00) => {
                println!("LD V{:x}, V{:x}", x, y);
                let y = self.registers[y];
                self.registers[x] = y;
            },
            // 8xy1 - OR Vx, Vy
            // Set Vx = Vx OR Vy.
            (0x08, _, _, 0x01) => {
                println!("OR V{:x}, V{:x}", x, y);
                self.registers[x] = self.registers[x] | self.registers[y];
            },
            // 8xy2 - AND Vx, Vy
            // Set Vx = Vx AND Vy.
            (0x08, _, _, 0x02) => {
                println!("AND V{:x}, V{:x}", x, y);
                self.registers[x] = self.registers[x] & self.registers[y];
            },
            // 8xy3 - XOR Vx, Vy
            // Set Vx = Vx XOR Vy.
            (0x08, _, _, 0x03) => {
                println!("XOR V{:x}, V{:x}", x, y);
                self.registers[x] = self.registers[x] ^ self.registers[y];
            },
            // 8xy4 - ADD Vx, Vy
            // Set Vx = Vx + Vy, set VF = carry.
            (0x08, _, _, 0x04) => {
                println!("ADD V{:x}, V{:x}", x, y);
                let ret = (Wrapping(self.registers[x] as u16)
                    + Wrapping(self.registers[y] as u16)).0;
                self.set_carry_flag(if ret > 0xFF { 1 } else { 0 });
                // lower 8 bits
                self.registers[x] = (ret & 0x00FF) as u8;
            },
            // 8xy5 - SUB Vx, Vy
            // Set Vx = Vx - Vy, set VF = NOT borrow.
            (0x08, _, _, 0x05) => {
                println!("SUB V{:x}, V{:X}", x, y);
                let x_val = self.registers[x];
                let y_val = self.registers[y];
                println!("{} - {} = {}", x_val, y_val, x_val.wrapping_sub(y_val));
                self.set_carry_flag(if x_val > y_val { 1 } else { 0 });
                self.registers[x] = x_val.wrapping_sub(y_val);
            },
            // 8xy6 - SHR Vx, Vy
            // Set Vx = Vx SHR 1.
            (0x08, _, _, 0x06) => {
                println!("SHR V{:x}, V{:x}", x, y);
                self.set_carry_flag(self.registers[x] & 0x1);
                self.registers[x] >>= 1;
            },
            // 8xy7 - SUBN Vx, Vy
            // Set Vx = Vy - Vx, set VF = NOT borrow.
            (0x08, _, _, 0x07) => {
                println!("SUBN V{:x}, V{:x}", x, y);
                let x_val = self.registers[x];
                let y_val = self.registers[y];
                self.set_carry_flag(if y_val > x_val { 1 } else { 0 });
                self.registers[x] = y_val.wrapping_sub(x_val);
            },
            // 8xyE - SHL Vx, Vy
            // Set Vx = Vx SHL 1.
            (0x08, _, _, 0x0E) => {
                println!("SHL V{:x}, V{:x}", x, y);
                self.set_carry_flag((self.registers[x] & 0b10000000) >> 7);
                self.registers[x] <<= 1;
            },
            // 9xy0 - SNE Vx, Vy
            // Skip next instruction if Vx != Vy
            (0x09, _, _, 0x00) => {
                println!("SNE V{:x}, V{:x}", x, y);
                if self.registers[x] != self.registers[y] {
                    self.program_counter += 2;
                }
            },
            // Annn - LD I, addr
            // Set register I to nnn
            (0x0A, _, _, _) => {
                println!("LD I, {:X}", nnn);
                self.i = nnn as usize;
            },
            // Bnnn - JP V0, addr
            // Jump to location nnn + V0
            (0x0B, _, _, _) => {
                println!("JP V0, {:x}", nnn);
                self.program_counter = self.registers[0] as u16 + nnn
            },
            // Cxkk - RND Vx, byte
            // Set Vx = random byte (0-255) AND kk
            (0x0C, _, _, _) => {
                println!("RND V{:x}, rng", x);
                let rng = rand::thread_rng().gen_range(0, 256) as u8;
                self.registers[x] = rng & kk;
            },
            // Dxyn - DRW Vx, Vy, nibble
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            (0x0D, _, _, _) => {
                println!("DXYN V{:x}, V{:x} {}", x, y, z);
                let sprite_height = z;
                let x = self.registers[x] as usize;
                let y = self.registers[y] as usize;

                self.set_carry_flag(0);
                for height in 0..sprite_height {
                    // get current sprite stored in i + offset by fx29
                    let cur_line = self.memory[self.i + height];
                    // iterate each bit in byte
                    for bit in 0..8 {
                        // check if current bit (pixel) is set, if it is we xor it with
                        // existing
//                        if cur_line & (0x80 >> bit) != 0 {
//                            let pos = (x + bit + ((y + line) * SCREEN_WIDTH as usize) % 0x800);
//                            if self.screen[pos] == 1 {
//                                self.screen[pos] ^= 1;
//                            }
//                        }
                        if cur_line & (0x80 >> bit) != 0 {
                            let pos_x = (x + bit) as u16 % SCREEN_WIDTH;
                            let pos_y = (y + height) as u16 % SCREEN_HEIGHT;
                            let pos = (pos_x + (pos_y * SCREEN_WIDTH)) as usize;
                            if self.screen[pos] == 1 {
                                self.registers[0x0f] = 1;
                            }
                            self.screen[pos] ^= 1;
                        }
                    }
                }
                if self.registers[0x0f] > 0 {
                    self.redraw = true;
                }
            },
            // Ex9E - SKP Vx
            // Skip next instruction if key with the value of Vx is pressed.
            (0x0E, _, 0x09, 0x0E) => {
                println!("SKP V{:x}", x);
                let x = self.registers[x];
                if self.keys_pressed[x as usize] {
                    self.program_counter += 2;
                }
            },
            // ExA1 - SKNP Vx
            // Skip next instruction if key with the value of Vx is not pressed.
            (0x0E, _, 0x0A, 0x01) => {
                println!("SKNP V{:x}", x);
                let x = self.registers[x];
                if !self.keys_pressed[x as usize] {
                    self.program_counter += 2;
                }
            }
            // Fx07 - LD Vx, DT
            // Set Vx = delay timer value.
            (0x0F, _, _, 0x07) => {
                println!("LD V{:x} DT", x);
                self.registers[x] = self.delay_timer;
            },
            // Fx0A - LD Vx, K
            // Wait for a key press, store the value of the key in Vx.
            (0x0F, _, _, 0x0A) => {
                println!("LD V{} K", x);
                self.awaiting_keypress = true;
                self.awaiting_keypress_register = x;
            },
            // Fx15 - LD DT, Vx
            // Set delay timer = Vx.
            (0x0F, _, 0x01, 0x05) => {
                println!("LD DT V{:x}", x);
                self.delay_timer = self.registers[x]
            },
            // Fx18 - LD ST, Vx
            // Set sound timer = Vx.
            (0x0F, _, 0x01, 0x08) => {
                println!("LD ST V{:x}", x);
                self.sound_timer = self.registers[x];
            },
            // Fx1E - ADD I, Vx
            // Set I = I + Vx.
            (0x0F, _, 0x01, 0x0E) => {
                println!("ADD I V{:x}", x);
                self.i += self.registers[x] as usize;
                self.set_carry_flag(if self.i > 0x0F00 { 1 } else { 0 });
            },
            // Fx29 - LD F, Vx
            // Set I = location of sprite for digit Vx.
            (0x0F, _, 0x02, 0x09) => {
                println!("LD F V{:x}", x);
                self.i = (self.registers[x] as usize) * FONT_LEN as usize;
            },
            // Fx33 - LD B, Vx
            // Store BCD representation of Vx in memory locations I, I+1, and I+2.
            (0x0F, _, 0x03, 0x03) => {
                println!("LD B V{:x}", x);
                let x = self.registers[x];

                self.memory[self.i] = x / 100;
                self.memory[(self.i + 1)] = (x % 100) / 10;
                self.memory[(self.i + 2)] = x % 10;
            },
            // Fx55 - LD [I], Vx
            // Store registers V0 through Vx in memory starting at location I.
            (0x0F, _, 0x05, 0x05) => {
                println!("LD [I] V{:x}", x);
                for i in 0..x + 1 {
                    let val = self.registers[i];
                    self.memory[self.i + i] = val;
                }
            },
            // Fx65 - LD Vx, [I]
            // Read registers V0 through Vx from memory starting at location I.
            (0x0F, _, 0x06, 0x05) => {
                println!("LD V{:x} [I]", x);
                for i in 0..x + 1 {
                    self.registers[i] = self.memory[self.i + i];
                }
            },
            _ => panic!("Unrecognized nibbles ({:x}, {:x}, {:x}, {:x})", nibbles.0, nibbles.1, nibbles.2, nibbles.3)
        }
        println!("---Registers---");
        println!("V0: {:x}, V1: {:x}, V2: {:x}, V3: {:x}, V4: {:x}, V5: {:x}, V6: {:x}, V7: {:x}",
                 self.registers[0], self.registers[1], self.registers[2], self.registers[3],
                 self.registers[4], self.registers[5], self.registers[6], self.registers[7]);
        println!("V8: {:x}, V9: {:x}, VA: {:x}, VB: {:x}, VC: {:x}, VD: {:x}, VE: {:x}, VF: {:x}",
                 self.registers[8], self.registers[9], self.registers[10], self.registers[11],
                 self.registers[12], self.registers[13], self.registers[14], self.registers[15]);
        println!("I: {:x}", self.i);
        println!("SP: {:x}", self.stack_pointer);
        println!("---Keys Pressed---");
        println!("{:?}", self.get_keys_pressed());
        println!();
    }
}