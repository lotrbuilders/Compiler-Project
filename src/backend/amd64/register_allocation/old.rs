use super::is_two_address;
use super::ralloc::RegisterLocation::*;
use super::ralloc::*;
use super::registers::*;
use super::BackendAMD64;

impl RegisterAllocator for RegisterAllocatorNormal {
    // Should be generated in a seperate file preferably
    // Allocates registers for an entire function
    fn allocate_registers(backend: &mut BackendAMD64) -> () {
        let length = backend.definition_index.len();
        let register_use = backend.find_uses();
        log::debug!("Initialization of vregisters:\n{:?}", register_use.creation);
        log::debug!("Last use of vregisters:\n{:?}", register_use.last_use);

        let mut assignments = RegisterAssignment {
            reg_occupied_by: [None; REG_COUNT],
            vreg2reg: vec![NotAllocated; length],
            vreg2reg_original: vec![NotAllocated; length],
            reg_relocations: vec![Vec::new(); backend.instructions.len()],
        };

        for instruction in 0..backend.instructions.len() {
            let rule = backend.rules[instruction];
            if backend.is_instruction(rule) {
                allocate_register(
                    backend,
                    rule,
                    instruction as u32,
                    &register_use,
                    &mut assignments,
                )
            }
        }

        backend.vreg2reg = assignments.vreg2reg_original;
        //.iter()
        //.map(|reg| reg.unwrap_or(Register::Rax))
        // .collect();
        backend.reg_relocations = assignments.reg_relocations;

        log::debug!(
            "vreg2reg at start {:?}",
            backend
                .vreg2reg
                .iter()
                .map(|reg| reg.to_string())
                .collect::<Vec<String>>()
        );
    }
}
fn allocate_register(
    backend: &BackendAMD64,
    rule: u16,
    index: u32,
    register_use: &RegisterUse,
    assignments: &mut RegisterAssignment,
) {
    let length = backend.definition_index.len();

    // Clobber registers if necessary
    let clobbered_registers = backend
        .clobber(index as usize)
        .iter()
        .collect::<RegisterClass>();

    for reg in &clobbered_registers {
        if let Some(vreg) = assignments.reg_occupied_by[reg as usize] {
            assignments.spill(index, reg, vreg);
            //unimplemented!();
        }
    }

    let (used_vregs, result_vreg) = backend.get_vregisters(index, rule);

    let used_regs: RegisterClass = assignments
        .reg_occupied_by
        .iter()
        .filter_map(|&vreg| vreg)
        .map(|vreg| assignments.vreg2reg[vreg as usize].reg().unwrap())
        .collect();

    for (vreg, class) in &used_vregs {
        let vreg = *vreg;
        let reg = assignments.vreg2reg[vreg as usize];
        if let NotAllocated = reg {
            if !assignments.try_reload(index, vreg, &(*class - &clobbered_registers))
                && !assignments.try_reload(index, vreg, &(&REG_CLASS_IREG - &clobbered_registers))
            {
                assignments.force_reload(
                    register_use,
                    index,
                    vreg,
                    &(&REG_CLASS_IREG - &clobbered_registers),
                )
            }
        }

        let reg = reg.reg().unwrap();
        if !class[reg] {
            if let Some(reg) = try_allocate2(&((*class).clone() - used_regs.clone())) {
                assignments.reg_relocations[index as usize]
                    .push(RegisterRelocation::Move(vreg, reg));

                assignments.reg_occupied_by
                    [assignments.vreg2reg[vreg as usize].reg().unwrap() as usize] = None;

                assignments.reg_occupied_by[reg as usize] = Some(vreg);
                assignments.vreg2reg[vreg as usize] = Reg(reg);
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
            .map(|vreg| assignments.vreg2reg[vreg as usize].reg().unwrap())
            .collect();

        //Registers that are in use at the start, but not at the end of the instruction
        let last_used_regs: RegisterClass = assignments
            .vreg2reg
            .iter()
            .zip(register_use.last_use.iter())
            .filter_map(|(reg, last_use)| reg.reg().zip(Some(*last_use)))
            .filter(|(_reg, last_use)| *last_use == index)
            .map(|(reg, _last_use)| reg)
            .collect();

        //Registers that will still be in use after the instruction
        let used_after_regs = &used_regs - &last_used_regs;

        //The two operand target register
        let first_used_reg = used_vregs
            .get(0)
            .map(|(vreg, _)| assignments.vreg2reg[*vreg as usize].reg())
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
            let left = assignments.vreg2reg[backend.get_left_vreg(index) as usize]
                .reg()
                .unwrap();
            if let Some(_reg) = assignments.try_allocate(&(&first_used_reg & result_class), vreg) {
            } else if let Some(reg) = assignments.try_allocate(
                &(result_class.clone() & (&preferred_regs - &used_regs)),
                vreg,
            ) {
                assignments.two_address_move(index, left, reg);
            } else if let Some(reg) = assignments.try_allocate(&(result_class - &used_regs), vreg) {
                assignments.two_address_move(index, left, reg);
            } else {
                log::error!("No register available and no solution currently implemented");
                unimplemented!();
            }
        }
    }
    for i in 0..length {
        if index == register_use.last_use[i] && register_use.creation[i] != u32::MAX {
            let reg = assignments.vreg2reg[i].reg().unwrap();

            // Check if the register has not been reassigned this instruction
            if let Some(vreg) = assignments.reg_occupied_by[reg as usize] {
                if vreg == i as u32 {
                    assignments.reg_occupied_by[reg as usize] = None;
                }
            }
            assignments.vreg2reg[i] = NotAllocated;
        }
    }
}
