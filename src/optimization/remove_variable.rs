use std::collections::HashSet;

use crate::ir::{IRFunction, IRInstruction};

impl IRFunction {
    pub fn remove_variables(&mut self) {
        let mut used_variables = HashSet::new();
        for instruction in &self.instructions {
            if let &IRInstruction::AddrL(_, _, variable) = instruction {
                used_variables.insert(variable as u32);
            }
        }
        self.variables
            .retain(|v| used_variables.contains(&v.number));
    }
}
