use std::collections::HashSet;

use super::super::{
    ralloc::*, RegisterAllocation, RegisterBackend, RegisterClass, RegisterInterface,
    RegisterLocation::*,
};

use super::{RegisterAllocator, RegisterAllocatorSimple};
use crate::backend::ir::control_flow_graph::ControlFlowGraph;
use crate::backend::ir::IRInstruction;

impl<R: RegisterInterface, B: RegisterBackend<RegisterType = R>> RegisterAllocator<R, B>
    for RegisterAllocatorSimple
{
    // Should be generated in a seperate file preferably
    // Allocates registers for an entire function
    fn allocate_registers(backend: &mut B) -> () {
        let length = backend.get_function_length();
        let register_use = backend.find_uses();

        log::debug!("Initialization of vregisters:\n{:?}", register_use.creation);
        log::debug!("Last use of vregisters:\n{:?}", register_use.last_use);

        let mut assignments = RegisterAssignment::<R> {
            reg_occupied_by: vec![None; R::REG_COUNT],
            vreg2reg: vec![NotAllocated; length],
            allocation: vec![RegisterAllocation::empty(); length],
            reg_relocations: vec![Vec::new(); backend.get_instructions().len()],
        };
        let mut used_registers = vec![false; R::REG_COUNT];

        let mut index = 0;
        for arg in backend.get_arguments() {
            if let Some(arg) = arg {
                assignments.allocation[*arg as usize]
                    .start(*try_allocate2(&R::CALL_REGS[index]).unwrap(), 0);
                index += 1;
            }
        }

        let cfg = ControlFlowGraph::construct(backend.get_instructions());

        for instruction in 1..backend.get_instructions().len() {
            let rule = backend.get_rule(instruction);
            if backend.is_instruction(rule) {
                allocate_register(
                    backend,
                    rule,
                    instruction as u32,
                    &register_use,
                    &mut assignments,
                    &mut used_registers,
                )
            }
            if let IRInstruction::Label(Some(phi), _lbl) =
                backend.get_instructions()[instruction].clone()
            {
                for (&target, sources) in phi.targets.iter().zip(phi.sources.iter()) {
                    let register_index: usize = R::REG_DEFAULT.into(); //This is likely not safe
                    used_registers[register_index] = true;

                    for &(block, source) in sources {
                        let last = cfg[block as usize].last();
                        let index = last;
                        let last = &mut assignments.reg_relocations[last as usize];

                        let target = backend.simple_get_spot(target);
                        let vreg = backend.simple_get_spot(source);
                        log::trace!("Insert memmove {}<-{} at {}", vreg, target, index);
                        last.push(RegisterRelocation::MemMove(vreg, target, R::REG_DEFAULT));
                        last.sort_unstable_by_key(move_cmp);
                    }
                }
            }
        }

        backend.simple_adjust_stack_size(assignments.vreg2reg.len() as i32);
        backend.set_allocation(assignments.allocation);
        log::debug!("Before optimization {:?}", assignments.reg_relocations);
        peephole_optimization(&mut assignments.reg_relocations);
        log::debug!("After optimization {:?}", assignments.reg_relocations);
        backend.set_reg_relocations(assignments.reg_relocations);
        backend.set_used_registers(used_registers);
    }
}
fn find_possible_registers<R: RegisterInterface>(
    class: &RegisterClass<R>,
    clobbered_registers: &Vec<R>,
    used_registers: &HashSet<R>,
) -> Vec<R> {
    class
        .iter()
        .filter(|r| !clobbered_registers.contains(&r) && !used_registers.contains(&r))
        .cloned()
        .collect()
}

fn allocate_register<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &mut B,
    rule: u16,
    index: u32,
    _register_use: &RegisterUse<R>,
    assignments: &mut RegisterAssignment<R>,
    global_used_register: &mut Vec<bool>,
) {
    // Clobber registers if necessary
    let clobbered_registers = backend.get_clobbered(index);

    let mut used_registers = HashSet::new();

    let (used_vregs, result_vreg) = backend.get_vregisters(index, rule);

    for (vreg, class) in used_vregs {
        let reg = if let Some(reg) = assignments.allocation[vreg as usize][index] {
            let i: usize = reg.into();
            global_used_register[i] = true;
            reg
        } else {
            let reg = try_allocate2(&find_possible_registers(
                &class,
                &clobbered_registers,
                &used_registers,
            ))
            .unwrap()
            .clone();

            let i: usize = reg.into();
            global_used_register[i] = true;

            let mem = backend.simple_get_spot(vreg);
            assignments.reg_relocations[index as usize].push(RegisterRelocation::Reload(reg, mem));

            assignments.allocation[vreg as usize].start(reg, index);
            reg
        };

        assignments.allocation[vreg as usize].end(index);
        used_registers.insert(reg);
    }

    // perform register allocation if necessary
    if let Some((vreg, result_class)) = result_vreg {
        let reg = *try_allocate2(&result_class).unwrap();
        let mem = backend.simple_get_spot(vreg);
        let i: usize = reg.into();
        global_used_register[i] = true;
        assignments.reg_relocations[(index) as usize].push(RegisterRelocation::Spill(reg, mem));
        assignments.allocation[vreg as usize].start(reg, index);
        assignments.allocation[vreg as usize].end(index);
    }
}

fn move_cmp<R: RegisterInterface>(mov: &RegisterRelocation<R>) -> i32 {
    use RegisterRelocation::*;
    match mov {
        MemMove(..) => 10,
        Move(..) => 20,
        TwoAddressMove(..) => 20,
        Spill(..) => 0,
        Reload(..) => 20,
        ReloadTemp(..) => 20,
        _ => unreachable!(),
    }
}

// Because of non-instructions this does not really work well
fn peephole_optimization<R: RegisterInterface>(relocations: &mut Vec<Vec<RegisterRelocation<R>>>) {
    use RegisterRelocation::*;
    for relocations in relocations {
        match (relocations.get(0), relocations.get(1)) {
            (Some(Spill(reg1, mem1)), Some(Reload(reg2, mem2))) if reg1 == reg2 && mem1 == mem2 => {
                relocations.remove(1);
            }

            _ => (),
        }
    }
}
