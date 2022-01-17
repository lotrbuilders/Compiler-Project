pub mod briggs;
mod build;
mod coalesce;
mod graph;
mod instruction_information;
mod live_range;
mod renumber;
mod select;
mod simplify;
mod spill_code;
mod write_back;

pub use graph::Graph;
pub use live_range::LiveRange;
pub(self) use renumber::{renumber, Renumber};
