use crate::{backend::ir::IRFunction, options::OptimizationSettings};

use self::analysis::{ControlFlowGraph, DominatorTree};

mod analysis;
mod mem2reg;

pub fn optimize(ir_functions: &mut [IRFunction], optimization_settings: &OptimizationSettings) {
    if optimization_settings.optimization_level >= 1 {
        for ir_function in ir_functions {
            let cfg = ControlFlowGraph::construct(&ir_function.instructions);
            let _ = DominatorTree::new(&cfg);
        }
    }
}
