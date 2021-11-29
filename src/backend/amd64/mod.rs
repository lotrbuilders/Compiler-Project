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
const REG_COUNT: usize = 6;
const REG_CLASS_EAX: [bool; REG_COUNT] = [true, false, false, false, false, false];
const REG_CLASS_IREG: [bool; REG_COUNT] = [true; REG_COUNT];
const REG_CLASS_EMPTY: [bool; REG_COUNT] = [false; REG_COUNT];
const REG_LOOKUP: [Register; REG_COUNT] = {
    use Register::*;
    [Rax, Rcx, Rdx, R8, R9, R10]
};
#[derive(Clone, Debug, Copy)]
enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    R8 = 3,
    R9 = 4,
    R10 = 5,
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
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum RegisterRelocation {
    Move(u32, Register), //from to
    Spill(Register),
    Reload(Register),
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
        log::info!("Starting register allocation");
        self.allocate_registers();

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
        let mut preferred_class: Vec<[bool; REG_COUNT]> = vec![REG_CLASS_IREG; length];
        for i in (0..self.instructions.len()).rev() {
            let rule = self.rules[i];
            if self.is_instruction(rule) {
                let (used_vreg, result_vreg) = self.get_vregisters(i as u32, rule);

                if let Some(vreg) = result_vreg {
                    first_use[vreg as usize] = i as u32;
                }
                for (vreg, class) in used_vreg {
                    last_use[vreg as usize] = i as u32;
                    if class != &REG_CLASS_IREG {
                        preferred_class[vreg as usize] = *class;
                    }
                }
            }
        }
        log::debug!("Initialization of vregisters:\n{:?}", first_use);
        log::debug!("First use of vregisters:\n{:?}", last_use);

        let mut reg_occupied_by: [Option<u32>; REG_COUNT] = [None; REG_COUNT];
        let mut vreg2reg: Vec<Option<Register>> = vec![None; length];
        let mut vreg2reg_original = vreg2reg.clone();

        for instruction in 0..self.instructions.len() {
            let rule = self.rules[instruction];
            if self.is_instruction(self.rules[instruction]) {
                // Implement clobber
                // Implement two address solving

                let (used_vreg, result_vreg) = self.get_vregisters(instruction as u32, rule);

                // perform register allocation if necessary
                let _result_reg = if let Some(vreg) = result_vreg {
                    let mut assigned_reg = None;
                    for i in 0..REG_COUNT {
                        let reg = REG_LOOKUP[i];
                        // Will also need to be dependent on the used register class eventually
                        // Skip unavailable registers
                        if !REG_CLASS_IREG[i] {
                            continue;
                        }
                        // Skip occupied registers
                        if reg_occupied_by[reg as usize] != None {
                            continue;
                        } else {
                            log::trace!("Using register {} for vreg {}", reg.to_string(), vreg);
                            reg_occupied_by[reg as usize] = Some(vreg);
                            vreg2reg[vreg as usize] = Some(reg);
                            vreg2reg_original[vreg as usize] = Some(reg);
                            assigned_reg = Some(reg);
                            break;
                        }
                    }
                    if let None = assigned_reg {
                        // Should do a relocation or spill
                        log::error!("No register available and no solution currently implemented")
                    }

                    assigned_reg
                } else {
                    None
                };

                // perform register relocation if necessary
                for (_vreg, _class) in used_vreg {}

                // If something has gone out of scope: remove it
                for i in 0..length {
                    if instruction == last_use[i] as usize {
                        let reg = vreg2reg[i].unwrap();
                        reg_occupied_by[reg as usize] = None;
                        vreg2reg[i] = None;
                    }
                }
            }
        }

        self.vreg2reg = vreg2reg_original.iter().map(|reg| reg.unwrap()).collect();
        self.reg_relocations = vec![Vec::new(); self.instructions.len()];

        log::debug!(
            "vreg2reg at start {:?}",
            self.vreg2reg
                .iter()
                .map(|reg| reg.to_string())
                .collect::<Vec<&str>>()
        );
    }
}
