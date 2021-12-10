use crate::backend::ir::IRInstruction;

//use super::is_two_address;
use super::ralloc::RegisterLocation::*;
use super::ralloc::*;
use super::ralloc_linear::ControlFlowGraph;
use super::registers::*;
use super::BackendAMD64;

impl RegisterAllocator for RegisterAllocatorSimple {
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
            allocation: vec![RegisterAllocation::empty(); length],
            reg_relocations: vec![Vec::new(); backend.instructions.len()],
        };

        let cfg = ControlFlowGraph::construct(&backend.instructions);

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
            if let IRInstruction::Label(Some(phi), _lbl) = &backend.instructions[instruction] {
                for ((&block, source), &target) in phi
                    .locations
                    .iter()
                    .zip(phi.sources.iter())
                    .zip(phi.targets.iter())
                {
                    let last = cfg[block as usize].last();
                    let last = &mut assignments.reg_relocations[last as usize];
                    for &vreg in source {
                        last.insert(0, RegisterRelocation::MemMove(vreg, target, Register::Rax));
                    }
                }
                //Todo
            }
        }

        backend.allocation = assignments.allocation;
        backend.reg_relocations = assignments.reg_relocations;
    }
}
fn allocate_register(
    backend: &mut BackendAMD64,
    rule: u16,
    index: u32,
    _register_use: &RegisterUse,
    assignments: &mut RegisterAssignment,
) {
    // Clobber registers if necessary
    let clobbered_registers = backend
        .clobber(index as usize)
        .iter()
        .collect::<RegisterClass>();

    let mut used_registers = REG_CLASS_EMPTY.clone();

    let (used_vregs, result_vreg) = backend.get_vregisters(index, rule);

    for (vreg, class) in used_vregs {
        let reg = try_allocate2(&(class - &clobbered_registers - &used_registers)).unwrap();
        let mem = get_spot(backend, vreg);
        assignments.reg_relocations[index as usize].push(RegisterRelocation::Reload(reg, mem));

        assignments.allocation[vreg as usize].start(RegisterLocation::Reg(reg), index);
        assignments.allocation[vreg as usize].end(index);
        used_registers.add(reg);
    }

    // perform register allocation if necessary
    if let Some((vreg, result_class)) = result_vreg {
        let reg = try_allocate2(result_class).unwrap();
        let mem = get_spot(backend, vreg);
        assignments.reg_relocations[(index + 1) as usize].push(RegisterRelocation::Spill(reg, mem));
        assignments.allocation[vreg as usize].start(RegisterLocation::Reg(reg), index);
        assignments.allocation[vreg as usize].end(index);
    }
}

fn get_spot(backend: &mut BackendAMD64, vreg: u32) -> u32 {
    (backend.stack_size.abs() as u32) + 4 + 4 * vreg
}
