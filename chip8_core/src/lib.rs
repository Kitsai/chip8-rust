use rand::random;

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




pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

pub struct Emu { //main object for the emulator backend
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_regs: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt:u8,
    st:u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu  = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            v_regs: [0; NUM_REGS],
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

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0;RAM_SIZE];
        self.screen = [false; SCREEN_HEIGHT*SCREEN_WIDTH];
        self.v_regs = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0;STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET)
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16{
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                //BEEP
            }
            self.st -= 1;
        }
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0,0,0,0) => return,
            //clear display
            (0,0,0xE,0) => {
                self.screen = [false; SCREEN_HEIGHT*SCREEN_WIDTH];
            },
            //ret
            (0,0,0xE,0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            //jump
            (0x1,_,_,_) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },
            //call
            (0x2,_,_,_) => {
                let nnn = op&0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }, 
            //skip if vx == nn
            (0x3,_,_,_) => {
                let nn = (op&0xFF) as u8;
                let x = digit2 as  usize;
                if self.v_regs[x] == nn {
                    self.pc += 2;
                }
            },   
            //skip if vx != nn
            (0x4,_,_,_) => {
                let nn = (op&0xFF) as u8;
                let x = digit2 as  usize;
                if self.v_regs[x] != nn {
                    self.pc += 2;
                }
            },
            //skip if vx == vy
            (0x5,_,_,0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_regs[x] == self.v_regs[y] {
                    self.pc += 2;
                }
            },
            //li
            (0x6,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op&0xFF) as u8;
                self.v_regs[x] = nn;
            },
            // vx += nn
            (0x7,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op&0xFF) as u8;
                self.v_regs[x] = self.v_regs[x].wrapping_add(nn); //lida com overflow
            },
            
            //mv   
            (0x8,_,_,0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] = self.v_regs[y];
            },

            //bitwise ops
            //###############################################################################

            // vx |= vy
            (0x8,_,_,0x1) => {
               let x = digit2 as usize;
               let y = digit3 as usize;
               self.v_regs[x] |= self.v_regs[y];  
            },
            // vx &= vy
            (0x8,_,_,0x2) => {
               let x = digit2 as usize;
               let y = digit3 as usize;
               self.v_regs[x] &= self.v_regs[y];  
            },
            // vx ^= vy
            (0x8,_,_,0x3) => {
               let x = digit2 as usize;
               let y = digit3 as usize;
               self.v_regs[x] ^= self.v_regs[y];  
            },
            //###############################################################################

            // vx += vy
            (0x8,_,_,0x4) => {
               let x = digit2 as usize;
               let y = digit3 as usize;

               let (new_vx, carry) = self.v_regs[x].overflowing_add(self.v_regs[y]);
               let new_vf = if carry {1} else {0};

               self.v_regs[x] = new_vx;
               self.v_regs[0xF] = new_vf;  
            },
            // vx -= vy
            (0x8,_,_,0x5) => {
               let x = digit2 as usize;
               let y = digit3 as usize;

               let (new_vx, carry) = self.v_regs[x].overflowing_sub(self.v_regs[y]);
               let new_vf = if carry {0} else {1};

               self.v_regs[x] = new_vx;
               self.v_regs[0xF] = new_vf;  
            },
            // vx >>= vy
            (0x8,_,_,0x6) => {
                let x = digit2 as usize;
                
                let lsb = self.v_regs[x] & 1;
                self.v_regs[x] >>= 1;
                self.v_regs[0xF] = lsb;
            },  
            //vx = vy - vx
            (0x8,_,_,0x7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                
                let (new_vx, carry) = self.v_regs[y].overflowing_sub(self.v_regs[x]);
                let new_vf = if carry {0} else {1};
                
                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf;  
            },
            // vx <<= vy
            (0x8,_,_,0xE) => {
                let x = digit2 as usize;
                
                let msb = (self.v_regs[x] >> 7) & 1;
                self.v_regs[x] <<= 1;
                self.v_regs[0xF] = msb;
            },
            //skip if vx != vy
            (0x9,_,_,0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_regs[x] != self.v_regs[y] {
                    self.pc += 2;
                }
            },
            // I = nnn
            (0xA,_,_,_) => {
                let nnn = op&0xFFF;
                self.i_reg = nnn;    
            },
            // jump to v0 + nnn
            (0xB,_,_,_) => {
                let nnn = op&0xFFF;
                self.pc = (self.v_regs[0] as u16) + nnn;    
            },
            // vx = rng & nn
            (0xC,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op&0xFF) as u8;
                let rng: u8 = random();
                self.v_regs[x] = rng & nn; 
            },
            //draw
            (0xD,_,_,_) => {
                //get cords
                let x_coord = self.v_regs[digit2 as usize] as u16;
                let y_coord = self.v_regs[digit3 as usize] as u16;
                // digit 4 is height
                let num_rows = digit4;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    //determine mem addr of row
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    //iterate over column(always 8 bits)
                    for x_line in 0..8 {
                        //use mask for current bit. if 1 flip
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            //wrap sprite araund screen
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            //get pixel i
                            let idx = x + SCREEN_WIDTH * y;
                            //check if the bit is going to be flipped
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;

                        }
                    }
                }
                //populate VF register
                if flipped {
                    self.v_regs[0xF] = 1;
                } else {
                    self.v_regs[0xF] = 0;
                }
            },
            // skip if key press
            (0xE,_,0x9,0xE) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // skip if key released
            (0xE,_,0xA,0x1) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            // vx = dt(Delay timer)
            (0xF,_,0x0,0x7) => {
                let x = digit2 as usize;
                self.v_regs[x] = self.dt;
            },
            //wait for key
            (0xF,_,0x0,0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_regs[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            },
            //dt = vx
            (0xF,_,0x1,0x5) => {
                let x = digit2 as usize;
                self.dt = self.v_regs[x];
            },
            //st = vx
            (0xF,_,0x1,0x8) => {
                let x = digit2 as usize;
                self.st = self.v_regs[x];
            },
            // I += vx
            (0xF,_,0x1,0xE) => {
               let x = digit2 as usize;
               let vx = self.v_regs[x] as u16;
               self.i_reg = self.i_reg.wrapping_add(vx); 
            },
            // I to font addr
            (0xF,_,0x2,0x9) => {
                let x = digit2 as usize;
                let c = self.v_regs[x] as u16;
                self.i_reg = c * 5;
            },

            (0xF,_,0x3,0x3) => {
               let x = digit2 as usize;
               let vx = self.v_regs[x] as f32;

               let hundreds = (vx / 100.0).floor() as u8;

               let tens = ((vx / 10.0) % 10.0).floor() as u8;

               let ones = (vx% 10.0) as u8;

               self.ram[self.i_reg as usize] = hundreds;
               self.ram[(self.i_reg + 1) as usize] = tens;
               self.ram[(self.i_reg + 2) as usize] = ones; 
            },
            //store v0 - vx
            (0xF,_,0x5,0x5) => {
               let x = digit2 as usize;
               let i = self.i_reg as usize;
               for idx in 0..=x {
                self.ram[i + idx] = self.v_regs[idx];
               } 
            },
            //load v0- vx
            (0xF,_,0x6,0x5) => {
               let x = digit2 as usize;
               let i = self.i_reg as usize;
               for idx in 0..=x {
                    self.v_regs[idx] = self.ram[i + idx];
               } 
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}",op)
        }
    }
    
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc+1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    // FRONTEND
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
}