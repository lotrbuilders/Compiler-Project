pub mod dominator_tree;
pub mod live_variable;
pub mod loop_analysis;
pub mod use_analysis;
pub mod vreg_size;

pub use crate::ir::ControlFlowGraph;
pub use dominator_tree::*;
pub use live_variable::*;
pub use loop_analysis::*;
pub use use_analysis::*;
pub use vreg_size::*;
