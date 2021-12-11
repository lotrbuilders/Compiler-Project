use std::fmt;
use std::fmt::Display;
use std::ops::Range;

use crate::backend::ir::IRInstruction;

use super::is_two_address;
use super::ralloc::RegisterLocation::*;
use super::ralloc::*;
use super::registers::*;
use super::BackendAMD64;

pub struct ControlFlowGraph {
    predecessors: Vec<u32>,
    successors: Vec<u32>,
    instructions: Range<usize>,
    label: u32,
}

impl Display for ControlFlowGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} -> {} -> {:?}",
            self.predecessors, self.label, self.successors
        )
    }
}

type CFG<'a> = ControlFlowGraph;

impl ControlFlowGraph {
    pub fn to_string(cfg: &Vec<ControlFlowGraph>) -> String {
        let mut result = String::new();
        for block in cfg {
            result.push_str(&format!("{}\n", block));
        }
        result
    }

    pub fn new(range: Range<usize>, label: u32) -> ControlFlowGraph {
        ControlFlowGraph {
            predecessors: Vec::new(),
            successors: Vec::new(),
            instructions: range,
            label,
        }
    }

    pub fn last(&self) -> u32 {
        return (self.instructions.end - 1) as u32;
    }

    pub fn check(cfg: &Vec<ControlFlowGraph>) {
        for (block, i) in cfg.iter().zip(0..) {
            assert_eq!(
                block.label, i,
                "The label of a cfg block({}) and it's index({}) must be equal.",
                block.label, i
            )
        }
    }

    pub fn find_successors(cfg: &mut Vec<ControlFlowGraph>, instructions: &Vec<IRInstruction>) {
        for (block, i) in cfg.iter_mut().zip(0..) {
            let end = block.instructions.end - 1;
            use IRInstruction::*;
            match instructions[end] {
                Jmp(next) => block.successors.push(next),
                Jcc(.., next) | Jnc(.., next) => {
                    block.successors.push(next);
                    block.successors.push(i + 1)
                }
                Ret(..) => (),
                _ => block.successors.push(i + 1),
            }
        }

        let length = cfg.len();
        if let Some(block) = cfg.last_mut() {
            block.successors = block
                .successors
                .iter()
                .filter(|&&i| (i as usize) < length)
                .map(|i| *i)
                .collect();
        }
    }

    pub fn find_predecessors(cfg: &mut Vec<ControlFlowGraph>) {
        for block in 0..cfg.len() {
            for successor in cfg[block].successors.clone() {
                cfg[successor as usize].predecessors.push(block as u32);
            }
        }
    }

    pub fn construct(instructions: &Vec<IRInstruction>) -> Vec<ControlFlowGraph> {
        log::info!("Constructing CFG");
        let mut cfg = Vec::new();
        let mut start = 0;
        let mut label = 0;
        for (ins, i) in instructions.iter().zip(0usize..) {
            use IRInstruction::*;
            match ins {
                Label(_, lbl) => {
                    if !(start..i).is_empty() {
                        cfg.push(ControlFlowGraph::new(start..i, label))
                    }
                    start = i;
                    label = *lbl;
                }
                _ => (),
            }
        }
        if !(start..instructions.len()).is_empty() {
            cfg.push(ControlFlowGraph::new(start..instructions.len(), label))
        }
        log::info!("CFG:\n{}", CFG::to_string(&cfg));
        CFG::check(&cfg);
        CFG::find_successors(&mut cfg, instructions);
        log::info!("CFG:\n{}", CFG::to_string(&cfg));
        CFG::find_predecessors(&mut cfg);
        log::info!("CFG:\n{}", CFG::to_string(&cfg));

        cfg
    }
}

impl RegisterAllocator for RegisterAllocatorLinear {
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

        let /*mut*/ cfg = CFG::construct(&backend.instructions);
        let mut cfg_vreg2reg = vec![Vec::new(); cfg.len()];
        let mut _cfg_used = vec![false; cfg.len()];

        for block in 0..cfg.len() {
            let _label = cfg[block].label;
            /*insert_moves(
                &mut assignments,
                &mut cfg,
                &mut cfg_vreg2reg,
                &mut cfg_used,
                label,
            );*/
            for instruction in cfg[block].instructions.clone() {
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
            cfg_vreg2reg[block] = assignments.vreg2reg.clone();
        }

        backend.allocation = assignments.allocation;
        backend.reg_relocations = assignments.reg_relocations;
    }
}

fn _insert_moves(
    assignments: &mut RegisterAssignment,
    cfg: &mut Vec<ControlFlowGraph>,
    cfg_vreg2reg: &mut Vec<Vec<RegisterLocation>>,
    cfg_used: &mut Vec<bool>,
    label: u32,
) {
    let block = label as usize;
    assert_eq!(cfg.len(), cfg_vreg2reg.len());
    assert!(cfg[block].predecessors.len() <= 2); // For now we only support two way predecessors

    // If a block only has one predecessor, no major modifications are necessary
    let pred_count = cfg[block].predecessors.len();
    if pred_count > 1 {
        let predecessors_used = cfg[block]
            .predecessors
            .iter()
            .filter(|&&pred| cfg_used[pred as usize])
            .map(|pred| *pred)
            .collect::<Vec<u32>>();

        assert!(predecessors_used.len() < 2); // The registers of atleast one predecessor must be changeable

        let first_predecessor = *predecessors_used
            .get(0)
            .unwrap_or(&cfg[block].predecessors[0]);

        let other_predecessors = cfg[block]
            .predecessors
            .iter()
            .filter(|&&pred| pred != first_predecessor)
            .map(|p| *p);

        let mut live_registers: Vec<bool> = vec![true; assignments.vreg2reg.len()];
        for pred in cfg[block].predecessors.clone() {
            for (live, i) in live_registers.iter_mut().zip(0..) {
                *live &= assignments.allocation[i]
                    .live_at((cfg[pred as usize].instructions.end - 1) as u32)
            }
        }

        for _pred in other_predecessors {
            for (_reg, vreg) in assignments.allocation.iter().zip(0..) {
                if live_registers[vreg] {
                    //if first_predecessor
                }
            }
        }

        todo!();
    }
    // If a block has one predecessor that is not the the previous block, we must reload the register allocation
    else if pred_count == 1 && cfg[block].predecessors[0] != label - 1 {
        let start = cfg[block].instructions.start;
        let pred = cfg[block].predecessors[0] as usize;
        let vreg2reg = cfg_vreg2reg[pred].clone();
        assignments.reg_relocations[start].push(RegisterRelocation::Jump(vreg2reg))
    }

    // Notate that this predecessor has already been used to limit overwriting of registers that are in use
    for &pred in &cfg[block].predecessors {
        cfg_used[pred as usize] = true;
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
    let clobbered_registers = backend.get_clobbered(index);

    for reg in &clobbered_registers {
        if let Some(vreg) = assignments.reg_occupied_by[reg as usize] {
            assignments.spill(index, reg, vreg);
        }
    }

    let (used_vregs, result_vreg) = backend.get_vregisters(index, rule);

    let in_use_regs: RegisterClass = assignments.in_use_registers();

    // Load all virtual registers that are used into an actual register
    let mut _fail = false;
    for (vreg, class) in &used_vregs {
        let vreg = *vreg;
        let reg = assignments.vreg2reg[vreg as usize];
        if let NotAllocated = reg {
            if !assignments.try_reload(index, vreg, &(*class - &clobbered_registers)) {
                assignments.force_reload(
                    register_use,
                    index,
                    vreg,
                    &(&REG_CLASS_IREG - &clobbered_registers),
                )
            } else {
                _fail = true;
            }
        }
    }

    for (vreg, class) in &used_vregs {
        let vreg = *vreg;
        let reg = assignments.vreg2reg[vreg as usize];
        let reg = reg.reg().unwrap();
        if !class[reg] {
            let old_reg = reg;
            if let Some(reg) = try_allocate2(&((*class).clone() - in_use_regs.clone())) {
                assignments.reg_relocations[index as usize]
                    .push(RegisterRelocation::Move(old_reg, reg));

                assignments.reg_occupied_by
                    [assignments.vreg2reg[vreg as usize].reg().unwrap() as usize] = None;
                assignments.allocation[vreg as usize].end_prev(index);

                assignments.reg_occupied_by[reg as usize] = Some(vreg);
                assignments.vreg2reg[vreg as usize] = Reg(reg);
                assignments.allocation[vreg as usize].start(Reg(reg), index);
            } else {
                unimplemented!();
            }
        }
    }

    // perform register allocation if necessary
    if let Some((vreg, result_class)) = result_vreg {
        //let mut assigned_reg = None;

        //Registers that are in use at the start of the instruction
        let used_regs: RegisterClass = assignments.in_use_registers();

        //Registers that are in use at the start, but not at the end of the instruction
        let last_used_regs: RegisterClass = assignments.final_use_registers(register_use, index);

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

        if let Some(_reg) = assignments.try_allocate(
            &(&preferred_regs & &first_used_reg & result_class),
            vreg,
            index,
        ) {
        } else if !two_address {
            if let Some(_reg) = assignments.try_allocate(
                &(result_class.clone() & (&preferred_regs - &used_after_regs)),
                vreg,
                index,
            ) {
            } else if let Some(_reg) =
                assignments.try_allocate(&(result_class - &used_after_regs), vreg, index)
            {
            } else {
                log::error!("No register available and no solution currently implemented");
                unimplemented!();
            }
        } else if two_address {
            let left = assignments.vreg2reg[backend.get_left_vreg(index) as usize]
                .reg()
                .unwrap();
            if let Some(_reg) =
                assignments.try_allocate(&(&first_used_reg & result_class), vreg, index)
            {
            } else if let Some(reg) = assignments.try_allocate(
                &(result_class.clone() & (&preferred_regs - &used_regs)),
                vreg,
                index,
            ) {
                assignments.two_address_move(index, left, reg);
            } else if let Some(reg) =
                assignments.try_allocate(&(result_class - &used_regs), vreg, index)
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
            if let Reg(reg) = assignments.vreg2reg[i] {
                // Check if the register has not been reassigned this instruction
                if let Some(vreg) = assignments.reg_occupied_by[reg as usize] {
                    if vreg == i as u32 {
                        assignments.reg_occupied_by[reg as usize] = None;
                    }
                }
            }
            assignments.allocation[i].end(index as u32);
            assignments.vreg2reg[i] = NotAllocated;
        }
    }
}
