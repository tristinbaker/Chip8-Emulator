use rand::random;

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
            v_reg: [0; NUM_REGS],
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

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
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
            if self.st == 1 {
                // Beep
            }

            self.st -= 1;
        }
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
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },

            // 00EE:  RET
            // Returns from subroutine. Sets the return address as the last place the pc was before
            // entering the subrouting, then sets the pc to that location, returning from the
            // subroutine.
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },

            // 1NNN:  JMP NNN
            // Sets the pc to the location specified in the opcode, jumping to that location in the
            // program.
            // Ex: 0x122A
            // 0x122A & 0xFFF -> 0x22A
            // pc = 0x22A.
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },

            // 2NNN:  CALL NNN
            // The opposite of RET, basically. Sets the PC to NNN, jumping to that location in the
            // program. The difference between CALL and JMP is that we store where we were before
            // JMPing to the stack.
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },

            // 3XNN:  SKIP VX == NN
            // First of a few conditional opcodes. Essentially building if/else statements with the
            // next few opcodes. This one, we check if V register X is equal to the byte at NN.
            // Ex: 32A2
            // nn = (32A2 & 0xFF) -> nn = A2
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },

            // 4XNN:  SKIP VX != NN
            // Ex: 42A2
            // nn = (42A2 & 0xFF) -> nn = A2
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xff) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },

            // 5XY0:  SKIP VX == VY
            // Ex: 52A0
            // x = 2, Y = A
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },

            // 6XNN:  VX = NN
            // Ex: 62A2
            // nn = (62A2 & 0xFF) -> nn = A2
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },

            // 7XNN:  VX += NN
            // Ex: 72A2
            // x = 2
            // nn = (72A2 & 0xFF) -> nn = A2
            // wrapping_add is needed because we need to prevent overflow
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },

            // 8XY0:  VX = VY
            // Ex: 82A0
            // x = 2
            // y = A
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },

            // 8XY1:  VX |= VY
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y]
            },

            // 8XY2:  VX &= VY
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y]
            },

            // 8XY3:  VX ^= VY
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y]
            },

            // 8XY4:  VX += VY
            // First opcode that has to deal with the carry flag (VF). This is done by tracking
            // whether adding VY to VX causes an overflow, and if so, VF is set to 1 if it
            // overflowed, 0 otherwise.
            // Ex: 82A4, V2 = 200, VA = 100, VF = 0
            // x = 2
            // y = A
            // new_vx = 44, carry = true
            // new_vf = 1 because carry = true
            // V2 = 44, VA = 100, VF = 1
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = carry as u8;

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // 8XY5:  VX -= VY
            // This functions the same as 8XY4, but in reverse. If the operation underflows, we set
            // the carry flag to 0 rather than 1.
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = !borrow as u8;

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // 8XY6:  VX >>= 1
            // Shifts VX right by one value, with the bit that drops off being stored in the VF register.
            // Rust doesn't have a way to catch a dropped bit so it has to be kept track of manually
            // Ex: 82A6, V2 = 0x07, VF = 0x0
            // x = 2
            // lsb = 0x07 & 1 -> 0000 0111 & 0000 0001 -> 0000 0001 -> 1
            // self.v_reg[x] >>= 1 -> 0x07 >> 1 -> 0000 0111 >> 1 -> 0000 0011 -> 3 (0x3)
            // self.v_reg[0xF] = 1
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },

            // 8XY7:  VX = VY - VX
            // Functions very similarly to 8XY5, but we're subtracting VX from VY instead of VY from
            // VX. Again, carry flag is set to 0 if subtracting underflows.
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = !borrow as u8; 

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // 8XYE:  VX <<= 1
            // Shifts VX left by one value, with the bit that drops off being stored in the VF
            // register, much like 8XY6, but in the other direction. msb is Most Significant Bit
            // Ex: 82AE, V2 = 0x87, VF = 0x0
            // x = 2
            // msb = (0x87 >> 7) & 1 -> (1000 0111 >> 7) & 1 -> 0000 0001 & 1 -> 1
            // self.v_reg[x] <<= 1 -> 0x87 << 1 -> 1000 0111 << 1 -> 0000 1110 -> 14 (0xE)
            // self.v_reg[0xF] = 1
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },

            // 9XY0:  SKIP VX != VY
            // Simply skips ahead one instruction if VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },

            // ANNN:  I = NNN
            // Sets the I register to NNN, simply does what it says on the tin.
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            
            // BNNN:  JMP V0 + NNN
            // Moves the PC to the location at whatever is at V0 plus the value in NNN. This varies
            // from JMP NNN since it lets ROMs use V0 as an index into a jump table.
            // Ex: BA2A, V0 = 0x23
            // nnn = 0xA2A
            // self.pc = (0x23 as u16) + 0xA2A -> 0x0023 + 0x0A2A -> 0x0A4D
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            
            // CXNN: VX = rand() & NN
            // This gets a random value and ANDs it with NN, and sets the result to the VX register.
            // Ex: C2A2
            // x = 2
            // nn = 0xA2
            // rng = 0x31
            // self.v_reg[x] = 0x31 & 0xA2 -> 0011 0001 & 1010 0010 -> 0010 0000 -> 32 (0x20)
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            
            // DXYN:  DRAW
            // The most complex opcode for Chip-8. To start with, C8 draws by sprite rather than
            // individual pixels. These sprites are stored in memory that are then copied to the
            // screen at a specified (x,y). The XY in the opcode gives us the V registers where we
            // the (x,y) coords are located at. Sprites in C8 are always 8 pixels wide, but are
            // variable in height from 1-16 pixels. This is where the N part of the opcode comes
            // into play. Sprites are drawn row by row beginning at the address stored in the I
            // register. So if N = 3, we start drawing the first row's data from *I, then *I + 1,
            // and finally *I + 2. This is why sprites are 8 pixels wide, each row is assigned a
            // byte, one bit for each pixel (black or white). The final quirk is that if any pixel is
            // flipped from on to off, VF is set. If not, it's cleared.
            // Ex: D122 with V1 = 2, V2 = 1, I = 0x300, RAM[0x300] = 0xC3, RAM[0x301] = 0x81
            // digit2 = 1, digit3 = 2, digit4 = 2
            // x_coord = V1 = 2, y_coord = V2 = 1, num_rows = 2
            //
            // Row 0 (y_line=0): read RAM[0x300] = 0xC3 = 1100 0011
            //   mask 1000_0000 >> 0 = 1000 0000 -> bit set → draw at (2,1)
            //   mask 1000_0000 >> 1 = 0100 0000 -> bit set → draw at (3,1)
            //   mask 1000_0000 >> 2 = 0010 0000 -> bit clear, skip
            //   mask 1000_0000 >> 3 = 0001 0000 -> bit clear, skip
            //   mask 1000_0000 >> 4 = 0000 1000 -> bit clear, skip
            //   mask 1000_0000 >> 5 = 0000 0100 -> bit clear, skip
            //   mask 1000_0000 >> 6 = 0000 0010 -> bit set → draw at (8,1)
            //   mask 1000_0000 >> 7 = 0000 0001 -> bit set → draw at (9,1)
            //
            // Row 1 (y_line=1): read RAM[0x301] = 0x81 = 1000 0001
            //   only bits 0 and 7 set -> draw at (2,2) and (9,2)
            //
            // Screen after (assuming empty before):
            //   (2,1) (3,1)             (8,1) (9,1)
            //   (2,2)                         (9,2)
            //
            // No pixels were toggled from on→off, so VF = 0.
            (0xD, _, _, _) => {
                // Get the (x,y) coords for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                // Get the height of sprite in rows
                let num_rows = digit4;

                // Keep track of pixels flipping
                let mut flipped = false;

                // Iterate over each row of the sprite
                for y_line in 0..num_rows {
                    // Determine which memory address the row's data is stored at
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // Iterate over each column in the row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get the pixel's index for the 1D screen array
                            let idx = x + SCREEN_WIDTH * y;

                            // Check if we're about to flip the pixel and set flipped
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            
            // EX9E:  SKIP KEY PRESS
            // Checks if the index stored in VX is pressed, and if so, skips the next instruction
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            
            // EXA1:  SKIP KEY RELEASE
            // Essentially does the opposite as the above instruction (key released)
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            
            // FX07:  VX = DT
            // Stores current value of DT to VX so that we can see what the DT is for game timing
            // purposes since DT decrements every frame rather than every CPU cycle.
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            
            // FX0A:  WAIT KEY
            // This is somewhat similar to SKIP KEY PRESS and SKIP KEY RELEASE, but not exactly.
            // This is a blocking instruction, which means the entire game pauses and waits for as
            // long as it needs to until a key is pressed. It will loop endlessly until something in
            // the keys array is true. Once true, it is stored in VX. If multiple keys are pressed,
            // it takes the lowest indexed key.
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    // Redo the opcode
                    self.pc -= 2;
                }
            },
            
            // FX15:  DT = VX
            // The opposite of FX07. The reason we'd want to do this is to set the DT to a value,
            // which is how it's able to have a nonzero starting point to decrement from in the
            // first place.
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            
            // FX18:  ST = VX
            // Exactly the same as above but for ST
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            
            // FX1E:  I += VX
            // Adds the current value of VX to the I register. In case of overflow, we should roll
            // back to 0.
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16)
            },
            
            // FX29:  I = FONT
            // Gets the RAM address of a sprite and stores it to the I register. Because we set
            // the fonts array to the beginning of RAM, and each font sprite takes up 5 bytes, we
            // can find their RAM address by multiplying their value by 5. This is not enforced by
            // the opcode, but rather a convention set when initializing RAM.
            // Address    Sprite       Digit
            // 0x00–0x04  font[0..5]   "0"
            // 0x05–0x09  font[5..10]  "1"
            // 0x0A–0x0E  font[10..15] "2"
            // ...
            // 0x4B–0x4F  font[75..80] "F"
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            
            // FX33:  BCD VX
            // Bit confusing. Essentially, every digit has a corresponding binary-coded digit. This
            // instruction converts a single byte number to a three-byte representation of that digit, to
            // make it easier for users to see numbers we're more familiar with. For example, in
            // hexidecimal, 100 is 0x64. 0x64 is easily read by a computer, but not a human.
            // Therefore, a BCD of 100 would be stored as "1", "0", "0", and when we want to show
            // that value, we can just show the BCD of the value. These values, once calculated, are
            // written to RAM starting at the location pointed at by the I register.
            // Ex: F233, VX = 112
            // x = 2
            // vx = 112
            //
            // hundreds = 112 / 100 -> 1
            // tens = (112 / 10) % 10 -> (11) % 10 -> 1
            // ones = 112 % 10 -> 2
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];

                let hundreds = vx / 100;
                let tens = (vx / 10) % 10;
                let ones = vx % 10;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            
            // FX55:  STO V0 through VX
            // This stores values of V0 through VX (inclusive) into RAM, starting at the location
            // from the I register.
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            },
            
            // FX65:  LD V0 - VX
            // The opposite of the above, loads the values from RAM starting at the address at the I
            // register into V0 through VX (inclusive)
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            },
            
            // the fallback state in case we reach an opcode that is unimplemented
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:04X}", op),
        }
    }
}


