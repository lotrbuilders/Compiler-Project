use crate::{
    backend::{TypeInfo, TypeInfoTable},
    parser::r#type::StructType,
};
use std::{collections::HashMap, ops::Index};

#[derive(Clone)]
pub struct StructTable {
    counter: usize,
    pub structs: Vec<StructType>,
    pub info: Vec<TypeInfo>,
    pub offsets: Vec<Vec<usize>>,
    local_table: Vec<HashMap<String, usize>>,
    global_table: HashMap<String, usize>,
    anonymous_table: Vec<usize>,
}

impl StructTable {
    pub fn new() -> StructTable {
        StructTable {
            counter: 0,
            structs: Vec::new(),
            info: Vec::new(),
            offsets: Vec::new(),
            local_table: Vec::new(),
            global_table: HashMap::new(),
            anonymous_table: Vec::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.local_table.push(HashMap::new());
    }

    pub fn leave_scope(&mut self) {
        self.local_table.pop();
    }

    pub fn contains(&self, key: &String) -> bool {
        for map in self.local_table.iter().rev() {
            if map.get(key).is_some() {
                return true;
            }
        }
        self.global_table.get(key).is_some()
    }

    pub fn try_insert(&mut self, key: Option<&String>) -> Result<usize, ()> {
        let index = self.counter;
        let name = key.map(|s| s.clone());
        let symbol = StructType {
            name,
            members: None,
        };
        if let Some(key) = key {
            if let Some(map) = self.local_table.last_mut() {
                StructTable::try_insert2(map, key, index)
            } else {
                StructTable::try_insert2(&mut self.global_table, key, index)
            }?;
        } else {
            self.anonymous_table.push(index);
        }
        self.structs.push(symbol);
        self.info.push(TypeInfo::new(0, 1, 1));
        self.offsets.push(Vec::new());

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

    pub fn qualify(&mut self, type_info: &TypeInfoTable, index: usize, entry: StructType) {
        let (info, offsets) = entry.to_info(type_info, &self.info);
        self.structs[index] = entry;
        self.offsets[index] = offsets;
        self.info[index] = info;
    }

    pub fn get_index(&self, key: &String) -> Option<usize> {
        for map in self.local_table.iter().rev() {
            let result = map.get(key).map(|i| *i);
            if result.is_some() {
                return result;
            }
        }
        self.global_table.get(key).map(|i| *i)
    }

    pub fn get<'a>(&'a self, key: &String) -> Option<&'a StructType> {
        let index = self.get_index(key)?;
        Some(&self.structs[index])
    }

    pub fn get_mut<'a>(&'a mut self, key: &String) -> Option<&'a mut StructType> {
        let index = self.get_index(key)?;
        Some(&mut self.structs[index])
    }
}
impl<'a> Index<usize> for &'a StructTable {
    type Output = StructType;
    fn index(&self, index: usize) -> &'a Self::Output {
        &self.structs[index]
    }
}
