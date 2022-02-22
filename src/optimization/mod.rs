use crate::{ir::*, options::OptimizationSettings};

pub mod analysis;
mod dead_block_elimination;
mod mem2reg;
mod remove_variable;

use dead_block_elimination as dbe;

pub fn optimize(module: &mut IRModule, optimization_settings: &OptimizationSettings) {
    if optimization_settings.optimization_level >= 1 {
        for function in &mut module.functions {
            dbe::eliminate_dead_blocks(function);
        }
    }
    if optimization_settings.optimization_level >= 1 {
        for function in &mut module.functions {
            mem2reg::mem2reg(function);
        }
    }
}
