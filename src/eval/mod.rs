use std::collections::{HashMap, HashSet};

use crate::backend::{ir::*, Backend, TypeInfo};
use crate::parser::ast::*;
use crate::table::struct_table::StructTable;
use crate::table::Symbol;
use crate::utility::padding;

use self::evaluation_context::EvaluationContext;

mod evaluation_context;
mod expression_eval;
mod global_eval;
mod statement_eval;

// This module is used to evaluate the AST into an IR

// The trait Evaluate is used by statements and expressions
// The vreg counter should be updated every use
// The function returns the virtual register representing its result
trait Evaluate {
    fn eval(&self, result: &mut Vec<IRInstruction>, context: &mut EvaluationContext) -> u32;
}

// The public function used to evaluate the ast
pub fn evaluate(
    ast: &TranslationUnit,
    map: &HashMap<String, Symbol>,
    backend: &mut dyn Backend,
    struct_table: StructTable,
) -> (Vec<IRFunction>, Vec<IRGlobal>, HashSet<String>) {
    let (struct_table, struct_offsets) = struct_table.to_info(backend);
    let mut functions = Vec::<IRFunction>::new();
    let mut function_names = HashSet::<String>::new();
    for global in &ast.global_declarations {
        if let Some(declaration) = global.eval(&struct_table, &struct_offsets, backend) {
            function_names.insert(declaration.name.clone());
            functions.push(declaration);
        }
    }

    let mut globals = Vec::new();
    let mut defined = HashSet::new();
    for global in &ast.global_declarations {
        log::trace!("Evaluating individual global");
        if let Some(declaration) = global.eval_global(map, &mut defined) {
            globals.push(declaration);
        }
    }
    (functions, globals, function_names)
}

impl StructTable {
    pub fn to_info(&self, backend: &dyn Backend) -> (Vec<TypeInfo>, Vec<Vec<usize>>) {
        let mut size_list: Vec<TypeInfo> = Vec::new();
        let mut offset_list: Vec<Vec<usize>> = Vec::new();
        for object in &self.structs {
            let (size, align, offsets) = match &object.members {
                Some(members) => members.iter().fold(
                    (0, 1, Vec::<usize>::new()),
                    |(offset, alignment, mut offset_list), (_, typ)| {
                        let element_alignment = backend.sizeof_element(typ) as usize;
                        let alignment = std::cmp::max(alignment, element_alignment);
                        let offset = offset + padding(offset, element_alignment);
                        let sizeof = backend.sizeof(typ) as usize;
                        offset_list.push(offset);
                        (offset + sizeof, alignment, offset_list)
                    },
                ),
                None => (0, 1, Vec::new()),
            };

            let size = size + padding(size, align);
            size_list.push(TypeInfo::new(size, align, align));
            offset_list.push(offsets);
        }
        (size_list, offset_list)
    }
}
