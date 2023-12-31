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
                        if Self::opcode_q(Self::first_byte(bytes)) {
                            // ADD HL, rp[p]
                        } else {
                            // LD rp[p], nn
                        }
                    }
                    2 => {
                        if Self::opcode_q(Self::first_byte(bytes)) {
                            match Self::opcode_p(Self::first_byte(bytes)) {
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
                            match Self::opcode_p(Self::first_byte(bytes)) {
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
                        // HALT
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
                        match Self::opcode_y(Self::first_byte(bytes)) {
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
                        match Self::opcode_y(Self::first_byte(bytes)) {
                            0 | 1 | 2 | 3 => {
                                // CALL cc[y], nn
                            }
                            4 | 5 | 6 | 7 | _ => {
                                // ILLEGAL OPCODE
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
                                    // ILLEGAL OPCODE
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

        Instruction::NOP
    }
}
