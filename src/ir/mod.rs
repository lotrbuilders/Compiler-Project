pub mod control_flow_graph;
pub mod ir;
pub mod ir_phi;
pub mod print_ir;

pub use self::control_flow_graph::*;
pub use self::ir::*;
pub use ir_phi::*;

// Get the indices at which virtual registers are defined
pub fn get_definition_indices(function: &IRFunction) -> Vec<u32> {
    let instructions = &function.instructions;
    let count = instructions
        .iter()
        .filter_map(|ins| {
            ins.get_result().or_else(|| {
                if let IRInstruction::Label(Some(phi), ..) = ins {
                    Some(phi.targets.iter().cloned().max().unwrap_or(0))
                } else {
                    None
                }
            })
        })
        .max()
        .unwrap_or(0)
        + 1;

    let mut result = vec![0; count as usize];

    for (i, instruction) in instructions.iter().enumerate() {
        match instruction {
            IRInstruction::Label(Some(phi), _) => {
                for &r in &phi.targets {
                    result[r as usize] = i as u32;
                }
            }
            _ => {
                if let Some(r) = instructions[i].get_result() {
                    result[r as usize] = i as u32;
                }
            }
        }
    }

    result
}
