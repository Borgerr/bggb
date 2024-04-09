use crate::cpu::{c_flag, CPU, CpuError, hi_byte};
use crate::cpu::instructions::RegisterID;
use crate::memory::Memory;

impl CPU {
    pub fn rotate_left(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let r2 = r.clone();
        let mut n = self.r_table_lookup(r2, mem)?;
        if (n >> 7) != 0 {
            n <<= 1;
            n |= 0b1;
            self.set_carry_flag_on();
        } else {
            n <<= 1;
            self.set_carry_flag_off();
        }

        self.zero_flag_check(n);
        self.r_table_assign(r, n, mem)?;

        Ok(())
    }
    pub fn rotate_right(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let r2 = r.clone();
        let mut n = self.r_table_lookup(r2, mem)?;
        if (n & 0b1) != 0 {
            n >>= 1;
            n |= 0x80;
            self.set_carry_flag_on();
        } else {
            n >>= 1;
            self.set_carry_flag_off();
        }

        self.zero_flag_check(n);
        self.r_table_assign(r, n, mem)?;

        Ok(())
    }

    pub fn rotate_left_thru_carry(
        &mut self,
        r: RegisterID,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        let r2 = r.clone();
        let mut n = self.r_table_lookup(r2, mem)?;
        if (n >> 7) != 0 {
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        n <<= 1;
        if c_flag(self.af) {
            n |= 0b1;
        }

        self.zero_flag_check(n);
        self.r_table_assign(r, n, mem)?;

        Ok(())
    }
    pub fn rotate_right_thru_carry(
        &mut self,
        r: RegisterID,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        let r2 = r.clone();
        let mut n = self.r_table_lookup(r2, mem)?;
        if (n & 0b1) != 0 {
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        n >>= 1;
        if c_flag(self.af) {
            n |= 0x80;
        }

        self.zero_flag_check(n);
        self.r_table_assign(r, n, mem)?;

        Ok(())
    }

    pub fn shift_left_arithmetic(
        &mut self,
        r: RegisterID,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        let r2 = r.clone();
        let n = self.r_table_lookup(r2, mem)?;
        if (n >> 7) != 0 {
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        self.zero_flag_check(n << 1);
        self.r_table_assign(r, n << 1, mem)?;

        Ok(())
    }
    pub fn shift_right_arithmetic(
        &mut self,
        r: RegisterID,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        let r2 = r.clone();
        let n = self.r_table_lookup(r2, mem)? as i8;
        if (n & 0b1) != 0 {
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        let result = (n >> 1) as u8;
        self.zero_flag_check(result);
        self.r_table_assign(r, result, mem)?;

        Ok(())
    }

    pub fn swap_nibbles_instr(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let r2 = r.clone();
        let n = self.r_table_lookup(r2, mem)?;
        let nibble1 = n >> 4;
        let nibble2 = n & 0x0f;

        let result = (nibble2 << 4) | nibble1;
        self.zero_flag_check(result);
        self.r_table_assign(r, result, mem)?;

        Ok(())
    }

    pub fn shift_right_logical(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let r2 = r.clone();
        let n = self.r_table_lookup(r2, mem)?;
        if (n & 0b1) != 0 {
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        let result = n >> 1;
        self.zero_flag_check(result);
        self.r_table_assign(r, result, mem)?;

        Ok(())
    }

    pub fn complement_accumulator(&mut self) {
        let a = hi_byte(self.af);
        self.set_register_a(!a);
        self.set_subtraction_flag_on();
        self.set_halfcarry_flag_on();
    }

    pub fn rotate_right_accumulator(&mut self) {
        let mut a = hi_byte(self.af);
        let rightmost_on = a & 0b1 != 0;

        a >>= 1;
        if rightmost_on {
            self.set_carry_flag_on();
            a |= 0x80;
        } else {
            self.set_carry_flag_off();
        }

        self.set_register_a(a);
    }
    pub fn rotate_left_accumulator(&mut self) {
        let mut a = hi_byte(self.af);
        let leftmost_on = a & 0x80 != 0;

        a <<= 1;
        if leftmost_on {
            self.set_carry_flag_on();
            a |= 0b1;
        } else {
            self.set_carry_flag_off();
        }

        self.set_register_a(a);
    }

    pub fn rotate_right_thru_carry_accumulator(&mut self) {
        let carry_bit = if c_flag(self.af) { 0x80 } else { 0x0 };
        let mut a = hi_byte(self.af);
        if (a & 0b1) != 0 {
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        a = (a >> 1) | carry_bit;
        self.set_register_a(a);
    }
    pub fn rotate_left_thru_carry_accumulator(&mut self) {
        let carry_bit = if c_flag(self.af) { 0b1 } else { 0b0 };
        let mut a = hi_byte(self.af);
        if (a & 0x80) != 0 {
            self.set_carry_flag_on();
        } else {
            self.set_carry_flag_off();
        }

        a = (a << 1) | carry_bit;
        self.set_register_a(a);
    }
}
