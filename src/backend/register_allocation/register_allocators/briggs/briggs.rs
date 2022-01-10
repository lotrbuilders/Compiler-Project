use std::collections::HashSet;

use crate::backend::register_allocation::register_allocators::{
    RegisterAllocator, RegisterAllocatorBriggs,
};
use crate::backend::register_allocation::{
    ralloc::*, RegisterAllocation, RegisterBackend, RegisterClass, RegisterInterface,
    RegisterLocation::*,
};

use crate::backend::ir::control_flow_graph::ControlFlowGraph;
use crate::backend::ir::IRInstruction;

impl<R: RegisterInterface, B: RegisterBackend<RegisterType = R>> RegisterAllocator<R, B>
    for RegisterAllocatorBriggs
{
    fn allocate_registers(backend: &mut B) -> () {}
}
