use std::collections::HashSet;

use super::registers::Register;
use super::{stmt_NT, BackendAMD64, State};
use crate::backend::register_allocation::{RegisterBackend, RegisterInterface};
use crate::backend::register_allocation::{RegisterClass, RegisterUse};
use crate::backend::{self, ir::*, Backend, TypeInfoTable};

impl Backend for BackendAMD64 {
    fn backend_type(&self) -> &'static str {
        "burg"
    }

    backend::rburg_template::generate! {}

    fn generate_global_prologue(&mut self) -> String {
        format!("default rel\nsection .text\n")
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

    fn argument_evaluation_direction_registers(&self) -> crate::backend::Direction {
        crate::backend::Direction::Left2Right
    }

    fn argument_evaluation_direction_stack(&self) -> crate::backend::Direction {
        crate::backend::Direction::Right2Left
    }

    fn get_type_info_table(&self) -> TypeInfoTable {
        use crate::backend::TypeInfo;
        use crate::parser::TypeNode::*;
        TypeInfoTable {
            char: TypeInfo {
                size: 1,
                align: 1,
                stack_align: 4,
                irsize: IRSize::S8,
            },
            short: TypeInfo {
                size: 2,
                align: 2,
                stack_align: 4,
                irsize: IRSize::S16,
            },
            int: TypeInfo {
                size: 4,
                align: 4,
                stack_align: 4,
                irsize: IRSize::S32,
            },
            long: TypeInfo {
                size: 8,
                align: 8,
                stack_align: 8,
                irsize: IRSize::S64,
            },
            pointer: TypeInfo {
                size: 8,
                align: 8,
                stack_align: 8,
                irsize: IRSize::P,
            },

            size_t: Long,
        }
    }
}

impl RegisterBackend for BackendAMD64 {
    type RegisterType = Register;

    fn is_instruction(&self, rule: u16) -> bool {
        BackendAMD64::is_instruction(&self, rule)
    }

    fn set_allocation(
        &mut self,
        allocation: Vec<
            crate::backend::register_allocation::RegisterAllocation<Self::RegisterType>,
        >,
    ) {
        self.allocation = allocation;
    }

    fn get_clobbered(&self, index: u32) -> Vec<Self::RegisterType> {
        self.clobber(index as usize)
    }

    fn get_clobbered_after(&self, index: u32) -> Vec<Self::RegisterType> {
        self.clobber_after(index as usize)
    }

    fn find_uses(
        &mut self,
    ) -> crate::backend::register_allocation::RegisterUse<Self::RegisterType> {
        let length = self.definition_index.len();
        let mut creation = vec![u32::MAX; length];
        let mut uses = vec![Vec::new(); length];
        let mut last_use = vec![0u32; length];
        let mut preferred_class: Vec<RegisterClass<Self::RegisterType>> =
            vec![Self::RegisterType::REG_DEFAULT_CLASS; length];

        for arg in self.arguments.arguments.iter().filter_map(|arg| *arg) {
            creation[arg as usize] = 0;
        }

        for i in (1..self.instructions.len()).rev() {
            let rule = self.rules[i];
            if self.is_instruction(rule) {
                let (used_vreg, result_vreg) = self.get_vregisters(i as u32, rule);

                if let Some((vreg, _)) = result_vreg {
                    creation[vreg as usize] = i as u32;
                }
                for (vreg, class) in used_vreg {
                    uses[vreg as usize].push(i as u32);
                    if last_use[vreg as usize] == 0 {
                        last_use[vreg as usize] = i as u32;
                    }
                    if class != Self::RegisterType::REG_DEFAULT_CLASS {
                        preferred_class[vreg as usize] = class;
                    }
                }
            }
            if let IRInstruction::Phi(phi) = &self.instructions[i] {
                for target in &phi.targets {
                    creation[*target as usize] = i as u32;
                }
                for source in phi.sources.iter().flat_map(|src| src.iter()) {
                    let vreg = *source as usize;
                    uses[vreg].push(i as u32);
                    assert_eq!(last_use[vreg], 0);
                    last_use[vreg] = i as u32;
                }
            }
        }
        RegisterUse {
            creation,
            last_use,
            uses,
            preferred_class,
        }
    }

    fn get_instructions<'a>(&'a self) -> &'a Vec<IRInstruction> {
        &self.instructions
    }

    fn get_rule(&self, index: usize) -> u16 {
        self.rules[index]
    }

    fn get_arguments<'a>(&'a self) -> &'a Vec<Option<u32>> {
        &self.arguments.arguments
    }
    fn get_vreg_count(&self) -> u32 {
        self.vreg_count
    }

    fn set_reg_relocations(
        &mut self,
        reg_relocations: Vec<
            Vec<crate::backend::register_allocation::RegisterRelocation<Self::RegisterType>>,
        >,
    ) {
        self.reg_relocations = reg_relocations;
    }

    fn get_vregisters(
        &self,
        index: u32,
        rule: u16,
    ) -> (
        smallvec::SmallVec<[(u32, RegisterClass<Self::RegisterType>); 4]>,
        Option<(u32, RegisterClass<Self::RegisterType>)>,
    ) {
        BackendAMD64::get_vregisters(&self, index, rule)
    }

    fn is_two_address(&self, rule: u16) -> bool {
        crate::backend::amd64::is_two_address(rule)
    }

    fn get_function_length(&self) -> usize {
        self.definition_index.len()
    }

    fn simple_get_spot(&self, vreg: u32) -> u32 {
        self.stack_size.abs() as u32 + 4 + 8 * vreg
    }

    fn simple_adjust_stack_size(&mut self, vreg: i32) {
        self.stack_size += 8 * vreg;
    }

    fn set_used_registers(&mut self, used_registers: Vec<bool>) {
        self.used_registers = used_registers;
    }
}
