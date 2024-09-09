mod font;
mod instruction;

use chip8_base::{Display, Interpreter, Keys, Pixel};
use instruction::Instruction;
use rand::random;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};

// TODO: add log macros (task 2.3/3.8)

pub struct ChipState {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: u16,
    index: u16,
    pointer: u8,
    stack: [u16; 16],
    display: [[Pixel; 64]; 32],
    speed: Duration,
    ticker: Duration,
    delay_timer: u8,
    sound_timer: u8,
}

impl Interpreter for ChipState {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        let opcode = self.fetch();
        let instruction = Instruction::decode(opcode);

        // Handle timers
        self.ticker = self.ticker.saturating_sub(self.speed());
        if self.ticker == Duration::ZERO {
            self.delay_timer = self.delay_timer.saturating_sub(1);
            self.sound_timer = self.sound_timer.saturating_sub(1);
            self.ticker = Duration::from_nanos(16666667);
        }

        log::debug!("Executing instruction {:?}", instruction);
        self.execute(instruction, keys)
    }

    fn speed(&self) -> Duration {
        self.speed
    }

    fn buzzer_active(&self) -> bool {
        self.sound_timer != 0
    }
}

impl ChipState {
    pub fn new(clock_freq: u32) -> Self {
        let mut memory = [0; 4096];
        // Load font into memory (font is 80 bytes)
        memory[0x50..0xA0].copy_from_slice(&font::FONT);

        ChipState {
            memory,
            registers: [0; 16],
            pc: 0x200,
            index: 0,
            pointer: 0,
            stack: [0; 16],
            display: [[Pixel::default(); 64]; 32],
            speed: Duration::from_secs_f64(1_f64 / clock_freq as f64),
            ticker: Duration::from_nanos(16666667),
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn load(&mut self, rom: PathBuf) -> io::Result<()> {
        fs::read(rom).and_then(|bytes| {
            self.memory[0x200..0x200 + bytes.len()].copy_from_slice(&bytes);
            self.pc = 0x200;
            Ok(())
        })
    }

    fn fetch(&mut self) -> u16 {
        let instruction = u16::from_be_bytes([
            self.memory[self.pc as usize],
            self.memory[(self.pc + 1) as usize],
        ]);

        self.increment_pc();
        instruction
    }

    fn execute(&mut self, instruction: Instruction, keys: &Keys) -> Option<Display> {
        match instruction {
            Instruction::Nop => (),
            Instruction::Cls => {
                self.display = [[Pixel::default(); 64]; 32];
                return Some(self.display);
            }
            Instruction::Ret => {
                self.pc = self.stack[self.pointer as usize];
                self.pointer -= 1;
            }
            Instruction::Jmp(addr) => self.pc = addr,
            Instruction::Call(addr) => {
                self.pointer += 1;
                self.stack[self.pointer as usize] = self.pc;
                self.pc = addr;
            }
            Instruction::Ske(x, byte) => {
                if self.registers[x as usize] == byte {
                    self.increment_pc();
                }
            }
            Instruction::Skne(x, byte) => {
                if self.registers[x as usize] != byte {
                    self.increment_pc();
                }
            }
            Instruction::Skre(x, y) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.increment_pc();
                }
            }
            Instruction::Setr(x, byte) => self.registers[x as usize] = byte,
            Instruction::Addr(x, byte) => {
                self.registers[x as usize] = self.registers[x as usize].wrapping_add(byte);
            }
            Instruction::Move(x, y) => self.registers[x as usize] = self.registers[y as usize],
            Instruction::Or(x, y) => self.registers[x as usize] |= self.registers[y as usize],
            Instruction::And(x, y) => self.registers[x as usize] &= self.registers[y as usize],
            Instruction::Xor(x, y) => self.registers[x as usize] ^= self.registers[y as usize],
            Instruction::Add(x, y) => {
                let (value, carry) =
                    self.registers[x as usize].overflowing_add(self.registers[y as usize]);
                self.registers[x as usize] = value;
                self.registers[0xF] = if carry { 1 } else { 0 };
            }
            Instruction::Sub(x, y) => {
                let (value, borrow) =
                    self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
                self.registers[x as usize] = value;
                self.registers[0xF] = if borrow { 0 } else { 1 };
            }
            Instruction::Shr(x, y) => {
                self.registers[0xF] = self.registers[x as usize] & 0b1;
                self.registers[x as usize] >>= 1;
                log::trace!("The y value {} was ignored - not used in this version", y);
            }
            Instruction::Ssub(x, y) => {
                let (value, borrow) =
                    self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
                self.registers[x as usize] = value;
                self.registers[0xF] = if borrow { 0 } else { 1 };
            }
            Instruction::Shl(x, y) => {
                self.registers[0xF] = (self.registers[x as usize] & 0x80) >> 7;
                self.registers[x as usize] <<= 1;
                log::trace!("The y value {} was ignored - not used in this version", y);
            }
            Instruction::Skrne(x, y) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.increment_pc();
                }
            }
            Instruction::Seti(addr) => self.index = addr,
            Instruction::Jmpr(addr) => self.pc = (addr + self.registers[0] as u16) & 0x0FFF,
            Instruction::Rand(x, byte) => self.registers[x as usize] = random::<u8>() & byte,
            Instruction::Draw(vx, vy, n) => {
                self.registers[0xF] = 0;
                let n = n.min(15);

                let sprite: Vec<Vec<Pixel>> = self
                    .memory
                    .iter()
                    .skip(self.index as usize)
                    .take(n as usize)
                    .cloned()
                    .map(|byte| {
                        (0..u8::BITS)
                            .map(move |i| Pixel::try_from(byte >> i & 0b1).unwrap_or_default())
                            .rev()
                            .collect()
                    })
                    .collect();

                for (i, row) in sprite.iter().enumerate() {
                    let y = (self.registers[vy as usize] % 32) + i as u8;

                    if y >= 32 {
                        break;
                    }

                    for (j, bit) in row.iter().enumerate() {
                        let x = (self.registers[vx as usize] % 64) + j as u8;

                        if x >= 64 {
                            break;
                        }

                        let pixel = &mut self.display[y as usize][x as usize];
                        // Set VF if pixel was erased, then set pixel on display
                        if (*pixel & *bit).into() {
                            self.registers[0xF] = 1;
                        }
                        *pixel ^= *bit;
                    }
                }

                return Some(self.display);
            }
            Instruction::Skp(x) => {
                if keys[self.registers[x as usize] as usize] {
                    self.increment_pc();
                }
            }
            Instruction::Sknp(x) => {
                if !keys[self.registers[x as usize] as usize] {
                    self.increment_pc();
                }
            }
            Instruction::Moved(x) => self.registers[x as usize] = self.delay_timer,
            Instruction::Key(x) => {
                if keys.iter().all(|k| !k) {
                    self.pc -= 2;
                } else {
                    self.registers[x as usize] = keys.iter().position(|&key| key).unwrap() as u8;
                    log::debug!("Key {:?} was pressed", self.registers[x as usize]);
                }
            }
            Instruction::Setrd(x) => self.delay_timer = self.registers[x as usize],
            Instruction::Setrs(x) => self.sound_timer = self.registers[x as usize],
            Instruction::Addi(x) => {
                self.index += self.registers[x as usize] as u16;
                self.index &= 0x0FFF;
            }
            Instruction::Ldfnt(x) => self.index = 0x50 + (5 * self.registers[x as usize] as u16),
            Instruction::Bcd(x) => {
                let mem_slice = &mut self.memory[(self.index as usize)..(self.index as usize + 3)];

                mem_slice[0] = self.registers[x as usize] / 100;
                mem_slice[1] = self.registers[x as usize] % 100 / 10;
                mem_slice[2] = self.registers[x as usize] % 10;
            }
            Instruction::Store(x) => {
                for r in 0..=x as usize {
                    self.memory[self.index as usize + r] = self.registers[r];
                }
            }
            Instruction::Load(x) => {
                for r in 0..=x as usize {
                    self.registers[r] = self.memory[self.index as usize + r];
                }
            }
        };

        None
    }

    fn increment_pc(&mut self) {
        self.pc += 2;
        // Reset PC to 0 when memory address 4096 is reached.
        // This mask works as addr is 12-bit but PC is 16-bit.
        self.pc &= 0x0FFF;
    }
}
