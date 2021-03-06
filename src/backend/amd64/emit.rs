use super::{registers::Register, BackendAMD64};
use crate::{ir::*, utility::padding};

impl BackendAMD64 {
    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits handwritten assembly for instruction that are too complex to process normally
    // The boolean specifies wether the normal assembly should also be generated
    pub fn emit_asm2(&self, index: usize) -> (String, bool) {
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
            Call(_size, _vreg, _, arguments) | CallV(_size, _vreg, _, arguments) => (
                {
                    let length = arguments.count;
                    let alignment = self.get_stack_alignment(arguments);
                    let alignment_instruction = if length <= 6 {
                        self.stack_alignment_instruction(alignment)
                    } else {
                        String::new()
                    };

                    let (name, addr) = match instruction {
                        Call(.., name, _) => (Some(name), None),
                        CallV(.., addr, _) => (None, Some(*addr)),
                        _ => unreachable!(),
                    };

                    let outside_file =
                        if name.is_some() && !self.function_names.contains(name.unwrap()) {
                            "wrt ..plt"
                        } else {
                            ""
                        };

                    let callable = if name.is_some() {
                        name.unwrap().clone()
                    } else {
                        format!(
                            "{:.64}",
                            self.allocation[addr.unwrap() as usize][index].unwrap()
                        )
                    };

                    format!(
                        "{}\txor eax,eax\n{}",
                        alignment_instruction,
                        if length > 6 || alignment != 0 {
                            // Only hold for integer and pointer arguments
                            format!(
                                "\tcall {} {}\n\tadd rsp,{}\n",
                                callable,
                                outside_file,
                                8 * (std::cmp::max(6, length) - 6) + alignment as usize
                            )
                        } else {
                            format!("\tcall {} {}\n", callable, outside_file,)
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
                let width = if *to_s == IRSize::S64 || *from_s == IRSize::S64 {
                    64
                } else {
                    32
                };
                let to = self.allocation[*to_r as usize][index].unwrap();
                let from = self.allocation[*from_r as usize][index].unwrap();
                if to != from {
                    (
                        format!("\tmov {:.width$},{:.width$}\n", to, from, width = width),
                        false,
                    )
                } else {
                    (String::new(), false)
                }
            }

            _ => (String::new(), true),
        }
    }

    pub fn emit_function_declaration(&self, name: &String) -> String {
        format!("section .text\nextern {}\n", name)
    }

    pub fn emit_global_definition(
        &self,
        name: &String,
        value: i128,
        size: &IRSize,
        count: usize,
    ) -> String {
        let _ = count;
        let (align, c) = match size {
            IRSize::S8 => (1, 'b'),
            IRSize::S16 => (2, 'w'),
            IRSize::S32 => (4, 'd'),
            IRSize::P | IRSize::S64 => (8, 'q'),
            IRSize::V | IRSize::B(_) => unreachable!(),
        };
        format!(
            "section .data\n\talign {}\n{}:\n\td{} {}\n",
            align, name, c, value
        )
    }

    pub fn emit_common(&self, name: &String, size: &IRSize, count: usize) -> String {
        let (align, size) = match size {
            IRSize::S8 => (1, 1),
            IRSize::S16 => (2, 2),
            IRSize::S32 => (4, 4),
            IRSize::P | IRSize::S64 => (8, 8),
            IRSize::B(size) => (std::cmp::min(*size, 16) as i32, *size as i32),
            IRSize::V => unreachable!(),
        };
        format!(
            "section .bss\n\talignb {}\n{}:\n\tresb {}\n",
            align,
            name,
            size * count as i32
        )
    }

    pub fn get_callee_saved_registers(&self) -> Vec<Register> {
        use Register::*;
        let callee_saved_registers = [Rbx, R12, R13, R14, R15];
        let callee_saved_registers: Vec<_> = callee_saved_registers
            .iter()
            .filter(|&&r| {
                let r: usize = r.into();
                self.used_registers[r]
            })
            .cloned()
            .collect();
        callee_saved_registers
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits the prologue for a function, such that it will be correct for the compiler
    pub fn emit_prologue(&mut self) -> String {
        let mut prologue = format!(
            "global {}\nsection .text\n{}:\n",
            self.function_name, self.function_name
        );

        let callee_saved_registers = self.get_callee_saved_registers();
        let stack_arguments = self.arguments.arguments.contains(&None);

        let offset =
            if self.stack_size != 0 || !callee_saved_registers.is_empty() || stack_arguments {
                prologue.push_str("\tpush rbp\n\tmov rbp,rsp\n");
                0
            } else {
                8
            };

        for reg in &callee_saved_registers {
            self.stack_size += 8;
            prologue.push_str(&format!("\tpush {:.64}\n", reg));
        }

        self.stack_size += padding(self.stack_size, 16);

        if self.stack_size != 0 || offset != 0 {
            prologue.push_str(&format!(
                "\tsub rsp, {}\n",
                self.stack_size + offset - 8 * callee_saved_registers.len() as i32
            ));
        }
        prologue
    }

    // Should be handwritten for any backend
    // Might use a macro to generate parts
    // Emits the epilogue for a function, such that it will be correct for the compiler
    pub fn emit_epilogue(&self) -> String {
        let mut epilogue = ".end:\n".to_string();
        let callee_saved_registers = self.get_callee_saved_registers();
        let stack_arguments = self.arguments.arguments.contains(&None);

        let offset =
            if self.stack_size != 0 || !callee_saved_registers.is_empty() || stack_arguments {
                0
            } else {
                8
            };

        if self.stack_size != 0 || offset != 0 {
            epilogue.push_str(&format!(
                "\tadd rsp, {}\n",
                self.stack_size + offset - 8 * callee_saved_registers.len() as i32
            ));
        }

        for reg in callee_saved_registers.iter().rev() {
            epilogue.push_str(&format!("\tpop {:.64}\n", reg));
        }

        if self.stack_size != 0 || !callee_saved_registers.is_empty() || stack_arguments {
            epilogue.push_str(&format!("\tpop rbp\n"));
        }
        epilogue.push_str(&format!("\tret\n"));
        epilogue
    }

    pub fn emit_strings(&self, strings: &Vec<String>) -> String {
        let mut result = String::from("section .data\n");
        for (string, i) in strings.iter().zip(0..) {
            result.push_str(&format!(".__string{}:\n\tdb ", i));
            for c in string.chars() {
                let b = c as u8;
                result.push_str(&format!("{},", b))
            }
            result.push_str("0\n")
        }
        result
    }

    pub fn emit_move(&self, modification: &super::RegisterRelocation<Register>) -> String {
        use super::RegisterRelocation::*;
        match modification {
            &TwoAddressMove(size, from, to)
            | &Move(size, from, to)
            | &MoveAfter(size, from, to) => {
                let size = self.get_default_register_width2(size);
                format!("\tmov {:.size$}, {:.size$}\n", to, from, size = size)
            }
            &Reload(size, reg, mem) => {
                let size = self.get_default_register_width2(size);
                format!("\tmov {:.size$}, [rbp-{}]\n", reg, mem, size = size)
            }
            &Spill(size, reg, mem) | &SpillEarly(size, reg, mem) => {
                let size = self.get_default_register_width2(size);
                format!("\tmov [rbp-{}], {:.size$} \n", mem, reg, size = size)
            }
            &MemMove(size, from, to, reg) => {
                let size = self.get_default_register_width2(size);
                format!(
                    "\tmov {:.size$}, [rbp-{}]\n\tmov [rbp-{}], {:.size$}\n",
                    reg,
                    from,
                    to,
                    reg,
                    size = size
                )
            }
        }
    }
}
