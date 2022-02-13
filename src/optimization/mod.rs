use crate::{backend::ir::IRFunction, options::OptimizationSettings};

pub mod analysis;
mod dead_code_elimination;
mod mem2reg;
mod remove_variable;

use dead_code_elimination as dce;

pub fn optimize(ir_functions: &mut [IRFunction], optimization_settings: &OptimizationSettings) {
    if optimization_settings.optimization_level >= 1 {
        for function in &mut *ir_functions {
            dce::eliminate_dead_code(function);
        }
    }
    if optimization_settings.optimization_level >= 1 {
        for function in &mut *ir_functions {
            mem2reg::mem2reg(function);
        }
    }
}
