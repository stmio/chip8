type Addr = u16;
type Reg = u8;

#[derive(Debug)]
pub enum Instruction {
    Nop,                // 0nnn (SYS addr) - Ignored by modern interpreters
    Cls,                // 00E0 (CLS) - Clears the display
    Ret,                // 00EE (RET) - Returns from the subroutine (PC set to top of stack)
    Jmp(Addr),          // 1nnn (JP addr) - Jump to location (Sets PC to nnn/addr)
    Call(Addr),         // 2nnn (CALL addr) - Call subroutine at nnn/addr
    Ske(Reg, u8),       // 3xkk (SE Vx, byte) - Skip next instruction if Vx == kk
    Skne(Reg, u8),      // 4xkk (SNE Vx, byte) - Skip next instruction if Vx != kk
    Skre(Reg, Reg),     // 5xy0 (SE Vx, Vy) - Skip next instruction if Vx == Vy
    Setr(Reg, u8),      // 6xkk (LD Vx, byte) - Set register Vx to kk
    Addr(Reg, u8),      // 7xkk (ADD Vx, byte) - Set Vx = Vx + kk
    Move(Reg, Reg),     // 8xy0 (LD Vx, Vy) - Stores the value of Vy in Vx
    Or(Reg, Reg),       // 8xy1 (OR Vx, Vy) - Set Vx = Vx OR Vy
    And(Reg, Reg),      // 8xy2 (AND Vx, Vy) - Set Vx = Vx AND Vy
    Xor(Reg, Reg),      // 8xy3 (XOR Vx, Vy) - Set Vx = Vx XOR Vy
    Add(Reg, Reg),      // 8xy4 (ADD Vx, Vy) - Set Vx = Vx + Vy (and VF = carry)
    Sub(Reg, Reg),      // 8xy5 (SUB Vx, Vy) - Set Vx = Vx - Vy (if Vx > Vy, VF = 1)
    Shr(Reg, Reg),      // 8xy6 (SHR Vx, Vy) - Set Vx = Vy >> 1
    Ssub(Reg, Reg),     // 8xy7 (SUBN Vx, Vy) - Set Vx = Vy - Vx (if Vy > Vx, VF = 1)
    Shl(Reg, Reg),      // 8xyE (SHL Vx, Vy) - Set Vx = Vy << 1
    Skrne(Reg, Reg),    // 9xy0 (SNE Vx, Vy) - Skip next instruction if Vx != Vy
    Seti(Addr),         // Annn (LD I, addr) - Set index register to nnn/addr
    Jmpr(Addr),         // Bnnn (JP V0, addr) - Jump to location nnn/addr + V0
    Rand(Reg, u8),      // Cxkk (RND Vx, byte) - Set Vx = random byte AND kk
    Draw(Reg, Reg, u8), // Dxyn (DRW Vx, Vy, nibble) - Display n-byte sprite starting at location I at co-ord (Vx, Vy) (VF = collision)
    Skp(Reg),           // Ex9E (SKP Vx) - Skip next instruction if key with value of Vx down
    Sknp(Reg),          // ExA1 (SKNP Vx) - Skip next instruction if key with value of Vx not down
    Moved(Reg),         // Fx07 (LD Vx, DT) - Set Vx to delay timer
    Key(Reg),           // Fx0A (LD Vx, K) - Wait for a key press, store key value in Vx
    Setrd(Reg),         // Fx15 (LD DT, Vx) - Set delay timer to Vx
    Setrs(Reg),         // Fx18 (LD ST, Vx) - Set sound timer to Vx
    Addi(Reg),          // Fx1E (ADD I, Vx) - Add Vx to value of index register
    Ldfnt(Reg),         // Fx29 (LD F, Vx) - Set index register to location of sprite for digit Vx
    Bcd(Reg),           // Fx33 (LD B, Vx) - Store BCD representation of Vx in locations I..=I+2
    Store(Reg),         // Fx55 (LD [I], Vx) - Store regs V0 to Vx in memory starting at location I
    Load(Reg),          // Fx65 (LD Vx, [I]) - Read regs V0 to Vx from memory starting at location I
}

fn nibbles(n: u16) -> (u8, u8, u8, u8) {
    let n3 = (n >> 12) as u8;
    let n2 = ((n >> 8) & 0b1111) as u8;
    let n1 = ((n >> 4) & 0b1111) as u8;
    let n0 = (n & 0b1111) as u8;
    (n3, n2, n1, n0)
}

use Instruction::*;
impl Instruction {
    pub fn decode(opcode: u16) -> Self {
        let addr = opcode & 0x0fff;
        let byte = (opcode & 0x00ff) as u8;

        match nibbles(opcode) {
            (0, 0, 0xE, 0xE) => Ret,
            (0, 0, 0xE, 0) => Cls,
            (0, _, _, _) => Nop,
            (1, _, _, _) => Jmp(addr),
            (2, _, _, _) => Call(addr),
            (3, x, _, _) => Ske(x, byte),
            (4, x, _, _) => Skne(x, byte),
            (5, x, y, 0) => Skre(x, y),
            (6, x, _, _) => Setr(x, byte),
            (7, x, _, _) => Addr(x, byte),
            (8, x, y, 0) => Move(x, y),
            (8, x, y, 1) => Or(x, y),
            (8, x, y, 2) => And(x, y),
            (8, x, y, 3) => Xor(x, y),
            (8, x, y, 4) => Add(x, y),
            (8, x, y, 5) => Sub(x, y),
            (8, x, y, 6) => Shr(x, y),
            (8, x, y, 7) => Ssub(x, y),
            (8, x, y, 0xE) => Shl(x, y),
            (9, x, y, 0) => Skrne(x, y),
            (0xA, _, _, _) => Seti(addr),
            (0xB, _, _, _) => Jmpr(addr),
            (0xC, x, _, _) => Rand(x, byte),
            (0xD, x, y, n) => Draw(x, y, n),
            (0xE, x, 9, 0xE) => Skp(x),
            (0xE, x, 0xA, 1) => Sknp(x),
            (0xF, x, 0, 7) => Moved(x),
            (0xF, x, 0, 0xA) => Key(x),
            (0xF, x, 1, 5) => Setrd(x),
            (0xF, x, 1, 8) => Setrs(x),
            (0xF, x, 1, 0xE) => Addi(x),
            (0xF, x, 2, 9) => Ldfnt(x),
            (0xF, x, 3, 3) => Bcd(x),
            (0xF, x, 5, 5) => Store(x),
            (0xF, x, 6, 5) => Load(x),
            _ => panic!("Unsupported instruction found: {:#06X}", opcode),
        }
    }
}
