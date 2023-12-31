use std::os::linux::raw;

pub enum Register {
    AF,
    BC,
    DE,
    HL,
    SP,

    A,
    B,
    C,
    D,
    E,
    H,
    L,

    HLplus,
    HLminus,
}
impl Register {
    pub fn r_lookup(val: u8) -> Register {
        // val is assumed to be less than 8
        match val {
            0 => Register::B,
            1 => Register::C,
            2 => Register::D,
            3 => Register::E,
            4 => Register::H,
            5 => Register::L,
            6 => Register::HL,
            7 | _ => Register::A,
        }
    }
    pub fn rp_lookup(val: u8) -> Register {
        // val is assumed to be less than 4
        match val {
            0 => Register::BC,
            1 => Register::DE,
            2 => Register::HL,
            3 | _ => Register::SP,
        }
    }
    pub fn rp2_lookup(val: u8) -> Register {
        // val is assumed to be less than 4
        match val {
            0 => Register::BC,
            1 => Register::DE,
            2 => Register::HL,
            3 | _ => Register::AF,
        }
    }
}
pub enum Flag {
    NZ,
    Z,
    NC,
    C,
}
impl Flag {
    pub fn cc_lookup(val: u8) -> Flag {
        // val is assumed to be less than 4
        match val {
            0 => Flag::NZ,
            1 => Flag::Z,
            2 => Flag::NC,
            3 | _ => Flag::C,
        }
    }
}

pub enum Instruction {
    NOP,
    STOP,
    HALT,
    ILLEGAL,

    Load16 { r: Register, nn: u16 },
    Load8 { r: Register, n: u8 },
    LoadFF00Plus { r: Register, n: u8 },
    LoadReg { r1: Register, r2: Register },

    Jump { nn: u16 },
    JumpConditional { f: Flag, nn: u16 },

    RLC { r: Register },
    RRC { r: Register },
    RL { r: Register },
    RR { r: Register },
    SLA { r: Register },
    SRA { r: Register },
    SWAP { r: Register },
    SRL { r: Register },
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
    fn third_and_fourth_bytes(bytes: u32) -> u16 {
        (bytes & 0xffff) as u16
    }
    fn second_and_third_bytes(bytes: u32) -> u16 {
        ((bytes >> 8) & 0xffff) as u16
    }

    // "x", "y", "z", "p", and "q" below are referencing the following document:
    // https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
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
        // in this function there's a lot of things like
        // using a `| _` after the last match case
        // this is just to appease the rust gods
        // there shouldn't be any other possible value beyond those, but this is all u8

        // check if first byte is a prefix byte
        if Self::first_byte(bytes) == 0xcb {
            let y = Self::opcode_y(Self::second_byte(bytes));
            let z = Self::opcode_z(Self::second_byte(bytes));

            match Self::opcode_x(Self::second_byte(bytes)) {
                0 => {
                    // rot[y] r[z]
                    match y {
                        0 => Instruction::RLC {
                            r: Register::r_lookup(z),
                        },
                        1 => Instruction::RRC {
                            r: Register::r_lookup(z),
                        },
                        2 => Instruction::RL {
                            r: Register::r_lookup(z),
                        },
                        3 => Instruction::RR {
                            r: Register::r_lookup(z),
                        },
                        4 => Instruction::SLA {
                            r: Register::r_lookup(z),
                        },
                        5 => Instruction::SRA {
                            r: Register::r_lookup(z),
                        },
                        6 => Instruction::SWAP {
                            r: Register::r_lookup(z),
                        },
                        7 | _ => Instruction::SRL {
                            r: Register::r_lookup(z),
                        },
                    }
                }
                1 => {
                    // BIT y, r[z]
                }
                2 => {
                    // RES y, r[z]
                }
                3 | _ => {
                    // SET y, r[z]
                }
            };
            Instruction::NOP
        } else {
            // first byte is not a prefix byte
            match Self::opcode_x(Self::first_byte(bytes)) {
                0 => Self::prefixless_x0(bytes),
                1 => Self::prefixless_x1(bytes),
                2 => Self::prefixless_x2(bytes),
                3 | _ => Self::prefixless_x3(bytes),
            }
        }
    }

    fn prefixless_x0(bytes: u32) -> Instruction {
        let y = Self::opcode_y(Self::first_byte(bytes));
        let q = Self::opcode_q(Self::first_byte(bytes));
        let p = Self::opcode_p(Self::first_byte(bytes));

        match Self::opcode_z(Self::first_byte(bytes)) {
            0 => {
                match y {
                    0 => {
                        // NOP
                    }
                    1 => {
                        // LD (nn), SP
                    }
                    2 => {
                        // STOP
                    }
                    3 => {
                        // JR d
                    }
                    4 | 5 | 6 | 7 | _ => {
                        // JR cc[y-4], d
                    }
                }
            }
            1 => {
                if q {
                    // ADD HL, rp[p]
                } else {
                    // LD rp[p], nn
                }
            }
            2 => {
                if q {
                    match p {
                        0 => {
                            // LD A, (BC)
                        }
                        1 => {
                            // LD A, (DE)
                        }
                        2 => {
                            // LD A, (HL+)
                        }
                        3 | _ => {
                            // LD A, (HL-)
                        }
                    }
                } else {
                    match p {
                        0 => {
                            // LD (BC), A
                        }
                        1 => {
                            // LD (DE), A
                        }
                        2 => {
                            // LD (HL+), A
                        }
                        3 | _ => {
                            // LD (HL-), A
                        }
                    }
                }
            }
            3 => {
                if q {
                    // DEC rp[p]
                } else {
                    // INC rp[p]
                }
            }
            4 => {
                // INC r[y]
            }
            5 => {
                // DEC r[y]
            }
            6 => {
                // LD r[y], n
            }
            7 | _ => {
                match y {
                    0 => {
                        // RLCA
                    }
                    1 => {
                        // RRCA
                    }
                    2 => {
                        // RLA
                    }
                    3 => {
                        // RRA
                    }
                    4 => {
                        // DAA
                    }
                    5 => {
                        // CPL
                    }
                    6 => {
                        // SCF
                    }
                    7 | _ => {
                        // CCF
                    }
                }
            }
        };
        Instruction::NOP
    }

    fn prefixless_x1(bytes: u32) -> Instruction {
        let y = Self::opcode_y(Self::first_byte(bytes));
        let z = Self::opcode_z(Self::first_byte(bytes));

        if (z == 6) && (y == 6) {
            Instruction::HALT
        } else {
            // LD r[y], r[z]
            Instruction::LoadReg {
                r1: Register::r_lookup(y),
                r2: Register::r_lookup(z),
            }
        }
    }

    fn prefixless_x2(bytes: u32) -> Instruction {
        match Self::opcode_y(Self::first_byte(bytes)) {
            0 => {
                // ADD
            }
            1 => {
                // ADC
            }
            2 => {
                // SUB
            }
            3 => {
                // SBC
            }
            4 => {
                // AND
            }
            5 => {
                // XOR
            }
            6 => {
                // OR
            }
            7 | _ => {
                // CP
            }
        };

        Instruction::NOP
    }

    fn prefixless_x3(bytes: u32) -> Instruction {
        let y = Self::opcode_y(Self::first_byte(bytes));
        let q = Self::opcode_q(Self::first_byte(bytes));
        let p = Self::opcode_p(Self::first_byte(bytes));

        match Self::opcode_z(Self::first_byte(bytes)) {
            0 => match y {
                0 | 1 | 2 | 3 => {
                    // RET cc[y]
                }
                4 => {
                    // LD (0xFF00 + n), A
                }
                5 => {
                    // ADD SP, d
                }
                6 => {
                    // LD A, (0xFF00 + n)
                }
                7 | _ => {
                    // LD HL, SP+ d
                }
            },
            1 => {
                if q {
                    match p {
                        0 => {
                            // RET
                        }
                        1 => {
                            // RETI
                        }
                        2 => {
                            // JP HL
                        }
                        3 | _ => {
                            // LD SP, HL
                        }
                    }
                } else {
                    // POP rp2[p]
                }
            }
            2 => {
                match y {
                    0 | 1 | 2 | 3 => {
                        // JP cc[y], nn
                    }
                    4 => {
                        // LD (0xFF00+C), A
                    }
                    5 => {
                        // LD (nn), A
                    }
                    6 => {
                        // LD A, (0xFF00+C)
                    }
                    7 | _ => {
                        // LD A, (nn)
                    }
                }
            }
            3 => {
                match y {
                    0 => {
                        // JP nn
                    } /*
                    1 => {
                    // CB prefix, never encountered.
                    }
                     */
                    2 | 3 | 4 | 5 => {
                        // ILLEGAL OPCODE
                    }
                    6 => {
                        // DI
                    }
                    7 | _ => {
                        // EI
                    }
                }
            }
            4 => {
                match y {
                    0 | 1 | 2 | 3 => {
                        // CALL cc[y], nn
                    }
                    4 | 5 | 6 | 7 | _ => {
                        // ILLEGAL OPCODE
                    }
                }
            }
            5 => {
                if q {
                    match p {
                        0 => {
                            // CALL nn
                        }
                        1 | 2 | 3 | _ => {
                            // ILLEGAL OPCODE
                        }
                    }
                } else {
                    // CALL cc[y]
                }
            }
            6 => {
                // alu[y] n
                match y {
                    0 => {
                        // ADD
                    }
                    1 => {
                        // ADC
                    }
                    2 => {
                        // SUB
                    }
                    3 => {
                        // SBC
                    }
                    4 => {
                        // AND
                    }
                    5 => {
                        // XOR
                    }
                    6 => {
                        // OR
                    }
                    7 | _ => {
                        // CP
                    }
                }
            }
            7 | _ => {
                // RST y*8
            }
        };

        Instruction::NOP
    }
}
