use super::ir::*;
use super::Backend;
use std::collections::HashSet;

mod register_allocation;
mod registers;
use self::register_allocation::*;
use self::registers::*;

rburg::rburg_main! {
    BackendAMD64,
:       Ret pi64i32i16i8(_a %eax)    #"return\n"
:       Store i8(r %ireg, a %ireg)   "mov [{a:.64}],{r:.8}\n"
:       Store i8(r %ireg, a adr)     "mov [{a}],{r:.8}\n"
:       Store i16(r %ireg, a %ireg)  "mov [{a:.64}],{r:.16}\n"
:       Store i16(r %ireg, a adr)    "mov [{a}],{r:.16}\n"
:       Store i32(r %ireg, a %ireg)  "mov [{a:.64}],{r}\n"
:       Store i32(r %ireg, a adr)    "mov [{a}],{r}\n"
:       Store Pi64(r %ireg, a %ireg) "mov [{a:.64}],{r:.64}\n"
:       Store Pi64(r %ireg, a adr)   "mov [{a}],{r:.64}\n"
:       Store(Imm(#i),a %ireg)       "mov dword[{a:.64}],{i}\n"
:       Store(Imm(#i),a adr)         "mov dword[{a}],{i}\n"
:       Store Pi64(Imm(#i),a adr)    "mov qword[{a}],{i}\n"
:       Store Pi64(Imm(#i),a %ireg)  "mov qword[{a:.64}],{i}\n"
:       Label(#i)                    ".L{i}:\n"
%ireg:  Label(#i)                    ".L{i}:\n"
:       Jmp(#i)                      "jmp .L{i}\n"
:       Jcc(r %ireg,#l)              "test {r},{r}\n\tjnz .L{l}\n" {2}
:       Jnc(r %ireg,#l)              "test {r},{r}\n\tjz .L{l}\n"  {2}
:       Jcc pi64(r %ireg,#l)         "test {r:.64},{r:.64}\n\tjnz .L{l}\n" {2}
:       Jnc pi64(r %ireg,#l)         "test {r:.64},{r:.64}\n\tjz .L{l}\n"  {2}


scale:  Imm i32i64(#i)              "{i}" {self.scale(index)}
con:    Imm i32i64(#i)              "{i}"
adr:    AddrL(#a)                   "rbp+{a}"
adr:    AddrG(#a)                   "{a}"
adr:    Add p(a %ireg,  Mul s64(r %ireg,i scale))  "{a:.64}+{i}*{r:.64}"
adr:    Add p(a %ireg,  r %ireg)    "{a:.64}+{r:.64}"
mem:    Load(a adr)                 "[{a}]"
mem:    Load(r %ireg)               "[{r:.64}]"
acon:   i con                       "{i}"
acon:   a adr                       "{a}"
mcon:   i con                       "{i}"
mcon:   m mem                       "{m}"

mem64:  Load pi64(a adr)            "[{a}]"
mem64:  Load pi64(r %ireg)          "[{r:.64}]"
mcon64:  i con                      "{i}"
mcon64:  m mem64                    "{m}"

%ireg:  m mcon                      "mov {res}, {m}\n"          {1}
%ireg:  m mcon64                    "mov {res:.64}, {m}\n"      {1}
%ireg:  a adr                       "lea {res:.64}, [{a}]\n"    {1}

%ireg:  Imm pi64(#i)                "mov {res:.64}, {i}\n"

%ireg: Load i8(a adr)               "mov {res:.8}, [{a}]\n"
%ireg: Load i8(a %ireg)             "mov {res:.8}, [{a:.64}]\n"
%ireg: Load i16(a adr)              "mov {res:.16}, [{a}]\n"
%ireg: Load i16(a %ireg)            "mov {res:.16}, [{a:.64}]\n"

%ireg:  Cvs s32(Load s8(a adr))     "movsx {res:.32}, byte [{a}]\n"
%ireg:  Cvs s32(Load s8(r %ireg))   "movsx {res:.32}, byte [{r:.64}]\n"
%ireg:  Cvs s32(Load s16(a adr))    "movsx {res:.32}, word [{a}]\n"
%ireg:  Cvs s32(Load s16(r %ireg))  "movsx {res:.32}, word [{r:.64}]\n"
%ireg:  Cvs s64(Load s8(a adr))     "movsx {res:.64}, byte [{a}]\n"
%ireg:  Cvs s64(Load s8(r %ireg))   "movsx {res:.64}, byte [{r:.64}]\n"
%ireg:  Cvs s64(Load s16(a adr))     "movsx {res:.64}, word [{a}]\n"
%ireg:  Cvs s64(Load s16(r %ireg))   "movsx {res:.64}, word [{r:.64}]\n"
%ireg:  Cvs s64(Load s32(a adr))     "movsx {res:.64}, dword [{a}]\n"
%ireg:  Cvs s64(Load s32(r %ireg))   "movsx {res:.64}, dword [{r:.64}]\n"

%ireg:  Add(a %ireg , b %ireg)      ?"add {res}, {b} ; {res} = {a} + {b}\n"   {1}
%ireg:  Add pi64(a %ireg , b %ireg) ?"add {res:.64}, {b:.64} ; {res:.64} = {a:.64} + {b:.64}\n"   {1}

%ireg:  Sub(a %ireg , b %ireg)          ?"sub {res}, {b} ; {res} = {a} - {b}\n"         {1}
%ireg:  Sub pi64(a %ireg , b %ireg)     ?"sub {res:.64}, {b:.64} ; {res} = {a} - {b}\n" {1}
%ireg:  Sub(Imm(#_i), b %ireg)          ?"neg {res} ; {res} = -{b}\n"   {self.range(self.get_left_index(index),0,0)+1}
%ireg:  Sub s64(Imm s64(#_i), b %ireg)  ?"neg {res} ; {res} = -{b}\n"   {self.range(self.get_left_index(index),0,0)+1}

%ireg:  Mul s32(a %ireg , b %ireg)  ?"imul {res}, {b} ; {res} = {a} * {b}\n"  {1}
%eax:   Div s32(a %eax  , b %ireg)  ?"cdq\n\tidiv {b} ; {res} = {a} / {b}\n"  {1}
%ireg:  Mul s64(a %ireg , b %ireg)  ?"imul {res:.64}, {b:.64} ; {res:.64} = {a:.64} * {b:.64}\n"  {1}
%eax:   Div s64(a %eax  , b %ireg)  ?"cdq\n\tidiv {b:.64}     ; {res:.64} = {a:.64} / {b:.64}\n"  {1}

%ireg:  And(a %ireg , b %ireg)      ?"and {res}, {b} ; {res} = {a} & {b}\n"   {1}
%ireg:  Or(a %ireg , b %ireg)       ?"or  {res}, {b} ; {res} = {a} | {b}\n"   {1}
%ireg:  Xor(a %ireg , b %ireg)      ?"xor {res}, {b} ; {res} = {a} ^ {b}\n"   {1}
%ireg:  Xor(a %ireg , Imm(#_i))     ?"not {res} ; {res} = ~{a}\n"             {self.range(self.get_right_index(index),-1,-1)+1}

%ireg:  And i64(a %ireg , b %ireg)  ?"and {res:.64}, {b:.64} ; {res:.64} = {a:.64} & {b:.64}\n"   {1}
%ireg:  Or  i64(a %ireg , b %ireg)  ?"or  {res:.64}, {b:.64} ; {res:.64} = {a:.64} | {b:.64}\n"   {1}
%ireg:  Xor i64(a %ireg , b %ireg)  ?"xor {res:.64}, {b:.64} ; {res:.64} = {a:.64} ^ {b:.64}\n"   {1}
%ireg:  Xor i64(a %ireg , Imm(#_i)) ?"not {res:.64} ; {res:.64} = ~{a:.64}\n"   {self.range(self.get_right_index(index),-1,-1)+1}

%ireg:  Eq i32(a %ireg , b %ireg)      "cmp {a}, {b}\n\tsete {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"    {3}
%ireg:  Ne i32(a %ireg , b %ireg)      "cmp {a}, {b}\n\tsetne {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"   {3}
%ireg:  Eq i32(a %ireg , Imm(#i))      "test {a}, {a}\n\tsetz {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {i}\n"   {self.range(self.get_right_index(index),0,0)+2}
%ireg:  Ne i32(a %ireg , Imm(#i))      "test {a}, {a}\n\tsetnz {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {i}\n"  {self.range(self.get_right_index(index),0,0)+2}

%ireg:  Lt s32 (a %ireg , b %ireg)  "cmp {a}, {b}\n\tsetl {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Le s32 (a %ireg , b %ireg)  "cmp {a}, {b}\n\tsetle {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}
%ireg:  Gt s32 (a %ireg , b %ireg)  "cmp {a}, {b}\n\tsetg {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Ge s32 (a %ireg , b %ireg)  "cmp {a}, {b}\n\tsetge {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}

%ireg:  Eq pi64 (a %ireg , b %ireg)    "cmp {a:.64}, {b:.64}\n\tsete {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"    {3}
%ireg:  Ne pi64 (a %ireg , b %ireg)    "cmp {a:.64}, {b:.64}\n\tsetne {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"   {3}
%ireg:  Eq pi64 (a %ireg , Imm p (#i)) "test {a:.64}, {a:.64}\n\tsetz {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {i}\n"   {self.range(self.get_right_index(index),0,0)+2}
%ireg:  Ne pi64 (a %ireg , Imm p (#i)) "test {a:.64}, {a:.64}\n\tsetnz {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {i}\n"  {self.range(self.get_right_index(index),0,0)+2}

%ireg:  Lt p (a %ireg , b %ireg)    "cmp {a:.64}, {a:.64}\n\tsetb {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Le p (a %ireg , b %ireg)    "cmp {a:.64}, {a:.64}\n\tsetbe {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}
%ireg:  Gt p (a %ireg , b %ireg)    "cmp {a:.64}, {a:.64}\n\tseta {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Ge p (a %ireg , b %ireg)    "cmp {a:.64}, {a:.64}\n\tsetae {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}

%ireg:  Lt s64 (a %ireg , b %ireg)  "cmp {a:.64}, {b:.64}\n\tsetl  {res:.8}\n\tmovsx {res:.64},{res:.8}; {res:.64} = {a:.64} == {b:.64}\n"      {3}
%ireg:  Le s64 (a %ireg , b %ireg)  "cmp {a:.64}, {b:.64}\n\tsetle {res:.8}\n\tmovsx {res:.64},{res:.8}; {res:.64} = {a:.64} == {b:.64}\n"      {3}
%ireg:  Gt s64 (a %ireg , b %ireg)  "cmp {a:.64}, {b:.64}\n\tsetg  {res:.8}\n\tmovsx {res:.64},{res:.8}; {res:.64} = {a:.64} == {b:.64}\n"      {3}
%ireg:  Ge s64 (a %ireg , b %ireg)  "cmp {a:.64}, {b:.64}\n\tsetge {res:.8}\n\tmovsx {res:.64},{res:.8}; {res:.64} = {a:.64} == {b:.64}\n"      {3}

%ireg:  Cvp (_r %ireg)              #"#extend/truncuate" {2}
%ireg:  Cvs s64s32s16s8(_r %ireg)   #"#extend/truncuate" {2}

:       Arg pi32i64(r %ireg)        #"push {r:.64}\n" {1}
%eax:   Call pi64i32i16i8(#name)    #"#call {name}\n" {20}
}

impl Backend for BackendAMD64 {
    fn backend_type(&self) -> &'static str {
        "burg"
    }

    // Generates assembly for a single function
    // Modifies the storage for the backend to allow for this
    fn generate(&mut self, function: &IRFunction, function_names: &HashSet<String>) -> String {
        self.function_name = function.name.clone();
        self.instructions = function.instructions.clone();
        self.definition_index = get_definition_indices(&function);
        self.use_count = BackendAMD64::get_use_count(&self.instructions, &self.definition_index);
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

    fn generate_global_prologue(&mut self) -> String {
        format!("default rel\nsection .text\n")
        //String::new()
    }

    fn argument_evaluation_direction_registers(&self) -> super::Direction {
        super::Direction::Left2Right
    }

    fn argument_evaluation_direction_stack(&self) -> super::Direction {
        super::Direction::Right2Left
    }

    fn get_arguments_in_registers(&self, sizes: &Vec<IRSize>) -> Vec<bool> {
        let mut result = Vec::with_capacity(sizes.len());
        let mut ireg = 0;
        for _size in sizes {
            result.push(ireg < 6);
            ireg += 1;
        }
        result
    }

    fn get_size(&self, typ: &crate::parser::TypeNode) -> IRSize {
        use crate::parser::TypeNode::*;
        match typ {
            Char => IRSize::S8,
            Short => IRSize::S16,
            Int => IRSize::S32,
            Long => IRSize::S64,
            Pointer => IRSize::P,
            _ => unreachable!(),
        }
    }

    fn sizeof_pointer(&self) -> u32 {
        return 8;
    }

    fn typeof_size_t(&self) -> crate::parser::TypeNode {
        return crate::parser::TypeNode::Long;
    }
}

impl BackendAMD64 {
    // Automatically generated
    // Checks if the instruction at index is the last instruction of the function for return optimization
    #[allow(dead_code)]
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
    // Automatically generated
    // Checks if the instruction at index is the last instruction of the function for return optimization
    #[allow(dead_code)]
    fn is_last_instruction(&self, index: usize) -> bool {
        self.instructions.len() - 1 == index
    }

    fn scale(&self, index: u32) -> u16 {
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

impl BackendAMD64 {
    // Should be automatically generated
    // Gets the rule that is assocatiated with the specific non_terminal used
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

        let child_non_terminals: Vec<usize> =
            self.get_child_non_terminals(instruction, rule_number);
        let kids: Vec<u32> = self.get_kids(instruction, rule_number);
        for i in 0..kids.len() {
            self.reduce_instruction(kids[i], child_non_terminals[i]);
        }
        self.rules[instruction as usize] = rule_number;
    }

    // Should be automatically generated
    // Emits handwritten assembly if necessary, otherwise uses the automatic generated function
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

    fn get_stack_alignment(&self, arguments: &IRArguments) -> i32 {
        let length = arguments.count as i32;
        let extra_stack_size = (std::cmp::max(length, 6) - 6) * 8;
        let next_alignment = self.stack_size + extra_stack_size as i32 + 8;
        match next_alignment % 16 {
            0 => 0,
            i => 16 - i,
        }
    }

    fn stack_alignment_instruction(&self, alignment: i32) -> String {
        match alignment {
            0 => String::new(),
            i => format!("\tsub rsp,{}\n", i),
        }
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits handwritten assembly for instruction that are too complex to process normally
    // The boolean specifies wether the normal assembly should also be generated
    fn emit_asm2(&self, index: usize) -> (String, bool) {
        let instruction = &self.instructions[index];
        let _rule = self.rules[index];
        use IRInstruction::*;
        match instruction {
            Ret(_size, _vreg) => (
                if !self.is_last_instruction(index) {
                    format!("\tjmp .end\n")
                } else {
                    String::new()
                },
                false,
            ),
            Call(_size, _vreg, name, arguments) => (
                {
                    let length = arguments.count;
                    let alignment = self.get_stack_alignment(arguments);
                    let alignment_instruction = if length <= 6 {
                        self.stack_alignment_instruction(alignment)
                    } else {
                        String::new()
                    };
                    let outside_file = if !self.function_names.contains(name) {
                        "wrt ..plt"
                    } else {
                        ""
                    };

                    format!(
                        "{}{}",
                        alignment_instruction,
                        if length > 6 || alignment != 0 {
                            // Only hold for integer and pointer arguments
                            format!(
                                "\tcall {} {}\n\tadd rsp,{}\n",
                                name,
                                outside_file,
                                8 * (std::cmp::max(6, length) - 6) + alignment as usize
                            )
                        } else {
                            format!("\tcall {} {}\n", name, outside_file,)
                        }
                    )
                },
                false,
            ),
            Arg(_size, _vreg, Some(index)) => (
                {
                    if let IRInstruction::Call(_size, _result, _name, arguments) =
                        &self.instructions[*index]
                    {
                        let alignment = self.get_stack_alignment(arguments);
                        self.stack_alignment_instruction(alignment)
                    } else {
                        String::new()
                    }
                },
                true,
            ),
            Cvs(
                to_s @ (IRSize::S64 | IRSize::S32 | IRSize::S16 | IRSize::S8),
                to_r,
                from_s @ (IRSize::S64 | IRSize::S32 | IRSize::S16 | IRSize::S8),
                from_r,
            ) if to_s > from_s => (
                format!(
                    "\tmovsx {:.to_w$},{:.from_w$}\n",
                    self.allocation[*to_r as usize][index].unwrap(),
                    self.allocation[*from_r as usize][index].unwrap(),
                    from_w = from_s.to_bit_width(),
                    to_w = to_s.to_bit_width()
                ),
                false,
            ),
            Cvs(
                to_s @ (IRSize::S64 | IRSize::S32 | IRSize::S16 | IRSize::S8),
                to_r,
                from_s @ (IRSize::S64 | IRSize::S32 | IRSize::S16 | IRSize::S8),
                from_r,
            ) => {
                let _ = (to_s, to_r, from_s, from_r);
                (String::new(), false)
            }

            _ => (String::new(), true),
        }
    }

    fn emit_function_declaration(&self, name: &String) -> String {
        format!("section .text\nextern {}\n", name)
    }

    fn emit_global_definition(&self, name: &String, value: i128, size: &IRSize) -> String {
        let _ = size;
        format!("section .data\n{}:\n\tdq {}\n", name, value)
    }

    fn emit_common(&self, name: &String, size: &IRSize) -> String {
        let _ = size;
        format!("section .bss\n{}:\n\tresb 4\n", name)
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
            prologue.push_str(&format!("\tpush rbp\n\tmov rbp,rsp\n"));
            prologue.push_str(&format!("\tsub rsp, {}\n", self.stack_size));
        }
        prologue
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits the epilogue for a function, such that it will be correct for the compiler
    fn emit_epilogue(&self) -> String {
        let mut epilogue = ".end:\n".to_string();
        if self.stack_size != 0 {
            epilogue.push_str(&format!("\tadd rsp, {}\n", self.stack_size));
            epilogue.push_str(&format!("\tpop rbp\n"));
        }
        epilogue.push_str(&format!("\tret\n"));
        epilogue
    }

    fn emit_strings(&self, strings: &Vec<String>) -> String {
        let mut result = String::from("section .data\n");
        for (string, i) in strings.iter().zip(0..) {
            result.push_str(&format!(".__string{}:\n\tdb {:?},0\n", i, string));
        }
        result
    }

    fn emit_move(&self, modification: &RegisterRelocation) -> String {
        use RegisterRelocation::*;
        match modification {
            &TwoAddressMove(from, to) => format!("\tmov {:.64},{:.64}\n", to, from),
            &Move(from, to) => {
                format!("\tmov {:.64},{:.64}\n", to, from)
            }
            &Reload(reg, mem) => format!("\tmov {:.64}, [rbp-{}]\n", reg, mem),
            &Spill(reg, mem) => format!("\tmov [rbp-{}],{:.64} \n", mem, reg),
            &MemMove(from, to, reg) => {
                format!(
                    "\tmov {:.64}, [rbp-{}]\n\tmov [rbp-{}], {:.64}\n",
                    reg, from, to, reg
                )
            }
            _ => unimplemented!(),
        }
    }

    fn clobber(&self, index: usize) -> Vec<Register> {
        let instruction = &self.instructions[index];
        use IRInstruction::*;
        match instruction {
            Div(..) => vec![Register::Rdx],
            Call(..) => vec![],
            _ => Vec::new(),
        }
    }

    fn get_call_regs(&self, sizes: &Vec<IRSize>) -> Vec<&'static RegisterClass> {
        let mut result = Vec::with_capacity(sizes.len());
        let mut ireg_index = 0usize;
        for _size in sizes {
            if ireg_index < 6 {
                result.push(CALL_REGS[ireg_index]);
                ireg_index += 1;
            }
        }
        result
    }

    // Should depend on sizes and allignment as given by the backend
    // Is currently handwritten for x86-64
    fn find_local_offsets(
        variable_types: &Vec<IRVariable>,
        arguments: &IRArguments,
    ) -> (Vec<i32>, i32) {
        let mut arg_offset = 8;
        let mut offset = 0;
        let mut result = Vec::new();

        for i in 0..variable_types.len() {
            let count = variable_types[i].count as i32;
            result.push(match arguments.arguments.get(i) {
                // Either a normal variable or an argument passed via register
                None | Some(Some(..)) => match variable_types[i].size {
                    IRSize::S32 | IRSize::S8 | IRSize::S16 => {
                        offset += -4 * count; // Wrong for types smaller then S32 at the moment
                        offset //+ 4 * (count - 1)
                    }
                    IRSize::P | IRSize::S64 => {
                        offset += -8 * count;
                        offset //+ 4 * (count - 1)
                    }
                },
                // Stack argument
                Some(None) => {
                    arg_offset += 8;
                    arg_offset
                }
            });
        }

        (result, -offset + 8)
    }

    fn get_use_count(instructions: &Vec<IRInstruction>, definitions: &Vec<u32>) -> Vec<u32> {
        log::debug!(
            "Started get use count with {} definitions",
            definitions.len()
        );
        let mut use_count = vec![0u32; definitions.len()];
        for instruction in instructions {
            if let Some(left) = instruction.get_left() {
                use_count[left as usize] += 1;
            }
            if let Some(right) = instruction.get_right() {
                use_count[right as usize] += 1;
            }
        }
        log::debug!("Use count: {:?}", use_count);
        use_count
    }
}
