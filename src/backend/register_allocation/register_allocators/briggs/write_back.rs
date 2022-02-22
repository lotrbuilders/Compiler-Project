use smallvec::SmallVec;

use crate::backend::register_allocation::{
    briggs::spill_code::MemoryCopy, RegisterAllocation, RegisterBackend, RegisterInterface,
    RegisterRelocation,
};

use super::{
    renumber::{Vreg2Live, VregCopy},
    spill_code::SpillCode,
    Graph,
};

fn translate_color<R: RegisterInterface>(
    colors: &Vec<R>,
    vreg2live: &Vreg2Live,
    vreg: u32,
    location: u32,
) -> R {
    colors[vreg2live[vreg as usize][location] as usize]
}

pub(super) fn write_back<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &mut B,
    graph: &Graph<R>,
    colors: &Vec<R>,
    copies: &Vec<SmallVec<[VregCopy; 2]>>,
    spill_code: &SpillCode,
) {
    let vreg_count = backend.get_vreg_count() as usize;
    let instruction_count = copies.len();
    let everywhere = 0..(instruction_count + 1) as u32;
    let vreg2live = &graph.vreg2live;
    let mut allocation = vec![RegisterAllocation::empty(); vreg_count];
    let mut relocation = vec![Vec::new(); instruction_count];
    let mut used_registers = vec![false; R::REG_COUNT];

    //log::debug!("writeback");
    //log::debug!("copies: {:?}", copies);
    //log::debug!("spill_code: {:?}", spill_code);

    for (i, live_range) in graph.live_ranges.iter().enumerate() {
        for &vreg in &live_range.vregs {
            if !spill_code.contains(vreg) {
                allocation[vreg as usize] =
                    RegisterAllocation::range(colors[i], everywhere.clone());
                let color_index: usize = colors[i].into();
                used_registers[color_index] = true;
            }
        }
    }

    for &vreg in spill_code.spills() {
        for (range, live_range) in &vreg2live[vreg] {
            let live_range = *live_range as usize;
            let reg = colors[live_range];
            allocation[vreg as usize].insert(reg, range.start);
            let color_index: usize = colors[live_range].into();
            used_registers[color_index] = true;
        }
    }

    for (i, copies) in spill_code.code.iter().enumerate() {
        let location = i as u32;
        let relocation = &mut relocation[i];
        for copy in copies {
            match copy {
                &MemoryCopy::Reload(vreg) => {
                    let size = backend.get_vreg_size(vreg);
                    let from = backend.simple_get_spot(spill_code.get_slot(vreg));
                    let live_range = vreg2live[vreg][location];
                    let to = colors[live_range as usize];
                    relocation.push(RegisterRelocation::Reload(size, to, from));
                }
                _ => (),
            }
        }
    }

    for (i, copies) in copies.iter().enumerate() {
        let location = i as u32;
        let relocation = &mut relocation[i];
        for copy in copies {
            use RegisterRelocation::*;
            match copy {
                VregCopy::ArgumentCopy { reg, vreg } => {
                    let size = backend.get_vreg_size(*vreg);
                    let from = R::REG_LOOKUP[*reg as usize];
                    let to = translate_color(colors, vreg2live, *vreg, location);
                    relocation.push(Move(size, from, to));
                    used_registers[*reg as usize] = true;
                }
                VregCopy::TwoAddress { from, to } => {
                    let size = backend.get_vreg_size(*to);
                    let from = translate_color(colors, vreg2live, *from, location);
                    let to = translate_color(colors, vreg2live, *to, location);
                    relocation.push(TwoAddressMove(size, from, to));
                }
                VregCopy::TargetBefore { vreg, reg } => {
                    let size = backend.get_vreg_size(*vreg);
                    let from = translate_color(colors, vreg2live, *vreg, location);
                    let to = R::REG_LOOKUP[*reg as usize];
                    relocation.push(Move(size, from, to));
                    allocation[*vreg as usize].insert(to, i as u32);
                    used_registers[*reg as usize] = true;
                }
                VregCopy::TargetAfter { reg, vreg } => {
                    let size = backend.get_vreg_size(*vreg);
                    let from = R::REG_LOOKUP[*reg as usize];
                    let to = translate_color(colors, vreg2live, *vreg, location);
                    relocation.push(MoveAfter(size, from, to));
                    allocation[*vreg as usize].insert(from, i as u32);
                    used_registers[*reg as usize] = true;
                }
                VregCopy::PhiCopy { from, to } => {
                    let size = backend.get_vreg_size(*to);
                    let from = translate_color(colors, vreg2live, *from, location);
                    let to = translate_color(colors, vreg2live, *to, location);
                    relocation.push(Move(size, from, to));
                }
                VregCopy::Coalesced => (),
            }
        }
    }

    for (i, copies) in spill_code.code.iter().enumerate() {
        let location = i as u32;
        let relocation = &mut relocation[i];
        for spill in copies.iter().filter(|&code| code.is_spill()) {
            let vreg = spill.vreg();
            let live_range = vreg2live[vreg][location];
            let from = colors[live_range as usize];
            let to = backend.simple_get_spot(spill_code.get_slot(vreg));
            let size = backend.get_vreg_size(vreg);
            if backend.is_jump(i) {
                relocation.push(RegisterRelocation::SpillEarly(size, from, to));
            } else {
                relocation.push(RegisterRelocation::Spill(size, from, to));
            }
        }
    }

    log::trace!("used registers: {:?}", used_registers);
    backend.set_allocation(allocation);
    backend.set_reg_relocations(relocation);
    backend.set_used_registers(used_registers);
    backend.simple_adjust_stack_size(spill_code.get_last_slot() as i32)
}
