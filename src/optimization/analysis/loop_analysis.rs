use std::collections::HashSet;

use smallvec::{smallvec, SmallVec};

use super::{ControlFlowGraph, DominatorTree};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Loop {
    header: u32,
    body: SmallVec<[u32; 4]>,
    back_edges: SmallVec<[u32; 4]>,
}

impl Loop {
    pub fn depth(loops: &[Loop], length: usize) -> Vec<u32> {
        let mut result = vec![0; length];

        for lp in loops {
            for &b in &lp.body {
                result[b as usize] += 1;
            }
        }

        result
    }
}

pub fn loops(cfg: &ControlFlowGraph, dom_tree: &DominatorTree) -> Vec<Loop> {
    let mut loops = Vec::new();

    for b in 0..cfg.len() {
        for &header in cfg[b]
            .successors
            .iter()
            .filter(|&&succ| dom_tree.dominates(succ, b as u32))
        {
            let body = search_loop_body(cfg, b, header);
            let back_edges = smallvec![b as u32];
            loops.push(Loop {
                header,
                body,
                back_edges,
            });
        }
    }

    loops
}

fn search_loop_body(cfg: &ControlFlowGraph, back_edge: usize, header: u32) -> SmallVec<[u32; 4]> {
    let mut loop_body = smallvec![header];
    let header = header as usize;
    let mut stack = vec![back_edge];
    let mut visited = HashSet::new();

    while let Some(block) = stack.pop() {
        visited.insert(block);
        loop_body.push(block as u32);

        stack.extend(
            cfg[block]
                .predecessors
                .iter()
                .map(|&p| p as usize)
                .filter(|&pred| pred != header)
                .filter(|pred| !visited.contains(pred)),
        );
    }

    loop_body
}
