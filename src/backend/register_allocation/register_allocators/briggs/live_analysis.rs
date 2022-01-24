use std::fmt::Debug;
use std::{collections::VecDeque, ops::RangeBounds};

use bitvec::prelude::BitVec;
use smallvec::SmallVec;

use super::{instruction_information::InstructionInformation, spill_code::SpillCode, Renumber};
use crate::backend::{
    ir::control_flow_graph::ControlFlowGraph,
    register_allocation::{RegisterBackend, RegisterInterface},
};

#[derive(Clone)]
pub struct LiveIn {
    pub live_in: Vec<BitVec>,
    pub last_used: Vec<SmallVec<[u32; 4]>>,
    pub loop_depth: Vec<u32>,
}

impl Debug for LiveIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.live_in.len();
        for i in 0..len {
            writeln!(
                f,
                "{} => live_in: {:?}",
                i,
                self.live_in[i].iter_ones().collect::<Vec<_>>(),
            )?;
        }
        writeln!(f, "last_used: {:?}", self.last_used)?;
        writeln!(f, "loop_depth: {:?}", self.loop_depth)?;
        Ok(())
    }
}

pub fn find_live_in<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    ins_info: &InstructionInformation<R>,
    cfg: &ControlFlowGraph,
    numbers: &Renumber<R>,
    spill_code: &SpillCode,
) -> LiveIn {
    let mut visited = vec![false; cfg.len()];
    //let mut loop_header = Vec::new();
    //let mut last_block = Vec::new();
    let mut loop_depth = vec![0; cfg.len()];

    let (gen_used, uses) = find_gen_used(backend, ins_info, cfg, numbers, spill_code);

    //let instructions = backend.get_instructions();
    /*for block in cfg.iter().rev() {
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
    }*/

    let (live_in, live_out) = live_in_out(backend, cfg, &gen_used, numbers.length);

    for b in 0..cfg.len() {
        log::trace!(
            "gen[{:?}]:{:?}",
            b,
            gen_used[b].0.iter_ones().collect::<Vec<_>>()
        );
        log::trace!(
            "used[{:?}]:{:?}",
            b,
            gen_used[b].1.iter_ones().collect::<Vec<_>>()
        );
        log::trace!(
            "live_in:[{:?}]:{:?}",
            b,
            live_in[b].iter_ones().collect::<Vec<_>>()
        );
        log::trace!(
            "live_out:[{:?}]:{:?}",
            b,
            live_out[b].iter_ones().collect::<Vec<_>>()
        );
    }

    let last_used = find_last_use(backend, numbers, cfg, ins_info, spill_code, &live_out);

    let result = LiveIn {
        live_in,
        last_used,
        loop_depth,
    };
    log::trace!("LiveIn:{:?}", result);
    result
}

fn find_last_use<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    numbers: &Renumber<R>,
    cfg: &ControlFlowGraph,
    ins_info: &InstructionInformation<R>,
    spill_code: &SpillCode,
    live_out: &[BitVec],
) -> Vec<SmallVec<[u32; 4]>> {
    let mut last_used = vec![SmallVec::new(); numbers.length];
    let instructions = backend.get_instructions();
    for block in cfg {
        let mut already_seen = live_out[block.label as usize].clone();

        for phi in block
            .successors
            .iter()
            .filter_map(|s| cfg[*s as usize].phi(instructions))
        {
            for (index, &target) in phi.targets.iter().enumerate() {
                for &(label, source) in &phi.sources[index] {
                    if !spill_code.contains(source) {
                        // Might have been changed wrong during phi change
                        let last_instruction = (block.instructions.end - 1) as u32;
                        let source = numbers.translate(source, last_instruction) as usize;
                        let target = numbers.translate(target, last_instruction) as usize;

                        if label == block.label && !already_seen.replace(source as usize, true) {
                            last_used[source].push(last_instruction);
                        }
                        if label == block.label && !already_seen.replace(target as usize, false) {
                            last_used[target].push(last_instruction);
                        }
                    }
                }
            }
        }

        // used |= block.operands.input
        // gen |= block.operands.output
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
                if !already_seen.replace(result, false) {
                    last_used[result].push(location);
                }
            }

            for &(vreg, _) in used {
                let live = numbers.translate(vreg, location) as usize;
                if !already_seen.replace(live, true) {
                    last_used[live].push(location);
                }
            }
        }

        /*
        // This is not completely accurate due to the insertion of phicopies at an earlier stage
        // Instead the variables should already be live for all live_out in predececessors
        // However this should not matter
        // gen |= block.phi.output
        if let Some(phi) = block.phi(instructions) {
            for (i, &target) in phi.targets.iter().enumerate() {
                if !spill_code.contains(target) {
                    let location = block.instructions.start as u32;
                    let target = numbers.translate(target, location) as usize;

                    if !already_seen.replace(target, true) {
                        for &(location, _source) in &phi.sources[i] {
                            last_used[target].push(location);
                        }
                    }
                }
            }
        }*/
    }
    last_used
}

fn live_in_out<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    cfg: &ControlFlowGraph,
    gen_used: &[(BitVec, BitVec)],
    live_count: usize,
) -> (Vec<BitVec>, Vec<BitVec>) {
    let mut live_in = vec![BitVec::repeat(false, live_count); cfg.len()]; //live_ranges x blocks
    let mut live_out = vec![BitVec::repeat(false, live_count); cfg.len()];
    let instructions = backend.get_instructions();

    let mut work_list: VecDeque<u32> = (0..cfg.len() as u32).collect();
    while let Some(node) = work_list.pop_back() {
        log::trace!("Processing {}", node);
        let n = node as usize;
        let (gen, used) = &gen_used[n];
        let old_in = live_in[n].clone();

        for &succ in &cfg[node].successors {
            live_out[n] |= &*live_in[succ as usize];
        }

        live_in[n] = (!gen.clone() & &live_out[n]) | used;

        if old_in != *live_in[n] {
            for &pred in &cfg[node].predecessors {
                work_list.push_front(pred);
            }
        }
    }

    /*for &arg in backend.get_arguments().iter().flat_map(|arg| arg) {
        live_in[0].set(arg as usize, true)
    }*/

    (live_in, live_out)
}

#[derive(Debug, Clone)]
struct Locations {
    use_loc: SmallVec<[u32; 4]>,
    gen_loc: u32,
}

impl Locations {
    fn new() -> Locations {
        Locations {
            gen_loc: 0,
            use_loc: SmallVec::new(),
        }
    }
}

fn find_gen_used<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    ins_info: &InstructionInformation<R>,
    cfg: &ControlFlowGraph,
    numbers: &Renumber<R>,
    spill_code: &SpillCode,
) -> (Vec<(BitVec, BitVec)>, Vec<Locations>) {
    let mut result = Vec::new();
    let use_gen_loc = vec![Locations::new(); numbers.length];
    let instructions = backend.get_instructions();

    for block in cfg {
        let mut gen_set = BitVec::repeat(false, numbers.length);
        let mut use_set = BitVec::repeat(false, numbers.length);

        // This is not completely accurate due to the insertion of phicopies at an earlier stage
        // Instead the variables should already be live for all live_out in predececessors
        // However this should not matter
        // gen |= block.phi.output
        /*if let Some(phi) = block.phi(instructions) {
            for (i, &target) in phi.targets.iter().enumerate() {
                for &(location, _source) in &phi.sources[i] {
                    let location = cfg[location].last();
                    let target = numbers.translate(target, location) as usize;
                    gen_set.set(target, true);
                }
                //use_gen_loc[target as usize].gen_loc = block.instructions.start as u32;
            }
        }*/

        // used |= block.operands.input
        // gen |= block.operands.output
        for index in block
            .instructions
            .clone()
            .filter(|&i| ins_info.is_instruction[i])
        {
            let used = &ins_info.used[index];
            let result = &ins_info.result[index];
            let location = index as u32;

            for &(vreg, _) in used {
                let live = numbers.translate(vreg, location) as usize;
                if !gen_set[live] {
                    use_set.set(live, true);
                }
                //use_gen_loc[live].use_loc.push(index as u32);
            }
            for &(result, _) in result {
                let result = numbers.translate(result, location) as usize;
                gen_set.set(result, true);
                // use_gen_loc[result].gen_loc = index as u32;
            }
        }

        // use |= union succesors.phi.input from b
        for phi in block
            .successors
            .iter()
            .filter_map(|s| cfg[*s as usize].phi(instructions))
        {
            for (index, &target) in phi.targets.iter().enumerate() {
                for &(location, source) in &phi.sources[index] {
                    // Might have been changed wrong during phi change
                    if location == block.label {
                        let last_instruction = block.instructions.end - 1;
                        let location = last_instruction as u32;

                        let source = numbers.translate(source, location) as usize;
                        let target = numbers.translate(target, location) as usize;
                        if !gen_set[source] {
                            use_set.set(source, true);
                        }
                        gen_set.set(target, true);
                        //use_gen_loc[target].use_loc.push(location as u32);
                    }
                }
            }
        }

        // gen -= spilled variables
        // kill -= spilled variables
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
                gen_set.set(live_range, false);
                use_set.set(live_range, false);
            }
        }

        result.push((gen_set, use_set));
    }
    (result, use_gen_loc)
}
