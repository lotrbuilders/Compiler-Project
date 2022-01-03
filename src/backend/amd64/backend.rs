use std::collections::HashSet;

use super::register_allocation::{ralloc::RegisterAllocator, RegisterAllocatorSimple};
use super::{stmt_NT, BackendAMD64, State};
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
