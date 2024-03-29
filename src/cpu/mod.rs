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

fn z_flag(af: u16) -> bool {
    ((af >> 6) & 0b1) == 0b1
}
fn nz_flag(af: u16) -> bool {
    !z_flag(af)
}

fn c_flag(af: u16) -> bool {
    ((af >> 3) & 0b1) == 0b1
}
fn nc_flag(af: u16) -> bool {
    !c_flag(af)
}

fn hi_byte(x: u16) -> u8 {
    (x >> 8) as u8
}
fn lo_byte(x: u16) -> u8 {
    (x & 0xff) as u8
}

struct CPU {
    af: u16, // accumulator & flags
    bc: u16, // BC register
    de: u16, // DE register
    hl: u16, // HL register
    sp: u16, // stack pointer
    pc: u16, // program counter/pointer

    interrupts_enabled: bool,
}

impl CPU {
    fn set_carry_flag_on(&mut self) {
        let flags = lo_byte(self.af) | 0b1000;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }
    fn set_carry_flag_off(&mut self) {
        let flags = lo_byte(self.af) & 0b0111;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }
    fn complement_carry_flag(&mut self) {
        if c_flag(self.af) {
            self.set_carry_flag_off();
        } else {
            self.set_carry_flag_on();
        }
    }

    fn set_zero_flag_on(&mut self) {
        let flags = lo_byte(self.af) | 0b1000000;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }
    fn set_zero_flag_off(&mut self) {
        let flags = lo_byte(self.af) & 0b0111111;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }

    fn set_halfcarry_flag_on(&mut self) {
        let flags = lo_byte(self.af) | 0b10000;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }
    fn set_halfcarry_flag_off(&mut self) {
        let flags = lo_byte(self.af) & 0b01111;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }

    fn set_subtraction_flag_on(&mut self) {
        let flags = lo_byte(self.af) | 0b100000;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }
    fn set_subtraction_flag_off(&mut self) {
        let flags = lo_byte(self.af) & 0b011111;
        self.af &= 0xff00;
        self.af |= flags as u16;
    }

    fn reset_flags(&mut self) {
        self.af &= 0xff00;
    }

    fn registerid_to_u16(&mut self, r: RegisterID) -> u16 {
        match r {
            RegisterID::AF => self.af,
            RegisterID::BC => self.bc,
            RegisterID::DE => self.de,
            RegisterID::HL | RegisterID::HLplus | RegisterID::HLminus | RegisterID::HLaddress => {
                self.hl
            }
            RegisterID::SP => self.sp,

            RegisterID::A => hi_byte(self.af) as u16,
            RegisterID::B => hi_byte(self.bc) as u16,
            RegisterID::C => lo_byte(self.bc) as u16,
            RegisterID::D => hi_byte(self.de) as u16,
            RegisterID::E => lo_byte(self.de) as u16,
            RegisterID::H => hi_byte(self.hl) as u16,
            RegisterID::L => lo_byte(self.hl) as u16,
        }
    }

    /*
       again, the following lookups and assigns come from the following link
       https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
    */
    fn rp_table_lookup(&mut self, r: RegisterID) -> Result<u16, CpuError> {
        let result = match r {
            RegisterID::BC => self.bc,
            RegisterID::DE => self.de,
            RegisterID::HL => self.hl,
            RegisterID::SP => self.sp,

            _ => return Err(CpuError::ReadingFromInvalidReg { r, pc: self.pc }),
        };

        Ok(result)
    }
    fn rp_table_assign(&mut self, r: RegisterID, val: u16) -> Result<(), CpuError> {
        match r {
            RegisterID::BC => self.bc = val,
            RegisterID::DE => self.de = val,
            RegisterID::HL => self.hl = val,
            RegisterID::SP => self.sp = val,

            _ => return Err(CpuError::ReadingIntoInvalidReg { r, pc: self.pc }),
        }

        Ok(())
    }

    fn r_table_lookup(&mut self, r: RegisterID, mem: &mut Memory) -> Result<u8, CpuError> {
        let result = match r {
            RegisterID::A => hi_byte(self.af),
            RegisterID::B => hi_byte(self.bc),
            RegisterID::C => lo_byte(self.bc),
            RegisterID::D => hi_byte(self.de),
            RegisterID::E => lo_byte(self.de),
            RegisterID::H => hi_byte(self.hl),
            RegisterID::L => lo_byte(self.hl),
            RegisterID::HLaddress => mem[self.hl as usize],

            _ => return Err(CpuError::ReadingFromInvalidReg { r, pc: self.pc }),
        };

        Ok(result)
    }

    fn r_table_assign(&mut self, r: RegisterID, val: u8, mem: &mut Memory) -> Result<(), CpuError> {
        match r {
            RegisterID::A => self.set_register_a(val),
            RegisterID::B => {
                self.bc &= 0x00ff;
                self.bc |= (val as u16) << 8;
            }
            RegisterID::C => {
                self.bc &= 0xff00;
                self.bc |= val as u16;
            }
            RegisterID::D => {
                self.de &= 0x00ff;
                self.de |= (val as u16) << 8;
            }
            RegisterID::E => {
                self.de &= 0xff00;
                self.de |= val as u16;
            }
            RegisterID::H => {
                self.hl &= 0x00ff;
                self.hl |= (val as u16) << 8;
            }
            RegisterID::L => {
                self.hl &= 0xff00;
                self.hl |= val as u16;
            }
            RegisterID::HLaddress => mem[self.hl as usize] = val,

            _ => return Err(CpuError::ReadingIntoInvalidReg { r, pc: self.pc }),
        }

        Ok(())
    }

    fn set_register_a(&mut self, new_val: u8) {
        self.af &= 0x00ff;
        self.af |= (new_val as u16) << 8;
    }

    pub fn fetch_decode_execute(&mut self, mem: &mut Memory) -> Result<(), CpuError> {
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
            let result = mem[self.pc as usize];
            self.pc += 1;
            Ok(result)
        }
    }
    fn fetch_instr_u32(&mut self, mem: &Memory) -> Result<u32, CpuError> {
        // always increments program counter by 3
        Ok(((self.fetch_pc_u8(mem)? as u32) << 24)
            | ((self.fetch_pc_u8(mem)? as u32) << 16)
            | ((self.fetch_pc_u8(mem)? as u32) << 8))
    }

    fn execute(&mut self, instr: Instruction, mem: &mut Memory) -> Result<(), CpuError> {
        // This function is kind of smelly since it decreases the program counter after determining the instruction
        // but the motivation is that instructions should store data critical to the operation,
        // and do that in one call to `Instruction::from_bytes`.
        // The other alternative would be to repeatedly call that
        // and potentially lose information within the opcodes themselves
        match instr {
            Instruction::NOP => self.pc -= 2, // no operation
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
                self.load_immediate8(r, n, mem)?;
            }
            Instruction::LoadFF00PlusImmediate { n } => {
                self.pc -= 1;
                self.load_ff00_plus_n(mem, n);
            }
            Instruction::LoadReg16 { r1, r2 } => {
                self.pc -= 2;
                self.load_registers16(r1, r2, mem)?;
            }
            Instruction::LoadReg8 { r1, r2 } => {
                self.pc -= 2;
                self.load_registers8(r1, r2, mem)?;
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

            // really no need to change the program counter prior to some jumps
            Instruction::Jump { nn } => self.jump(nn),
            Instruction::JumpConditional { f, nn } => self.jump_conditional(f, nn),
            Instruction::JR { d } => self.jump_reg(d),
            Instruction::JumpRegConditional { f, d } => {
                self.pc -= 1;
                self.jump_reg_conditional(f, d);
            }
            Instruction::JumpToHL => self.jump_to_hl(),

            // CB-prefixed
            Instruction::RLC { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.rotate_left(r, mem)?;
            }
            Instruction::RRC { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.rotate_right(r, mem)?;
            }
            Instruction::RL { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.rotate_left_thru_carry(r, mem)?;
            }
            Instruction::RR { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.rotate_right_thru_carry(r, mem)?;
            }
            Instruction::SLA { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.shift_left_arithmetic(r, mem)?;
            }
            Instruction::SRA { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.shift_right_arithmetic(r, mem)?;
            }
            Instruction::SWAP { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.swap_nibbles_instr(r, mem)?;
            }
            Instruction::SRL { r } => {
                self.pc -= 1;
                self.reset_flags();
                self.shift_right_logical(r, mem)?;
            }

            // also CB-prefixed (y is included in opcode)
            Instruction::BIT { y, r } => self.pc -= 1,
            Instruction::RES { y, r } => self.pc -= 1,
            Instruction::SET { y, r } => self.pc -= 1,

            Instruction::AddRegisters { r1, r2 } => self.pc -= 2,
            Instruction::AddSigned { r, d } => self.pc -= 1,

            Instruction::DEC8b { r } => {
                self.pc -= 2;
                self.decrement_8b(r, mem)?;
            }
            Instruction::INC8b { r } => {
                self.pc -= 2;
                self.increment_8b(r, mem)?;
            }

            Instruction::DEC16b { r } => {
                self.pc -= 2;
                self.decrement_16b(r)?;
            }
            Instruction::INC16b { r } => {
                self.pc -= 2;
                self.increment_16b(r)?;
            }
            Instruction::AddHLAndR16 { r } => {
                self.pc -= 2;
                self.add_hl_and_r16(r)?;
            }

            Instruction::RLCA => {
                self.pc -= 2;
                self.rotate_left_accumulator();
            }
            Instruction::RRCA => {
                self.pc -= 2;
                self.rotate_right_accumulator();
            }
            Instruction::RLA => {
                self.pc -= 2;
                self.rotate_left_thru_carry_accumulator();
            }
            Instruction::RRA => {
                self.pc -= 2;
                self.rotate_right_thru_carry_accumulator();
            }
            Instruction::DAA => self.pc -= 2,
            Instruction::CPL => {
                self.pc -= 2;
                self.complement_accumulator();
            }
            Instruction::SCF => {
                self.pc -= 2;
                self.set_carry_flag_on();
            }
            Instruction::CCF => {
                self.pc -= 2;
                self.complement_carry_flag();
            }

            Instruction::RET { f } => self.pc -= 2,

            Instruction::RETNoParam => self.pc -= 2,
            Instruction::RETI => self.pc -= 2,

            Instruction::POP { r } => self.pc -= 2,

            // interrupts
            Instruction::DI => {
                self.pc -= 2;
                self.disable_interrupts();
            }
            Instruction::EI => {
                self.pc -= 2;
                self.enable_interrupts();
            }
            Instruction::STOP => self.pc -= 2, // low power standby mode
            Instruction::HALT => self.pc -= 1, // halt until interrupt occurs... somehow.
            // TODO: research STOP and HALT instructions
            Instruction::CallConditional { f, nn } => self.pc -= 0,
            Instruction::Call { nn } => self.pc -= 0,

            Instruction::PUSH { r } => self.pc -= 2,

            Instruction::RST { arg } => self.pc -= 2, // "arg" is included in opcode

            Instruction::AddImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.add_immediate(n);
            }
            Instruction::AdcImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.adc_immediate(n);
            }
            Instruction::SubImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.sub_immediate(n);
            }
            Instruction::SbcImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.sbc_immediate(n);
            }
            Instruction::AndImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.logical_template(n, |n1, n2| n1 & n2);
                self.set_halfcarry_flag_on(); // AND sets half-carry on
            }
            Instruction::XorImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.logical_template(n, |n1, n2| n1 ^ n2);
            }
            Instruction::OrImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.logical_template(n, |n1, n2| n1 | n2);
            }
            Instruction::CpImmediate { n } => {
                self.pc -= 1;
                self.reset_flags();
                self.compare_immediate(n);
            }

            Instruction::AddRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.add_register(r, mem)?;
            }
            Instruction::AdcRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.adc_register(r, mem)?;
            }
            Instruction::SubRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.sub_register(r, mem)?;
            }
            Instruction::SbcRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.sbc_register(r, mem)?;
            }
            Instruction::AndRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.logical_reg_template(r, mem, |n1, n2| n1 & n2)?;
            }
            Instruction::XorRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.logical_reg_template(r, mem, |n1, n2| n1 ^ n2)?;
            }
            Instruction::OrRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.logical_reg_template(r, mem, |n1, n2| n1 | n2)?;
            }
            Instruction::CpRegister { r } => {
                self.pc -= 2;
                self.reset_flags();
                self.compare_register(r, mem)?;
            }
        }

        Ok(())
    }

    /*
       instructions
    */
    fn jump(&mut self, nn: u16) {
        self.pc = nn;
    }

    fn jump_to_hl(&mut self) {
        self.pc = self.hl;
    }

    fn jump_conditional(&mut self, f: FlagID, nn: u16) {
        if match f {
            FlagID::C => c_flag(self.af),
            FlagID::NC => nc_flag(self.af),
            FlagID::Z => z_flag(self.af),
            FlagID::NZ => nz_flag(self.af),
        } {
            self.pc = nn;
        }
    }

    fn jump_reg(&mut self, d: i8) {
        let new_pc = ((self.pc as i32) + (d as i32)) as u16;
        self.pc = new_pc;
    }

    fn jump_reg_conditional(&mut self, f: FlagID, d: i8) {
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

    fn load_immediate16(&mut self, r: RegisterID, nn: u16) -> Result<(), CpuError> {
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

    fn load_immediate8(&mut self, r: RegisterID, n: u8, mem: &mut Memory) -> Result<(), CpuError> {
        self.r_table_assign(r, n, mem)?;

        Ok(())
    }

    fn load_sp_to_hl_with_offset(&mut self, mem: &Memory, d: i8) -> Result<(), CpuError> {
        let index = ((self.sp as i32) + (d as i32)) as usize;
        if index > 0xffff {
            return Err(CpuError::IndexOutOfBounds { index, pc: self.pc });
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
        self.af &= mem[(0xff00) + lo_byte(self.bc) as usize] as u16;
    }

    fn load_registers16(
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

    fn load_registers8(
        &mut self,
        r1: RegisterID,
        r2: RegisterID,
        mem: &mut Memory,
    ) -> Result<(), CpuError> {
        let new_val = self.r_table_lookup(r2, mem)?;
        self.r_table_assign(r1, new_val, mem)?;

        Ok(())
    }

    fn enable_interrupts(&mut self) {
        self.interrupts_enabled = true;
    }
    fn disable_interrupts(&mut self) {
        self.interrupts_enabled = false;
    }

    fn sub_flag_checks(&mut self, mut result: i16, prev_val: i16) {
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
    fn add_flag_checks(&mut self, mut result: u16, prev_val: u16) {
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
    fn zero_flag_check(&mut self, result: u8) {
        if result == 0 {
            self.set_zero_flag_on();
        } else {
            self.set_zero_flag_off();
        }
    }

    fn add_immediate(&mut self, n: u8) {
        let n = n as u16;
        let a = hi_byte(self.af) as u16;
        let result = n + a;

        self.add_flag_checks(result, n);
    }
    fn adc_immediate(&mut self, n: u8) {
        let n = n as u16;
        let a = hi_byte(self.af) as u16;
        let mut result = n + a;

        if c_flag(self.af) {
            result += 1;
        }

        self.add_flag_checks(result, n);
    }

    fn sub_immediate(&mut self, n: u8) {
        let n = n as i16;
        let a = hi_byte(self.af) as i16;
        let result = a - n;

        self.sub_flag_checks(result, n);
    }
    fn sbc_immediate(&mut self, n: u8) {
        let n = n as i16;
        let a = hi_byte(self.af) as i16;
        let mut result = a - n;

        if c_flag(self.af) {
            result -= 1;
        }

        self.sub_flag_checks(result, n);
    }

    fn compare_immediate(&mut self, n: u8) {
        // basically a sub immediate but not changing A
        let n = n as i16;
        let a = hi_byte(self.af) as i16;
        let result = a - n;

        self.sub_flag_checks(result, n);
        self.set_register_a(a as u8);
    }

    fn logical_template<F: Fn(u8, u8) -> u8>(&mut self, n: u8, op: F) {
        let a = hi_byte(self.af);
        let result = op(a, n);

        self.set_register_a(result as u8);
        self.zero_flag_check(result);
    }

    fn add_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.add_immediate(n);

        Ok(())
    }
    fn adc_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.adc_immediate(n);

        Ok(())
    }

    fn increment_8b(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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

    fn sub_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.sub_immediate(n);

        Ok(())
    }
    fn sbc_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.sbc_immediate(n);

        Ok(())
    }

    fn decrement_8b(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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

    fn compare_register(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.compare_immediate(n);

        Ok(())
    }

    fn logical_reg_template<F: Fn(u8, u8) -> u8>(
        &mut self,
        r: RegisterID,
        mem: &mut Memory,
        op: F,
    ) -> Result<(), CpuError> {
        let n = self.r_table_lookup(r, mem)?;
        self.logical_template(n, op);

        Ok(())
    }

    fn rotate_left(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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
    fn rotate_right(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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

    fn rotate_left_thru_carry(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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
    fn rotate_right_thru_carry(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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

    fn shift_left_arithmetic(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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
    fn shift_right_arithmetic(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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

    fn swap_nibbles_instr(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
        let r2 = r.clone();
        let n = self.r_table_lookup(r2, mem)?;
        let nibble1 = n >> 4;
        let nibble2 = n & 0x0f;

        let result = (nibble2 << 4) | nibble1;
        self.zero_flag_check(result);
        self.r_table_assign(r, result, mem)?;

        Ok(())
    }

    fn shift_right_logical(&mut self, r: RegisterID, mem: &mut Memory) -> Result<(), CpuError> {
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

    fn complement_accumulator(&mut self) {
        let a = hi_byte(self.af);
        self.set_register_a(!a);
        self.set_subtraction_flag_on();
        self.set_halfcarry_flag_on();
    }

    fn rotate_right_accumulator(&mut self) {
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
    fn rotate_left_accumulator(&mut self) {
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

    fn rotate_right_thru_carry_accumulator(&mut self) {
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
    fn rotate_left_thru_carry_accumulator(&mut self) {
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

    fn decrement_16b(&mut self, r: RegisterID) -> Result<(), CpuError> {
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

    fn increment_16b(&mut self, r: RegisterID) -> Result<(), CpuError> {
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

    fn add_hl_and_r16(&mut self, r: RegisterID) -> Result<(), CpuError> {
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
