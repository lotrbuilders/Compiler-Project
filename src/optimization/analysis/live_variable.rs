use std::{
    collections::{HashSet, VecDeque},
    iter::repeat,
};

use crate::backend::ir::*;

use super::ControlFlowGraph;
#[derive(Debug, Clone)]
pub struct LiveVariableAnalysis {
    live_in: Vec<HashSet<u32>>,
    live_out: Vec<HashSet<u32>>,
    gen: Vec<HashSet<u32>>,
    used: Vec<HashSet<u32>>,
}
pub fn live_variable(
    cfg: &ControlFlowGraph,
    function: &IRFunction,
    variables: Option<&HashSet<u32>>,
) -> LiveVariableAnalysis {
    let len = cfg.len();
    let mut analysis = LiveVariableAnalysis {
        live_in: vec![HashSet::new(); len],
        live_out: vec![HashSet::new(); len],
        gen: vec![HashSet::new(); len],
        used: vec![HashSet::new(); len],
    };

    find_local_use(
        &mut analysis,
        cfg,
        &function.instructions,
        variables.unwrap(),
    );

    work_list(&mut analysis, cfg);
    log::debug!("Live variable analysis output: {:?}", analysis);

    analysis
}

fn work_list(analysis: &mut LiveVariableAnalysis, cfg: &ControlFlowGraph) {
    let mut work_list: VecDeque<u32> = (0..cfg.len() as u32).collect();

    while let Some(node) = work_list.pop_back() {
        log::trace!("Processing {}", node);
        let n = node as usize;
        let old_in = analysis.live_in[n].clone();

        for &succ in &cfg[node].successors {
            analysis.live_out[n] = &analysis.live_out[n] | &analysis.live_in[succ as usize];
        }
        analysis.live_in[n] = &analysis.used[n] | &(&analysis.live_out[n] - &analysis.gen[n]);
        if old_in != analysis.live_in[n] {
            for &pred in &cfg[node].predecessors {
                work_list.push_front(pred);
            }
        }
    }
}

fn find_local_use(
    analysis: &mut LiveVariableAnalysis,
    cfg: &ControlFlowGraph,
    instructions: &[IRInstruction],
    variables: &HashSet<u32>,
) {
    for (block, index, variable) in (0..cfg.len())
        .flat_map(|block| cfg[block].instructions.clone().zip(repeat(block)))
        .filter_map(|(index, block)| {
            if let &IRInstruction::AddrL(_, _, variable) = &instructions[index] {
                let variable = variable as u32;
                Some((block, index, variable))
            } else {
                None
            }
        })
        .filter(|(_, _, variable)| variables.contains(&variable))
    {
        let destination = instructions.get(index + 1);
        match destination {
            Some(IRInstruction::Load(..)) => {
                if !analysis.gen[block].contains(&variable) {
                    analysis.used[block].insert(variable);
                }
            }
            Some(IRInstruction::Store(..)) => {
                analysis.gen[block].insert(variable);
            }
            _ => (),
        }
    }
}
