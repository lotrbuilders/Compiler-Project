use super::analysis::ControlFlowGraph;
use crate::backend::ir::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::mem;

/// Optimization which removes any block in the cfg without any predecessors.
/// Only removes dead blocks, not any unused variables.
// It first finds all blocks that need to be modified, removes them and renumbers them
pub fn eliminate_dead_blocks(function: &mut IRFunction) {
    let mut cfg = ControlFlowGraph::construct(&function.instructions);
    let dead_blocks = find_dead_blocks(&mut cfg);
    remove_blocks(function, &dead_blocks);
    renumber_blocks(function, &dead_blocks)
}

/// Finds all blocks of code without predecessors using a worklist.
/// Never removes block 0 and cannot remove loops.
/// Disconnects the block from the cfg.
fn find_dead_blocks(cfg: &mut ControlFlowGraph) -> HashSet<u32> {
    let mut work_list: VecDeque<_> = (1..cfg.len() as u32).collect();
    let mut dead_code = HashSet::new();

    while let Some(block) = work_list.pop_front() {
        if block == 0 {
            continue;
        }
        if cfg[block].predecessors.is_empty() {
            dead_code.insert(block);
            let successors = mem::take(&mut cfg[block].successors);
            for succ in successors {
                work_list.push_back(succ);
                cfg[succ].predecessors.retain(|b| *b != block);
            }
        }
    }

    dead_code
}

/// Removes block by only keeping instructions outside the loops
fn remove_blocks(function: &mut IRFunction, dead_blocks: &HashSet<u32>) {
    let instruction = mem::take(&mut function.instructions);
    function.instructions = instruction
        .into_iter()
        .scan(false, |remove, instruction| {
            if let &IRInstruction::Label(_, number) = &instruction {
                *remove = dead_blocks.contains(&number);
            }
            if *remove {
                Some(None)
            } else {
                Some(Some(instruction))
            }
        })
        .filter_map(|p| p)
        .collect();
}

/// Renumbers all blocks. Then rewrites all instructions which reference labels.
fn renumber_blocks(function: &mut IRFunction, dead_blocks: &HashSet<u32>) {
    let label_map: HashMap<u32, u32> = function
        .instructions
        .iter()
        .scan(0, |counter, instruction| {
            if let &IRInstruction::Label(_, number) = instruction {
                let kv = (number, *counter);
                *counter += 1;
                Some(Some(kv))
            } else {
                Some(None)
            }
        })
        .filter_map(|kv| kv)
        .collect();

    for instruction in &mut function.instructions {
        match instruction {
            IRInstruction::Jcc(.., label)
            | IRInstruction::Jnc(.., label)
            | IRInstruction::Label(None, label)
            | IRInstruction::Jmp(label) => *label = label_map[label],

            IRInstruction::Label(Some(phi), label) => {
                *label = label_map[label];
                for sources in &mut phi.sources {
                    sources.retain(|(label, _)| !dead_blocks.contains(&label));
                    for (label, _) in sources {
                        *label = label_map[label];
                    }
                }
            }
            _ => (),
        }
    }
}
