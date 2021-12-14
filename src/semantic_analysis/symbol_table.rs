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
            SymbolTable::try_insert2(map, key, symbol_type, declaration_type, number)
        } else {
            SymbolTable::try_insert2(
                &mut self.global_table,
                key,
                symbol_type,
                declaration_type,
                number,
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
    ) -> Result<(), ()> {
        if !map.contains_key(key) {
            map.insert(
                key.clone(),
                Symbol {
                    number,
                    symbol_type: symbol_type.clone(),
                    declaration_type,
                    global: false,
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

/*
pub struct SymbolTableIter<'a> {
    vec_iterator: slice::Iter<'a, HashMap<String, Symbol>>,
    map_iterator: hash_map::Iter<'a, String, Symbol>,
}

impl<'a> Iterator for SymbolTableIter<'a> {
    type Item = (&'a String, &'a Symbol);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.map_iterator.next();
        if next.is_none() {
            let map_iterator = self.vec_iterator.next().map(|map| map.iter());
            if let Some(map_iterator) = map_iterator {
                self.map_iterator = map_iterator;
                self.next()
            } else {
                None
            }
        } else {
            next
        }
    }
}*/
