use crate::parser::r#type::{DeclarationType, Type};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Symbol {
    pub number: u32,
    pub symbol_type: Type,
    pub declaration_type: DeclarationType,
    pub global: bool,
}

#[derive(Clone, Debug)]
pub struct SymbolTable {
    counter: u32,
    local_table: Vec<HashMap<String, Symbol>>,
    pub global_table: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            counter: 0,
            local_table: Vec::new(),
            global_table: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.local_table.push(HashMap::new());
    }

    pub fn leave_scope(&mut self) {
        self.local_table.pop();
        if self.local_table.is_empty() {
            self.counter = 0;
        }
    }

    #[allow(dead_code)]
    pub fn contains(&self, key: &String) -> bool {
        for map in self.local_table.iter().rev() {
            if map.contains_key(key) {
                return true;
            }
        }
        false
    }

    pub fn try_insert(
        &mut self,
        key: &String,
        symbol_type: &Type,
        declaration_type: DeclarationType,
    ) -> Result<(), ()> {
        let number = self.counter;
        if let Some(map) = self.local_table.last_mut() {
            log::trace!("Local insertion of {} with type {}", key, symbol_type);
            SymbolTable::try_insert2(map, key, symbol_type, declaration_type, number, false)
        } else {
            log::trace!("Global insertion of {} with type {}", key, symbol_type);
            SymbolTable::try_insert2(
                &mut self.global_table,
                key,
                symbol_type,
                declaration_type,
                number,
                true,
            )
        }?;
        if !self.local_table.is_empty() {
            self.counter += 1;
        } else {
            self.counter = 0;
        }
        Ok(())
    }

    fn try_insert2(
        map: &mut HashMap<String, Symbol>,
        key: &String,
        symbol_type: &Type,
        declaration_type: DeclarationType,
        number: u32,
        global: bool,
    ) -> Result<(), ()> {
        if !map.contains_key(key) {
            map.insert(
                key.clone(),
                Symbol {
                    number,
                    symbol_type: symbol_type.clone(),
                    declaration_type,
                    global,
                },
            );
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get<'a>(&'a self, key: &String) -> Option<&'a Symbol> {
        for map in self.local_table.iter().rev() {
            let result = map.get(key);
            if result.is_some() {
                return result;
            }
        }
        self.global_table.get(key)
    }

    pub fn get_mut<'a>(&'a mut self, key: &String) -> Option<&'a mut Symbol> {
        for map in self.local_table.iter_mut().rev() {
            let result = map.get_mut(key);
            if result.is_some() {
                return result;
            }
        }
        self.global_table.get_mut(key)
    }

    pub fn update_declaration_type(&mut self, key: &String, declaration_type: DeclarationType) {
        if let Some(symbol) = self.get_mut(key) {
            symbol.declaration_type = declaration_type;
        }
    }
}
