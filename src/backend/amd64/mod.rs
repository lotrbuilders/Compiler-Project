//use std::io::Read;

use super::ir::*;
use super::Backend;

//#[allow(dead_code)]
/*pub struct BackendAMD64 {
    instructions: Vec<IRInstruction>,
    definition_index: Vec<u32>,
}*/

rburg::rburg_main! {
    BackendAMD64,
:       Ret(a %eax) "#\n"
%ireg:  Imm(#i) "mov {res},{i}\n" {1}
}

//Currently only caller safed registers
const reg_count: usize = 7;
const reg_class_eax: [bool; reg_count] = [true, false, false, false, false, false, false];
enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    R8 = 3,
    R9 = 4,
    R10 = 5,
    R11 = 6,
}

impl Register {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Rax => "eax",
            Self::Rcx => "ecx",
            Self::Rdx => "edx",
            Self::R8 => "r8d",
            Self::R9 => "r9d",
            Self::R10 => "r10d",
            Self::R11 => "r11d",
        }
    }
}

impl Backend for BackendAMD64 {
    fn backend_type(&self) -> &'static str {
        "burg"
    }

    fn generate(&mut self, function: &IRFunction) -> String {
        self.instructions = function.instructions.clone();
        self.definition_index = get_definition_indices(&function.instructions);
        self.instruction_states = vec![State::new(); self.instructions.len()];
        self.rules = vec![0xffff; self.instructions.len()];
        log::trace!(
            "State at construction:\n{}\n{:?}\n{:?}\n",
            self.to_string(),
            self.instructions,
            self.definition_index
        );
        for instruction in (0..function.instructions.len()).rev() {
            log::trace!("Labeling instruction tree starting at {}", instruction);
            self.label(instruction as u32);
        }

        log::info!("Labeled function {}:\n{}", function.name, self.to_string(),);

        for i in (0..self.instructions.len()).rev() {
            self.reduce_instruction(i as u32, stmt_NT);
        }

        log::info!("definitive rules:\n{:?}", self.rules);
        String::new()
    }
}

impl BackendAMD64 {
    fn get_rule(&self, index: u32, non_terminal: usize) -> u16 {
        let state = &self.instruction_states[index as usize];
        let rule = state.rule[non_terminal];
        if rule == 0xffff {
            log::warn!(
                "No valid rule for instruction {} with non_terminal {}",
                index,
                non_terminal
            );
        }
        rule
    }

    // Does not currrently support instructions with seperate levels of terminals, these need to be weeded out of the tree first
    // This could be done by only labelling nodes that we know to be terminals(registers that are used more then once and instructions that don't return values)
    // This also not supported in the labelizer due to the lack of a condition
    fn reduce_instruction(&mut self, instruction: u32, non_terminal: usize) -> () {
        if self.rules[instruction as usize] != 0xffff {
            return ();
        }

        let rule_number = self.get_rule(instruction, non_terminal);
        let child_non_terminals: Vec<usize> = self.get_child_non_terminals(rule_number);
        let kids: Vec<u32> = self.get_kids(instruction, rule_number);
        for i in 0..kids.len() {
            self.reduce_instruction(kids[i], child_non_terminals[i]);
        }
        self.rules[instruction as usize] = rule_number;
    }

    // Gives wether the current node is actually an instruction.
    // Currently everything should be an instruction
    fn is_instruction(&self, rule: u16) -> bool {
        let _ = rule;
        true
    }

    fn allocate_registers(&mut self) -> () {
        let length = self.definition_index.len();
        let mut last_use = vec![0u32; length];
        let mut first_use = vec![u32::MAX; length];
        for i in 0..self.instructions.len() {
            let rule = self.rules[i];
            if self.is_instruction(rule) {
                let (used_vreg, result_vreg): (Vec<u32>, Option<u32>) =
                    self.get_vregisters(i, rule);

                if let Some(vreg) = result_vreg {
                    first_use[vreg as usize] = i as u32;
                }
                for vreg in used_vreg {
                    last_use[vreg as usize] = i as u32;
                }
            }
        }
    }
}
