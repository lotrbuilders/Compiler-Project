use super::BackendAMD64;
use crate::backend::ir::*;

impl BackendAMD64 {
    crate::backend::rburg_template::default_utility!();
    pub fn scale(&self, index: u32) -> u16 {
        let ins: &IRInstruction = &self.instructions[index as usize];
        match ins {
            &IRInstruction::Imm(_, _, value) => match value {
                1 | 2 | 4 | 8 => 0,
                _ => 0xfff,
            },
            _ => {
                log::error!("scale called on unsupported instruction");
                0xfff
            }
        }
    }
}
