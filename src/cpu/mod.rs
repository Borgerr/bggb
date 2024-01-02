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

            _ => return Err(CpuError::ReadingFromInvalidReg { r, pc: self.pc - 1 }),
            // so far this error is only for fetching from 8bit registers, which is 1 byte long
            // so the pc is decremented only by 1
            // can possibly adjust this later
        };

        Ok(result)
    }

    pub fn fetch_decode_execute(&mut self, mem: &Memory) -> Result<(), CpuError> {
        let bytes = self.fetch_pc_u32(mem)?;
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
    fn fetch_pc_u32(&mut self, mem: &Memory) -> Result<u32, CpuError> {
        Ok(((self.fetch_pc_u8(mem)? as u32) << 24)
            | ((self.fetch_pc_u8(mem)? as u32) << 16)
            | ((self.fetch_pc_u8(mem)? as u32) << 8)
            | (self.fetch_pc_u8(mem)?) as u32)
    }

    fn execute(&mut self, instr: Instruction, mem: &Memory) -> Result<(), CpuError> {
        // remember if the instruction ends up being short, like NOP, reduce the program counter again
        match instr {
            Instruction::NOP => {}
            Instruction::STOP => {}
            Instruction::HALT => {}
            Instruction::ILLEGAL => return Err(CpuError::IllegalInstruction { pc: self.pc - 4 }),

            Instruction::Load16 { r, nn } => self.load_immediate16(r, nn)?,
            Instruction::Load8 { r, n } => self.load_immediate8(r, n)?,
            Instruction::LoadFF00PlusImmediate { n } => self.load_ff00_plus_n(mem, n),
            Instruction::LoadReg16 { r1, r2 } => self.load_registers16(r1, r2)?,
            Instruction::LoadReg8 { r1, r2 } => self.load_registers8(r1, r2)?,
            Instruction::LoadSPToHLWithOffset { d } => self.load_sp_to_hl_with_offset(mem, d)?,
            Instruction::LoadFF00PlusC => self.load_ff00_plus_c(mem),

            Instruction::StoreFF00Plus { r, n } => {}
            Instruction::StoreReg { r1, loc } => {}
            Instruction::StoreImmediate { loc } => {}
            Instruction::StoreFF00PlusC => {}

            Instruction::Jump { nn } => self.jump(nn),
            Instruction::JumpConditional { f, nn } => {}
            Instruction::JR { d } => {}
            Instruction::JumpRegConditional { f, d } => {}
            Instruction::JumpToHL => {}

            Instruction::RLC { r } => {}
            Instruction::RRC { r } => {}
            Instruction::RL { r } => {}
            Instruction::RR { r } => {}
            Instruction::SLA { r } => {}
            Instruction::SRA { r } => {}
            Instruction::SWAP { r } => {}
            Instruction::SRL { r } => {}

            Instruction::BIT { y, r } => {}
            Instruction::RES { y, r } => {}
            Instruction::SET { y, r } => {}

            Instruction::AddRegisters { r1, r2 } => {}
            Instruction::AddSigned { r, d } => {}

            Instruction::DEC { r } => {}
            Instruction::INC { r } => {}

            Instruction::RLCA => {}
            Instruction::RRCA => {}
            Instruction::RLA => {}
            Instruction::RRA => {}
            Instruction::DAA => {}
            Instruction::CPL => {}
            Instruction::SCF => {}
            Instruction::CCF => {}

            Instruction::RET { f } => {}

            Instruction::RETNoParam => {}
            Instruction::RETI => {}

            Instruction::POP { r } => {}

            Instruction::DI => {}
            Instruction::EI => {}

            Instruction::CallConditional { f, nn } => {}
            Instruction::Call { nn } => {}

            Instruction::PUSH { r } => {}

            Instruction::RST { arg } => {}

            Instruction::AddImmediate { n } => {}
            Instruction::AdcImmediate { n } => {}
            Instruction::SubImmediate { n } => {}
            Instruction::SbcImmediate { n } => {}
            Instruction::AndImmediate { n } => {}
            Instruction::XorImmediate { n } => {}
            Instruction::OrImmediate { n } => {}
            Instruction::CpImmediate { n } => {}

            Instruction::AddRegister { r } => {}
            Instruction::AdcRegister { r } => {}
            Instruction::SubRegister { r } => {}
            Instruction::SbcRegister { r } => {}
            Instruction::AndRegister { r } => {}
            Instruction::XorRegister { r } => {}
            Instruction::OrRegister { r } => {}
            Instruction::CpRegister { r } => {}
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
        self.pc -= 1; // instruction has 3 bytes

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
        self.pc -= 2;

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
        self.pc -= 2;

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
        self.pc -= 2;

        self.af |= 0xff00;
        self.af &= mem[(0xff00) + (n as usize)] as u16;
    }

    fn load_ff00_plus_c(&mut self, mem: &Memory) {
        self.pc -= 3;

        self.af |= 0xff00;
        self.af &= mem[(0xff00) + Self::lo_byte(self.bc) as usize] as u16;
    }

    fn load_registers16(&mut self, r1: RegisterID, r2: RegisterID) -> Result<(), CpuError> {
        self.pc -= 3;

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
        self.pc -= 3;

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
