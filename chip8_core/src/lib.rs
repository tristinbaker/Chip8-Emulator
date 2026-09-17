pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize= 16;

const START_ADDR: u16 = 0x200; // For C8, programs start at 0x200 since 0x000-0x200 are reserved

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
        Self {
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
        }
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
}


