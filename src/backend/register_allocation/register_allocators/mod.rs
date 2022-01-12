use super::{RegisterBackend, RegisterInterface};

pub mod briggs;
pub mod simple;

trait RegisterAllocator<R: RegisterInterface, B: RegisterBackend<RegisterType = R>> {
    fn allocate_registers(backend: &mut B) -> ();
}

pub fn allocate_registers<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &mut B,
    register_allocator: &str,
) -> () {
    log::info!(
        "Allocating registers using {} register allocator",
        register_allocator
    );
    match register_allocator {
        "briggs" => RegisterAllocatorBriggs::allocate_registers(backend),
        "simple" => RegisterAllocatorSimple::allocate_registers(backend),
        _ => log::error!("unallowed register allocator {}", register_allocator),
    }
}

pub struct RegisterAllocatorBriggs {}
pub struct RegisterAllocatorSimple {}
#[allow(dead_code)]
pub struct RegisterAllocatorLinear {}
