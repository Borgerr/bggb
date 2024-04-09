use crate::cpu::CPU;

impl CPU {
    pub fn enable_interrupts(&mut self) {
        self.interrupts_enabled = true;
    }
    pub fn disable_interrupts(&mut self) {
        self.interrupts_enabled = false;
    }

    pub fn sub_flag_checks(&mut self, mut result: i16, prev_val: i16) {
        if (prev_val >= 0b00010000) && (result < 0b00010000) {
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
        self.set_register_a(result as u8);
    }
    pub fn add_flag_checks(&mut self, mut result: u16, prev_val: u16) {
        if (prev_val <= 0b00001111) && (result >= 0b00010000) {
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
        self.set_register_a(result as u8);
    }
    pub fn zero_flag_check(&mut self, result: u8) {
        if result == 0 {
            self.set_zero_flag_on();
        } else {
            self.set_zero_flag_off();
        }
    }
}
