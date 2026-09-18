pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize= 16;

const START_ADDR: u16 = 0x200; // For C8, programs start at 0x200 since 0x000-0x200 are reserved

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emu {
    pc: u16, // program counter, keeps the index of the current instruction
    ram: [u8; RAM_SIZE], // RAM, 4096 bytes
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // 64x32 array of 0/1
    v_reg: [u8; NUM_REGS], // general registers
    i_reg: u16, // I register - used for indexing into RAM for reads/writes
    sp: u16, // stack pointer
    stack: [u16; STACK_SIZE], // stack (LIFO)
    keys: [bool; NUM_KEYS], // input array
    dt: u8, // delay timer
    st: u8, // sound timer
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0, NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    // pushes from stack, increments stack pointer
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    
    // pops off stack, decrements stack pointer
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // resets emulator to default state (obviously)
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        // Fetch the opcode
        let op = self.fetch();

        // Decode the opcode
        // Execute the opcode
        // Done in one function
        self.execute(op);
    }

    // Fetches the opcode from RAM so it can be decoded and executed
    // All opcodes in Chip-8 are 2 bytes by definition
    // For example: 
    // Address: 0x200 0x201 0x202 0x203 0x204 0x205
    // Bytes:   0xA2  0x2A  0x60  0x0C  0xD0  0x11
    // Higher Byte: 0x00A2
    // Lower Byte:  0x002A
    // Higher Byte << 8: 0xA200
    // 0xA200 | 0x002A: 0xA22A -> Full Opcode
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    // Decrements both timers down once every frame, instead of once every CPU cycle
    // If st > 0, the emulator emits a beep
    // TODO: Beep when st > 0
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            // Beep
        }
         self.st -= 1;
    }

    // Decodes and Executes the opcode, the main meat of the emulator
    // Using our example from above, op = 0xA22A
    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12; // (0xA22A & 0xF000) -> 0xA000, 0xA000 >> 12 -> 0xA
        let digit2 = (op & 0x0F00) >> 8;  // (0xA22A & 0x0F00) -> 0x0200, 0x0200 >> 8 ->  0x2
        let digit3 = (op & 0x00F0) >> 4;  // (0xA22A & 0x00F0) -> 0x0020, 0x0020 >> 4 ->  0x2
        let digit4 = op & 0x000F;         // (0xA22A & 0x000F) -> 0x000A, 0x000A >> 0 ->  0xA

        match (digit1, digit2, digit3, digit4) {
            // 0000:  NOP
            (0, 0, 0, 0) => return,

            // 00E0:  CLS
            (0, 0, 0xE, 0) => {
            }

            // 00EE:  RET
            (0, 0, 0xE, 0xE) => {
            }

            // 1NNN:  JMP NNN
            (1, _, _, _) => {
            }

            // 2NNN:  CALL NNN
            (2, _, _, _) => {
            }

            // 3XNN:  SKIP VX == NN
            (3, _, _, _) => {
            }

            // 4XNN:  SKIP VX != NN
            (4, _, _, _) => {
            }

            // 5XY0:  SKIP VX == VY
            (5, _, _, 0) => {
            }

            // 6XNN:  VX = NN
            (6, _, _, _) => {
            }

            // 7XNN:  VX += NN
            (7, _, _, _) => {
            }

            // 8XY0:  VX = VY
            (8, _, _, 0) => {
            }

            // 8XY1:  VX |= VY
            (8, _, _, 1) => {
            }

            // 8XY2:  VX &= VY
            (8, _, _, 2) => {
            }

            // 8XY3:  VX ^= VY
            (8, _, _, 3) => {
            }

            // 8XY4:  VX += VY
            (8, _, _, 4) => {
            }

            // 8XY5:  VX -= VY
            (8, _, _, 5) => {
            }

            // 8XY6:  VX >>= 1
            (8, _, _, 6) => {
            }

            // 8XY7:  VX = VY - VX
            (8, _, _, 7) => {
            }

            // 8XYE:  VX <<= 1
            (8, _, _, 0xE) => {
            }

            // 9XY0:  SKIP VX != VY
            (9, _, _, 0) => {
            }

            // ANNN:  I = NNN
            (0xA, _, _, _) => {
            }
            
            // BNNN:  JMP V0 + NNN
            (0xB, _, _, _) => {
            }
            
            // CXNN:  VX = rand() & NN
            (0xC, _, _, _) => {
            }
            
            // DXYN:  DRAW
            (0xD, _, _, _) => {
            }
            
            // EX9E:  SKIP KEY PRESS
            (0xE, _, 9, 0xE) => {
            }
            
            // EXA1:  SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => {
            }
            
            // FX07:  VX = DT
            (0xF, _, 0, 7) => {
            }
            
            // FX0A:  WAIT KEY
            (0xF, _, 0, 0xA) => {
            }
            
            // FX15:  DT = VX
            (0xF, _, 1, 5) => {
            }
            
            // FX18:  ST = VX
            (0xF, _, 1, 8) => {
            }
            
            // FX1E:  I += VX
            (0xF, _, 1, 0xE) => {
            }
            
            // FX29:  I = FONT
            (0xF, _, 2, 9) => {
            }
            
            // FX33:  BCD
            (0xF, _, 3, 3) => {
            }
            
            // FX55:  STO V0 - VX
            (0xF, _, 5, 5) => {
            }
            
            // FX65:  LD V0 - VX
            (0xF, _, 6, 5) => {
            }
            
            // the fallback state in case we reach an opcode that is unimplemented
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}


