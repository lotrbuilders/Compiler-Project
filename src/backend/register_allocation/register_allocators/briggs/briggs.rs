use crate::backend::register_allocation::briggs::instruction_information::InstructionInformation;
use crate::backend::register_allocation::briggs::spill_code::SpillCode;
use crate::backend::register_allocation::register_allocators::{
    RegisterAllocator, RegisterAllocatorBriggs,
};
use crate::backend::register_allocation::{RegisterBackend, RegisterInterface};
use crate::ir::ControlFlowGraph;

use super::build::build;
use super::renumber;
use super::select::select;
use super::simplify::simplify;
use super::write_back::write_back;

impl<R: RegisterInterface, B: RegisterBackend<RegisterType = R>> RegisterAllocator<R, B>
    for RegisterAllocatorBriggs
{
    fn allocate_registers(backend: &mut B) -> () {
        let mut spill_code = SpillCode::new(backend.get_instructions().len());
        let cfg = ControlFlowGraph::construct(backend.get_instructions());
        let ins_info = InstructionInformation::gather(backend);

        for _ in 0..100 {
            let renumber = renumber(backend, &ins_info, &cfg, &spill_code);
            let (mut graph, copies) = build(backend, &ins_info, &cfg, &spill_code, renumber);
            let stack = simplify(&mut graph);
            //log::trace!("stack:{:?}", stack);
            let color = select(&mut graph, stack);
            log::trace!("Color result:\n{:?}", color);
            match color {
                Ok(colors) => {
                    write_back(backend, &graph, &colors, &copies, &spill_code);
                    return;
                }
                Err(spills) => {
                    spill_code.generate(backend, &ins_info, &cfg, spills);
                    //break;
                }
            }
        }
        log::error!("Failed to allocate registers");
        std::process::exit(-3);
    }
}
