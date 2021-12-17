pub mod symbol_table;

mod analysis;
mod const_eval;
mod expression_analysis;
mod global_analysis;
mod statement_analysis;
mod type_checking;
mod type_class;

use std::collections::HashMap;

use self::analysis::Analysis;
use self::symbol_table::{Symbol, SymbolTable};
use crate::parser::ast::*;

// The semantic analyzer checks the entire syntax tree for problems
// The semantic analyzer is passed as a member and modified using traits

#[derive(Clone, Debug)]
pub struct SemanticAnalyzer {
    errors: Vec<String>,
    symbol_table: SymbolTable,
    loop_depth: u32,
}

impl SemanticAnalyzer {
    pub fn new() -> SemanticAnalyzer {
        SemanticAnalyzer {
            errors: Vec::new(),
            symbol_table: SymbolTable::new(),
            loop_depth: 0,
        }
    }

    pub fn get_global_table<'a>(&'a self) -> &'a HashMap<String, Symbol> {
        &self.symbol_table.global_table
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
