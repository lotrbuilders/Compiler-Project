use crate::backend::ir::*;
use smallvec::SmallVec;

pub struct UseAnalysis {
    pub use_count: Vec<u32>,
    pub uses: Vec<SmallVec<[u32; 4]>>,
}

#[allow(dead_code)]
pub fn use_analysis(function: &IRFunction) -> UseAnalysis {
    private_use_analysis(function)
}
#[allow(dead_code)]
pub fn find_vreg_uses(function: &IRFunction) -> Vec<SmallVec<[u32; 4]>> {
    private_use_analysis(function).uses
}

pub fn find_vreg_use_count(function: &IRFunction) -> Vec<u32> {
    private_use_analysis(function).use_count
}

#[inline(always)]
fn private_use_analysis(function: &IRFunction) -> UseAnalysis {
    let length = function.vreg_count as usize;
    let mut use_count = vec![0; length];
    let mut uses = vec![SmallVec::new(); length];
    for (index, instruction) in function.instructions.iter().enumerate() {
        for usage in instruction.get_used_vreg() {
            use_count[usage as usize] += 1;
            uses[usage as usize].push(index as u32);
        }
    }

    UseAnalysis { use_count, uses }
}
