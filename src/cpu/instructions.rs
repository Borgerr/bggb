pub enum Instruction {
    NOP,
}

impl Instruction {
    fn first_byte(bytes: u32) -> u8 {
        (bytes >> 24) as u8
    }
    fn second_byte(bytes: u32) -> u8 {
        ((bytes >> 16) & 0xff) as u8
    }
    fn third_byte(bytes: u32) -> u8 {
        ((bytes >> 8) & 0xff) as u8
    }
    fn fourth_byte(bytes: u32) -> u8 {
        (bytes & 0xff) as u8
    }

    // "x", "y", "z", "p", and "q" below are referencing the following document:
    // https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html#cb
    fn opcode_x(opcode: u8) -> u8 {
        opcode >> 6
    }
    fn opcode_y(opcode: u8) -> u8 {
        (opcode >> 3) & 0b0111
    }
    fn opcode_z(opcode: u8) -> u8 {
        opcode & 0b0111
    }
    fn opcode_p(opcode: u8) -> u8 {
        Self::opcode_y(opcode) >> 1
    }
    fn opcode_q(opcode: u8) -> bool {
        (Self::opcode_y(opcode) % 0b001) == 0b001
    }

    pub fn from_bytes(bytes: u32) -> Instruction {
        // check if first byte is a prefix byte
        match Self::first_byte(bytes) {
            0xcb => {}
            0xdd => {}
            0xed => {}
            0xfd => {}
            _ => {
                // first byte is not a prefix byte
            }
        };

        Instruction::NOP
    }
}
