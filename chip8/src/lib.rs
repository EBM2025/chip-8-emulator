use rand::random;

const RAM_SIZE: usize = 4096; // chip8 implemented for 4KB RAM
const NUM_REG: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0, 
    0xF0, 0x10, 0xF0, 0x10, 0xF0, 
    0x90, 0x90, 0xF0, 0x10, 0x10, 
    0xF0, 0x80, 0xF0, 0x10, 0xF0, 
    0xF0, 0x80, 0xF0, 0x90, 0xF0, 
    0xF0, 0x10, 0x20, 0x40, 0x40, 
    0xF0, 0x90, 0xF0, 0x90, 0xF0, 
    0xF0, 0x90, 0xF0, 0x10, 0xF0, 
    0xF0, 0x90, 0xF0, 0x90, 0x90, 
    0xE0, 0x90, 0xE0, 0x90, 0xE0, 
    0xF0, 0x80, 0x80, 0x80, 0xF0, 
    0xE0, 0x90, 0x90, 0x90, 0xE0, 
    0xF0, 0x80, 0xF0, 0x80, 0xF0, 
    0xF0, 0x80, 0xF0, 0x80, 0x80 
];

pub const SCALER: u32 = 15;
pub const WIDTH: u32 = 64;
pub const HEIGHT: u32 = 32;
pub const W_WIDTH: u32 = (SCALER * WIDTH) as u32;
pub const W_HEIGHT: u32 = (SCALER * WIDTH) as u32;

pub struct Emulator {
    pc: u16, // index of current instruction
    ram: [u8; RAM_SIZE],
    screen: [bool; (WIDTH * HEIGHT) as usize], // black and white pixel screen array
    v_reg: [u8; NUM_REG],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8, // count down every CPU cycle and performs some action
    st: u8 // emits noise every CPU cycle
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; (WIDTH * HEIGHT) as usize],
            v_reg: [0; NUM_REG],
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

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val; // push to top of the stack
        self.sp += 1; // update the stack pointer
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1; // sp points to open slot
        self.stack[self.sp as usize]
    }

    pub fn reset(&mut self) { 
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; ((WIDTH * HEIGHT) as usize)];
        self.v_reg = [0; NUM_REG];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) { 
        
        let op = self.fetch();
        // decode
        // execute
        self.execute(op);
    }

    pub fn execute(&mut self, op: u16) {
        let d1 = (op & 0xF000) >> 12; // high half-byte
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = (op & 0x000F); //low half-byte
        
        match(d1, d2, d3, d4) { // match the two bytes to opcode
            (0, 0, 0, 0) => return, //noop - no operation - for timing and alignment purposes
            (0, 0, 0xE, 0) => { // clear the screen == opcode 0x00E0
                self.screen = [false; (WIDTH * HEIGHT) as usize];
            },
            (0, 0, 0xE, 0xE) => { //return from subroutine - push current instruction to the stack
                let rtn_add = self.pop();
                self.pc = rtn_add;
            },
            (1, _, _, _) => { // anything starting with 0x1 represents jumping to last 12 bits address
                let new_addr = op & 0xFFF; // get the last 12 bits
                self.pc = new_addr; // move pointer to new instruction
            },
            (2, _, _, _) => { // opposite of RFS - jump to subroutine 
                let rdi = op & 0xFFF; // get instruction
                self.push(self.pc);
                self.pc = rdi;
            },
            (3, _, _, _) => { // skip next
                let xx = (op & 0xFF) as u8;
                if self.v_reg[d2 as usize] == xx {
                    self.pc += 2; // skip opcode if register at d2 == 0x__xx
                }
            },
            (4, _, _, _) => { // skip next if v_reg[d2] != 0x__xx
                let xx = (op & 0xFF) as u8;
                if self.v_reg[d2 as usize] != xx {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => { // skip the operation if v_reg equals middle 2 dig
                if self.v_reg[d2 as usize] == self.v_reg[d3 as usize] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => { // set the v register to the second digit to the value given
                let xx = (op & 0xFF) as u8;
                self.v_reg[d2 as usize] = xx;
            },
            (7, _, _, _) => { // V[x] += nn
                let xx = (op & 0xFF) as u8;
                self.v_reg[d2 as usize] = self.v_reg[d2 as usize].wrapping_add(xx); // in event of overflow
            },
            (8, _, _, 0) => { // V[x] = V[y], where op = 0x8xy0
                self.v_reg[d2 as usize] = self.v_reg[d3 as usize];
            },
            (8, _, _, 1) | (8, _, _, 2) | (8, _, _, 3) => { // V[x] |= V[y], where op = 0x8xy1or2or3
                self.v_reg[d2 as usize] |= self.v_reg[d3 as usize];
            },
            (8, _, _, 4) => { // V[x] += V[y]
                let (n_vx, carry) = self.v_reg[d2 as usize].overflowing_add(self.v_reg[d3 as usize]);
                let flag = if carry { 1 } else { 0 };

                self.v_reg[d2 as usize] = n_vx; // set the X reg 
                self.v_reg[0xF] = flag; // set the overflow v reg flag
            },
            (8, _, _, 5) => { //  V[x] -= V[y]
                let (n_vx, carry) = self.v_reg[d2 as usize].overflowing_sub(self.v_reg[d3 as usize]);
                let flag = if carry { 1 } else { 0 };

                self.v_reg[d2 as usize] = n_vx;
                self.v_reg[0xF] = flag;
            },
            (8, _, _, 6) => { // V[x] >> 1, store removed bit in 0xF register
                let bit = (self.v_reg[d2 as usize] & 0x1);  // capture the flag bit
                self.v_reg[d2 as usize] = (self.v_reg[d2 as usize] >> 1);
                self.v_reg[0xF] = bit;
            },
            (8, _, _, 7) => { // V[y] -= V[x] - same as 0x8xy5 in opposite order
                let (n_vy, carry) = self.v_reg[d3 as usize].overflowing_sub(self.v_reg[d2 as usize]);
                let flag = if carry { 1 } else { 0 };

                self.v_reg[d3 as usize] = n_vy;
                self.v_reg[0xF] = flag;
            },
            (8, _, _, 0xE) => { // V[x] << 1, store removed bit in 0xF register
                let bit = ((self.v_reg[d2 as usize] as u8) & 0x10); // capture the upper bit
                self.v_reg[d2 as usize] = (self.v_reg[d2 as usize] << 1);
                self.v_reg[0xF] = bit;
            },
            (9, _, _, 0) => { // if V[x] != V[y], skip instruction
                if self.v_reg[d2 as usize] != self.v_reg[d3 as usize] {
                    self.pc += 2;
                }
            },
            (0xA, _, _, _) => { // 0xAyyy, assign I-register 0xyyy - address pointer to RAM
                let yyy = (op & 0xFFF);
                self.i_reg = yyy;
            },
            (0xB, _, _, _) => { // JMP V0 + yyy, where op = 0xByyy - move pc 
                let yyy = (op & 0xFFF);
                self.pc = (self.v_reg[0] as u16) + yyy;
            },
            (0xC, _, _, _) => { // rand() & yy - generate rand number and & with last two of op
                let yy = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[d2 as usize] = yy & rng;
            },
            (0xD, _, _, _) => { // draw sprite - second and third digit for (x,y) coord - 0xDxyn
                // x and y for x, y coord
                let x_val = self.v_reg[d2 as usize] as u16;
                let y_val = self.v_reg[d3 as usize] as u16;
                
                // n represents height in rows of the sprite
                let n = d4;
                let mut flag = false; // keep track of fliped pixels

                for i in 0..n { // loop through bytes (row)

                    let pixel_val = self.ram[(self.i_reg + i) as usize];

                    for j in 0..8 { // iterate through each byte (col)

                        let mask = 0b00000001 << j; // left shift each time

                        if ((mask & pixel_val) == 1) {

                            let new_x = (x_val + j) as usize % (WIDTH as usize);
                            let new_y = (y_val + i) as usize % (HEIGHT as usize);

                            let index = new_x + (WIDTH as usize) * new_y;

                            flag = flag | self.screen[index];
                            if (self.screen[index]) { self.screen[index] = false; } else { self.screen[index] = true; } 
                        }

                    }

                }

                if flag { self.v_reg[0xF] = 1; } else { self.v_reg[0xF] = 0; }
            },
            (0xE, _, 9, 0xE) => { // index stored at V[X] is pressed? - check keys
                let key = self.keys[self.v_reg[d2 as usize] as usize]; //
                if (key) { // is key pressed? if not skip
                    self.pc += 2;
                }
            }, 
            (0xE, _, 0xA, 1) => { // index stored at V[x] is not pressed? check keys
                let key = self.keys[self.v_reg[d2 as usize] as usize];
                if (!key) { // // is key pressed? if no skip instruction
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => { // store the delay timer in v_reg at index d2
                self.v_reg[d2 as usize] = self.dt;
            },
            (0xF, _, 0, 0xA) => { // pause and wait until the player presses a key
                let mut pressed = false;

                for i in 0..0xF { // loop endlessly until key press, then break
                    if self.keys[i]{
                        self.v_reg[d2 as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if (!pressed) { // try again
                    self.pc -= 2;
                }
            },
            (0xF, _, 1, 5) => { // reset delay timer, copy value from v-reg
                self.dt = self.v_reg[d2 as usize];
            },
            (0xF, _, 1, 8) => { // reset the sound timer, copy value from v-reg
                self.st = self.v_reg[d2 as usize];
            },
            (0xF, _, 1, 0xE) => { // I += V[d2]
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[d2 as usize] as u16);
            },
            (0xF, _, 2, 9) => { // fonts are stored at the beginning of RAM - I = font
                let font = self.v_reg[d2 as usize] as u16;
                self.i_reg = font * 5;
            },
            (0xF, _, 3, 3) => { // take V[d2] and convert from hex to decimal and store in RAM starting at i_reg
                let vx = self.v_reg[d2 as usize] as f32;
                
                // retrieve each place
                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                // store in RAM
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            (0xF, _, 5, 5) => { // populate the RAM with the same value v[d2] from 0 to d2 + 1
                let i_ptr = self.i_reg as usize;
                let end = (d2 + 1) as usize;
                for index in 0..end {
                    self.ram[i_ptr + index] = self.v_reg[index];
                }
            },
            (0xF, _, 6, 5) => { // opposite of the one above
                let i_ptr = self.i_reg as usize;
                let end = (d2 + 1) as usize;
                for index in 0..end {
                    self.v_reg[index] = self.ram[index + i_ptr];
                }
            }, 

            (_, _, _, _) => { unimplemented!("Unimplemented operation code: {}", op); }
        }

    }

    pub fn get_display(&self) -> &[bool] { // pass pointer of our display
        &self.screen
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) { // function that allows frontend to update keys with keypresses from the user
        self.keys[index] = pressed;
    }

    pub fn load(&mut self, game_code: &[u8]) { // load game code in RAM - begin at 0x200 - first 512 bytes of RAM are empty
        let end = (START_ADDR as usize) + game_code.len();
        self.ram[(START_ADDR as usize)..end].copy_from_slice(game_code);
    }


    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // beep
            }
            self.st -= 1;
        }
    }

    pub fn fetch(&mut self) -> u16 {
        // big endian - op code from 2 bytes
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (high_byte << 8) | low_byte;
        self.pc += 2; // move two bytes past
        op
    }
}



