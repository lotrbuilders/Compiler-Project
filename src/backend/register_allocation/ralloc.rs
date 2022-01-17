use std::fmt::Display;
use std::ops::Range;

use crate::backend::register_allocation::RegisterInterface;

use super::RegisterClass;
use super::{RegisterAllocation, RegisterLocation};

// A vector of this is added to the instruction
// Shows operation that need to happen to make modifications to the register file
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RegisterRelocation<R: RegisterInterface> {
    MemMove(u32, u32, R), //from to using
    Move(R, R),           // from to
    MoveAfter(R, R),      // from to
    TwoAddressMove(R, R), // from to
    Spill(R, u32),
    SpillEarly(R, u32), // Only happens at jump instructions.
    Reload(R, u32),
    ReloadTemp(R, u32), // Reload temp is currently still the same as reload: Should be removed again after reloading
    Jump(Vec<RegisterLocation<R>>),
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

use RegisterLocation::*;
#[allow(dead_code)]
impl<R: RegisterInterface + Display> RegisterAssignment<R> {
    pub fn try_allocate(&mut self, class: &RegisterClass<R>, vreg: u32, index: u32) -> Option<R> {
        match try_allocate2(class) {
            Some(&reg) => {
                assign_register(reg, vreg, self, index);
                Some(reg)
            }
            _ => None,
        }
    }

    pub fn _force_allocate(
        &mut self,
        _register_use: &RegisterUse<R>,
        _vreg: u32,
        _class: &RegisterClass<R>,
    ) -> R {
        unreachable!()
    }

    pub fn try_reload(
        &mut self,
        //register_use: RegisterUse,
        index: u32,
        vreg: u32,
        class: &RegisterClass<R>,
    ) -> bool {
        if let Some(&reg) = try_allocate2(class) {
            log::debug!("Reloading {} to {} at {}", vreg, reg, index);
            self.reg_relocations[index as usize].push(RegisterRelocation::ReloadTemp(reg, vreg));
            self.reg_occupied_by[<R as Into<usize>>::into(reg)] = Some(vreg);
            self.vreg2reg[vreg as usize] = Reg(reg);
            true
        } else {
            log::debug!("No register available for reload of {} at {}", vreg, index);
            false
        }
    }

    pub fn force_reload(
        &mut self,
        register_use: &RegisterUse<R>,
        index: u32,
        vreg: u32,
        class: &RegisterClass<R>,
    ) {
        if !self.try_reload(index, vreg, class) {
            let _reg = self.spill_last(register_use, index, vreg, class);
            self.try_reload(index, vreg, class);
        }
    }

    pub fn spill_last(
        &mut self,
        register_use: &RegisterUse<R>,
        index: u32,
        vreg: u32,
        class: &RegisterClass<R>,
    ) -> R {
        let mut furthest_use = 0u32;
        let mut furthest_vreg = u32::MAX;
        for vreg in self.reg_occupied_by.iter().filter_map(|reg| *reg) {
            if class[self.vreg2reg[vreg as usize].reg().unwrap()] {
                let next_use = register_use.uses[index as usize]
                    .iter()
                    .find(|&&i| i > index)
                    .expect("No registers available to spill");
                if *next_use > furthest_use {
                    furthest_use = *next_use;
                    furthest_vreg = vreg;
                }
            }
        }
        let reg = self.vreg2reg[vreg as usize].reg().unwrap();
        self.spill(index, reg, furthest_vreg);
        reg
    }

    pub fn spill(&mut self, index: u32, reg: R, vreg: u32) {
        self.reg_relocations[index as usize].push(RegisterRelocation::Spill(reg, vreg));
        self.allocation[vreg as usize].end_prev(index);
        //self.allocation[vreg as usize].start(Vreg(0), index);
        self.vreg2reg[vreg as usize] = Vreg(0); //TODO!!
        self.reg_occupied_by[<R as Into<usize>>::into(reg)] = None;
    }

    pub fn two_address_move(&mut self, index: u32, from: R, to: R) {
        self.reg_relocations[index as usize].push(RegisterRelocation::TwoAddressMove(from, to));
    }
}

pub fn try_allocate2<'a, R: RegisterInterface, T: IntoIterator<Item = &'a R> + 'a>(
    class: T,
) -> Option<&'a R> {
    class.into_iter().next()
}

pub fn assign_register<R: RegisterInterface + Display>(
    reg: R,
    vreg: u32,
    assignments: &mut RegisterAssignment<R>,
    index: u32,
) {
    log::trace!("Using register {} for vreg {}", reg.to_string(), vreg);
    assignments.reg_occupied_by[<R as Into<usize>>::into(reg)] = Some(vreg);
    assignments.vreg2reg[vreg as usize] = Reg(reg);
    assignments.allocation[vreg as usize].start(reg, index);
}
