#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RegisterID {
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
    HLaddress,
}
impl RegisterID {
    pub fn r_lookup(val: u8) -> RegisterID {
        // val is assumed to be less than 8
        match val {
            0 => RegisterID::B,
            1 => RegisterID::C,
            2 => RegisterID::D,
            3 => RegisterID::E,
            4 => RegisterID::H,
            5 => RegisterID::L,
            6 => RegisterID::HLaddress,
            7 | _ => RegisterID::A,
        }
    }
    pub fn rp_lookup(val: u8) -> RegisterID {
        // val is assumed to be less than 4
        match val {
            0 => RegisterID::BC,
            1 => RegisterID::DE,
            2 => RegisterID::HL,
            3 | _ => RegisterID::SP,
        }
    }
    pub fn rp2_lookup(val: u8) -> RegisterID {
        // val is assumed to be less than 4
        match val {
            0 => RegisterID::BC,
            1 => RegisterID::DE,
            2 => RegisterID::HL,
            3 | _ => RegisterID::AF,
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FlagID {
    NZ,
    Z,
    NC,
    C,
}
impl FlagID {
    pub fn cc_lookup(val: u8) -> FlagID {
        // val is assumed to be less than 4
        match val {
            0 => FlagID::NZ,
            1 => FlagID::Z,
            2 => FlagID::NC,
            3 | _ => FlagID::C,
        }
    }
}

pub enum Instruction {
    NOP,
    STOP,
    HALT,
    ILLEGAL,

    Load16 { r: RegisterID, nn: u16 },
    Load8 { r: RegisterID, n: u8 },
    LoadReg16 { r1: RegisterID, r2: RegisterID },
    LoadReg8 { r1: RegisterID, r2: RegisterID },
    LoadSPToHLWithOffset { d: i8 },
    LoadFF00PlusC,                   // LD A, (0xFF00+C)
    LoadFF00PlusImmediate { n: u8 }, // LD A, (0xFF00+n)

    StoreFF00Plus { r: RegisterID, n: u8 },
    StoreReg { r1: RegisterID, loc: u16 },
    StoreImmediate { loc: u16 }, // LD (nn), A
    StoreFF00PlusC,              // LD (0xFF00+C), A

    Jump { nn: u16 },
    JumpConditional { f: FlagID, nn: u16 },
    JR { d: i8 },
    JumpRegConditional { f: FlagID, d: i8 },
    JumpToHL,

    RLC { r: RegisterID },
    RRC { r: RegisterID },
    RL { r: RegisterID },
    RR { r: RegisterID },
    SLA { r: RegisterID },
    SRA { r: RegisterID },
    SWAP { r: RegisterID },
    SRL { r: RegisterID },

    BIT { y: u8, r: RegisterID },
    RES { y: u8, r: RegisterID },
    SET { y: u8, r: RegisterID },

    AddRegisters { r1: RegisterID, r2: RegisterID },
    AddSigned { r: RegisterID, d: i8 },

    DEC8b { r: RegisterID },
    DEC16b { r: RegisterID },
    INC8b { r: RegisterID },
    INC16b { r: RegisterID },

    AddHLAndR16 { r: RegisterID },

    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,

    RET { f: FlagID },

    RETNoParam,
    RETI,

    POP { r: RegisterID },

    DI,
    EI,

    CallConditional { f: FlagID, nn: u16 },
    Call { nn: u16 },

    PUSH { r: RegisterID },

    RST { arg: u8 },

    AddImmediate { n: u8 },
    AdcImmediate { n: u8 },
    SubImmediate { n: u8 },
    SbcImmediate { n: u8 },
    AndImmediate { n: u8 },
    XorImmediate { n: u8 },
    OrImmediate { n: u8 },
    CpImmediate { n: u8 },

    AddRegister { r: RegisterID },
    AdcRegister { r: RegisterID },
    SubRegister { r: RegisterID },
    SbcRegister { r: RegisterID },
    AndRegister { r: RegisterID },
    XorRegister { r: RegisterID },
    OrRegister { r: RegisterID },
    CpRegister { r: RegisterID },
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
    fn second_and_third_bytes_reversed(bytes: u32) -> u16 {
        let hi = ((bytes >> 16) & 0xff) as u16;
        let lo = ((bytes >> 8) & 0xff) as u16;
        (lo << 8) & hi
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
            let r = RegisterID::r_lookup(z);

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
                    r1: RegisterID::SP,
                    loc: Self::second_and_third_bytes_reversed(bytes),
                },

                2 => Instruction::STOP,
                3 => Instruction::JR {
                    d: Self::second_byte(bytes) as i8,
                },
                4 | 5 | 6 | 7 | _ => Instruction::JumpRegConditional {
                    f: FlagID::cc_lookup(y - 4),
                    d: Self::second_byte(bytes) as i8,
                },
            },
            1 => {
                if q {
                    Instruction::AddHLAndR16 {
                        r: RegisterID::rp_lookup(p),
                    }
                } else {
                    Instruction::Load16 {
                        r: RegisterID::rp_lookup(p),
                        nn: Self::second_and_third_bytes_reversed(bytes),
                    }
                }
            }
            2 => {
                if q {
                    match p {
                        0 => Instruction::LoadReg16 {
                            r1: RegisterID::A,
                            r2: RegisterID::BC,
                        },
                        1 => Instruction::LoadReg16 {
                            r1: RegisterID::A,
                            r2: RegisterID::DE,
                        },
                        2 => Instruction::LoadReg16 {
                            r1: RegisterID::A,
                            r2: RegisterID::HLplus,
                        },
                        3 | _ => Instruction::LoadReg16 {
                            r1: RegisterID::A,
                            r2: RegisterID::HLminus,
                        },
                    }
                } else {
                    match p {
                        0 => Instruction::LoadReg16 {
                            r1: RegisterID::BC,
                            r2: RegisterID::A,
                        },
                        1 => Instruction::LoadReg16 {
                            r1: RegisterID::DE,
                            r2: RegisterID::A,
                        },
                        2 => Instruction::LoadReg16 {
                            r1: RegisterID::HLplus,
                            r2: RegisterID::A,
                        },
                        3 | _ => Instruction::LoadReg16 {
                            r1: RegisterID::HLminus,
                            r2: RegisterID::A,
                        },
                    }
                }
            }
            3 => {
                if q {
                    Instruction::DEC16b {
                        r: RegisterID::rp_lookup(p),
                    }
                } else {
                    Instruction::INC16b {
                        r: RegisterID::rp_lookup(p),
                    }
                }
            }
            4 => Instruction::INC8b {
                r: RegisterID::r_lookup(y),
            },
            5 => Instruction::DEC8b {
                r: RegisterID::r_lookup(y),
            },
            6 => Instruction::Load8 {
                r: RegisterID::r_lookup(y),
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
            Instruction::LoadReg8 {
                r1: RegisterID::r_lookup(y),
                r2: RegisterID::r_lookup(z),
            }
        }
    }

    fn prefixless_x2(bytes: u32) -> Instruction {
        let z = Self::opcode_z(Self::first_byte(bytes));
        let r = RegisterID::r_lookup(z);

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
                    f: FlagID::cc_lookup(y),
                },
                4 => Instruction::StoreFF00Plus {
                    r: RegisterID::A,
                    n: Self::second_byte(bytes),
                },
                5 => Instruction::AddSigned {
                    r: RegisterID::SP,
                    d: Self::second_byte(bytes) as i8,
                },
                6 => Instruction::LoadFF00PlusImmediate {
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
                        3 | _ => Instruction::LoadReg16 {
                            r1: RegisterID::SP,
                            r2: RegisterID::HL,
                        },
                    }
                } else {
                    Instruction::POP {
                        r: RegisterID::rp2_lookup(p),
                    }
                }
            }
            2 => {
                match y {
                    0 | 1 | 2 | 3 => Instruction::JumpConditional {
                        f: FlagID::cc_lookup(y),
                        nn: Self::second_and_third_bytes_reversed(bytes),
                    },
                    4 => {
                        // LD (0xFF00+C), A
                        Instruction::StoreFF00PlusC
                    }
                    5 => {
                        // LD (nn), A
                        Instruction::StoreImmediate {
                            loc: Self::second_and_third_bytes_reversed(bytes),
                        }
                    }
                    6 => {
                        // LD A, (0xFF00+C)
                        Instruction::LoadFF00PlusC
                    }
                    7 | _ => {
                        // LD A, (nn)
                        Instruction::Load16 {
                            r: RegisterID::A,
                            nn: Self::second_and_third_bytes_reversed(bytes),
                        }
                    }
                }
            }
            3 => {
                match y {
                    0 => Instruction::Jump {
                        nn: Self::second_and_third_bytes_reversed(bytes),
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
                    f: FlagID::cc_lookup(y),
                    nn: Self::second_and_third_bytes_reversed(bytes),
                },
                4 | 5 | 6 | 7 | _ => Instruction::ILLEGAL,
            },
            5 => {
                if q {
                    match p {
                        0 => Instruction::Call {
                            nn: Self::second_and_third_bytes_reversed(bytes),
                        },
                        1 | 2 | 3 | _ => Instruction::ILLEGAL,
                    }
                } else {
                    // PUSH rp2[p]
                    Instruction::PUSH {
                        r: RegisterID::rp2_lookup(p),
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
