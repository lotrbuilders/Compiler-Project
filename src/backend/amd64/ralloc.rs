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
    Spill(Register),
    Reload(Register),
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

                if let Some(vreg) = result_vreg {
                    creation[vreg as usize] = i as u32;
                }
                for (vreg, class) in used_vreg {
                    uses[vreg as usize].push(i as u32);
                    last_use[vreg as usize] = i as u32;
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

        for reg in self.clobber(index as usize) {
            if let Some(_vreg) = assignments.reg_occupied_by[reg as usize] {
                unimplemented!();
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
            let reg = assignments.vreg2reg[vreg as usize].unwrap();
            if !class[reg] {
                if let Some(reg) = try_allocate(&((*class).clone() - used_regs.clone())) {
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
        if let Some(vreg) = result_vreg {
            //let mut assigned_reg = None;

            let used_regs: RegisterClass = assignments
                .reg_occupied_by
                .iter()
                .filter_map(|&vreg| vreg)
                .map(|vreg| assignments.vreg2reg[vreg as usize].unwrap())
                .collect();

            let last_used_regs: RegisterClass = assignments
                .vreg2reg
                .iter()
                .zip(register_use.last_use.iter())
                .filter_map(|(reg, last_use)| reg.zip(Some(*last_use)))
                .filter(|(_reg, last_use)| *last_use == index)
                .map(|(reg, _last_use)| reg)
                .collect();

            let used_after_regs = used_regs.clone() - last_used_regs.clone();

            let first_used_reg: Option<Register> = used_vregs
                .get(0)
                .map(|(vreg, _)| assignments.vreg2reg[*vreg as usize])
                .flatten();

            let first_used_reg = [first_used_reg]
                .iter()
                .filter_map(|&r| r)
                .collect::<RegisterClass>();

            let first_used_reg = first_used_reg - used_after_regs.clone();

            let preferred_regs = register_use.preferred_class[vreg as usize].clone();

            let two_address = is_two_address(rule);

            if let Some(reg) = try_allocate(&(preferred_regs.clone() & first_used_reg.clone())) {
                assign_register(reg, vreg, assignments);
            } else if let (Some(reg), true) = (try_allocate(&first_used_reg), two_address) {
                // Really only for two address architectures/instruction. Should be checked
                assign_register(reg, vreg, assignments);
            } else if !two_address {
                if let Some(reg) = try_allocate(&(preferred_regs.clone() - used_after_regs.clone()))
                {
                    assign_register(reg, vreg, assignments);
                } else if let Some(reg) =
                    try_allocate(&(REG_CLASS_IREG.clone() - used_after_regs.clone()))
                {
                    assign_register(reg, vreg, assignments);
                } else {
                    log::error!("No register available and no solution currently implemented");
                    unimplemented!();
                }
            } else {
                let left = assignments.vreg2reg[self.get_left_vreg(index) as usize].unwrap();
                if let Some(reg) = try_allocate(&(preferred_regs.clone() - used_regs.clone())) {
                    assignments.reg_relocations[index as usize]
                        .push(RegisterRelocation::TwoAddressMove(reg, left));
                    assign_register(reg, vreg, assignments);
                } else if let Some(reg) =
                    try_allocate(&(REG_CLASS_IREG.clone() - used_regs.clone()))
                {
                    assignments.reg_relocations[index as usize]
                        .push(RegisterRelocation::TwoAddressMove(reg, left));
                    assign_register(reg, vreg, assignments);
                } else {
                    log::error!("No register available and no solution currently implemented");
                    unimplemented!();
                }
            }

            /*for i in 0..REG_COUNT {
                let reg = REG_LOOKUP[i];
                // Will also need to be dependent on the used register class eventually
                // Skip unavailable registers
                if !REG_CLASS_IREG[i] {
                    continue;
                }
                // Skip occupied registers
                if assignments.reg_occupied_by[reg as usize] != None {
                    continue;
                } else {
                    log::trace!("Using register {} for vreg {}", reg.to_string(), vreg);
                    assignments.reg_occupied_by[reg as usize] = Some(vreg);
                    assignments.vreg2reg[vreg as usize] = Some(reg);
                    assignments.vreg2reg_original[vreg as usize] = Some(reg);
                    assigned_reg = Some(reg);
                    break;
                }
            }
            if let None = assigned_reg {
                // Should do a relocation or spill
            }

            assigned_reg*/
        }

        // perform register relocation if necessary

        // If something has gone out of scope: remove it
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

fn try_allocate(class: &RegisterClass) -> Option<Register> {
    class.iter().next()
}

fn assign_register(reg: Register, vreg: u32, assignments: &mut RegisterAssignment) {
    log::trace!("Using register {} for vreg {}", reg.to_string(), vreg);
    assignments.reg_occupied_by[reg as usize] = Some(vreg);
    assignments.vreg2reg[vreg as usize] = Some(reg);
    assignments.vreg2reg_original[vreg as usize] = Some(reg);
}
