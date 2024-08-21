mod interpreter;

use clap::Parser;
use env_logger;
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Cli::parse();

    let mut chip = interpreter::ChipState::new(args.freq);
    chip.load(PathBuf::from(&args.rom))?;

    chip8_base::run(chip);
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// A CHIP-8 ROM to load into the interpreter
    #[clap(validator = rom_exists)]
    rom: String,
    // Frequency to run the interpreter at
    #[clap(action, default_value_t = 700)]
    freq: u32,
}

fn rom_exists(f: &str) -> Result<(), &'static str> {
    let p = std::path::Path::new(f);
    if !p.is_file() {
        Err("File does not exist.")
    } else {
        Ok(())
    }
}
