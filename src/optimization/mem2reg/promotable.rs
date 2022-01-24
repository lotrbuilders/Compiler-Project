use crate::backend::ir::*;
use std::collections::HashSet;

impl IRSize {
    fn is_promotable(&self) -> bool {
        match self {
            IRSize::S8 | IRSize::S16 | IRSize::S32 | IRSize::S64 | IRSize::P => true,
            IRSize::V | IRSize::B(_) => false,
        }
    }
}

// Performs very simple escape analysis
// Any use of AddrL that not immediately used by a store or a load discards that variable
// This is about the smallest subset of possible promotions, but is always save
pub(super) fn find_promotable_variables(
    instructions: &[IRInstruction],
    use_count: &[u32],
    candidates: &[IRVariable],
    arguments: &IRArguments,
) -> HashSet<u32> {
    // If a variable is not a basic type (pointer,integer,[float]) it cannot be promoted
    let mut candidates: HashSet<_> = candidates
        .iter()
        .filter(|&candidate| candidate.size.is_promotable() && candidate.count == 1)
        .map(|candidate| candidate.number)
        .collect();

    // Remove all stack arguments(These could be promoted, but that requires a beter optimization framework then currently available)
    let arg_iter = arguments.arguments.iter().zip(arguments.variables.iter());
    for (_argument, variable) in arg_iter.filter(|&(&arg, _var)| arg.is_none()) {
        if let Some(variable) = variable {
            candidates.remove(variable);
        }
    }

    for (index, instruction) in instructions.iter().enumerate() {
        // AddrL is the only way to access the address of a variable, other instructions can be discarded for escape analysis
        let (variable, vreg) = if let &IRInstruction::AddrL(_, vreg, variable) = instruction {
            (variable as u32, vreg)
        } else {
            continue;
        };

        // Variables that have already been discarded do not need to be analyzed further
        if !candidates.contains(&variable) {
            continue;
        }

        // If the address is used more then once the variable might still be unsafe
        if use_count[vreg as usize] > 1 {
            candidates.remove(&variable);
        }

        // Only loads or stores immediately following the load are accepted(this is significantly more severe then technically necessary,but sufficient for most needs)
        match instructions.get(index + 1) {
            Some(&IRInstruction::Load(_, _, address)) if address == vreg => (),
            Some(&IRInstruction::Store(_, src, address)) if address == vreg && src != vreg => (),
            None => (),
            _ => {
                candidates.remove(&variable);
            }
        }
    }

    candidates
}
