use crate::parser::r#type::Type;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Symbol {
    pub number: u32,
    pub symbol_type: Vec<Type>,
    pub global: bool,
}

#[derive(Clone, Debug)]
pub struct SymbolTable {
    counter: u32,
    local_table: Vec<HashMap<String, Symbol>>,
    _global_table: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            counter: 0,
            local_table: Vec::new(),
            _global_table: HashMap::new(),
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

    pub fn try_insert(&mut self, key: &String, symbol_type: &Vec<Type>) -> Result<(), ()> {
        if let Some(map) = self.local_table.last_mut() {
            if !map.contains_key(key) {
                map.insert(
                    key.clone(),
                    Symbol {
                        number: self.counter,
                        symbol_type: symbol_type.clone(),
                        global: false,
                    },
                );
                self.counter += 1;
                Ok(())
            } else {
                Err(())
            }
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
        None
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
