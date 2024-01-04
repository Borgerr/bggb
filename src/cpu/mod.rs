mod instructions;
use instructions::{FlagID, Instruction, RegisterID};

use crate::memory::Memory;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CpuError {
    FetchError { pc: u16 },
    IllegalInstruction { pc: u16 },
    ReadingIntoInvalidReg { r: RegisterID, pc: u16 },
    ReadingFromInvalidReg { r: RegisterID, pc: u16 },
    IndexOutOfBounds { index: usize, pc: u16 },
}

struct CPU {
    af: u16, // accumulator & flags
    bc: u16, // BC register
    de: u16, // DE register
    hl: u16, // HL register
    sp: u16, // stack pointer
    pc: u16, // program counter/pointer
}

impl CPU {
    fn hi_byte(x: u16) -> u8 {
        (x >> 8) as u8
    }
    fn lo_byte(x: u16) -> u8 {
        (x & 0xff) as u8
    }

    fn registerid_to_u16(&mut self, r: RegisterID) -> u16 {
        match r {
            RegisterID::AF => self.af,
            RegisterID::BC => self.bc,
            RegisterID::DE => self.de,
            RegisterID::HL | RegisterID::HLplus | RegisterID::HLminus => self.hl,
            RegisterID::SP => self.sp,

            RegisterID::A => Self::hi_byte(self.af) as u16,
            RegisterID::B => Self::hi_byte(self.bc) as u16,
            RegisterID::C => Self::lo_byte(self.bc) as u16,
            RegisterID::D => Self::hi_byte(self.de) as u16,
            RegisterID::E => Self::lo_byte(self.de) as u16,
            RegisterID::H => Self::hi_byte(self.hl) as u16,
            RegisterID::L => Self::lo_byte(self.hl) as u16,
        }
    }

    fn registerid_to_u8(&mut self, r: RegisterID) -> Result<u8, CpuError> {
        let result = match r {
            RegisterID::A => Self::hi_byte(self.af),
            RegisterID::B => Self::hi_byte(self.bc),
            RegisterID::C => Self::lo_byte(self.bc),
            RegisterID::D => Self::hi_byte(self.de),
            RegisterID::E => Self::lo_byte(self.de),
            RegisterID::H => Self::hi_byte(self.hl),
            RegisterID::L => Self::lo_byte(self.hl),

            _ => return Err(CpuError::ReadingFromInvalidReg { r, pc: self.pc }),
        };

        Ok(result)
    }

    pub fn fetch_decode_execute(&mut self, mem: &Memory) -> Result<(), CpuError> {
        let bytes = self.fetch_instr_u32(mem)?;
        let instr = Instruction::from_bytes(bytes);
        self.execute(instr, mem)?;

        Ok(())
    }

    fn fetch_pc_u8(&mut self, mem: &Memory) -> Result<u8, CpuError> {
        if self.pc > 0x7fff {
            // possibly change to be more general for other fetches
            Err(CpuError::FetchError { pc: self.pc })
        } else {
            let result = Ok(mem[self.pc as usize]);
            self.pc += 1;
            result
        }
    }
    fn fetch_instr_u32(&mut self, mem: &Memory) -> Result<u32, CpuError> {
        // always increments program counter by 3
        Ok(((self.fetch_pc_u8(mem)? as u32) << 24)
            | ((self.fetch_pc_u8(mem)? as u32) << 16)
            | ((self.fetch_pc_u8(mem)? as u32) << 8))
    }

    fn execute(&mut self, instr: Instruction, mem: &Memory) -> Result<(), CpuError> {
        // This function is kind of smelly since it decreases the program counter after determining the instruction
        // but the motivation is that instructions should store data critical to the operation,
        // and do that in one call to `Instruction::from_bytes`.
        // The other alternative would be to repeatedly call that, and lose information within the opcodes themselves
        match instr {
            Instruction::NOP => self.pc -= 2,
            Instruction::STOP => self.pc -= 2,
            Instruction::HALT => self.pc -= 1,
            Instruction::ILLEGAL => {
                self.pc -= 3;
                return Err(CpuError::IllegalInstruction { pc: self.pc });
            }

            Instruction::Load16 { r, nn } => {
                self.pc -= 0;
                self.load_immediate16(r, nn)?;
            }
            Instruction::Load8 { r, n } => {
                self.pc -= 1;
                self.load_immediate8(r, n)?;
            }
            Instruction::LoadFF00PlusImmediate { n } => {
                self.pc -= 1;
                self.load_ff00_plus_n(mem, n);
            }
            Instruction::LoadReg16 { r1, r2 } => {
                self.pc -= 2;
                self.load_registers16(r1, r2)?;
            }
            Instruction::LoadReg8 { r1, r2 } => {
                self.pc -= 2;
                self.load_registers8(r1, r2)?;
            }
            Instruction::LoadSPToHLWithOffset { d } => {
                self.pc -= 1;
                self.load_sp_to_hl_with_offset(mem, d)?;
            }
            Instruction::LoadFF00PlusC => {
                self.pc -= 2;
                self.load_ff00_plus_c(mem);
            }

            Instruction::StoreFF00Plus { r, n } => self.pc -= 1,
            Instruction::StoreReg { r1, loc } => self.pc -= 0,
            Instruction::StoreImmediate { loc } => self.pc -= 0,
            Instruction::StoreFF00PlusC => self.pc -= 2,

            // really no need to change the program counter prior to jump
            Instruction::Jump { nn } => self.jump(nn),
            Instruction::JumpConditional { f, nn } => {}
            Instruction::JR { d } => {}
            Instruction::JumpRegConditional { f, d } => {}
            Instruction::JumpToHL => {}

            // CB-prefixed
            Instruction::RLC { r } => self.pc -= 1,
            Instruction::RRC { r } => self.pc -= 1,
            Instruction::RL { r } => self.pc -= 1,
            Instruction::RR { r } => self.pc -= 1,
            Instruction::SLA { r } => self.pc -= 1,
            Instruction::SRA { r } => self.pc -= 1,
            Instruction::SWAP { r } => self.pc -= 1,
            Instruction::SRL { r } => self.pc -= 1,

            // also CB-prefixed (y is included in opcode)
            Instruction::BIT { y, r } => self.pc -= 1,
            Instruction::RES { y, r } => self.pc -= 1,
            Instruction::SET { y, r } => self.pc -= 1,

            Instruction::AddRegisters { r1, r2 } => self.pc -= 2,
            Instruction::AddSigned { r, d } => self.pc -= 1,

            Instruction::DEC { r } => self.pc -= 2,
            Instruction::INC { r } => self.pc -= 2,

            Instruction::RLCA => self.pc -= 2,
            Instruction::RRCA => self.pc -= 2,
            Instruction::RLA => self.pc -= 2,
            Instruction::RRA => self.pc -= 2,
            Instruction::DAA => self.pc -= 2,
            Instruction::CPL => self.pc -= 2,
            Instruction::SCF => self.pc -= 2,
            Instruction::CCF => self.pc -= 2,

            Instruction::RET { f } => self.pc -= 2,

            Instruction::RETNoParam => self.pc -= 2,
            Instruction::RETI => self.pc -= 2,

            Instruction::POP { r } => self.pc -= 2,

            Instruction::DI => self.pc -= 2,
            Instruction::EI => self.pc -= 2,

            Instruction::CallConditional { f, nn } => self.pc -= 0,
            Instruction::Call { nn } => self.pc -= 0,

            Instruction::PUSH { r } => self.pc -= 2,

            Instruction::RST { arg } => self.pc -= 2, // "arg" is included in opcode

            Instruction::AddImmediate { n } => self.pc -= 1,
            Instruction::AdcImmediate { n } => self.pc -= 1,
            Instruction::SubImmediate { n } => self.pc -= 1,
            Instruction::SbcImmediate { n } => self.pc -= 1,
            Instruction::AndImmediate { n } => self.pc -= 1,
            Instruction::XorImmediate { n } => self.pc -= 1,
            Instruction::OrImmediate { n } => self.pc -= 1,
            Instruction::CpImmediate { n } => self.pc -= 1,

            Instruction::AddRegister { r } => self.pc -= 2,
            Instruction::AdcRegister { r } => self.pc -= 2,
            Instruction::SubRegister { r } => self.pc -= 2,
            Instruction::SbcRegister { r } => self.pc -= 2,
            Instruction::AndRegister { r } => self.pc -= 2,
            Instruction::XorRegister { r } => self.pc -= 2,
            Instruction::OrRegister { r } => self.pc -= 2,
            Instruction::CpRegister { r } => self.pc -= 2,
        }

        Ok(())
    }

    /*
       instructions
    */
    fn jump(&mut self, nn: u16) {
        self.pc = nn;
    }

    fn load_immediate16(&mut self, r: RegisterID, nn: u16) -> Result<(), CpuError> {
        match r {
            RegisterID::AF => self.af = nn,
            RegisterID::BC => self.bc = nn,
            RegisterID::DE => self.de = nn,
            RegisterID::HL => self.hl = nn,
            RegisterID::SP => self.sp = nn,
            _ => return Err(CpuError::ReadingIntoInvalidReg { r, pc: self.pc - 3 }),
        }
        Ok(())
    }

    fn load_immediate8(&mut self, r: RegisterID, n: u8) -> Result<(), CpuError> {
        match r {
            RegisterID::A => {
                self.af |= 0xff00;
                self.af &= (n as u16) << 8;
            }
            RegisterID::B => {
                self.bc |= 0xff00;
                self.bc &= (n as u16) << 8;
            }
            RegisterID::C => {
                self.bc |= 0x00ff;
                self.bc &= n as u16;
            }
            RegisterID::D => {
                self.de |= 0xff00;
                self.de &= (n as u16) << 8;
            }
            RegisterID::E => {
                self.de |= 0x00ff;
                self.de &= n as u16;
            }
            RegisterID::H => {
                self.hl |= 0xff00;
                self.hl &= (n as u16) << 8;
            }
            RegisterID::L => {
                self.hl |= 0x00ff;
                self.hl &= n as u16;
            }
            RegisterID::HL => {
                self.hl = n as u16;
            }
            _ => return Err(CpuError::ReadingIntoInvalidReg { r, pc: self.pc - 2 }),
        }

        Ok(())
    }

    fn load_sp_to_hl_with_offset(&mut self, mem: &Memory, d: i8) -> Result<(), CpuError> {
        let index = ((self.sp as i32) + (d as i32)) as usize;
        if index > 0xffff {
            return Err(CpuError::IndexOutOfBounds {
                index,
                pc: self.pc - 2,
            });
        }

        self.hl = mem[index] as u16;

        Ok(())
    }

    fn load_ff00_plus_n(&mut self, mem: &Memory, n: u8) {
        self.af |= 0xff00;
        self.af &= mem[(0xff00) + (n as usize)] as u16;
    }

    fn load_ff00_plus_c(&mut self, mem: &Memory) {
        self.af |= 0xff00;
        self.af &= mem[(0xff00) + Self::lo_byte(self.bc) as usize] as u16;
    }

    fn load_registers16(&mut self, r1: RegisterID, r2: RegisterID) -> Result<(), CpuError> {
        match r1 {
            RegisterID::SP => self.sp = self.registerid_to_u16(r2),
            RegisterID::A => match r2 {
                // special cases when HLplus or HLminus; doesn't apply to others
                // as those instructions don't exist
                RegisterID::HLplus => {
                    self.af = self.registerid_to_u16(r2);
                    self.hl += 1;
                }
                RegisterID::HLminus => {
                    self.af = self.registerid_to_u16(r2);
                    self.hl -= 1;
                }
                _ => (),
            },
            RegisterID::BC => self.bc = self.registerid_to_u16(r2),
            RegisterID::DE => self.de = self.registerid_to_u16(r2),
            RegisterID::HLplus => {
                self.hl = self.registerid_to_u16(r2);
                self.hl += 1;
            }
            RegisterID::HLminus => {
                self.hl = self.registerid_to_u16(r2);
                self.hl -= 1;
            }

            _ => {
                return Err(CpuError::ReadingIntoInvalidReg {
                    r: r1,
                    pc: self.pc - 1,
                });
            }
        }

        Ok(())
    }

    fn load_registers8(&mut self, r1: RegisterID, r2: RegisterID) -> Result<(), CpuError> {
        let new_val = self.registerid_to_u8(r2)?;
        match r1 {
            RegisterID::A => {
                self.af |= 0xff00;
                self.af &= (new_val as u16) << 8;
            }
            RegisterID::B => {
                self.bc |= 0xff00;
                self.bc &= (new_val as u16) << 8;
            }
            RegisterID::C => {
                self.bc |= 0x00ff;
                self.bc &= new_val as u16;
            }
            RegisterID::D => {
                self.de |= 0xff00;
                self.de &= (new_val as u16) << 8;
            }
            RegisterID::E => {
                self.de |= 0x00ff;
                self.de &= new_val as u16;
            }
            RegisterID::H => {
                self.hl |= 0xff00;
                self.hl &= (new_val as u16) << 8;
            }
            RegisterID::L => {
                self.hl |= 0x00ff;
                self.hl &= new_val as u16;
            }
            RegisterID::HL => self.hl = new_val as u16,

            _ => {
                return Err(CpuError::ReadingIntoInvalidReg {
                    r: r1,
                    pc: self.pc - 1,
                })
            }
        }

        Ok(())
    }
}
