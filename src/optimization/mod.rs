use crate::{backend::ir::IRFunction, options::OptimizationSettings};

mod analysis;
mod mem2reg;

pub fn optimize(ir_functions: &mut [IRFunction], optimization_settings: &OptimizationSettings) {
    if optimization_settings.optimization_level >= 1 {
        for function in ir_functions {
            mem2reg::mem2reg(function);
        }
    }
}
