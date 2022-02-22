use crate::ir::*;

#[allow(dead_code)]
pub fn vreg_size(function: &IRFunction, int_size: IRSize) -> Vec<IRSize> {
    let vreg_count = function.vreg_count;
    let mut result = vec![int_size; vreg_count as usize];

    for instruction in &function.instructions {
        for res in instruction.get_result() {
            result[res as usize] = instruction.get_result_size(int_size);
        }
    }

    result
}
