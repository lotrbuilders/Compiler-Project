use super::registers::*;
use super::BackendAMD64;

// A vector of this is added to the instruction
// Shows operation that need to happen to make modifications to the register file
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RegisterRelocation {
    Move(u32, Register),                // from to
    TwoAddressMove(Register, Register), // from to
    Spill(Register, u32),
    Reload(Register, u32),
    ReloadTemp(Register, u32), // Reload temp is currently still the same as reload: Should be removed again after reloading
}

pub struct RegisterUse {
    pub creation: Vec<u32>,
    pub uses: Vec<Vec<u32>>,
    pub last_use: Vec<u32>,
    pub preferred_class: Vec<&'static RegisterClass>,
}

pub struct RegisterAssignment {
    pub reg_occupied_by: [Option<u32>; REG_COUNT],
    pub vreg2reg: Vec<Option<Register>>,
    pub vreg2reg_original: Vec<Option<Register>>,
    pub reg_relocations: Vec<Vec<RegisterRelocation>>,
}

pub struct RegisterAllocatorNormal {}
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

        for i in (0..self.instructions.len()).rev() {
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
        }
        RegisterUse {
            creation,
            last_use,
            uses,
            preferred_class,
        }
    }
}

impl RegisterAssignment {
    pub fn try_allocate(&mut self, class: &RegisterClass, vreg: u32) -> Option<Register> {
        match try_allocate2(class) {
            Some(reg) => {
                assign_register(reg, vreg, self);
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
            self.vreg2reg[vreg as usize] = Some(reg);
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
        let _reg = self.spill_last(register_use, index, vreg, class);
        self.try_reload(index, vreg, class);
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
            if class[self.vreg2reg[vreg as usize].unwrap()] {
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
        let reg = self.vreg2reg[vreg as usize].unwrap();
        self.spill(index, reg, furthest_vreg);
        reg
    }

    pub fn spill(&mut self, index: u32, reg: Register, vreg: u32) {
        self.reg_relocations[index as usize].push(RegisterRelocation::Spill(reg, vreg));
        self.vreg2reg[vreg as usize] = None;
        self.reg_occupied_by[reg as usize] = None;
    }

    pub fn two_address_move(&mut self, index: u32, from: Register, to: Register) {
        self.reg_relocations[index as usize].push(RegisterRelocation::TwoAddressMove(from, to));
    }
}

pub fn try_allocate2(class: &RegisterClass) -> Option<Register> {
    class.iter().next()
}

pub fn assign_register(reg: Register, vreg: u32, assignments: &mut RegisterAssignment) {
    log::trace!("Using register {} for vreg {}", reg.to_string(), vreg);
    assignments.reg_occupied_by[reg as usize] = Some(vreg);
    assignments.vreg2reg[vreg as usize] = Some(reg);
    assignments.vreg2reg_original[vreg as usize] = Some(reg);
}
