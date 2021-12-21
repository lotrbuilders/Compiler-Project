pub mod control_flow_graph;
pub mod ir;
pub mod ir_phi;
pub mod print_ir;

pub use self::ir::*;
pub use ir_phi::*;

// Get the indices at which virtual registers are defined
pub fn get_definition_indices(function: &IRFunction) -> Vec<u32> {
    let instructions = &function.instructions;
    let arguments = &function.arguments.arguments;
    let mut result = arguments
        .iter()
        .filter_map(|arg| *arg)
        .map(|_| 0u32)
        .collect::<Vec<u32>>();

    for i in 0..instructions.len() {
        match &instructions[i] {
            IRInstruction::Label(Some(phi), _) => {
                for _ in &phi.targets {
                    result.push(i as u32);
                }
            }
            _ => {
                if let Some(_) = instructions[i].get_result() {
                    result.push(i as u32);
                }
            }
        }
    }

    result
}
