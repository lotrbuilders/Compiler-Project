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
use crate::backend::{Backend, TypeInfoTable};
use crate::eval::evaluation_context::EvaluateSize;
use crate::parser::{ast::*, Type};
use crate::table::{StructTable, Symbol, SymbolTable};

// The semantic analyzer checks the entire syntax tree for problems
// The semantic analyzer is passed as a member and modified using traits

#[derive(Clone)]
pub struct SemanticAnalyzer {
    errors: Vec<String>,
    symbol_table: SymbolTable,
    struct_table: StructTable,
    function_return_type: Type,
    type_info: TypeInfoTable,
    //backend: &'a dyn Backend,
    loop_depth: u32,
}

impl SemanticAnalyzer {
    pub fn new(backend: &dyn Backend) -> SemanticAnalyzer {
        SemanticAnalyzer {
            errors: Vec::new(),
            symbol_table: SymbolTable::new(),
            struct_table: StructTable::new(),
            loop_depth: 0,
            function_return_type: Type::empty(),
            type_info: backend.get_type_info_table(),
        }
    }

    pub fn get_struct_table(&mut self) -> StructTable {
        std::mem::replace(&mut self.struct_table, StructTable::new())
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

    fn enter_scope(&mut self) {
        self.symbol_table.enter_scope();
        self.struct_table.enter_scope();
    }

    fn leave_scope(&mut self) {
        self.symbol_table.leave_scope();
        self.struct_table.leave_scope();
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

impl EvaluateSize for SemanticAnalyzer {
    fn type_info(&self) -> &TypeInfoTable {
        &self.type_info
    }

    fn struct_size_table<'a>(&'a self) -> &'a Vec<crate::backend::TypeInfo> {
        &self.struct_table.info
    }
}
