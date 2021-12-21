use std::ops::Range;

use crate::backend::ir::IRInstruction;

//use self::RegisterLocation::*;
use super::super::registers::*;
use super::super::BackendAMD64;
use super::RegisterClass;
use super::{RegisterAllocation, RegisterLocation};

// A vector of this is added to the instruction
// Shows operation that need to happen to make modifications to the register file
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RegisterRelocation {
    MemMove(u32, u32, Register),        //from to
    Move(Register, Register),           // from to
    TwoAddressMove(Register, Register), // from to
    Spill(Register, u32),
    Reload(Register, u32),
    ReloadTemp(Register, u32), // Reload temp is currently still the same as reload: Should be removed again after reloading
    Jump(Vec<RegisterLocation>),
}

pub struct RegisterUse {
    pub creation: Vec<u32>,
    pub uses: Vec<Vec<u32>>,
    pub last_use: Vec<u32>,
    pub preferred_class: Vec<&'static RegisterClass>,
}

#[derive(Debug, Clone)]
pub struct RegisterRange {
    pub loc: Option<Register>,
    pub range: Range<u32>,
}

impl RegisterRange {
    pub fn new(loc: Register, range: Range<u32>) -> RegisterRange {
        RegisterRange {
            loc: Some(loc),
            range,
        }
    }
}

pub struct RegisterAssignment {
    pub reg_occupied_by: [Option<u32>; REG_COUNT],
    pub vreg2reg: Vec<RegisterLocation>,
    pub allocation: Vec<RegisterAllocation>,
    pub reg_relocations: Vec<Vec<RegisterRelocation>>,
}

pub struct RegisterAllocatorSimple {}
pub struct RegisterAllocatorLinear {}
pub trait RegisterAllocator {
    fn allocate_registers(backend: &mut BackendAMD64) -> ();
}

impl BackendAMD64 {
    pub fn find_uses(&mut self) -> RegisterUse {
        let length = self.definition_index.len();
        let mut creation = vec![u32::MAX; length];
        let mut uses = vec![Vec::new(); length];
        let mut last_use = vec![0u32; length];
        let mut preferred_class: Vec<&'static RegisterClass> = vec![&REG_CLASS_IREG; length];

        for arg in self.arguments.arguments.iter().filter_map(|arg| *arg) {
            creation[arg as usize] = 0;
        }

        for i in (1..self.instructions.len()).rev() {
            let rule = self.rules[i];
            if self.is_instruction(rule) {
                let (used_vreg, result_vreg) = self.get_vregisters(i as u32, rule);

                if let Some((vreg, _)) = result_vreg {
                    creation[vreg as usize] = i as u32;
                }
                for (vreg, class) in used_vreg {
                    uses[vreg as usize].push(i as u32);
                    if last_use[vreg as usize] == 0 {
                        last_use[vreg as usize] = i as u32;
                    }
                    if class != &REG_CLASS_IREG {
                        preferred_class[vreg as usize] = &class;
                    }
                }
            }
            if let IRInstruction::Phi(phi) = &self.instructions[i] {
                for target in &phi.targets {
                    creation[*target as usize] = i as u32;
                }
                for source in phi.sources.iter().flat_map(|src| src.iter()) {
                    let vreg = *source as usize;
                    uses[vreg].push(i as u32);
                    assert_eq!(last_use[vreg], 0);
                    last_use[vreg] = i as u32;
                }
            }
        }
        RegisterUse {
            creation,
            last_use,
            uses,
            preferred_class,
        }
    }
    pub fn get_clobbered(&self, index: u32) -> RegisterClass {
        self.clobber(index as usize)
            .iter()
            .collect::<RegisterClass>()
    }
}

impl RegisterAssignment {
    // Registers that are in use at the start of the function
    pub fn in_use_registers(&self) -> RegisterClass {
        self.reg_occupied_by
            .iter()
            .filter_map(|&vreg| vreg)
            .map(|vreg| self.vreg2reg[vreg as usize].reg().unwrap())
            .collect()
    }

    pub fn _now_used_registers(&self) -> RegisterClass {
        REG_CLASS_IREG.clone()
    }

    // Registers that are used last in this instruction
    pub fn final_use_registers(&self, register_use: &RegisterUse, index: u32) -> RegisterClass {
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
impl RegisterAssignment {
    pub fn try_allocate(
        &mut self,
        class: &RegisterClass,
        vreg: u32,
        index: u32,
    ) -> Option<Register> {
        match try_allocate2(class) {
            Some(reg) => {
                assign_register(reg, vreg, self, index);
                Some(reg)
            }
            _ => None,
        }
    }

    pub fn _force_allocate(
        &mut self,
        _register_use: &RegisterUse,
        _vreg: u32,
        _class: &RegisterClass,
    ) -> Register {
        Register::Rax
    }

    pub fn try_reload(
        &mut self,
        //register_use: RegisterUse,
        index: u32,
        vreg: u32,
        class: &RegisterClass,
    ) -> bool {
        if let Some(reg) = try_allocate2(class) {
            log::debug!("Reloading {} to {} at {}", vreg, reg, index);
            self.reg_relocations[index as usize].push(RegisterRelocation::ReloadTemp(reg, vreg));
            self.reg_occupied_by[reg as usize] = Some(vreg);
            self.vreg2reg[vreg as usize] = Reg(reg);
            true
        } else {
            log::debug!("No register available for reload of {} at {}", vreg, index);
            false
        }
    }

    pub fn force_reload(
        &mut self,
        register_use: &RegisterUse,
        index: u32,
        vreg: u32,
        class: &RegisterClass,
    ) {
        if !self.try_reload(index, vreg, class) {
            let _reg = self.spill_last(register_use, index, vreg, class);
            self.try_reload(index, vreg, class);
        }
    }

    pub fn spill_last(
        &mut self,
        register_use: &RegisterUse,
        index: u32,
        vreg: u32,
        class: &RegisterClass,
    ) -> Register {
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

    pub fn spill(&mut self, index: u32, reg: Register, vreg: u32) {
        self.reg_relocations[index as usize].push(RegisterRelocation::Spill(reg, vreg));
        self.allocation[vreg as usize].end_prev(index);
        //self.allocation[vreg as usize].start(Vreg(0), index);
        self.vreg2reg[vreg as usize] = Vreg(0); //TODO!!
        self.reg_occupied_by[reg as usize] = None;
    }

    pub fn two_address_move(&mut self, index: u32, from: Register, to: Register) {
        self.reg_relocations[index as usize].push(RegisterRelocation::TwoAddressMove(from, to));
    }
}

pub fn try_allocate2(class: &RegisterClass) -> Option<Register> {
    class.iter().next()
}

pub fn assign_register(reg: Register, vreg: u32, assignments: &mut RegisterAssignment, index: u32) {
    log::trace!("Using register {} for vreg {}", reg.to_string(), vreg);
    assignments.reg_occupied_by[reg as usize] = Some(vreg);
    assignments.vreg2reg[vreg as usize] = Reg(reg);
    assignments.allocation[vreg as usize].start(reg, index);
}
