pub struct CPU {
    //16 registers means hexadecimal number (0 to F) can address them
    pub register: [u8; 16],
    pub position_in_memory: usize,
    //the emulator has 4kb of memory, the first 512 bytes are reserved for thr system
    pub memory: [u8; 4096],
}

impl CPU {
    pub fn read_opcode(&self) -> u16 {
        let p = self.position_in_memory;
        let op_byte1 = self.memory[p] as u16;
        let op_byte2 = self.memory[p + 1] as u16;

        //To create a u16 opcode, we combine two values from memory with the logical OR operation. They need to be cast
        //as u16 to start with, otherwise the left-shift will set all of the bits to 0.
        op_byte1 << 8 | op_byte2
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            //increment position in memory to next instruction
            self.position_in_memory += 2;

            //extract high and low nibbles from byte
            //filter first bit by AND 0XF000 and move bits to lowest significant place
            let c = ((opcode & 0xF000) >> 12) as u8;
            //filter second bit by 0X0F00 and move bits to owest significant place
            let x = ((opcode & 0x0F00) >> 8) as u8;
            //filter third bit by 0X00F0 and move bits to owest significant place
            let y = ((opcode & 0x00F0) >> 4) as u8;
            //filter fourth bit 0X000F and move bits to owest significant place
            let d = ((opcode & 0x000F) >> 0) as u8;

            println!("nibbles c-{:?} x-{:?} y-{:?} d-{:?} ", c, x, y, d);

            match (c, x, y, d) {
                //terminate when 0,0,0,0 is encountered
                (0, 0, 0, 0) => {
                    return;
                }
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }

    pub fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.register[x as usize];
        let arg2 = self.register[y as usize];

        //Within the CHIP-8, the last register is used as a carry flag. When set, this flag indicates that
        //an operation has overflowed the u8 size of a register
        //overflow_detected is true if overflow happens
        let (val, overflow_detected) = arg1.overflowing_add(arg2);
        self.register[x as usize] = val;

        if overflow_detected {
            self.register[0xF] = 1;
        } else {
            self.register[0xF] = 0;
        }
    }
}
