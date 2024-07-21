mod interpreter;

fn main() {
    let chip = interpreter::ChipState::new(700);
    chip8_base::run(chip);
}
