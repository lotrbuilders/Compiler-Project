mod amd64;
pub mod asm;
pub mod ir;
pub mod print_ir;

use self::ir::*;

// Generates all functions for the specific backend specified

pub fn generate_code(
    functions: Vec<IRFunction>,
    globals: Vec<IRGlobal>,
    architecture: String,
) -> Result<String, String> {
    let mut backend: Box<dyn Backend> = match &architecture as &str {
        "amd64" => Box::new(amd64::BackendAMD64::new()),
        _ => {
            log::error!("There is no backend implemented for {}", architecture);
            return Err(format!("Unimplemented"));
        }
    };

    let mut assembly = backend.generate_global_prologue();

    for function in &functions {
        assembly.push_str(&backend.generate(&function));
    }
    assembly.push_str(&backend.generate_globals(&globals));

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

    fn generate_globals(&mut self, _globals: &Vec<IRGlobal>) -> String {
        log::error!("Generate global is not implemented for this backend");
        String::new()
    }

    fn generate_global_prologue(&mut self) -> String {
        log::error!("Generate prologue is not implemented for this backend");
        String::new()
    }

    fn get_arguments_in_registers(&self, _sizes: Vec<IRSize>) -> Vec<bool> {
        log::error!("Get arguments is not implemented for this backend");
        Vec::new()
    }
}
