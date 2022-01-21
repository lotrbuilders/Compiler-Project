use bitvec::prelude::BitVec;
use std::{
    collections::{HashSet, VecDeque},
    fmt::Debug,
    iter::repeat,
};

use crate::backend::ir::*;

use super::ControlFlowGraph;
#[derive(Clone)]
pub struct LiveVariableAnalysis {
    live_in: Vec<BitVec>,
    live_out: Vec<BitVec>,
    gen: Vec<BitVec>,
    used: Vec<BitVec>,
}

pub fn live_variable(
    cfg: &ControlFlowGraph,
    function: &IRFunction,
    variables: Option<&HashSet<u32>>,
) -> LiveVariableAnalysis {
    let len = cfg.len();
    let var_count = function.variables.len();
    let mut analysis = LiveVariableAnalysis {
        live_in: vec![BitVec::repeat(false, var_count); len],
        live_out: vec![BitVec::repeat(false, var_count); len],
        gen: vec![BitVec::repeat(false, var_count); len],
        used: vec![BitVec::repeat(false, var_count); len],
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
            analysis.live_out[n] |= &analysis.live_in[succ as usize];
        }
        analysis.live_in[n] =
            (!analysis.gen[n].clone() & &analysis.live_out[n]) | &analysis.used[n];
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
                if !analysis.gen[block][variable as usize] {
                    analysis.used[block].set(variable as usize, true);
                }
            }
            Some(IRInstruction::Store(..)) => {
                analysis.gen[block].set(variable as usize, true);
            }
            _ => (),
        }
    }
}

impl Debug for LiveVariableAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.live_in.len();
        for i in 0..len {
            writeln!(
                f,
                "{} => in:{:?} out:{:?} gen:{:?} use:{:?}",
                i,
                self.live_in[i].iter_ones().collect::<Vec<_>>(),
                self.live_out[i].iter_ones().collect::<Vec<_>>(),
                self.gen[i].iter_ones().collect::<Vec<_>>(),
                self.used[i].iter_ones().collect::<Vec<_>>(),
            )?;
        }
        Ok(())
    }
}
