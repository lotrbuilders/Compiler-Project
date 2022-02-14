use std::collections::{HashMap, HashSet};

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
    let mut loops = HashMap::new();

    for b in 0..cfg.len() {
        for &header in cfg[b]
            .successors
            .iter()
            .filter(|&&succ| dom_tree.dominates(succ, b as u32))
        {
            let mut body = search_loop_body(cfg, b, header);
            body.sort();
            let entry = loops.entry(header).or_insert_with(|| Loop {
                header,
                body: SmallVec::new(),
                back_edges: SmallVec::new(),
            });
            entry.back_edges.push(b as u32);
            entry.body.append(&mut body);
            entry.body.sort();
            entry.body.dedup();
        }
    }

    loops.into_values().collect()
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
