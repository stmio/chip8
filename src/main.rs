mod interpreter;

use env_logger;
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut chip = interpreter::ChipState::new(700);
    chip.load(PathBuf::from("/home/sam/repos/chip8/roms/uwcs.ch8"))?;

    chip8_base::run(chip);
}
