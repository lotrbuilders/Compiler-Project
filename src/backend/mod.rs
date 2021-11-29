mod amd64;
pub mod asm;
pub mod ir;
pub mod print_ir;

use self::ir::*;

// Generates all functions for the specific backend specified

pub fn generate_code(functions: Vec<IRFunction>, architecture: String) -> Result<String, String> {
    let mut backend: Box<dyn Backend> = match &architecture as &str {
        "amd64" => Box::new(amd64::BackendAMD64::new()),
        _ => {
            log::error!("There is no backend implemented for {}", architecture);
            return Err(format!("Unimplemented"));
        }
    };

    let mut assembly = String::new();

    for function in &functions {
        assembly.push_str(&backend.generate(&function));
    }

    Ok(assembly)
}

pub trait Backend {
    // Gives the backend type for processing
    fn backend_type(&self) -> &'static str;

    fn label(&mut self, _index: u32) -> () {
        log::error!("This is not an rburg based backend");
    }

    fn to_string(&self) -> String {
        log::error!("To string is not implemented for this backend");
        String::new()
    }

    // Generates the assembly for a function
    fn generate(&mut self, function: &IRFunction) -> String {
        log::error!("Generate is not implemented for this backend");
        format!(
            "/*This is not a properly implemented backend. Cannot implement {}*/",
            function.name
        )
    }
}
