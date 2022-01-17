use smallvec::SmallVec;

use crate::backend::register_allocation::{RegisterBackend, RegisterClass, RegisterInterface};

pub struct InstructionInformation<R: RegisterInterface> {
    pub is_instruction: Vec<bool>,
    pub used: Vec<SmallVec<[(u32, RegisterClass<R>); 4]>>,
    pub result: Vec<Option<(u32, RegisterClass<R>)>>,
    pub clobber: Vec<Vec<R>>,
    pub clobber_after: Vec<Vec<R>>,
}

#[allow(dead_code)]
impl<R: RegisterInterface> InstructionInformation<R> {
    pub fn is_instruction(&self, i: u32) -> bool {
        self.is_instruction[i as usize]
    }
    pub fn used<'a>(&'a self, i: u32) -> &'a [(u32, RegisterClass<R>)] {
        &self.used[i as usize]
    }
    pub fn result<'a>(&'a self, i: u32) -> &'a Option<(u32, RegisterClass<R>)> {
        &self.result[i as usize]
    }
    pub fn clobber<'a>(&'a self, i: u32) -> &'a [R] {
        &self.clobber[i as usize]
    }
    pub fn clobber_after<'a>(&'a self, i: u32) -> &'a [R] {
        &self.clobber_after[i as usize]
    }

    pub fn gather<B: RegisterBackend<RegisterType = R>>(backend: &B) -> InstructionInformation<R> {
        let instructions = backend.get_instructions();
        let mut is_instruction = vec![false; instructions.len()];
        let mut used = vec![SmallVec::new(); instructions.len()];
        let mut result = vec![None; instructions.len()];
        let mut clobber = vec![Vec::new(); instructions.len()];
        let mut clobber_after = vec![Vec::new(); instructions.len()];

        for i in 0..instructions.len() {
            let rule = backend.get_rule(i);
            if backend.is_instruction(rule) {
                let index = i as u32;
                is_instruction[i] = true;
                let (u, r) = backend.get_vregisters(index, rule);
                used[i] = u;
                result[i] = r;
                clobber[i] = backend.get_clobbered(index);
                clobber_after[i] = backend.get_clobbered_after(index);
            }
        }
        InstructionInformation {
            is_instruction,
            used,
            result,
            clobber,
            clobber_after,
        }
    }
}
