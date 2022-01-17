use std::collections::{HashMap, HashSet};

use smallvec::SmallVec;

use crate::backend::{
    ir::control_flow_graph::ControlFlowGraph,
    register_allocation::{RegisterBackend, RegisterInterface},
};

use super::instruction_information::InstructionInformation;

#[derive(Debug, Clone, Copy)]
pub enum MemoryCopy {
    Spill(u32),
    Reload(u32),
}

impl MemoryCopy {
    pub fn vreg(&self) -> u32 {
        match self {
            &MemoryCopy::Reload(vreg) | &MemoryCopy::Spill(vreg) => vreg,
        }
    }
    pub fn is_reload(&self) -> bool {
        matches!(self, MemoryCopy::Reload(..))
    }
    pub fn is_spill(&self) -> bool {
        matches!(self, MemoryCopy::Spill(..))
    }
}

#[derive(Clone, Debug)]
pub struct SpillCode {
    next_slot: u32,
    map: HashMap<u32, u32>,
    spills: HashSet<u32>,
    pub code: Vec<SmallVec<[MemoryCopy; 2]>>,
}

impl SpillCode {
    pub(super) fn new(length: usize) -> SpillCode {
        SpillCode {
            next_slot: 0,
            map: HashMap::new(),
            spills: HashSet::new(),
            code: vec![SmallVec::new(); length],
        }
    }

    pub fn get_slot(&self, vreg: u32) -> u32 {
        self.map[&vreg]
    }

    pub fn get_last_slot(&self) -> u32 {
        self.next_slot
    }

    pub fn contains(&self, vreg: u32) -> bool {
        self.spills.contains(&vreg)
    }

    pub fn spills<'a>(&'a self) -> &'a HashSet<u32> {
        &self.spills
    }

    fn insert(&mut self, vreg: u32) {
        assert!(!self.map.contains_key(&vreg));
        assert!(!self.spills.contains(&vreg));

        let slot = self.next_slot;
        self.next_slot += 1;
        self.map.insert(vreg, slot);
        self.spills.insert(vreg);
    }

    pub(super) fn generate<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
        &mut self,
        backend: &B,
        ins_info: &InstructionInformation<R>,
        cfg: &ControlFlowGraph,
        spills: HashSet<u32>,
    ) {
        for &spill in &spills {
            self.insert(spill)
        }

        for arg in backend.get_arguments().iter().filter_map(|x| *x) {
            if spills.contains(&arg) {
                self.code[0].push(MemoryCopy::Spill(arg))
            }
        }

        let instructions = backend.get_instructions();
        for block in cfg {
            if let Some(phi) = block.phi(instructions) {
                for (i, &result) in phi.targets.iter().enumerate() {
                    for (sources, &location) in phi.sources.iter().zip(phi.locations.iter()) {
                        let source = sources[i];
                        let loc = cfg.graph[location as usize].last() as usize;
                        if spills.contains(&source) {
                            self.code[loc].push(MemoryCopy::Reload(source))
                        }
                        if spills.contains(&result) {
                            self.code[loc].push(MemoryCopy::Spill(result))
                        }
                    }
                }
            }

            let instructions = block.instructions.clone();
            for index in instructions.filter(|&i| ins_info.is_instruction[i]) {
                let used = &ins_info.used[index];
                let result = &ins_info.result[index];
                for &(vreg, _) in used {
                    if spills.contains(&vreg) {
                        self.code[index].push(MemoryCopy::Reload(vreg))
                    }
                }
                for &(vreg, _) in result {
                    if spills.contains(&vreg) {
                        self.code[index].push(MemoryCopy::Spill(vreg))
                    }
                }
            }
        }

        log::info!("spill_code:{:?}", self)
    }
}
