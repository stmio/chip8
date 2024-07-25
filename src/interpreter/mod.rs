mod instruction;

use chip8_base::{Display, Interpreter, Keys, Pixel};
use instruction::Instruction;
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
    speed: Duration,
    display: [[Pixel; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
}

impl Interpreter for ChipState {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        let opcode = self.fetch();
        let instruction = Instruction::decode(opcode);

        log::debug!("Executing instruction {:?}", instruction);
        self.execute(instruction)
    }

    fn speed(&self) -> Duration {
        self.speed
    }

    fn buzzer_active(&self) -> bool {
        false
    }
}

impl ChipState {
    pub fn new(clock_freq: u32) -> Self {
        ChipState {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0x200,
            index: 0,
            pointer: 0,
            stack: [0; 16],
            speed: Duration::from_secs_f64(1_f64 / clock_freq as f64),
            display: [[Pixel::default(); 64]; 32],
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

    fn execute(&mut self, instruction: Instruction) -> Option<Display> {
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
            Instruction::Call(_) => todo!(),
            Instruction::Ske(_, _) => todo!(),
            Instruction::Skne(_, _) => todo!(),
            Instruction::Skre(_, _) => todo!(),
            Instruction::Setr(x, byte) => self.registers[x as usize] = byte,
            Instruction::Addr(x, byte) => {
                self.registers[x as usize] = self.registers[x as usize].wrapping_add(byte);
            }
            Instruction::Move(_, _) => todo!(),
            Instruction::Or(_, _) => todo!(),
            Instruction::And(x, y) => self.registers[x as usize] &= self.registers[y as usize],
            Instruction::Xor(_, _) => todo!(),
            Instruction::Add(_, _) => todo!(),
            Instruction::Sub(_, _) => todo!(),
            Instruction::Shr(_, _) => todo!(),
            Instruction::Ssub(_, _) => todo!(),
            Instruction::Shl(_, _) => todo!(),
            Instruction::Skrne(_, _) => todo!(),
            Instruction::Seti(addr) => self.index = addr,
            Instruction::Jmpr(_) => todo!(),
            Instruction::Rand(_, _) => todo!(),
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

                    for (j, bit) in row.iter().enumerate() {
                        let x = (self.registers[vx as usize] % 64) + j as u8;

                        if x >= 64 || y >= 32 {
                            // TODO: need to move out
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
            Instruction::Skp(_) => todo!(),
            Instruction::Sknp(_) => todo!(),
            Instruction::Moved(_) => todo!(),
            Instruction::Key(_) => todo!(),
            Instruction::Setrd(_) => todo!(),
            Instruction::Setrs(_) => todo!(),
            Instruction::Addi(_) => todo!(),
            Instruction::Ldfnt(_) => todo!(),
            Instruction::Bcd(_) => todo!(),
            Instruction::Store(_) => todo!(),
            Instruction::Load(_) => todo!(),
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
