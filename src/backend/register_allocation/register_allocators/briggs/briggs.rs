use crate::backend::ir::control_flow_graph::ControlFlowGraph;
use crate::backend::register_allocation::register_allocators::{
    RegisterAllocator, RegisterAllocatorBriggs,
};
use crate::backend::register_allocation::{RegisterBackend, RegisterInterface};

use super::build::build;
use super::renumber;
use super::select::select;
use super::simplify::simplify;
use super::write_back::write_back;

impl<R: RegisterInterface, B: RegisterBackend<RegisterType = R>> RegisterAllocator<R, B>
    for RegisterAllocatorBriggs
{
    fn allocate_registers(backend: &mut B) -> () {
        let cfg = ControlFlowGraph::construct(backend.get_instructions());
        let renumber = renumber(backend, &cfg);
        let (mut graph, copies) = build(backend, &cfg, renumber);
        let stack = simplify(&mut graph);
        log::trace!("stack:{:?}", stack);
        let color = select(&mut graph, stack);
        log::trace!("Color result:\n{:?}", color);
        match &color {
            Ok(colors) => write_back(backend, &graph, colors, &copies),
            Err(spills) => {
                let _ = spills;
                todo!();
            }
        }
    }
}
