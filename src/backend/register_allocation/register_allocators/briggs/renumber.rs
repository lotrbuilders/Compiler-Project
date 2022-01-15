use smallvec::SmallVec;

use crate::backend::{
    ir::{control_flow_graph::ControlFlowGraph, IRInstruction},
    register_allocation::{RegisterBackend, RegisterInterface},
};

use super::live_range::LiveRange;

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
    pub fn from(&self, vreg2live: &Vec<Option<u32>>) -> u32 {
        match self {
            VregCopy::ArgumentCopy { reg, .. } | VregCopy::TargetAfter { reg, .. } => *reg,
            VregCopy::TargetBefore { vreg: from, .. }
            | VregCopy::TwoAddress { from, .. }
            | VregCopy::PhiCopy { from, .. } => vreg2live[*from as usize].unwrap(),
            VregCopy::Coalesced => unreachable!(),
        }
    }
    pub fn to(&self, vreg2live: &Vec<Option<u32>>) -> u32 {
        match self {
            VregCopy::TargetBefore { reg, .. } => *reg,
            VregCopy::ArgumentCopy { vreg: to, .. }
            | VregCopy::TargetAfter { vreg: to, .. }
            | VregCopy::TwoAddress { to, .. }
            | VregCopy::PhiCopy { to, .. } => vreg2live[*to as usize].unwrap(),
            VregCopy::Coalesced => unreachable!(),
        }
    }

    /// Gives merge destination and source
    pub fn destination<R: RegisterInterface>(&self, vreg2live: &Vec<Option<u32>>) -> (u32, u32) {
        match self {
            VregCopy::ArgumentCopy { .. } | VregCopy::TargetAfter { .. } => {
                (self.from(vreg2live), self.to(vreg2live))
            }
            VregCopy::TwoAddress { from, .. } | VregCopy::PhiCopy { from, .. } => {
                if vreg2live[*from as usize].unwrap() < R::REG_COUNT as u32 {
                    (self.from(vreg2live), self.to(vreg2live))
                } else {
                    (self.to(vreg2live), self.from(vreg2live))
                }
            }

            VregCopy::TargetBefore { .. } => (self.to(vreg2live), self.from(vreg2live)),
            VregCopy::Coalesced => unreachable!(),
        }
    }
}

pub struct Renumber<R: RegisterInterface> {
    pub vreg2live: Vec<Option<u32>>,
    pub live_ranges: Vec<LiveRange<R>>,
    //live_intervals: Vec<LiveInterval>,
    pub copies: Vec<SmallVec<[VregCopy; 2]>>,
    pub length: usize,
}

impl<R: RegisterInterface> Renumber<R> {
    pub fn translate(&self, vreg: u32) -> u32 {
        self.vreg2live[vreg as usize].unwrap()
    }
}

pub(super) fn renumber<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &B,
    cfg: &ControlFlowGraph,
) -> Renumber<R> {
    let instructions: &[_] = backend.get_instructions();
    let mut live_ranges = Vec::new();
    let mut copies = vec![SmallVec::new(); instructions.len()];
    let mut vreg2live = vec![None; backend.get_vreg_count() as usize];

    for &reg in R::REG_LOOKUP {
        live_ranges.push(LiveRange::reg(reg))
    }

    let mut index = 0;
    for arg in backend.get_arguments() {
        if let Some(arg) = *arg {
            let range = live_ranges.len() as u32;
            vreg2live[arg as usize] = Some(range);
            live_ranges.push(LiveRange::new(arg));

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

    for index in 0..instructions.len() {
        let rule = backend.get_rule(index);
        if backend.is_instruction(rule) {
            log::trace!("renumber index:{}", index);
            log::trace!("vreg2live: {:?}", vreg2live);
            if let IRInstruction::Label(Some(phi), _) = &instructions[index] {
                for (i, &result) in phi.targets.iter().enumerate() {
                    let live = live_ranges.len() as u32;
                    vreg2live[result as usize] = Some(live);
                    live_ranges.push(LiveRange::new(result as u32));

                    //let sources = &phi.sources[i];
                    for (sources, &location) in phi.sources.iter().zip(phi.locations.iter()) {
                        let source = sources[i];
                        let loc = cfg.graph[location as usize].last() as usize;
                        let _from = vreg2live[source as usize].unwrap();
                        let _to = vreg2live[result as usize].unwrap();
                        copies[loc].push(VregCopy::PhiCopy {
                            from: source,
                            to: result,
                        })
                    }
                }
            } else {
                let (used, result) = backend.get_vregisters(index as u32, rule);
                log::trace!("used: {:?}", used);
                log::trace!("result: {:?}", result);

                for (vreg, class) in used.iter() {
                    if let Some(target) = class.is_target() {
                        let _from = vreg2live[*vreg as usize].unwrap();
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
                    vreg2live[result as usize] = Some(live);
                    live_ranges.push(LiveRange::new(result));

                    if let Some(source) = class.is_target() {
                        let _to = vreg2live[result as usize].unwrap();
                        let copy = VregCopy::TargetAfter {
                            reg: source.into(),
                            vreg: result,
                        };
                        copies[index].push(copy);
                    }
                }

                if backend.is_two_address(rule) {
                    let (used, result) = (used[0].0, result.as_ref().unwrap().0);
                    let _from = vreg2live[used as usize].unwrap();
                    let _to = vreg2live[result as usize].unwrap();
                    let copy = VregCopy::TwoAddress {
                        from: used,
                        to: result,
                    };
                    copies[index].push(copy);
                }
            }
        }
    }

    /*let last = live_ranges
    .iter()
    .rev()
    .find(|&r| r.is_vreg())
    .map(|r| r.vregs[0])
    .unwrap_or(0);*/

    log::trace!("ranges:{:?}", live_ranges);
    log::trace!("vreg2live:{:?}", vreg2live);

    for index in 0..live_ranges.len() {
        let range = &live_ranges[index];
        if range.is_vreg() {
            let vreg = range.vregs[0];
            assert_eq!(vreg2live[vreg as usize], Some(index as u32))
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
