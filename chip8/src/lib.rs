const RAM_SIZE: usize = 4096; // chip8 implemented for 4KB RAM
const NUM_REG: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Emu {
    pc: u16, // index of current instruction
    ram [u8; RAM_SIZE],
    screen: [bool; WIDTH * HEIGHT], // black and white pixel screen array
    v_reg: [u8; NUM_REG],
    i_reg: u16,
    stck_ptr: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8, // count down every CPU cycle and performs some action
    sound_timer: u8 // emits noise every CPU cycle
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; WIDTH * HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONSET);
        new_emu
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val; // push to top of the stack
        self.sp += 1; // update the stack pointer
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1; // sp points to open slot
        self.stack[self.sp as usize];
    }

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
                self.screen = [false; WIDTH * HEIGHT];
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

            (_, _, _, _) => unimplemented!("Unimplemented operation code: {}", op);
        }

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
        if self.pc + 2 > self.ram.len() {
            return Err("Passed length of RAM".into());
        }
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (high_byte << 8) | low_byte;
        self.pc += 2; // move two bytes past
        op
    }
}



