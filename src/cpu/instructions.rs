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
    LoadSPToHLWithOffset { d: i8 },
    LoadFF00PlusC, // LD A, (0xFF00+C)

    StoreFF00Plus { r: Register, n: u8 },
    StoreReg { r1: Register, loc: u16 },
    StoreImmediate { loc: u16 }, // LD (nn), A
    StoreFF00PlusC,              // LD (0xFF00+C), A

    Jump { nn: u16 },
    JumpConditional { f: Flag, nn: u16 },
    JR { d: i8 },
    JumpRegConditional { f: Flag, d: i8 },
    JumpToHL,

    RLC { r: Register },
    RRC { r: Register },
    RL { r: Register },
    RR { r: Register },
    SLA { r: Register },
    SRA { r: Register },
    SWAP { r: Register },
    SRL { r: Register },

    BIT { y: u8, r: Register },
    RES { y: u8, r: Register },
    SET { y: u8, r: Register },

    AddRegisters { r1: Register, r2: Register },
    AddSigned { r: Register, d: i8 },

    DEC { r: Register },
    INC { r: Register },

    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,

    RET { f: Flag },

    RETNoParam,
    RETI,

    POP { r: Register },

    DI,
    EI,

    CallConditional { f: Flag, nn: u16 },
    Call { nn: u16 },

    PUSH { r: Register },

    RST { arg: u8 },

    AddImmediate { n: u8 },
    AdcImmediate { n: u8 },
    SubImmediate { n: u8 },
    SbcImmediate { n: u8 },
    AndImmediate { n: u8 },
    XorImmediate { n: u8 },
    OrImmediate { n: u8 },
    CpImmediate { n: u8 },

    AddRegister { r: Register },
    AdcRegister { r: Register },
    SubRegister { r: Register },
    SbcRegister { r: Register },
    AndRegister { r: Register },
    XorRegister { r: Register },
    OrRegister { r: Register },
    CpRegister { r: Register },
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
            let r = Register::r_lookup(z);

            match Self::opcode_x(Self::second_byte(bytes)) {
                0 => {
                    // rot[y] r[z]
                    match y {
                        0 => Instruction::RLC { r },
                        1 => Instruction::RRC { r },
                        2 => Instruction::RL { r },
                        3 => Instruction::RR { r },
                        4 => Instruction::SLA { r },
                        5 => Instruction::SRA { r },
                        6 => Instruction::SWAP { r },
                        7 | _ => Instruction::SRL { r },
                    }
                }
                1 => Instruction::BIT { y, r },
                2 => Instruction::RES { y, r },
                3 | _ => Instruction::SET { y, r },
            }
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
            0 => match y {
                0 => Instruction::NOP,
                1 => Instruction::StoreReg {
                    r1: Register::SP,
                    loc: Self::second_and_third_bytes(bytes),
                },

                2 => Instruction::STOP,
                3 => Instruction::JR {
                    d: Self::second_byte(bytes) as i8,
                },
                4 | 5 | 6 | 7 | _ => Instruction::JumpRegConditional {
                    f: Flag::cc_lookup(y - 4),
                    d: Self::second_byte(bytes) as i8,
                },
            },
            1 => {
                if q {
                    Instruction::AddRegisters {
                        r1: Register::HL,
                        r2: Register::rp_lookup(p),
                    }
                } else {
                    Instruction::Load16 {
                        r: Register::rp_lookup(p),
                        nn: Self::second_and_third_bytes(bytes),
                    }
                }
            }
            2 => {
                if q {
                    match p {
                        0 => Instruction::LoadReg {
                            r1: Register::A,
                            r2: Register::BC,
                        },
                        1 => Instruction::LoadReg {
                            r1: Register::A,
                            r2: Register::BC,
                        },
                        2 => Instruction::LoadReg {
                            r1: Register::A,
                            r2: Register::HLplus,
                        },
                        3 | _ => Instruction::LoadReg {
                            r1: Register::A,
                            r2: Register::HLminus,
                        },
                    }
                } else {
                    match p {
                        0 => Instruction::LoadReg {
                            r1: Register::BC,
                            r2: Register::A,
                        },
                        1 => Instruction::LoadReg {
                            r1: Register::DE,
                            r2: Register::A,
                        },
                        2 => Instruction::LoadReg {
                            r1: Register::HLplus,
                            r2: Register::A,
                        },
                        3 | _ => Instruction::LoadReg {
                            r1: Register::HLminus,
                            r2: Register::A,
                        },
                    }
                }
            }
            3 => {
                if q {
                    Instruction::DEC {
                        r: Register::rp_lookup(p),
                    }
                } else {
                    Instruction::INC {
                        r: Register::rp_lookup(p),
                    }
                }
            }
            4 => Instruction::INC {
                r: Register::r_lookup(y),
            },
            5 => Instruction::DEC {
                r: Register::r_lookup(y),
            },
            6 => Instruction::Load8 {
                r: Register::r_lookup(y),
                n: Self::second_byte(bytes),
            },
            7 | _ => match y {
                0 => Instruction::RLCA,
                1 => Instruction::RRCA,
                2 => Instruction::RLA,
                3 => Instruction::RRA,
                4 => Instruction::DAA,
                5 => Instruction::CPL,
                6 => Instruction::SCF,
                7 | _ => Instruction::CCF,
            },
        }
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
        let z = Self::opcode_z(Self::first_byte(bytes));
        let r = Register::r_lookup(z);

        match Self::opcode_y(Self::first_byte(bytes)) {
            0 => Instruction::AddRegister { r },
            1 => Instruction::AdcRegister { r },
            2 => Instruction::SubRegister { r },
            3 => Instruction::SbcRegister { r },
            4 => Instruction::AndRegister { r },
            5 => Instruction::XorRegister { r },
            6 => Instruction::OrRegister { r },
            7 | _ => Instruction::CpRegister { r },
        }
    }

    fn prefixless_x3(bytes: u32) -> Instruction {
        let y = Self::opcode_y(Self::first_byte(bytes));
        let q = Self::opcode_q(Self::first_byte(bytes));
        let p = Self::opcode_p(Self::first_byte(bytes));

        match Self::opcode_z(Self::first_byte(bytes)) {
            0 => match y {
                0 | 1 | 2 | 3 => Instruction::RET {
                    f: Flag::cc_lookup(y),
                },
                4 => Instruction::StoreFF00Plus {
                    r: Register::A,
                    n: Self::second_byte(bytes),
                },
                5 => Instruction::AddSigned {
                    r: Register::SP,
                    d: Self::second_byte(bytes) as i8,
                },
                6 => Instruction::LoadFF00Plus {
                    r: Register::A,
                    n: Self::second_byte(bytes),
                },
                7 | _ => Instruction::LoadSPToHLWithOffset {
                    d: Self::second_byte(bytes) as i8,
                },
            },
            1 => {
                if q {
                    match p {
                        0 => Instruction::RETNoParam,
                        1 => Instruction::RETI,
                        2 => Instruction::JumpToHL,
                        3 | _ => Instruction::LoadReg {
                            r1: Register::SP,
                            r2: Register::HL,
                        },
                    }
                } else {
                    Instruction::POP {
                        r: Register::rp2_lookup(p),
                    }
                }
            }
            2 => {
                match y {
                    0 | 1 | 2 | 3 => Instruction::JumpConditional {
                        f: Flag::cc_lookup(y),
                        nn: Self::second_and_third_bytes(bytes),
                    },
                    4 => {
                        // LD (0xFF00+C), A
                        Instruction::StoreFF00PlusC
                    }
                    5 => {
                        // LD (nn), A
                        Instruction::StoreImmediate {
                            loc: Self::second_and_third_bytes(bytes),
                        }
                    }
                    6 => {
                        // LD A, (0xFF00+C)
                        Instruction::LoadFF00PlusC
                    }
                    7 | _ => {
                        // LD A, (nn)
                        Instruction::Load16 {
                            r: Register::A,
                            nn: Self::second_and_third_bytes(bytes),
                        }
                    }
                }
            }
            3 => {
                match y {
                    0 => Instruction::Jump {
                        nn: Self::second_and_third_bytes(bytes),
                    }, /*
                    1 => {
                    // CB prefix, never encountered.
                    }
                     */
                    2 | 3 | 4 | 5 => Instruction::ILLEGAL,
                    6 => Instruction::DI,
                    7 | _ => Instruction::EI,
                }
            }
            4 => match y {
                0 | 1 | 2 | 3 => Instruction::CallConditional {
                    f: Flag::cc_lookup(y),
                    nn: Self::second_and_third_bytes(bytes),
                },
                4 | 5 | 6 | 7 | _ => Instruction::ILLEGAL,
            },
            5 => {
                if q {
                    match p {
                        0 => Instruction::Call {
                            nn: Self::second_and_third_bytes(bytes),
                        },
                        1 | 2 | 3 | _ => Instruction::ILLEGAL,
                    }
                } else {
                    // PUSH rp2[p]
                    Instruction::PUSH {
                        r: Register::rp2_lookup(p),
                    }
                }
            }
            6 => {
                // alu[y] n
                match y {
                    0 => Instruction::AddImmediate {
                        n: Self::second_byte(bytes),
                    },
                    1 => Instruction::AdcImmediate {
                        n: Self::second_byte(bytes),
                    },
                    2 => Instruction::SubImmediate {
                        n: Self::second_byte(bytes),
                    },
                    3 => Instruction::SbcImmediate {
                        n: Self::second_byte(bytes),
                    },
                    4 => Instruction::AndImmediate {
                        n: Self::second_byte(bytes),
                    },
                    5 => Instruction::XorImmediate {
                        n: Self::second_byte(bytes),
                    },
                    6 => Instruction::OrImmediate {
                        n: Self::second_byte(bytes),
                    },
                    7 | _ => Instruction::CpImmediate {
                        n: Self::second_byte(bytes),
                    },
                }
            }
            7 | _ => Instruction::RST { arg: y << 3 },
        }
    }
}
