use std::{
    fmt::{self, Display},
    ops::{Index, IndexMut, Range},
};

use smallvec::SmallVec;

use super::{ir_phi::IRPhi, IRInstruction};

pub struct ControlFlowNode {
    pub predecessors: SmallVec<[u32; 4]>,
    pub successors: SmallVec<[u32; 4]>,
    pub instructions: Range<usize>,
    pub label: u32,
}

impl Display for ControlFlowNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} -> {} -> {:?}",
            self.predecessors, self.label, self.successors
        )
    }
}

impl ControlFlowNode {
    pub fn new(range: Range<usize>, label: u32) -> ControlFlowNode {
        ControlFlowNode {
            predecessors: SmallVec::new(),
            successors: SmallVec::new(),
            instructions: range,
            label,
        }
    }
    pub fn last(&self) -> u32 {
        return (self.instructions.end - 1) as u32;
    }
}

impl<'a> ControlFlowNode {
    pub fn phi(&self, instructions: &'a Vec<IRInstruction>) -> Option<&'a IRPhi> {
        if let IRInstruction::Label(Some(phi), _) = &instructions[self.instructions.start] {
            Some(phi)
        } else {
            None
        }
    }
}

pub struct ControlFlowGraph {
    pub graph: Vec<ControlFlowNode>,
}

pub type CFG = ControlFlowGraph;

impl ControlFlowGraph {
    pub fn to_string(cfg: &ControlFlowGraph) -> String {
        let mut result = String::new();
        for block in &cfg.graph {
            result.push_str(&format!("{}\n", block));
        }
        result
    }

    pub fn check(cfg: &ControlFlowGraph) {
        for (block, i) in cfg.graph.iter().zip(0..) {
            assert_eq!(
                block.label, i,
                "The label of a cfg block({}) and it's index({}) must be equal.",
                block.label, i
            )
        }
    }

    pub fn find_successors(cfg: &mut ControlFlowGraph, instructions: &Vec<IRInstruction>) {
        let length = cfg.len() as u32;
        for (block, i) in cfg.graph.iter_mut().zip(0..) {
            let end = block.instructions.end - 1;
            use IRInstruction::*;
            match instructions[end] {
                Jmp(next) => block.successors.push(next),
                Jcc(.., next) | Jnc(.., next) => {
                    block.successors.push(next);
                    block.successors.push(i + 1)
                }
                Ret(..) => (),
                _ if (i + 1) < length => block.successors.push(i + 1), // The last instruction in the last block has no successors
                _ => (),
            }
        }

        let length = cfg.graph.len();
        if let Some(block) = cfg.graph.last_mut() {
            block.successors = block
                .successors
                .iter()
                .filter(|&&i| (i as usize) < length)
                .map(|i| *i)
                .collect();
        }
    }

    pub fn find_predecessors(cfg: &mut ControlFlowGraph) {
        for block in 0..cfg.graph.len() {
            for successor in cfg.graph[block].successors.clone() {
                cfg.graph[successor as usize]
                    .predecessors
                    .push(block as u32);
            }
        }
    }

    pub fn construct(instructions: &Vec<IRInstruction>) -> ControlFlowGraph {
        log::info!("Constructing CFG");
        let mut cfg = Vec::new();
        let mut start = 0;
        let mut label = 0;
        for (ins, i) in instructions.iter().zip(0usize..) {
            use IRInstruction::*;
            match ins {
                Label(_, lbl) => {
                    if !(start..i).is_empty() {
                        cfg.push(ControlFlowNode::new(start..i, label))
                    }
                    start = i;
                    label = *lbl;
                }
                _ => (),
            }
        }
        if !(start..instructions.len()).is_empty() {
            cfg.push(ControlFlowNode::new(start..instructions.len(), label))
        }
        let mut cfg = ControlFlowGraph { graph: cfg };
        log::info!("CFG:\n{}", CFG::to_string(&cfg));
        CFG::check(&cfg);
        CFG::find_successors(&mut cfg, instructions);
        log::info!("CFG:\n{}", CFG::to_string(&cfg));
        CFG::find_predecessors(&mut cfg);
        log::info!("CFG:\n{}", CFG::to_string(&cfg));

        cfg
    }
}

impl Index<usize> for ControlFlowGraph {
    type Output = ControlFlowNode;
    fn index(&self, index: usize) -> &Self::Output {
        &self.graph[index]
    }
}
impl IndexMut<usize> for ControlFlowGraph {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.graph[index]
    }
}
impl Index<u32> for ControlFlowGraph {
    type Output = ControlFlowNode;
    fn index(&self, index: u32) -> &Self::Output {
        &self[index as usize]
    }
}
impl IndexMut<u32> for ControlFlowGraph {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
impl<'a> IntoIterator for &'a ControlFlowGraph {
    type IntoIter = std::slice::Iter<'a, ControlFlowNode>;
    type Item = &'a ControlFlowNode;
    fn into_iter(self) -> Self::IntoIter {
        self.graph.iter()
    }
}

impl<'a> ControlFlowGraph {
    pub fn iter(&'a self) -> std::slice::Iter<'a, ControlFlowNode> {
        self.into_iter()
    }
    pub fn len(&self) -> usize {
        self.graph.len()
    }
}

/*
pub struct CFGRevPostIter<'a> {
    cfg: &'a ControlFlowGraph,
    visited: HashSet<u32>,
    stack: Vec<u32>,
}

impl<'a> Iterator for CFGRevPostIter<'a> {
    type Item = ControlFlowNode;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            if !self.visited.contains(&node) {
                self.visited.insert(node);

            }
        }
        None
    }
}*/
