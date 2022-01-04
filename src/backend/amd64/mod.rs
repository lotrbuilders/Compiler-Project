use super::ir::*;
use smallvec::SmallVec;
use std::collections::HashSet;

mod backend;
mod emit;
mod register_allocation;
mod registers;
mod utility;
use self::register_allocation::*;
use self::registers::*;

pub use self::emit::*;

rburg::rburg_main! {
    BackendAMD64,
:       Ret pi64i32i16i8v(_a %eax)    #"return\n"
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

:       Arg pi32i64(r %ireg)         #"push {r:.64}\n" {1}
%eax:   Call pi64i32i16i8v(#name)    #"#call {name}\n" {20}
%eax:   CallV pi64i32i16i8v(r %ireg) #"#call {r}\n"    {20}
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
                    IRSize::B(size) => {
                        let size = size as i32;
                        offset += -size * count;
                        offset
                    }
                    IRSize::V => unreachable!(),
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
}
