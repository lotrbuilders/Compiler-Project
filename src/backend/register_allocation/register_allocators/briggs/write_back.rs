use smallvec::SmallVec;

use crate::backend::register_allocation::{
    RegisterAllocation, RegisterBackend, RegisterInterface, RegisterRelocation,
};

use super::{renumber::VregCopy, Graph};

fn translate_color<R: RegisterInterface>(
    colors: &Vec<R>,
    vreg2live: &Vec<Option<u32>>,
    vreg: u32,
) -> R {
    colors[vreg2live[vreg as usize].unwrap() as usize]
}

pub(super) fn write_back<R: RegisterInterface, B: RegisterBackend<RegisterType = R>>(
    backend: &mut B,
    graph: &Graph<R>,
    colors: &Vec<R>,
    copies: &Vec<SmallVec<[VregCopy; 2]>>,
) {
    let vreg_count = backend.get_vreg_count() as usize;
    let instruction_count = copies.len();
    let everywhere = 0..(instruction_count + 1) as u32;
    let vreg2live = &graph.vreg2live;
    let mut allocation = vec![RegisterAllocation::empty(); vreg_count];
    let mut relocation = vec![Vec::new(); instruction_count];
    let mut used_registers = vec![false; R::REG_COUNT];

    for (i, live_range) in graph.live_ranges.iter().enumerate() {
        for &vreg in &live_range.vregs {
            allocation[vreg as usize] = RegisterAllocation::range(colors[i], everywhere.clone());
            let color_index: usize = colors[i].into();
            used_registers[color_index] = true;
        }
    }

    for (i, copies) in copies.iter().enumerate() {
        let relocation = &mut relocation[i];
        for copy in copies {
            use RegisterRelocation::*;
            match copy {
                VregCopy::ArgumentCopy { reg, vreg } => {
                    let from = R::REG_LOOKUP[*reg as usize];
                    let to = translate_color(colors, vreg2live, *vreg);
                    relocation.push(Move(from, to));
                    used_registers[*reg as usize] = true;
                }
                VregCopy::TwoAddress { from, to } => {
                    let from = translate_color(colors, vreg2live, *from);
                    let to = translate_color(colors, vreg2live, *to);
                    relocation.push(TwoAddressMove(from, to));
                }
                VregCopy::TargetBefore { vreg, reg } => {
                    let from = translate_color(colors, vreg2live, *vreg);
                    let to = R::REG_LOOKUP[*reg as usize];
                    relocation.push(Move(from, to));
                    allocation[*vreg as usize].insert(to, i as u32);
                    used_registers[*reg as usize] = true;
                }
                VregCopy::TargetAfter { reg, vreg } => {
                    let from = R::REG_LOOKUP[*reg as usize];
                    let to = translate_color(colors, vreg2live, *vreg);
                    relocation.push(MoveAfter(from, to));
                    allocation[*vreg as usize].insert(from, i as u32);
                    used_registers[*reg as usize] = true;
                }
                VregCopy::PhiCopy { from, to } => {
                    let from = translate_color(colors, vreg2live, *from);
                    let to = translate_color(colors, vreg2live, *to);
                    relocation.push(Move(from, to));
                }
                VregCopy::Coalesced => (),
            }
        }
    }
    log::trace!("used registers: {:?}", used_registers);
    backend.set_allocation(allocation);
    backend.set_reg_relocations(relocation);
    backend.set_used_registers(used_registers);
}
