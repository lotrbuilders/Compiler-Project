static mut GLOBAL_FILE_TABLE: Vec<String> = Vec::new();

pub fn add_sourcefile(s: &String) {
    unsafe {
        GLOBAL_FILE_TABLE.push(s.clone());
    }
}

pub fn get_sourcefile(index: u32) -> &'static String {
    unsafe { &GLOBAL_FILE_TABLE[index as usize] }
}

pub unsafe fn reset() -> () {
    GLOBAL_FILE_TABLE = Vec::new();
}
