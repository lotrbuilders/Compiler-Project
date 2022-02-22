use crate::backend::register_allocation::{RegisterBackend, RegisterInterface};
use crate::ir::*;
use smallvec::SmallVec;
use std::ops::{Index, IndexMut, Range};

use super::{
    instruction_information::InstructionInformation,
    live_range::{IntervalVector, LiveRange},
    spill_code::{MemoryCopy, SpillCode},
};

// These use virtual registers and not liveranges
// The live ranges can move
// The virtual registers stay the same
#[derive(Debug, Clone, Copy)]
pub enum VregCopy {
    ArgumentCopy { reg: u32, vreg: u32 },
    TwoAddress { from: u32, to: u32 },
    TargetBefore { vreg: u32, reg: u32 },
    TargetAfter { reg: u32, vreg: u32 },
    PhiCopy { from: u32, to: u32 },
    Coalesced,
}

impl VregCopy {
    pub fn from(&self, vreg2live: &Vreg2Live, location: u32) -> u32 {
        match self {
            VregCopy::ArgumentCopy { reg, .. } | VregCopy::TargetAfter { reg, .. } => *reg,
            VregCopy::TargetBefore { vreg: from, .. }
            | VregCopy::TwoAddress { from, .. }
            | VregCopy::PhiCopy { from, .. } => vreg2live[*from as usize][location],
            VregCopy::Coalesced => unreachable!(),
        }
    }
    pub fn to(&self, vreg2live: &Vreg2Live, location: u32) -> u32 {
        match self {
            VregCopy::TargetBefore { reg, .. } => *reg,
            VregCopy::ArgumentCopy { vreg: to, .. }
            | VregCopy::TargetAfter { vreg: to, .. }
            | VregCopy::TwoAddress { to, .. }
            | VregCopy::PhiCopy { to, .. } => vreg2live[*to as usize][location],
            VregCopy::Coalesced => unreachable!(),
        }
    }

    /// Gives merge destination and source
    pub fn destination<R: RegisterInterface>(
        &self,
        vreg2live: &Vreg2Live,
        location: u32,
    ) -> (u32, u32) {
        match self {
            VregCopy::ArgumentCopy { .. } | VregCopy::TargetAfter { .. } => {
                (self.from(vreg2live, location), self.to(vreg2live, location))
            }
            VregCopy::TwoAddress { from, .. } | VregCopy::PhiCopy { from, .. } => {
                if vreg2live[*from as usize][location] < R::REG_COUNT as u32 {
                    (self.from(vreg2live, location), self.to(vreg2live, location))
                } else {
                    (self.to(vreg2live, location), self.from(vreg2live, location))
                }
            }

            VregCopy::TargetBefore { .. } => {
                (self.to(vreg2live, location), self.from(vreg2live, location))
            }
            VregCopy::Coalesced => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vreg2Live {
    map: Vec<IntervalVector<u32, u32>>,
}

impl Vreg2Live {
    pub fn new(length: u32) -> Vreg2Live {
        Vreg2Live {
            map: vec![IntervalVector::empty(); length as usize],
        }
    }
    pub fn iter<'a>(&'a self) -> std::slice::Iter<IntervalVector<u32, u32>> {
        self.map.iter()
    }
    pub fn insert(&mut self, vreg: u32, range: Range<u32>, live_range: u32) -> bool {
        if self.map[vreg as usize].is_empty() {
            self.map[vreg as usize].insert(range, live_range);
            true
        } else {
            false
        }
    }
}

impl Index<usize> for Vreg2Live {
    type Output = IntervalVector<u32, u32>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.map[index]
    }
}

impl Index<u32> for Vreg2Live {
    type Output = IntervalVector<u32, u32>;

    fn index(&self, index: u32) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<usize> for Vreg2Live {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.map[index]
    }
}

impl IndexMut<u32> for Vreg2Live {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

pub struct Renumber<R: RegisterInterface> {
    pub vreg2live: Vreg2Live,
    pub live_ranges: Vec<LiveRange<R>>,
    pub copies: Vec<SmallVec<[VregCopy; 2]>>,
    pub length: usize,
}

impl<R: RegisterInterface> Renumber<R> {
    pub fn translate(&self, vreg: u32, location: u32) -> u32 {
        self.vreg2live[vreg as usize][location]
    }
}

pub(super) fn renumber<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    ins_info: &InstructionInformation<R>,
    cfg: &ControlFlowGraph,
    spill_code: &SpillCode,
) -> Renumber<R> {
    let instructions: &[_] = backend.get_instructions();
    let range = 0..instructions.len() as u32;
    let mut live_ranges = Vec::new();
    let mut copies = vec![SmallVec::new(); instructions.len()];
    let mut vreg2live_old = vec![None; backend.get_vreg_count() as usize];
    let mut vreg2live = Vreg2Live::new(backend.get_vreg_count());

    log::debug!("Starting renumber phase");

    for &reg in R::REG_LOOKUP {
        live_ranges.push(LiveRange::reg(reg))
    }

    for (index, instruction) in spill_code
        .code
        .iter()
        .enumerate()
        .filter(|&(i, _)| ins_info.is_instruction[i])
    {
        let start = index as u32;
        let range = start..start + 1;
        for copy in instruction {
            match copy {
                &MemoryCopy::Spill(vreg) => {
                    let live_range = live_ranges.len() as u32;
                    live_ranges.push(LiveRange::new(vreg, range.clone()));
                    vreg2live[vreg].insert(range.clone(), live_range);
                }
                &MemoryCopy::Reload(vreg) => {
                    let live_range = live_ranges.len() as u32;
                    live_ranges.push(LiveRange::new(vreg, range.clone()));
                    vreg2live[vreg].insert(range.clone(), live_range);
                }
            }
        }
    }

    let mut index = 0;
    for arg in backend.get_arguments() {
        if let Some(arg) = *arg {
            let live_range = live_ranges.len() as u32;
            vreg2live_old[arg as usize] = Some(live_range);
            if vreg2live.insert(arg, range.clone(), live_range) {
                live_ranges.push(LiveRange::new(arg, range.clone()));
            }

            let source = R::CALL_REGS[index].is_target().unwrap();
            let source: usize = source.into();

            let copy = VregCopy::ArgumentCopy {
                reg: source as u32,
                vreg: arg,
            };
            copies[0].push(copy);
            index += 1;
        }
    }

    for (index, instruction) in instructions
        .iter()
        .enumerate()
        .filter(|&(i, _)| ins_info.is_instruction[i])
    {
        let rule = backend.get_rule(index);
        if let IRInstruction::Label(Some(phi), _) = instruction {
            for (i, &result) in phi.targets.iter().enumerate() {
                let live = live_ranges.len() as u32;
                vreg2live_old[result as usize] = Some(live);
                if vreg2live.insert(result, range.clone(), live) {
                    live_ranges.push(LiveRange::new(result as u32, range.clone()));
                }

                //let sources = &phi.sources[i];
                for &(location, source) in phi.sources[i].iter() {
                    //let = sources[i];
                    let loc = cfg.graph[location as usize].last() as usize;
                    copies[loc].push(VregCopy::PhiCopy {
                        from: source,
                        to: result,
                    })
                }
            }
        } else {
            let used = &ins_info.used[index];
            let result = &ins_info.result[index];

            for (vreg, class) in used.iter() {
                if let Some(target) = class.is_target() {
                    //let _from = vreg2live_old[*vreg as usize].unwrap();
                    let copy = VregCopy::TargetBefore {
                        vreg: *vreg,
                        reg: target.into(),
                    };
                    copies[index].push(copy);
                }
            }

            if let Some((result, class)) = &result {
                let result = *result;
                let live = live_ranges.len() as u32;
                vreg2live_old[result as usize] = Some(live);
                if vreg2live.insert(result, range.clone(), live) {
                    live_ranges.push(LiveRange::new(result, range.clone()));
                }

                if let Some(source) = class.is_target() {
                    //let _to = vreg2live_old[result as usize].unwrap();
                    let copy = VregCopy::TargetAfter {
                        reg: source.into(),
                        vreg: result,
                    };
                    copies[index].push(copy);
                }
            }

            if backend.is_two_address(rule) {
                let (used, result) = (used[0].0, result.as_ref().unwrap().0);
                //let _from = vreg2live_old[used as usize].unwrap();
                //let _to = vreg2live_old[result as usize].unwrap();
                let copy = VregCopy::TwoAddress {
                    from: used,
                    to: result,
                };
                copies[index].push(copy);
            }
        }
    }

    let length = live_ranges.len();
    Renumber {
        vreg2live,
        live_ranges,
        copies,
        length,
    }
}
