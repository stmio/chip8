use chip8_base::{Interpreter, Pixel};

pub struct ChipState {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: u16,
    index: u16,
    pointer: u8,
    stack: [u16; 16],
    display: [[Pixel; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
}

impl ChipState {
    pub fn new() -> Self {
        ChipState {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0x200,
            index: 0,
            pointer: 0,
            stack: [0; 16],
            display: [[Pixel::default(); 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

impl Interpreter for ChipState {
    fn step(&mut self, keys: &chip8_base::Keys) -> Option<chip8_base::Display> {
        todo!()
    }

    fn speed(&self) -> std::time::Duration {
        todo!()
    }

    fn buzzer_active(&self) -> bool {
        todo!()
    }
}
