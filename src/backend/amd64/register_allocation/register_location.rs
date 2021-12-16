use super::super::registers::Register;
use std::fmt::Display;
use RegisterLocation::*;

#[derive(Debug, Clone, Copy)]
pub enum RegisterLocation {
    Reg(Register),
    Vreg(u32),
    NotAllocated,
}

pub const NOT_ALLOCATED: Option<Register> = None;

impl RegisterLocation {
    pub fn reg(&self) -> Option<Register> {
        match self {
            Reg(reg) => Some(*reg),
            _ => None,
        }
    }
}

impl From<RegisterLocation> for Option<Register> {
    fn from(reg: RegisterLocation) -> Self {
        match reg {
            Reg(register) => Some(register),
            _ => None,
        }
    }
}

impl From<RegisterLocation> for Option<u32> {
    fn from(reg: RegisterLocation) -> Self {
        match reg {
            Vreg(vreg) => Some(vreg),
            _ => None,
        }
    }
}

impl Display for RegisterLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg(reg) => {
                if let Some(precision) = f.precision() {
                    write!(f, "{:.precision$}", reg, precision = precision)
                } else {
                    write!(f, "{}", reg)
                }
            }
            Vreg(vreg) => write!(f, "[{}]", vreg),
            NotAllocated => write!(f, "-"),
        }
    }
}
