mod instructions;
use instructions::Instruction;

use crate::memory::Memory;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CpuError {
    FetchError { pc: u16 },
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
            Instruction::ILLEGAL => {}

            Instruction::Load16 { r, nn } => {}
            Instruction::Load8 { r, n } => {}
            Instruction::LoadFF00Plus { r, n } => {}
            Instruction::LoadReg { r1, r2 } => {}
            Instruction::LoadSPToHLWithOffset { d } => {}
            Instruction::LoadFF00PlusC => {}

            Instruction::StoreFF00Plus { r, n } => {}
            Instruction::StoreReg { r1, loc } => {}
            Instruction::StoreImmediate { loc } => {}
            Instruction::StoreFF00PlusC => {}

            Instruction::Jump { nn } => {}
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
}
