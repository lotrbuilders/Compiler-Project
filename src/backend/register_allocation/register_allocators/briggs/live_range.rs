use smallvec::{smallvec, SmallVec};

use crate::backend::register_allocation::RegisterInterface;

#[derive(Clone, Debug)]
pub struct LiveRange<R: RegisterInterface> {
    pub vregs: SmallVec<[u32; 4]>,
    pub spill_cost: f32,
    pub precolor: Option<R>,
}

impl<R: RegisterInterface> LiveRange<R> {
    pub fn new(vreg: u32) -> LiveRange<R> {
        LiveRange {
            vregs: smallvec![vreg],
            spill_cost: 0.,
            precolor: None,
        }
    }
    pub fn reg(reg: R) -> LiveRange<R> {
        LiveRange {
            vregs: SmallVec::new(),
            spill_cost: f32::MAX,
            precolor: Some(reg),
        }
    }
    pub fn is_vreg(&self) -> bool {
        !self.vregs.is_empty()
    }
}

// Blocks are semi-inclusive: the end is defined somewhere in the last block
// Ranges are semi-inclusive: the end is at the start of the instruction
#[allow(dead_code)]
pub struct LiveInterval {
    blocks: SmallVec<[(i32, i32); 2]>,
    ranges: SmallVec<[(i32, i32); 2]>,
}
