use smallvec::SmallVec;
use std::collections::HashSet;

use super::{
    coalesce::CoalesceSettings,
    instruction_information::InstructionInformation,
    live_analysis::{find_live_in, LiveIn},
    renumber::VregCopy,
    spill_code::SpillCode,
    Graph, Renumber,
};
use crate::backend::register_allocation::{RegisterBackend, RegisterInterface};
use crate::ir::ControlFlowGraph;

// The build function covers both the build-coalesce loop and also keeps track of the spill cost directly
pub fn build<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    ins_info: &InstructionInformation<R>,
    cfg: &ControlFlowGraph,
    spill_code: &SpillCode,
    mut numbers: Renumber<R>,
) -> (Graph<R>, Vec<SmallVec<[VregCopy; 2]>>) {
    log::debug!("Starting build stage");
    log::trace!("LiveRanges: {:?}", numbers.live_ranges);
    log::trace!(
        "Vreg2Live: {:?}",
        numbers
            .vreg2live
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{} => {:?}", i, v))
            .collect::<Vec<_>>()
    );
    let mut graph = Graph::new(
        numbers.live_ranges.clone(),
        numbers.vreg2live.clone(),
        numbers.length,
    );
    //log::trace!("Copies:{:?}", numbers.copies);

    build_first_iteration(backend, ins_info, cfg, &numbers, &mut graph, spill_code);
    graph.fill_adjacency_list();
    //log::trace!("Graph:{:?}", graph);

    log::trace!("Copies:{:?}", numbers.copies);
    log::trace!("Graph:{:?}", graph);

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
                !last_used[live_range as usize].contains(&(block.instructions.start as u32))
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
                        for &interference in &live_in {
                            if interference != to {
                                graph.let_interfere(to, interference);
                            }
                        }
                        graph.copy_interfer_live(&live_in, to, from);
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
                        log::trace!("live_in: {:?}", live_in);
                        log::trace!("phi copy from {} to {}", from, to);
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
                    log::trace!("removing {}", live_range);
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
