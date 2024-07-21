use chip8_base::{Display, Interpreter, Keys, Pixel};
use std::time::Duration;

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
}

impl Interpreter for ChipState {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        None
    }

    fn speed(&self) -> Duration {
        self.speed
    }

    fn buzzer_active(&self) -> bool {
        false
    }
}
