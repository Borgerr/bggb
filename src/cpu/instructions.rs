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

pub enum Flag {
    NZ,
    Z,
    NC,
    C,
}

pub enum Instruction {
    NOP,
    STOP,
    HALT,
    ILLEGAL,

    Load16 {r: Register, nn: u16},
    LoadReg {r1: Register, r2: Register},
    Load8 {r: Register, n: u8},

    Jump{nn: u16},
    JumpConditional{f: Flag, nn: u16},
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
            match Self::opcode_x(Self::second_byte(bytes)) {
                0 => {
                    // rot[y] r[z]
                    match Self::opcode_y(Self::second_byte(bytes)) {
                        0 => {
                            // RLC
                        }
                        1 => {
                            // RRC
                        }
                        2 => {
                            // RL
                        }
                        3 => {
                            // RR
                        }
                        4 => {
                            // SLA
                        }
                        5 => {
                            // SRA
                        }
                        6 => {
                            // SWAP
                        }
                        7 | _ => {
                            // SRL
                        }
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
            }
        } else {
            // first byte is not a prefix byte
            match Self::opcode_x(Self::first_byte(bytes)) {
                0 => match Self::opcode_z(Self::first_byte(bytes)) {
                    0 => {
                        match Self::opcode_y(Self::first_byte(bytes)) {
                            0 => {
                                Instruction::NOP
                            }
                            1 => {
                                // LD (nn), SP
                                Instruction::Load16{r: Register::SP, nn: Self::second_and_third_bytes(bytes)}
                            }
                            2 => {
                                Instruction::STOP
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
                        if Self::opcode_q(Self::first_byte(bytes)) {
                            // ADD HL, rp[p]
                        } else {
                            // LD rp[p], nn
                            match Self::opcode_p(Self::first_byte(bytes)) {
                                0 => Instruction::Load16{r: Register::BC, nn: Self::second_and_third_bytes(bytes)},
                                1 => Instruction::Load16{r: Register::DE, nn: Self::second_and_third_bytes(bytes)},
                                2 => Instruction::Load16{r: Register::HL, nn: Self::second_and_third_bytes(bytes)},
                                3 | _ => Instruction::Load16{r: Register::SP, nn: Self::second_and_third_bytes(bytes)},
                            }
                        }
                    }
                    2 => {
                        if Self::opcode_q(Self::first_byte(bytes)) {
                            match Self::opcode_p(Self::first_byte(bytes)) {
                                0 => {
                                    // LD A, (BC)
                                    Instruction::LoadReg{r1: Register::A, r2: Register::BC}
                                }
                                1 => {
                                    // LD A, (DE)
                                    Instruction::LoadReg{r1: Register::A, r2: Register::DE}
                                }
                                2 => {
                                    // LD A, (HL+)
                                    Instruction::LoadReg{r1: Register::A, r2: Register::HLplus}
                                }
                                3 | _ => {
                                    // LD A, (HL-)
                                    Instruction::LoadReg{r1: Register::A, r2: Register::HLminus}
                                }
                            }
                        } else {
                            match Self::opcode_p(Self::first_byte(bytes)) {
                                0 => {
                                    // LD (BC), A
                                    Instruction::LoadReg{r1: Register::BC, r2: Register::A}
                                }
                                1 => {
                                    // LD (DE), A
                                    Instruction::LoadReg{r1: Register::DE, r2: Register::A}
                                }
                                2 => {
                                    // LD (HL+), A
                                    Instruction::LoadReg{r1: Register::HLplus, r2: Register::A}
                                }
                                3 | _ => {
                                    // LD (HL-), A
                                    Instruction::LoadReg{r1: Register::HLminus, r2: Register::A}
                                }
                            }
                        }
                    }
                    3 => {
                        if Self::opcode_q(Self::first_byte(bytes)) {
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
                        match Self::opcode_y(Self::first_byte(bytes)) {
                            0 => Instruction::Load8{r: Register::B, n: Self::second_byte(bytes)},
                            1 => Instruction::Load8{r: Register::C, n: Self::second_byte(bytes)},
                            2 => Instruction::Load8{r: Register::D, n: Self::second_byte(bytes)},
                            3 => Instruction::Load8{r: Register::E, n: Self::second_byte(bytes)},
                            4 => Instruction::Load8{r: Register::H, n: Self::second_byte(bytes)},
                            5 => Instruction::Load8{r: Register::L, n: Self::second_byte(bytes)},
                            6 => Instruction::Load8{r: Register::HL, n: Self::second_byte(bytes)},
                            7 | _ => Instruction::Load8{r: Register::A, n: Self::second_byte(bytes)},
                        }
                    }
                    7 | _ => {
                        match Self::opcode_y(Self::first_byte(bytes)) {
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
                },
                1 => {
                    if (Self::opcode_z(Self::first_byte(bytes)) == 6)
                        && (Self::opcode_y(Self::first_byte(bytes)) == 6)
                    {
                        Instruction::HALT
                    } else {
                        // LD r[y], r[z]
                    }
                }
                2 => {
                    // alu[y] r[z]
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
                    }
                }
                3 | _ => match Self::opcode_z(Self::first_byte(bytes)) {
                    0 => match Self::opcode_y(Self::first_byte(bytes)) {
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
                        if Self::opcode_q(Self::first_byte(bytes)) {
                            match Self::opcode_p(Self::first_byte(bytes)) {
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
                        match Self::opcode_y(Self::first_byte(bytes)) {
                            // JP cc[y], nn for 0-3
                            0 => Instruction::JumpConditional { f: Flag::NZ, nn: Self::second_and_third_bytes(bytes) },
                            1 => Instruction::JumpConditional { f: Flag::Z, nn: Self::second_and_third_bytes(bytes) },
                            2 => Instruction::JumpConditional { f: Flag::NC, nn: Self::second_and_third_bytes(bytes) },
                            3 => Instruction::JumpConditional { f: Flag::C, nn: Self::second_and_third_bytes(bytes) },
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
                        match Self::opcode_y(Self::first_byte(bytes)) {
                            0 => {
                                // JP nn
                                Instruction::Jump{nn: Self::second_and_third_bytes(bytes)}
                            } /*
                            1 => {
                            // CB prefix, never encountered.
                            }
                             */
                            2 | 3 | 4 | 5 => {
                                Instruction::ILLEGAL
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
                        match Self::opcode_y(Self::first_byte(bytes)) {
                            0 | 1 | 2 | 3 => {
                                // CALL cc[y], nn
                            }
                            4 | 5 | 6 | 7 | _ => {
                                Instruction::ILLEGAL
                            }
                        }
                    }
                    5 => {
                        if Self::opcode_q(Self::first_byte(bytes)) {
                            match Self::opcode_p(Self::first_byte(bytes)) {
                                0 => {
                                    // CALL nn
                                }
                                1 | 2 | 3 | _ => {
                                    Instruction::ILLEGAL
                                }
                            }
                        } else {
                            // call with condition y < 4
                        }
                    }
                    6 => {
                        // alu[y] n
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
                        }
                    }
                    7 | _ => {
                        // RST y*8
                    }
                },
            }
        }
    }
}
