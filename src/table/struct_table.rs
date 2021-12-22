use crate::parser::r#type::{StructType, Type};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct StructTable {
    counter: usize,
    structs: Vec<StructType>,
    local_table: Vec<HashMap<String, usize>>,
    global_table: HashMap<String, usize>,
}

impl StructTable {
    pub fn new() -> StructTable {
        StructTable {
            counter: 0,
            structs: Vec::new(),
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

    pub fn try_insert(
        &mut self,
        key: Option<&String>,
        members: Vec<(String, Type)>,
    ) -> Result<usize, ()> {
        let index = self.counter;
        let symbol = StructType { members };
        if let Some(key) = key {
            if let Some(map) = self.local_table.last_mut() {
                StructTable::try_insert2(map, key, index)
            } else {
                StructTable::try_insert2(&mut self.global_table, key, index)
            }?;
        }
        self.structs.push(symbol);

        self.counter += 1;
        Ok(index)
    }

    fn try_insert2(map: &mut HashMap<String, usize>, key: &String, index: usize) -> Result<(), ()> {
        if !map.contains_key(key) {
            map.insert(key.clone(), index);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get<'a>(&'a self, key: &String) -> Option<&'a StructType> {
        for map in self.local_table.iter().rev() {
            let result = map.get(key);
            if result.is_some() {
                return result.map(|&i| &self.structs[i]);
            }
        }
        self.global_table.get(key).map(|&i| &self.structs[i])
    }

    pub fn get_mut<'a>(&'a mut self, key: &String) -> Option<&'a mut StructType> {
        for map in self.local_table.iter_mut().rev() {
            let result = map.get(key);
            if result.is_some() {
                return result.map(|&i| &mut self.structs[i]);
            }
        }
        self.global_table.get(key).map(|&i| &mut self.structs[i])
    }
}
