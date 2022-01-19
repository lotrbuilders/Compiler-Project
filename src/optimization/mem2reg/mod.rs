mod promotable;

use promotable::find_promotable_variables;

use std::collections::HashSet;

use crate::backend::ir::*;

use super::analysis::{self, ControlFlowGraph};
use analysis::{find_vreg_use_count, DominatorTree};

pub fn mem2reg(function: &mut IRFunction) {
    let cfg = ControlFlowGraph::construct(&function.instructions);
    let vreg_use_count = find_vreg_use_count(function);
    let dominator_tree = DominatorTree::new(&cfg);

    let candidates = &function.variables;
    let promotions = find_promotable_variables(&function.instructions, &vreg_use_count, candidates);
    log::info!("Promotable variables:{:?}", promotions);
}
