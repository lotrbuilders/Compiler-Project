macro_rules! get_rule {
    {} => {
        fn get_rule(&self, index: u32, non_terminal: usize) -> u16 {
            let state = &self.instruction_states[index as usize];
            let rule = state.rule[non_terminal];
            if rule == 0xffff {
                log::warn!(
                    "No valid rule for instruction{} with non_terminal {}:",
                    index,
                    non_terminal
                );
                log::warn!("{}", self.instructions[index as usize],);
            }
            rule
        }
    };
}

macro_rules! reduce_instruction  {
    {} => {
        fn reduce_instruction(&mut self, instruction: u32, non_terminal: usize) -> () {
            if self.rules[instruction as usize] != 0xffff {
                log::trace!("{} already reduced correctly", instruction);
                return ();
            }

            let rule_number = self.get_rule(instruction, non_terminal);
            self.reduce_terminals(instruction, rule_number);

            let child_non_terminals: Vec<usize> =
                self.get_child_non_terminals(instruction, rule_number);
            let kids: Vec<u32> = self.get_kids(instruction, rule_number);
            for i in 0..kids.len() {
                self.reduce_instruction(kids[i], child_non_terminals[i]);
            }
            self.rules[instruction as usize] = rule_number;
        }
    };
}

macro_rules! emit_asm {
    {} => {
        fn emit_asm(&mut self, strings: &Vec<String>) -> String {
            let mut result = self.emit_prologue();
            for instruction in 0..self.instructions.len() {
                for modification in &self.reg_relocations[instruction] {
                    result.push_str(&self.emit_move(modification));
                    //use RegisterLocation::*;
                    use RegisterRelocation::*;
                    match modification {
                        Move(..) => continue, //  self.vreg2reg[vreg as usize] = Reg(to),
                        TwoAddressMove(..) => continue,
                        Spill(..) => continue,
                        Reload(..) => continue,
                        MemMove(..) => continue,
                        _ => unimplemented!(),
                    }
                }
                let rule = self.rules[instruction];
                if self.is_instruction(rule) {
                    let procede = if self.custom_print[rule as usize] {
                        let (handwritten, procede) = self.emit_asm2(instruction);
                        result.push_str(&handwritten);
                        procede
                    } else {
                        true
                    };
                    if procede {
                        result.push_str(&self.gen_asm(instruction));
                    }
                }
            }
            result.push_str(&self.emit_epilogue());
            result.push_str(&self.emit_strings(strings));
            result
        }
    };
}

macro_rules! default_utility {
    {} => {
        // Automatically generated
        // Checks if the instruction at index is the last instruction of the function for return optimization
        #[allow(dead_code)]
        pub fn range(&self, index: u32, from: i128, to: i128) -> u16 {
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
        // Automatically generated
        // Checks if the instruction at index is the last instruction of the function for return optimization
        #[allow(dead_code)]
        pub fn is_last_instruction(&self, index: usize) -> bool {
            self.instructions.len() - 1 == index
        }
    };
}

macro_rules! generate {
    {} => {
        // Generates assembly for a single function
        // Should be generated automatically
        // Modifies the storage for the backend to allow for this
        fn generate(&mut self, function: &IRFunction, function_names: &HashSet<String>) -> String {
            self.function_name = function.name.clone();
            self.instructions = function.instructions.clone();
            self.definition_index = get_definition_indices(&function);
            self.use_count = backend::get_use_count(&self.instructions, &self.definition_index);
            self.instruction_states = vec![State::new(); self.instructions.len()];
            self.rules = vec![0xffff; self.instructions.len()];
            self.arguments = function.arguments.clone();
            self.function_names = function_names.clone();

            let (local_offsets, stack_size) =
                BackendAMD64::find_local_offsets(&function.variables, &function.arguments);

            log::info!("Local offsets: {:?}", local_offsets);
            log::info!("Stack size: {}", stack_size);
            self.local_offsets = local_offsets;
            self.stack_size = stack_size;
            for instruction in (0..function.instructions.len()).rev() {
                log::trace!("Labeling instruction tree starting at {}", instruction);
                self.label(instruction as u32);
            }
            log::info!("Labeled function {}:\n{}", function.name, self.to_string(),);

            for i in (0..self.instructions.len()).rev() {
                self.reduce_instruction(i as u32, stmt_NT);
            }

            if cfg!(debug_assertions)
            {
                let bad_instruction=self.rules.iter().any(|&rule| rule==0xffff);
                if bad_instruction
                {
                    std::process::exit(2);
                }
            }

            log::info!("definitive rules:\n{:?}", self.rules);
            log::info!("Starting register allocation");
            RegisterAllocatorSimple::allocate_registers(self);
            log::debug!(
                "vreg2reg at start \n[\n{}]",
                self.allocation
                    .iter()
                    .map(|reg| format!("\t{}\n", reg))
                    .flat_map(|s| s.chars().collect::<Vec<char>>())
                    .collect::<String>()
            );

            log::info!("Starting assembly generation");
            let assembly = self.emit_asm(&function.strings);
            log::info!("Assembly:\n{}", assembly);
            assembly
        }

        // Generates assembly for globals
        // Should be generated automatically
        fn generate_globals(&mut self, globals: &Vec<IRGlobal>) -> String {
            let mut result = String::new();
            for global in globals {
                if global.function {
                    result.push_str(&self.emit_function_declaration(&global.name));
                } else if let Some(value) = global.value {
                    result.push_str(&self.emit_global_definition(&global.name, value, &global.size));
                } else {
                    result.push_str(&self.emit_common(&global.name, &global.size));
                }
            }
            result
        }
    };
}

pub(super) use default_utility;
pub(super) use emit_asm;
pub(super) use generate;
pub(super) use get_rule; // <-- the trick
pub(super) use reduce_instruction; // <-- the trick // <-- the trick
