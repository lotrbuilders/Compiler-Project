use crate::backend::ir::*;

pub fn find_vreg_use_count(function: &IRFunction) -> Vec<u32> {
    let length = function.vreg_count as usize;
    let mut use_count = vec![0; length];
    for instruction in &function.instructions {
        for usage in instruction.get_used_vreg() {
            use_count[usage as usize] += 1;
        }
    }
    use_count
}
