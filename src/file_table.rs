use std::sync::Mutex;

// This file contains a static filetable that is created by the lexer
// Used by the parser+analyzer to get access to the source location
lazy_static::lazy_static! {
    static ref GLOBAL_FILE_TABLE: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn add_sourcefile(s: &String) -> usize {
    let mut file_table = GLOBAL_FILE_TABLE.lock().unwrap();
    let index = file_table.len();
    file_table.push(s.clone());
    index
}

pub fn get_sourcefile(index: u32) -> String {
    let file_table = GLOBAL_FILE_TABLE.lock().unwrap();
    (*file_table)[index as usize].clone()
}
