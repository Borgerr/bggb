use crate::cpu::{c_flag, CPU, nc_flag, nz_flag, z_flag};
use crate::cpu::instructions::FlagID;

impl CPU {
    pub fn jump(&mut self, nn: u16) {
        self.pc = nn;
    }

    pub fn jump_to_hl(&mut self) {
        self.pc = self.hl;
    }

    pub fn jump_conditional(&mut self, f: FlagID, nn: u16) {
        if match f {
            FlagID::C => c_flag(self.af),
            FlagID::NC => nc_flag(self.af),
            FlagID::Z => z_flag(self.af),
            FlagID::NZ => nz_flag(self.af),
        } {
            self.pc = nn;
        }
    }

    pub fn jump_reg(&mut self, d: i8) {
        let new_pc = ((self.pc as i32) + (d as i32)) as u16;
        self.pc = new_pc;
    }

    pub fn jump_reg_conditional(&mut self, f: FlagID, d: i8) {
        if match f {
            FlagID::C => c_flag(self.af),
            FlagID::NC => nc_flag(self.af),
            FlagID::Z => z_flag(self.af),
            FlagID::NZ => nz_flag(self.af),
        } {
            let new_pc = ((self.pc as i32) + (d as i32)) as u16;
            self.pc = new_pc;
        }
    }
}
