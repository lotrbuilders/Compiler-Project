use std::ops::Range;

use crate::backend::register_allocation::RegisterInterface;
use crate::ir::ir::IRSize;

use super::RegisterClass;
use super::{RegisterAllocation, RegisterLocation};

// A vector of this is added to the instruction
// Shows operation that need to happen to make modifications to the register file
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RegisterRelocation<R: RegisterInterface> {
    MemMove(IRSize, u32, u32, R), //from to using
    Move(IRSize, R, R),           // from to
    MoveAfter(IRSize, R, R),      // from to
    TwoAddressMove(IRSize, R, R), // from to
    Spill(IRSize, R, u32),
    SpillEarly(IRSize, R, u32), // Only happens at jump instructions.
    Reload(IRSize, R, u32),
}

impl<R: RegisterInterface> RegisterRelocation<R> {
    pub fn after(&self) -> bool {
        matches!(self, Self::MoveAfter(..) | Self::Spill(..))
    }
}

pub struct RegisterUse<R: RegisterInterface + 'static> {
    pub creation: Vec<u32>,
    pub uses: Vec<Vec<u32>>,
    pub last_use: Vec<u32>,
    pub preferred_class: Vec<RegisterClass<R>>,
}

#[derive(Debug, Clone)]
pub struct RegisterRange<R: RegisterInterface> {
    pub loc: Option<R>,
    pub range: Range<u32>,
}

impl<R: RegisterInterface> RegisterRange<R> {
    pub fn new(loc: R, range: Range<u32>) -> RegisterRange<R> {
        RegisterRange {
            loc: Some(loc),
            range,
        }
    }
}

pub struct RegisterAssignment<R: RegisterInterface> {
    pub reg_occupied_by: Vec<Option<u32>>,
    pub vreg2reg: Vec<RegisterLocation<R>>,
    pub allocation: Vec<RegisterAllocation<R>>,
    pub reg_relocations: Vec<Vec<RegisterRelocation<R>>>,
}

#[allow(dead_code)]
impl<R: RegisterInterface> RegisterAssignment<R> {
    // Registers that are in use at the start of the function
    pub fn in_use_registers(&self) -> Vec<R> {
        self.reg_occupied_by
            .iter()
            .filter_map(|&vreg| vreg)
            .map(|vreg| self.vreg2reg[vreg as usize].reg().unwrap())
            .collect()
    }

    pub fn _now_used_registers(&self) -> RegisterClass<R> {
        todo!()
        //REG_CLASS_IREG.clone()
    }

    // Registers that are used last in this instruction
    pub fn final_use_registers(&self, register_use: &RegisterUse<R>, index: u32) -> Vec<R> {
        self.vreg2reg
            .iter()
            .zip(register_use.last_use.iter())
            .filter_map(|(reg, last_use)| reg.reg().zip(Some(*last_use)))
            .filter(|(_reg, last_use)| *last_use == index)
            .map(|(reg, _last_use)| reg)
            .collect()
    }
}

pub fn try_allocate2<'a, R: RegisterInterface, T: IntoIterator<Item = &'a R> + 'a>(
    class: T,
) -> Option<&'a R> {
    class.into_iter().next()
}
