use std::fmt::Display;
use thiserror::Error;

use super::instructions::RegisterID;

#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum CpuError {
    FetchError { pc: u16 },
    IllegalInstruction { pc: u16 },
    ReadingIntoInvalidReg { r: RegisterID, pc: u16 },
    ReadingFromInvalidReg { r: RegisterID, pc: u16 },
    IndexOutOfBounds { index: usize, pc: u16 },
}

impl Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FetchError { pc } => write!(f, "Error fetching instruction at pc={}", pc),
            Self::IllegalInstruction { pc } => write!(f, "Illegal instruction at pc={}", pc),
            Self::ReadingFromInvalidReg { r, pc } => write!(
                f,
                "Attempt to read from invalid register r={} at pc={}",
                r, pc
            ),

            Self::ReadingIntoInvalidReg { r, pc } => write!(
                f,
                "Attempt to read into invalid register r={} at pc={}",
                r, pc
            ),
            Self::IndexOutOfBounds { index, pc } => {
                write!(f, "Index out of bounds, index={}, pc={}", index, pc)
            }
        }
    }
}
