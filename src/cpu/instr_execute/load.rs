use crate::cpu::instructions::RegisterID;
use crate::cpu::{cpuerror::CpuError, lo_byte, CPU};
use crate::memory::Memory;

impl CPU {
    pub fn load_immediate16(&mut self, r: RegisterID, nn: u16) -> Result<(), CpuError> {
        match r {
            RegisterID::AF => self.af = nn,
            RegisterID::BC => self.bc = nn,
            RegisterID::DE => self.de = nn,
            RegisterID::HL => self.hl = nn,
            RegisterID::SP => self.sp = nn,
            _ => return Err(CpuError::ReadingIntoInvalidReg { r, pc: self.pc }),
        }
        Ok(())
    }

    pub fn load_immediate8(
        &mut self,
        r: RegisterID,
        n: u8,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        self.r_table_assign(r, n, mem)?;

        Ok(())
    }

    pub fn load_sp_to_hl_with_offset(&mut self, mem: &Memory, d: i8) -> Result<(), CpuError> {
        let index = ((self.sp as i32) + (d as i32)) as usize;
        if index > 0xffff {
            return Err(CpuError::IndexOutOfBounds { index, pc: self.pc });
        }

        self.hl = mem[index] as u16;

        Ok(())
    }

    pub fn load_ff00_plus_n(&mut self, mem: &Memory, n: u8) {
        self.af |= 0xff00;
        self.af &= mem[(0xff00) + (n as usize)] as u16;
    }

    pub fn load_ff00_plus_c(&mut self, mem: &Memory) {
        self.af |= 0xff00;
        self.af &= mem[(0xff00) + lo_byte(self.bc) as usize] as u16;
    }

    pub fn load_registers16(
        &mut self,
        r1: RegisterID,
        r2: RegisterID,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        match r1 {
            RegisterID::SP => self.sp = self.registerid_to_u16(r2),
            RegisterID::A => match r2 {
                RegisterID::HLplus => {
                    self.set_register_a(mem[self.hl as usize]);
                    self.hl -= 1;
                }
                RegisterID::HLminus => {
                    self.set_register_a(mem[self.hl as usize]);
                    self.hl += 1;
                }
                RegisterID::BC => self.set_register_a(mem[self.bc as usize]),
                RegisterID::DE => self.set_register_a(mem[self.de as usize]),
                _ => return Err(CpuError::ReadingFromInvalidReg { r: r2, pc: self.pc }),
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

            _ => return Err(CpuError::ReadingIntoInvalidReg { r: r1, pc: self.pc }),
        }

        Ok(())
    }

    pub fn load_registers8(
        &mut self,
        r1: RegisterID,
        r2: RegisterID,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        let new_val = self.r_table_lookup(r2, mem)?;
        self.r_table_assign(r1, new_val, mem)?;

        Ok(())
    }
}
