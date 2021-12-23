mod analysis;
mod const_eval;
mod expression_analysis;
mod global_analysis;
mod statement_analysis;
mod type_checking;
mod type_class;
pub mod type_promotion;

use std::collections::HashMap;

use self::analysis::Analysis;
use crate::backend::Backend;
use crate::parser::{ast::*, Type};
use crate::table::{StructTable, Symbol, SymbolTable};

// The semantic analyzer checks the entire syntax tree for problems
// The semantic analyzer is passed as a member and modified using traits

#[derive(Clone)]
pub struct SemanticAnalyzer<'a> {
    errors: Vec<String>,
    symbol_table: SymbolTable,
    struct_table: StructTable,
    function_return_type: Type,
    backend: &'a dyn Backend,
    loop_depth: u32,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(struct_table: StructTable, backend: &'a dyn Backend) -> SemanticAnalyzer<'a> {
        SemanticAnalyzer {
            errors: Vec::new(),
            symbol_table: SymbolTable::new(),
            struct_table,
            loop_depth: 0,
            function_return_type: Type::empty(),
            backend,
        }
    }

    pub fn get_global_table(&self) -> HashMap<String, Symbol> {
        self.symbol_table.global_table.clone()
    }

    pub fn analyze(&mut self, translation_unit: &mut TranslationUnit) -> Result<(), Vec<String>> {
        translation_unit.analyze(self);
        if !self.errors.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(())
        }
    }

    fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }

    fn leave_loop(&mut self) {
        self.loop_depth -= 1;
    }

    fn in_loop(&mut self) -> bool {
        return self.loop_depth > 0;
    }
}
