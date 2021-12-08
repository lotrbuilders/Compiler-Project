use super::is_two_address;
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

struct RegisterUse {
    creation: Vec<u32>,
    uses: Vec<Vec<u32>>,
    last_use: Vec<u32>,
    preferred_class: Vec<&'static RegisterClass>,
}

struct RegisterAssignment {
    reg_occupied_by: [Option<u32>; REG_COUNT],
    vreg2reg: Vec<Option<Register>>,
    vreg2reg_original: Vec<Option<Register>>,
    reg_relocations: Vec<Vec<RegisterRelocation>>,
}

impl BackendAMD64 {
    fn find_uses(&mut self) -> RegisterUse {
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

    // Should be generated in a seperate file preferably
    // Allocates registers for an entire function
    pub fn allocate_registers(&mut self) -> () {
        let length = self.definition_index.len();
        let register_use = self.find_uses();
        log::debug!("Initialization of vregisters:\n{:?}", register_use.creation);
        log::debug!("Last use of vregisters:\n{:?}", register_use.last_use);

        let mut assignments = RegisterAssignment {
            reg_occupied_by: [None; REG_COUNT],
            vreg2reg: vec![None; length],
            vreg2reg_original: vec![None; length],
            reg_relocations: vec![Vec::new(); self.instructions.len()],
        };

        for instruction in 0..self.instructions.len() {
            let rule = self.rules[instruction];
            if self.is_instruction(rule) {
                self.allocate_register(rule, instruction as u32, &register_use, &mut assignments)
            }
        }

        self.vreg2reg = assignments
            .vreg2reg_original
            .iter()
            .map(|reg| reg.unwrap_or(Register::Rax))
            .collect();
        self.reg_relocations = assignments.reg_relocations;

        log::debug!(
            "vreg2reg at start {:?}",
            self.vreg2reg
                .iter()
                .map(|reg| reg.to_string())
                .collect::<Vec<&str>>()
        );
    }

    fn allocate_register(
        &self,
        rule: u16,
        index: u32,
        register_use: &RegisterUse,
        assignments: &mut RegisterAssignment,
    ) {
        let length = self.definition_index.len();

        // Clobber registers if necessary
        let clobbered_registers = self
            .clobber(index as usize)
            .iter()
            .collect::<RegisterClass>();

        for reg in &clobbered_registers {
            if let Some(vreg) = assignments.reg_occupied_by[reg as usize] {
                assignments.spill(index, reg, vreg);
                //unimplemented!();
            }
        }

        let (used_vregs, result_vreg) = self.get_vregisters(index, rule);

        let used_regs: RegisterClass = assignments
            .reg_occupied_by
            .iter()
            .filter_map(|&vreg| vreg)
            .map(|vreg| assignments.vreg2reg[vreg as usize].unwrap())
            .collect();

        for (vreg, class) in &used_vregs {
            let vreg = *vreg;
            let reg = assignments.vreg2reg[vreg as usize];
            if let None = reg {
                if !assignments.try_reload(index, vreg, &(*class - &clobbered_registers))
                    && !assignments.try_reload(
                        index,
                        vreg,
                        &(&REG_CLASS_IREG - &clobbered_registers),
                    )
                {
                    assignments.force_reload(
                        register_use,
                        index,
                        vreg,
                        &(&REG_CLASS_IREG - &clobbered_registers),
                    )
                }
            }

            let reg = reg.unwrap();
            if !class[reg] {
                if let Some(reg) = try_allocate2(&((*class).clone() - used_regs.clone())) {
                    assignments.reg_relocations[index as usize]
                        .push(RegisterRelocation::Move(vreg, reg));

                    assignments.reg_occupied_by
                        [assignments.vreg2reg[vreg as usize].unwrap() as usize] = None;

                    assignments.reg_occupied_by[reg as usize] = Some(vreg);
                    assignments.vreg2reg[vreg as usize] = Some(reg);
                } else {
                    unimplemented!();
                }
            }
        }

        // perform register allocation if necessary
        if let Some((vreg, result_class)) = result_vreg {
            //let mut assigned_reg = None;

            //Registers that are in use at the start of the instruction
            let used_regs: RegisterClass = assignments
                .reg_occupied_by
                .iter()
                .filter_map(|&vreg| vreg)
                .map(|vreg| assignments.vreg2reg[vreg as usize].unwrap())
                .collect();

            //Registers that are in use at the start, but not at the end of the instruction
            let last_used_regs: RegisterClass = assignments
                .vreg2reg
                .iter()
                .zip(register_use.last_use.iter())
                .filter_map(|(reg, last_use)| reg.zip(Some(*last_use)))
                .filter(|(_reg, last_use)| *last_use == index)
                .map(|(reg, _last_use)| reg)
                .collect();

            //Registers that will still be in use after the instruction
            let used_after_regs = &used_regs - &last_used_regs;

            //The two operand target register
            let first_used_reg = used_vregs
                .get(0)
                .map(|(vreg, _)| assignments.vreg2reg[*vreg as usize])
                .flatten()
                .iter()
                .collect::<RegisterClass>();

            //First register is only an option if it's used later
            let first_used_reg = first_used_reg - used_after_regs.clone();

            //Registers that are in the preferred register class for this instruction
            let preferred_regs = register_use.preferred_class[vreg as usize].clone();

            //Wether this instruction is a two address instruction
            let two_address = is_two_address(rule);

            if let Some(_reg) =
                assignments.try_allocate(&(&preferred_regs & &first_used_reg & result_class), vreg)
            {
            }
            /*else if let (Some(reg), true) = (
                try_allocate2(&(first_used_reg.clone() & result_class.clone())),
                two_address,
            ) {
                assign_register(reg, vreg, assignments);
            }*/
            else if !two_address {
                if let Some(_reg) = assignments.try_allocate(
                    &(result_class.clone() & (&preferred_regs - &used_after_regs)),
                    vreg,
                ) {
                } else if let Some(_reg) =
                    assignments.try_allocate(&(result_class - &used_after_regs), vreg)
                {
                } else {
                    log::error!("No register available and no solution currently implemented");
                    unimplemented!();
                }
            } else if two_address {
                let left = assignments.vreg2reg[self.get_left_vreg(index) as usize].unwrap();
                if let Some(_reg) =
                    assignments.try_allocate(&(&first_used_reg & result_class), vreg)
                {
                } else if let Some(reg) = assignments.try_allocate(
                    &(result_class.clone() & (&preferred_regs - &used_regs)),
                    vreg,
                ) {
                    assignments.two_address_move(index, left, reg);
                } else if let Some(reg) =
                    assignments.try_allocate(&(result_class - &used_regs), vreg)
                {
                    assignments.two_address_move(index, left, reg);
                } else {
                    log::error!("No register available and no solution currently implemented");
                    unimplemented!();
                }
            }
        }
        for i in 0..length {
            if index == register_use.last_use[i] && register_use.creation[i] != u32::MAX {
                let reg = assignments.vreg2reg[i].unwrap();

                // Check if the register has not been reassigned this instruction
                if let Some(vreg) = assignments.reg_occupied_by[reg as usize] {
                    if vreg == i as u32 {
                        assignments.reg_occupied_by[reg as usize] = None;
                    }
                }
                assignments.vreg2reg[i] = None;
            }
        }
    }
}

impl RegisterAssignment {
    fn try_allocate(&mut self, class: &RegisterClass, vreg: u32) -> Option<Register> {
        match try_allocate2(class) {
            Some(reg) => {
                assign_register(reg, vreg, self);
                Some(reg)
            }
            _ => None,
        }
    }

    fn _force_allocate(
        &mut self,
        _register_use: &RegisterUse,
        _vreg: u32,
        _class: &RegisterClass,
    ) -> Register {
        Register::Rax
    }

    fn try_reload(
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

    fn force_reload(
        &mut self,
        register_use: &RegisterUse,
        index: u32,
        vreg: u32,
        class: &RegisterClass,
    ) {
        let _reg = self.spill_last(register_use, index, vreg, class);
        self.try_reload(index, vreg, class);
    }

    fn spill_last(
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

    fn spill(&mut self, index: u32, reg: Register, vreg: u32) {
        self.reg_relocations[index as usize].push(RegisterRelocation::Spill(reg, vreg));
        self.vreg2reg[vreg as usize] = None;
        self.reg_occupied_by[reg as usize] = None;
    }

    fn two_address_move(&mut self, index: u32, from: Register, to: Register) {
        self.reg_relocations[index as usize].push(RegisterRelocation::TwoAddressMove(from, to));
    }
}

fn try_allocate2(class: &RegisterClass) -> Option<Register> {
    class.iter().next()
}

fn assign_register(reg: Register, vreg: u32, assignments: &mut RegisterAssignment) {
    log::trace!("Using register {} for vreg {}", reg.to_string(), vreg);
    assignments.reg_occupied_by[reg as usize] = Some(vreg);
    assignments.vreg2reg[vreg as usize] = Some(reg);
    assignments.vreg2reg_original[vreg as usize] = Some(reg);
}
