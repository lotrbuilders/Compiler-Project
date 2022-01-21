use std::collections::HashSet;

use bitvec::prelude::BitVec;
use smallvec::SmallVec;

use super::{
    coalesce::CoalesceSettings, instruction_information::InstructionInformation,
    renumber::VregCopy, spill_code::SpillCode, Graph, Renumber,
};
use crate::backend::{
    ir::control_flow_graph::ControlFlowGraph,
    register_allocation::{RegisterBackend, RegisterInterface},
};

// The build function covers both the build-coalesce loop and also keeps track of the spill cost directly
pub fn build<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    ins_info: &InstructionInformation<R>,
    cfg: &ControlFlowGraph,
    spill_code: &SpillCode,
    mut numbers: Renumber<R>,
) -> (Graph<R>, Vec<SmallVec<[VregCopy; 2]>>) {
    log::debug!("Starting build stage");
    let mut graph = Graph::new(
        numbers.live_ranges.clone(),
        numbers.vreg2live.clone(),
        numbers.length,
    );
    //log::trace!("Copies:{:?}", numbers.copies);

    build_first_iteration(backend, ins_info, cfg, &numbers, &mut graph, spill_code);
    graph.fill_adjacency_list();
    //log::trace!("Graph:{:?}", graph);

    for i in 0..10 {
        let improved = graph.coalesce(
            &mut numbers.copies,
            CoalesceSettings {
                conservative: true,
                coalesce_split: false,
                coalesce_argument: true,
            },
        );
        log::debug!("Coalesce stage: {}\timproved: {}", i, improved);
        if !improved {
            break;
        }
    }

    log::trace!("Copies:{:?}", numbers.copies);
    log::trace!("Graph:{:?}", graph);
    graph.drop_bit_matrix();
    (graph, numbers.copies)
}

// Fills the bit_matrix using both copy and instructions
// Later (coalesce) iteration only modify this
fn build_first_iteration<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    ins_info: &InstructionInformation<R>,
    cfg: &ControlFlowGraph,
    numbers: &Renumber<R>,
    graph: &mut Graph<R>,
    spill_code: &SpillCode,
) {
    let LiveIn {
        live_in,
        last_used,
        loop_depth,
    } = find_live_in(backend, ins_info, cfg, numbers, spill_code);

    let loop_cost: Vec<_> = loop_depth
        .iter()
        .map(|&i| 10.0_f32.powi(i as i32))
        .collect();

    //log::trace!("Last Used: {:?}", last_used);
    for block in &cfg.graph {
        let label = block.label as usize;
        let block_cost = loop_cost[block.label as usize];
        // Possibly needs a method to ensure correctness

        // Make a hashset of the live_in bitset for this block
        // Filter out anything that is last_used at the start of the block
        // These are introduced by
        let mut live_in: HashSet<u32> = live_in[label]
            .iter()
            .enumerate()
            .filter_map(|(i, b)| if *b { Some(i as u32) } else { None })
            .filter(|&live_range| {
                last_used[live_range as usize].contains(&(block.instructions.start as u32))
            })
            .collect();

        for index in block
            .instructions
            .clone()
            .filter(|&i| ins_info.is_instruction[i])
        {
            //log::trace!("First build {}", index);
            //log::trace!("live_in: {:?}", live_in);
            let location = index as u32;
            let used = &ins_info.used[index];
            let defined = &ins_info.result[index];
            let clobbered_before = &ins_info.clobber[index];
            let clobbered_after = &ins_info.clobber_after[index];

            // Reloading is processed first, because it is possible for copies to use the reload result
            for reload in spill_code.code[index]
                .iter()
                .filter(|&spill| spill.is_reload())
            {
                let vreg = reload.vreg();
                let live_range = numbers.translate(vreg, location);
                graph.live_ranges[live_range as usize].spill_cost = f32::MAX;
                for &interference in &live_in {
                    graph.let_interfere(live_range, interference)
                }
                live_in.insert(live_range);
            }

            // Copies before should interfer
            //      everything that's live outside from
            //      from if this is the last use of from
            for copy in &numbers.copies[index] {
                match copy {
                    VregCopy::ArgumentCopy { reg, vreg } => {
                        let from = *reg;
                        let to = numbers.translate(*vreg, location);
                        graph.copy_interfer_live(&live_in, from, to);
                        graph.adjust_spill_cost(to, block_cost);
                        live_in.remove(&from);
                    }
                    VregCopy::TwoAddress { from, to } => {
                        let from = numbers.translate(*from, location);
                        let to = numbers.translate(*to, location);
                        graph.copy_interfer_live(&live_in, from, to);
                        graph.adjust_spill_cost(to, block_cost);
                        graph.adjust_spill_cost(from, block_cost);
                        //Can interfer later, but this is handled as normal
                    }
                    VregCopy::PhiCopy { from, to } => {
                        let from = numbers.translate(*from, location);
                        let to = numbers.translate(*to, location);
                        graph.copy_interfer_live(&live_in, from, to);
                        graph.adjust_spill_cost(to, block_cost);
                        graph.adjust_spill_cost(from, block_cost);
                        // If the vreg is last used in this instruction
                        // and only used by the phicopy
                        // it won't interfer with to
                        if !(last_used[from as usize].contains(&(index as u32))
                            && !used.iter().find(|(r, _)| *r == from).is_some())
                        {
                            graph.let_interfere(to, from);
                        }
                    }
                    VregCopy::TargetBefore { vreg, reg } => {
                        let from = numbers.translate(*vreg, location);
                        let to = *reg;
                        graph.copy_interfer_live(&live_in, from, to);
                        graph.adjust_spill_cost(from, block_cost);
                    }
                    VregCopy::TargetAfter { .. } => {}
                    VregCopy::Coalesced => unreachable!(),
                }
            }

            for &clobber in clobbered_before {
                for &interference in &live_in {
                    graph.let_interfere(clobber.into(), interference);
                }
            }

            for (vreg, class) in used.iter() {
                let vreg = *vreg;
                let live_range = numbers.translate(vreg, location);
                // Interfer with all register outside this registerclass
                if let Some(_target) = class.is_target() {
                } else {
                    for reg in !class.clone() {
                        let reg: u32 = reg.into();
                        graph.let_interfere(live_range, reg);
                    }
                    graph.adjust_spill_cost(live_range, block_cost);
                }

                // If this is the last use drop it from live
                if last_used[live_range as usize].contains(&(index as u32)) {
                    live_in.remove(&live_range);
                }
            }
            for (vreg, class) in defined {
                let vreg = *vreg;
                let _ = (&vreg, &class);
                let live_range = numbers.translate(vreg, location);
                if let Some(_target) = class.is_target() {
                } else {
                    // Interfer with all register outside this registerclass
                    for reg in !class.clone() {
                        let reg: u32 = reg.into();
                        graph.let_interfere(live_range, reg);
                    }
                    // Interfer with everything that is currently live
                }
                for &interference in &live_in {
                    graph.let_interfere(live_range, interference);
                }
                // Add defined register to life
                graph.adjust_spill_cost(live_range, block_cost);
                live_in.insert(live_range);
                if last_used[live_range as usize].contains(&(index as u32)) {
                    live_in.remove(&live_range);
                }
            }

            // Reloaded registers are removed before clobber after is processed
            // Since these registers are currently never saved they can savely be clobbered
            for reload in spill_code.code[index]
                .iter()
                .filter(|&spill| spill.is_reload())
            {
                let vreg = reload.vreg();
                let live_range = numbers.translate(vreg, location);

                live_in.remove(&live_range);
            }

            for &clobber in clobbered_after {
                //log::trace!("clobber {}", clobber);
                for &interference in &live_in {
                    graph.let_interfere(clobber.into(), interference);
                }
            }

            for copy in &numbers.copies[index] {
                if let VregCopy::TargetAfter { reg, vreg } = copy {
                    //log::trace!("Target after {}", R::REG_LOOKUP[*reg as usize]);
                    //log::trace!("Live in: {:?}", live_in);
                    let from = *reg;
                    let to = numbers.translate(*vreg, location);
                    graph.copy_interfer_live(&live_in, to, from);
                }
            }

            // Spills are processed last
            // They can still be targeted by target after or clobbered
            for spill in spill_code.code[index]
                .iter()
                .filter(|&spill| spill.is_spill())
            {
                let vreg = spill.vreg();
                let live_range = numbers.translate(vreg, location);
                graph.live_ranges[live_range as usize].spill_cost = f32::MAX;
                live_in.remove(&live_range);
            }
        }
    }
}

impl<R: RegisterInterface> Graph<R> {
    fn copy_interfer_live(&mut self, live_in: &HashSet<u32>, from: u32, to: u32) {
        for &interference in live_in {
            if interference != from {
                self.let_interfere(to, interference);
            }
        }
    }
}

/// for all block in cfg
///     for all instructions in that block
///         remove any defined register from the live set
///         add any used registers
///     

struct LiveIn {
    live_in: Vec<BitVec>,
    last_used: Vec<SmallVec<[u32; 4]>>,
    loop_depth: Vec<u32>,
}

fn find_live_in<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    ins_info: &InstructionInformation<R>,
    cfg: &ControlFlowGraph,
    numbers: &Renumber<R>,
    spill_code: &SpillCode,
) -> LiveIn {
    let mut live_in = vec![BitVec::repeat(false, numbers.length); cfg.len()]; //live_ranges x blocks
    let mut last_used = vec![SmallVec::new(); numbers.length]; //n x live_ranges
    let mut visited = vec![false; cfg.len()];
    let mut loop_header = Vec::new();
    let mut last_block = Vec::new();
    let mut loop_depth = vec![0; cfg.len()];

    let instructions = backend.get_instructions();
    for block in cfg.iter().rev() {
        let mut live = BitVec::repeat(false, numbers.length);
        let label = block.label as usize;

        // live = union succors.live
        for s in &block.successors {
            let s = *s as usize;
            if !visited[s] {
                loop_header.push(s as u32);
                last_block.push(block.label);
            } else {
                live |= live_in[s].clone();
            }
        }
        visited[label] = true;

        // live |= union succesors.phi.input from b
        for phi in block
            .successors
            .iter()
            .filter_map(|s| cfg[*s as usize].phi(instructions))
        {
            /*for (index, _) in phi
                .locations
                .iter()
                .enumerate()
                .filter(|(_, &label)| label == block.label)
            {*/
            for (index, _) in phi.targets.iter().enumerate() {
                for &(_location, target) in &phi.sources[index] {
                    // Might have been changed wrong during phi change
                    let last_instruction = block.instructions.end - 1;
                    let location = last_instruction as u32;

                    let target = numbers.translate(target, location) as usize;
                    if !live.get(target).unwrap() {
                        last_used[target].push(last_instruction as u32);
                        live.set(target, true);
                    }
                }
            }

            //}
        }

        // live |= block.operands.input
        // live -= block.operands.output
        for index in block
            .instructions
            .clone()
            .rev()
            .filter(|&i| ins_info.is_instruction[i])
        {
            let used = &ins_info.used[index];
            let result = &ins_info.result[index];
            let location = index as u32;

            for &(result, _) in result {
                let result = numbers.translate(result, location) as usize;
                if !live.get(result).unwrap() {
                    last_used[result].push(index as u32);
                }
                live.set(result, false);
            }
            for &(vreg, _) in used {
                let vreg = numbers.translate(vreg, location) as usize;
                if !live.get(vreg).unwrap() {
                    last_used[vreg].push(index as u32);
                    live.set(vreg, true);
                }
            }
            //}
        }

        // live -= block.phi.output
        if let Some(phi) = block.phi(instructions) {
            for (i, &target) in phi.targets.iter().enumerate() {
                for &(location, _source) in &phi.sources[i] {
                    let location = cfg[location].last();
                    let target = numbers.translate(target, location) as usize;
                    live.set(target, false);
                }
            }
        }

        // live -= spilled variables
        for index in block
            .instructions
            .clone()
            .rev()
            .filter(|&i| ins_info.is_instruction[i])
        {
            let location = index as u32;
            for copy in &spill_code.code[index] {
                let vreg = copy.vreg();
                let live_range = numbers.translate(vreg, location) as usize;
                live.set(live_range, false);
            }
        }

        // extend ranges if b is a loop header
        for (index, _) in loop_header
            .iter()
            .enumerate()
            .filter(|(_, &label)| label == block.label)
        {
            let loop_end = last_block[index];

            // For all blocks between the loop_header and loop end extend any variables live in the loop header
            for part in block.label..loop_end {
                //This migh overshoot the actual loop depth when continue is used a lot
                loop_depth[part as usize] += 1;
                let part = part as usize;
                let clone = live.clone();
                live_in[part] |= clone;
            }

            // For all live variables at the loop header move the last_use to the end of the loop if inside the loop
            let start = block.instructions.start as u32;
            let end = cfg[loop_end].instructions.end as u32;
            let range = start..end;
            let loop_end = end + 1;

            for (_, last_uses) in live.iter().zip(last_used.iter_mut()).filter(|(b, _)| **b) {
                for last_use in last_uses {
                    if range.contains(last_use) {
                        *last_use = loop_end;
                    }
                }
            }
        }

        live_in[label] = live;
    }

    for &arg in backend.get_arguments().iter().flat_map(|arg| arg) {
        live_in[0].set(arg as usize, true)
    }

    LiveIn {
        live_in,
        last_used,
        loop_depth,
    }
}
