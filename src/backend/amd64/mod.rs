use crate::utility::padding;

use super::ir::*;
use super::register_allocation::RegisterInterface;
use smallvec::SmallVec;
use std::collections::HashSet;

mod backend;
mod emit;
mod registers;
mod utility;
use self::registers::*;
use super::register_allocation::*;

pub use self::emit::*;

rburg::rburg_main! {
    BackendAMD64,
    int_size: 32
    default_register_sizes: {
        pi64 => 64,
        i32i16i8 => 32,
    }
    instructions:
:       Nop(#_l)                        ""
:       Ret pi64i32i16i8v(_a %eax)      #"return"
:       Store i8(r %ireg, a %ireg)      "\tmov [{a:.64}],{r:.8}\n"
:       Store i8(r %ireg, a adr)        "\tmov [{a}],{r:.8}\n"
:       Store i16(r %ireg, a %ireg)     "\tmov [{a:.64}],{r:.16}\n"
:       Store i16(r %ireg, a adr)       "\tmov [{a}],{r:.16}\n"
:       Store pi64i32(r %ireg, a %ireg) "\tmov [{a:.64}],{r}\n"
:       Store pi64i32(r %ireg, a adr)   "\tmov [{a}],{r}\n"
:       Store i32(Imm(#i),a %ireg)      "\tmov dword[{a:.64}],{i}\n"
:       Store i32(Imm(#i),a adr)        "\tmov dword[{a}],{i}\n"
:       Store Pi64(Imm(#i),a adr)       "\tmov qword[{a}],{i}\n"
:       Store Pi64(Imm(#i),a %ireg)     "\tmov qword[{a:.64}],{i}\n"
:       Label(#i)                    ".L{i}:\n"
%ireg:  Label(#i)                    ".L{i}:\n"
:       Jmp(#i)                      "\tjmp .L{i}\n" {2}
:       Jmp(#i)                      "\t;jmp .L{}\n" {self.empty_jump_target(index)}
:       Jcc pi64i32(r %ireg,#l)      "\ttest {r},{r}\n\tjnz .L{l}\n" {2}
:       Jnc pi64i32(r %ireg,#l)      "\ttest {r},{r}\n\tjz .L{l}\n"  {2}

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

%ireg:  m mcon                      "\tmov {res}, {m}\n"          {1}
%ireg:  m mcon64                    "\tmov {res:.64}, {m}\n"      {1}
%ireg:  a adr                       "\tlea {res:.64}, [{a}]\n"    {1}

%ireg:  Imm pi64(#i)                "\tmov {res:.64}, {i}\n"

%ireg: Load i8(a adr)               "\tmov {res:.8}, [{a}]\n"
%ireg: Load i8(a %ireg)             "\tmov {res:.8}, [{a:.64}]\n"
%ireg: Load i16(a adr)              "\tmov {res:.16}, [{a}]\n"
%ireg: Load i16(a %ireg)            "\tmov {res:.16}, [{a:.64}]\n"

%ireg:  Cvs s64s32(Load s8(a adr))     "\tmovsx {res}, byte [{a}]\n"
%ireg:  Cvs s64s32(Load s8(r %ireg))   "\tmovsx {res}, byte [{r:.64}]\n"
%ireg:  Cvs s64s32(Load s16(a adr))    "\tmovsx {res}, word [{a}]\n"
%ireg:  Cvs s64s32(Load s16(r %ireg))  "\tmovsx {res}, word [{r:.64}]\n"
%ireg:  Cvs s64(Load s32(a adr))       "\tmovsx {res:.64}, dword [{a}]\n"
%ireg:  Cvs s64(Load s32(r %ireg))     "\tmovsx {res:.64}, dword [{r:.64}]\n"

%ireg:  Add pi64i32(a %ireg , b %ireg)      ?"\tadd {res}, {b} ; {res} = {a} + {b}\n"   {1}

%ireg:  Sub pi64i32(a %ireg , b %ireg)      ?"\tsub {res}, {b} ; {res} = {a} - {b}\n"         {1}
%ireg:  Sub pi64i32(Imm(#_i), b %ireg)      ?"\tneg {res} ; {res} = -{b}\n"   {self.range(self.get_left_index(index),0,0)+1}

%ireg:  Mul s32s64(a %ireg , b %ireg)       ?"\timul {res}, {b} ; {res} = {a} * {b}\n"  {1}
%eax:   Div s32(a %eax  , b %ireg)          ?"\tcdq\n\tidiv {b} ; {res} = {a} / {b}\n"  {1}
%eax:   Div s64(a %eax  , b %ireg)          ?"\tcqo\n\tidiv {b:.64}     ; {res:.64} = {a:.64} / {b:.64}\n"  {1}

%ireg:  And i64i32 (a %ireg , b %ireg)      ?"\tand {res}, {b} ; {res} = {a} & {b}\n"   {1}
%ireg:  Or  i64i32 (a %ireg , b %ireg)      ?"\tor  {res}, {b} ; {res} = {a} | {b}\n"   {1}
%ireg:  Xor i64i32 (a %ireg , b %ireg)      ?"\txor {res}, {b} ; {res} = {a} ^ {b}\n"   {1}
%ireg:  Xor i64i32 (a %ireg , Imm(#_i))     ?"\tnot {res} ; {res} = ~{a}\n"             {self.range(self.get_right_index(index),-1,-1)+1}

%ireg:  Eq pi64i32(a %ireg , b %ireg)      "\tcmp {a}, {b}\n\tsete {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"    {3}
%ireg:  Ne pi64i32(a %ireg , b %ireg)      "\tcmp {a}, {b}\n\tsetne {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"   {3}
%ireg:  Eq pi64i32(a %ireg , Imm(#i))      "\ttest {a}, {a}\n\tsetz {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {i}\n"   {self.range(self.get_right_index(index),0,0)+2}
%ireg:  Ne pi64i32(a %ireg , Imm(#i))      "\ttest {a}, {a}\n\tsetnz {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {i}\n"  {self.range(self.get_right_index(index),0,0)+2}

%ireg:  Lt s32s64 (a %ireg , b %ireg)  "\tcmp {a}, {b}\n\tsetl {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Le s32s64 (a %ireg , b %ireg)  "\tcmp {a}, {b}\n\tsetle {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}
%ireg:  Gt s32s64 (a %ireg , b %ireg)  "\tcmp {a}, {b}\n\tsetg {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Ge s32s64 (a %ireg , b %ireg)  "\tcmp {a}, {b}\n\tsetge {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}

%ireg:  Lt p (a %ireg , b %ireg)    "\tcmp {a:.64}, {b:.64}\n\tsetb {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Le p (a %ireg , b %ireg)    "\tcmp {a:.64}, {b:.64}\n\tsetbe {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}
%ireg:  Gt p (a %ireg , b %ireg)    "\tcmp {a:.64}, {b:.64}\n\tseta {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"       {3}
%ireg:  Ge p (a %ireg , b %ireg)    "\tcmp {a:.64}, {b:.64}\n\tsetae {res:.8}\n\tmovsx {res},{res:.8}; {res} = {a} == {b}\n"      {3}

:  Jcc(Eq pi64i32(a %ireg , b %ireg),#l)      "\tcmp {a}, {b}\n\tje  .L{l}\n"   {1}
:  Jcc(Ne pi64i32(a %ireg , b %ireg),#l)      "\tcmp {a}, {b}\n\tjne .L{l}\n"   {1}
:  Jcc(Eq pi64i32(a %ireg , Imm(#_i)),#l)     "\ttest {a}, {a}\n\tjz  .L{l}\n"  {self.range(self.get_right_index(self.get_left_index(index)),0,0)+1}
:  Jcc(Ne pi64i32(a %ireg , Imm(#_i)),#l)     "\ttest {a}, {a}\n\tjnz .L{l}\n"  {self.range(self.get_right_index(self.get_left_index(index)),0,0)+1}

:  Jcc(Lt s32s64 (a %ireg , b %ireg),#l)  "\tcmp {a}, {b}\n\tjl  .L{l}\n"       {1}
:  Jcc(Le s32s64 (a %ireg , b %ireg),#l)  "\tcmp {a}, {b}\n\tjle .L{l}\n"       {1}
:  Jcc(Gt s32s64 (a %ireg , b %ireg),#l)  "\tcmp {a}, {b}\n\tjg .L{l}\n"       {1}
:  Jcc(Ge s32s64 (a %ireg , b %ireg),#l)  "\tcmp {a}, {b}\n\tjge .L{l}\n"       {1}

:  Jcc(Lt p (a %ireg , b %ireg),#l)    "\tcmp {a:.64}, {b:.64}\n\tjb  .L{l}\n"      {1}
:  Jcc(Le p (a %ireg , b %ireg),#l)    "\tcmp {a:.64}, {b:.64}\n\tjbe .L{l}\n"      {1}
:  Jcc(Gt p (a %ireg , b %ireg),#l)    "\tcmp {a:.64}, {b:.64}\n\tja  .L{l}\n"      {1}
:  Jcc(Ge p (a %ireg , b %ireg),#l)    "\tcmp {a:.64}, {b:.64}\n\tjae .L{l}\n"      {1}

%ireg:  Cvp (_r %ireg)              #"#extend/truncuate" {2}
%ireg:  Cvs s64s32s16s8(_r %ireg)   #"#extend/truncuate" {2}

:       Arg pi32i64(r %ireg)         #"\tpush {r:.64}\n" {1}
%eax:   Call pi64i32i16i8v(#name)    #"#call {name}\n" {20}
%eax:   CallV pi64i32i16i8v(r %ireg) #"#call {r}\n"    {20}
:       Call v(#name)                #"#call {name}\n" {20}
:       CallV v(r %ireg)             #"#call {r}\n"    {20}
}

impl BackendAMD64 {
    super::rburg_template::get_rule! {}
    super::rburg_template::reduce_instruction! {}
    super::rburg_template::emit_asm! {}

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

    // Returns the set of registers which are clobbered before an instruction
    // Or during an instruction with multiple steps(see x86-64 divide for example)
    fn clobber(&self, index: usize) -> Vec<Register> {
        let instruction = &self.instructions[index];
        use IRInstruction::*;
        match instruction {
            Div(..) => vec![Register::Rdx],
            _ => Vec::new(),
        }
    }

    // Returns the set of registers which are clobbered at the end of an instruction
    fn clobber_after(&self, index: usize) -> Vec<Register> {
        let instruction = &self.instructions[index];
        use IRInstruction::*;
        use Register::*;
        match instruction {
            Call(..) => vec![Rcx, Rdx, Rsi, Rdi, R8, R9, R10, R11],
            _ => Vec::new(),
        }
    }

    fn get_call_regs(&self, sizes: &Vec<IRSize>) -> Vec<&'static RegisterClass<Register>> {
        let mut result = Vec::with_capacity(sizes.len());
        let mut ireg_index = 0usize;
        for _size in sizes {
            if ireg_index < 6 {
                result.push(&Register::CALL_REGS[ireg_index]);
                ireg_index += 1;
            }
        }
        result
    }

    // Should depend on sizes and allignment as given by the backend
    // Is currently handwritten for x86-64
    fn find_local_offsets(
        &self,
        variable_types: &Vec<IRVariable>,
        arguments: &IRArguments,
    ) -> (Vec<i32>, i32) {
        let mut arg_offset = 8;
        let mut offset = 0;
        let mut result = Vec::new();
        let callee_saved_registers = self.get_callee_saved_registers();
        let saved_offset = -8 * callee_saved_registers.len() as i32;

        for i in 0..variable_types.len() {
            let count = variable_types[i].count as i32;
            result.push(match arguments.arguments.get(i) {
                // Either a normal variable or an argument passed via register
                None | Some(Some(..)) => {
                    offset += padding(
                        offset,
                        match variable_types[i].size {
                            IRSize::S8 | IRSize::S16 | IRSize::S32 => -4,
                            IRSize::P | IRSize::S64 => -8,
                            IRSize::B(size) => -(std::cmp::max(size, 4) as i32),
                            IRSize::V => unreachable!(),
                        },
                    );
                    offset += count
                        * match variable_types[i].size {
                            IRSize::S8 => -1,
                            IRSize::S16 => -2,
                            IRSize::S32 => -4,
                            IRSize::P | IRSize::S64 => -8,
                            IRSize::B(size) => -(size as i32),
                            IRSize::V => unreachable!(),
                        };
                    offset + saved_offset
                }
                // Stack argument
                Some(None) => {
                    arg_offset += 8;
                    arg_offset
                }
            });
        }

        (result, -offset + -saved_offset)
    }
}
