use crate::cpu::{c_flag, CPU, CpuError, hi_byte};
use crate::cpu::instructions::RegisterID;
use crate::memory::Memory;

impl CPU {
    pub fn add_immediate(&mut self, n: u8) {
        let n = n as u16;
        let a = hi_byte(self.af) as u16;
        let result = n + a;

        self.add_flag_checks(result, n);
    }
    pub fn adc_immediate(&mut self, n: u8) {
        let n = n as u16;
        let a = hi_byte(self.af) as u16;
        let mut result = n + a;

        if c_flag(self.af) {
            result += 1;
        }

        self.add_flag_checks(result, n);
    }

    pub fn sub_immediate(&mut self, n: u8) {
        let n = n as i16;
        let a = hi_byte(self.af) as i16;
        let result = a - n;

        self.sub_flag_checks(result, n);
    }
    pub fn sbc_immediate(&mut self, n: u8) {
        let n = n as i16;
        let a = hi_byte(self.af) as i16;
        let mut result = a - n;

        if c_flag(self.af) {
            result -= 1;
        }

        self.sub_flag_checks(result, n);
    }

    pub fn compare_immediate(&mut self, n: u8) {
        // basically a sub immediate but not changing A
        let n = n as i16;
        let a = hi_byte(self.af) as i16;
        let result = a - n;

        self.sub_flag_checks(result, n);
        self.set_register_a(a as u8);
    }

    pub fn logical_template<F: Fn(u8, u8) -> u8>(&mut self, n: u8, op: F) {
        let a = hi_byte(self.af);
        let result = op(a, n);

        self.set_register_a(result as u8);
        self.zero_flag_check(result);
    }

    pub fn add_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.add_immediate(n);

        Ok(())
    }
    pub fn adc_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.adc_immediate(n);

        Ok(())
    }

    pub fn increment_8b(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let r2 = r.clone();
        let n = self.r_table_lookup(r2, mem)? as u16;
        let mut result = n + 1;

        if (n <= 0b00001000) && (result > 0b00001000) {
            self.set_halfcarry_flag_on();
        } else {
            self.set_halfcarry_flag_off();
        }

        if result > 0xff {
            result -= 0xff;
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        self.zero_flag_check(result as u8);
        self.set_subtraction_flag_off();

        self.r_table_assign(r, result as u8, mem)?;
        Ok(())
    }

    pub fn sub_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.sub_immediate(n);

        Ok(())
    }
    pub fn sbc_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.sbc_immediate(n);

        Ok(())
    }

    pub fn decrement_8b(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let r2 = r.clone();
        let n = self.r_table_lookup(r2, mem)? as i16;
        let mut result = n - 1;

        // copied from sub_flag_checks
        if (n >= 0b00010000) && (result < 0b00001000) {
            self.set_halfcarry_flag_on();
        } else {
            self.set_halfcarry_flag_off();
        }

        if result < 0 {
            result += 0xff;
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        self.set_subtraction_flag_on();
        self.zero_flag_check(result as u8);

        self.r_table_assign(r, result as u8, mem)?;
        Ok(())
    }

    pub fn compare_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.compare_immediate(n);

        Ok(())
    }

    pub fn logical_reg_template<F: Fn(u8, u8) -> u8>(
        &mut self,
        r: RegisterID,
        mem: &mut Memory,
        op: F,
    ) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.logical_template(n, op);

        Ok(())
    }

    pub fn decrement_16b(&mut self, r: RegisterID) -> Result<(), CpuError> {
        let r2 = r.clone();
        let mut val = self.rp_table_lookup(r2)? as i32;
        val -= 1;
        if val < 0 {
            // check for underflow
            val += 0xffff;
        }
        self.rp_table_assign(r, val as u16)?;

        Ok(())
    }

    pub fn increment_16b(&mut self, r: RegisterID) -> Result<(), CpuError> {
        let r2 = r.clone();
        let mut val = self.rp_table_lookup(r2)? as u32;
        val += 1;
        if val > 0xffff {
            // check for overflow
            val -= 0xffff;
        }
        self.rp_table_assign(r, val as u16)?;

        Ok(())
    }

    pub fn add_hl_and_r16(&mut self, r: RegisterID) -> Result<(), CpuError> {
        let r2 = r.clone();
        let hl = self.hl as u32;
        let r16 = self.rp_table_lookup(r2)? as u32;
        let mut result = hl + r16;

        if result > 0xffff {
            result -= 0xffff;
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        self.set_subtraction_flag_off(); // N -> 0

        // check for overflow from bit 11
        let bit_11 = 0b1 << 12;
        if (hl < bit_11) && (result >= bit_11) {
            self.set_halfcarry_flag_on();
        } else {
            self.set_halfcarry_flag_off();
        }

        self.hl = result as u16;
        Ok(())
    }
}
