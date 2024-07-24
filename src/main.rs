mod interpreter;

use env_logger;

fn main() {
    env_logger::init();

    let chip = interpreter::ChipState::new(700);
    chip8_base::run(chip);
}
