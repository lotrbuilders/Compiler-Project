use crate::backend::register_allocation::RegisterInterface;

use std::fmt::Display;
use RegisterLocation::*;

#[derive(Debug, Clone, Copy)]
pub enum RegisterLocation<R: RegisterInterface> {
    Reg(R),
    Vreg(u32),
    NotAllocated,
}

//pub const NOT_ALLOCATED<R>: Option<R> = None;

impl<R: RegisterInterface> RegisterLocation<R> {
    pub const NOT_ALLOCATED: Option<R> = None;
    pub fn reg(&self) -> Option<R> {
        match self {
            Reg(reg) => Some(*reg),
            _ => None,
        }
    }
}

impl<R: RegisterInterface> From<RegisterLocation<R>> for Option<R> {
    fn from(reg: RegisterLocation<R>) -> Self {
        match reg {
            Reg(register) => Some(register),
            _ => None,
        }
    }
}

impl<R: RegisterInterface> From<RegisterLocation<R>> for Option<u32> {
    fn from(reg: RegisterLocation<R>) -> Self {
        match reg {
            Vreg(vreg) => Some(vreg),
            _ => None,
        }
    }
}

impl<R: RegisterInterface + Display> Display for RegisterLocation<R> {
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
