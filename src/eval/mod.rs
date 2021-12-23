use std::collections::{HashMap, HashSet};

use crate::backend::{ir::*, Backend};
use crate::parser::ast::*;
use crate::table::struct_table::StructTable;
use crate::table::Symbol;

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
    let mut struct_table = struct_table;
    let mut functions = Vec::<IRFunction>::new();
    let mut function_names = HashSet::<String>::new();
    for global in &ast.global_declarations {
        let ext = global.eval(struct_table, backend);
        struct_table = ext.1;
        if let Some(declaration) = ext.0 {
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
