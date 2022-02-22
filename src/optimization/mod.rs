use crate::{ir::IRFunction, options::OptimizationSettings};

pub mod analysis;
mod dead_block_elimination;
mod mem2reg;
mod remove_variable;

use dead_block_elimination as dbe;

pub fn optimize(ir_functions: &mut [IRFunction], optimization_settings: &OptimizationSettings) {
    if optimization_settings.optimization_level >= 1 {
        for function in &mut *ir_functions {
            dbe::eliminate_dead_blocks(function);
        }
    }
    if optimization_settings.optimization_level >= 1 {
        for function in &mut *ir_functions {
            mem2reg::mem2reg(function);
        }
    }
}
