use std::vec;

use super::ir::*;
use super::Backend;
mod ralloc;
mod registers;
use self::ralloc::*;
use self::registers::*;

//#[allow(dead_code)]
/*pub struct BackendAMD64 {
    instructions: Vec<IRInstruction>,
    definition_index: Vec<u32>,
}*/

rburg::rburg_main! {
    BackendAMD64,
:       Ret(_a %eax)               "#\n"
%ireg:  Imm(#i)                    "mov  {res}, {i}\n"   {1}
%ireg:  Add(a %ireg , b %ireg)    ?"add  {res}, {b} ; {res} = {a} + {b}\n"   {1}
%ireg:  Sub(a %ireg , b %ireg)    ?"sub  {res}, {b} ; {res} = {a} - {b}\n"   {1}
%ireg:  Mul(a %ireg , b %ireg)    ?"imul {res}, {b} ; {res} = {a} * {b}\n"   {1}
%ireg:  Div(a %ireg , b %ireg)    ?"sub edx,edx\n\tidiv {b}        ; {res} = {a} / {b}\n"   {1}
}

impl Backend for BackendAMD64 {
    fn backend_type(&self) -> &'static str {
        "burg"
    }

    // Generates assembly for a single function
    // Modifies the storage for the backend to allow for this
    fn generate(&mut self, function: &IRFunction) -> String {
        self.function_name = function.name.clone();
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
        log::info!("Starting register allocation");
        self.allocate_registers();

        log::info!("Starting assembly generation");
        let assembly = self.emit_asm();
        log::info!("Assembly:\n{}", assembly);
        assembly
    }
}

impl BackendAMD64 {
    // Should be automatically generated
    // Gets the rule that is assocatiated with the specific non_terminal used
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

    fn clobber(&self, index: usize) -> Vec<Register> {
        let instruction = &self.instructions[index];
        use IRInstruction::*;
        match instruction {
            Div(..) => vec![Register::Rdx],
            _ => Vec::new(),
        }
    }

    // Should be automatically generated
    // Emits handwritten assembly if necessary, otherwise uses the automatic generated function
    fn emit_asm(&mut self) -> String {
        let mut result = self.emit_prologue();
        for instruction in 0..self.instructions.len() {
            for modification in &self.reg_relocations[instruction] {
                self.emit_move(modification);
            }
            let handwritten = self.emit_asm2(instruction);
            if let Some(assembly) = handwritten {
                result.push_str(&assembly);
            } else {
                result.push_str(&self.gen_asm(instruction));
            }
        }
        result
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits handwritten assembly for instruction that are too complex to process normally
    fn emit_asm2(&self, index: usize) -> Option<String> {
        let instruction = &self.instructions[index];
        let _rule = self.rules[index];
        use IRInstruction::*;
        match instruction {
            Ret(_size, _vreg) => Some(format!("\tret\n")),
            _ => None,
        }
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits the prologue for a function, such that it will be correct for the compiler
    fn emit_prologue(&self) -> String {
        format!(
            "global {}\nsection .text\n{}:\n",
            self.function_name, self.function_name
        )
    }

    fn emit_move(&self, modification: &RegisterRelocation) -> String {
        use RegisterRelocation::*;
        match modification {
            &TwoAddressMove(from, to) => format!("\tmov {},{}\n", to, from),
            _ => unimplemented!(),
        }
    }
}
