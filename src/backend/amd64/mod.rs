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
:       Ret i32(_a %eax)               "#\n"
:       Store(r %ireg, AddrL(#a))  "mov [ebp{a}],{r}\n"     {1}

%ireg:  Imm(#i)                    "mov {res}, {i}\n"       {1}
%ireg:  AddrL(#a)                  "lea {res},[ebp{a}]\n"   {1}

%ireg:  Load(AddrL(#a))             "mov {res},[ebp{a}]\n"  {1}

%ireg:  Add(a %ireg , b %ireg)    ?"add {res}, {b} ; {res} = {a} + {b}\n"   {1}
%ireg:  Add(a %ireg , Imm(#i))     ?"add {res}, {i} ; {res} = {a} + {i}\n"  {1}

%ireg:  Sub(a %ireg , b %ireg)    ?"sub {res}, {b} ; {res} = {a} - {b}\n"   {1}
%ireg:  Sub(Imm(#_i), b %ireg)    ?"neg {res} ; {res} = -{b}\n"             {self.range(self.get_left_index(index),0,0)+1}

%ireg:  Mul(a %ireg , b %ireg)    ?"imul {res}, {b} ; {res} = {a} * {b}\n"  {1}
%eax:   Div(a %eax  , b %ireg)    ?"cdq\n\tidiv {b} ; {res} = {a} / {b}\n"  {1}

%ireg:  Xor(a %ireg , b %ireg)    ?"xor {res}, {b} ; {res} = {a} ^ {b}\n"   {1}
%ireg:  Xor(a %ireg , Imm(#_i))   ?"not {res} ; {res} = ~{a}\n"             {self.range(self.get_right_index(index),-1,-1)+1}

%ireg:  Eq (a %ireg , b %ireg)     "cmp {a}, {b}\n\tsete {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n "  {1}
%ireg:  Eq (a %ireg , Imm(#i))     "test {a}, {a}\n\tsetz {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {i}\n " {self.range(self.get_right_index(index),0,0)+1}
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
        self.use_count = BackendAMD64::get_use_count(&self.instructions, &self.definition_index);
        self.instruction_states = vec![State::new(); self.instructions.len()];
        self.rules = vec![0xffff; self.instructions.len()];

        let (local_offsets, stack_size) = BackendAMD64::find_local_offsets(&function.variables);
        self.local_offsets = local_offsets;
        self.stack_size = stack_size;

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
    fn range(&self, index: u32, from: i128, to: i128) -> u16 {
        let ins: &IRInstruction = &self.instructions[index as usize];
        match ins {
            &IRInstruction::Imm(_, _, value) => {
                if value >= from && value <= to {
                    0
                } else {
                    0xfff
                }
            }
            _ => {
                log::error!("range called on unsupported instruction");
                0xfff
            }
        }
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
            log::trace!("{} already reduced correctly", instruction);
            return ();
        }

        let rule_number = self.get_rule(instruction, non_terminal);
        self.reduce_terminals(instruction, rule_number);
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
        match rule {
            0xffff => false,
            0xfffe => false,
            _ => true,
        }
    }

    // Should be automatically generated
    // Emits handwritten assembly if necessary, otherwise uses the automatic generated function
    fn emit_asm(&mut self) -> String {
        let mut result = self.emit_prologue();
        for instruction in 0..self.instructions.len() {
            for modification in &self.reg_relocations[instruction] {
                result.push_str(&self.emit_move(modification));
                use RegisterRelocation::*;
                match modification {
                    &Move(vreg, to) => self.vreg2reg[vreg as usize] = to,
                    TwoAddressMove(..) => continue,
                    _ => unimplemented!(),
                }
            }
            let handwritten = self.emit_asm2(instruction);
            if let Some(assembly) = handwritten {
                result.push_str(&assembly);
            } else {
                result.push_str(&self.gen_asm(instruction));
            }
        }
        result.push_str(&self.emit_epilogue());
        result
    }

    // Automatically generated
    // Checks if the instruction at index is the last instruction of the function for return optimization
    #[allow(dead_code)]
    fn is_last_instruction(&self, index: usize) -> bool {
        self.instructions.len() - 1 == index
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits handwritten assembly for instruction that are too complex to process normally
    fn emit_asm2(&self, index: usize) -> Option<String> {
        let instruction = &self.instructions[index];
        let _rule = self.rules[index];
        use IRInstruction::*;
        match instruction {
            Ret(_size, _vreg) => Some(if !self.is_last_instruction(index) {
                format!("\tjmp .end\n")
            } else {
                String::new()
            }),
            _ => None,
        }
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits the prologue for a function, such that it will be correct for the compiler
    fn emit_prologue(&self) -> String {
        let mut prologue = format!(
            "global {}\nsection .text\n{}:\n",
            self.function_name, self.function_name
        );
        if self.stack_size != 0 {
            prologue.push_str(&format!("\tpush rbp\n\tmov rbp,rsp\n"))
        }
        prologue
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits the epilogue for a function, such that it will be correct for the compiler
    fn emit_epilogue(&self) -> String {
        let mut epilogue = ".end:\n".to_string();
        if self.stack_size != 0 {
            epilogue.push_str(&format!("\tpop rbp\n"));
        }
        epilogue.push_str(&format!("\tret\n"));
        epilogue
    }

    fn emit_move(&self, modification: &RegisterRelocation) -> String {
        use RegisterRelocation::*;
        match modification {
            &TwoAddressMove(from, to) => format!("\tmov {},{}\n", to, from),
            &Move(vreg, to) => {
                let from = self.vreg2reg[vreg as usize];
                format!("\tmov {},{}\n", to, from)
            }
            _ => unimplemented!(),
        }
    }

    fn clobber(&self, index: usize) -> Vec<Register> {
        let instruction = &self.instructions[index];
        use IRInstruction::*;
        match instruction {
            Div(..) => vec![Register::Rdx],
            _ => Vec::new(),
        }
    }

    // Should depend on sizes and allignment as given by the backend
    // Is currently handwritten for x86-64
    fn find_local_offsets(variable_types: &Vec<IRSize>) -> (Vec<i32>, i32) {
        let mut offset = -4;
        let result = variable_types
            .iter()
            .map(|typ| match typ {
                IRSize::S32 | IRSize::I32 => {
                    offset += -4;
                    offset
                }
                IRSize::P => {
                    offset += -8;
                    offset
                }
            })
            .collect();
        (result, -offset + 4)
    }

    fn get_use_count(instructions: &Vec<IRInstruction>, definitions: &Vec<u32>) -> Vec<u32> {
        let mut use_count = vec![0u32; definitions.len()];
        for instruction in instructions {
            if let Some(left) = instruction.get_left() {
                use_count[left as usize] += 1;
            }
            if let Some(right) = instruction.get_right() {
                use_count[right as usize] += 1;
            }
        }
        use_count
    }
}
