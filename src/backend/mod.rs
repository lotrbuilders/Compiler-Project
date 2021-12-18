mod amd64;
pub mod ir;

use crate::parser::TypeNode;

use self::ir::*;

// Generates all functions for the specific backend specified

pub fn generate_code(
    functions: Vec<IRFunction>,
    globals: Vec<IRGlobal>,
    backend: &mut dyn Backend,
) -> Result<String, String> {
    let mut assembly = backend.generate_global_prologue();

    for function in &functions {
        assembly.push_str(&backend.generate(&function));
    }
    assembly.push_str(&backend.generate_globals(&globals));

    Ok(assembly)
}

pub fn get_backend(architecture: String) -> Result<Box<dyn Backend>, String> {
    let backend = match &architecture as &str {
        "amd64" => Box::new(amd64::BackendAMD64::new()),
        _ => {
            log::error!("There is no backend implemented for {}", architecture);
            return Err(format!("Unimplemented"));
        }
    };
    Ok(backend)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Left2Right,
    Right2Left,
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

    fn get_arguments_in_registers(&self, _sizes: &Vec<IRSize>) -> Vec<bool> {
        log::error!("Get arguments is not implemented for this backend");
        Vec::new()
    }

    fn argument_evaluation_direction_registers(&self) -> Direction;
    fn argument_evaluation_direction_stack(&self) -> Direction;

    fn get_size(&self, typ: &TypeNode) -> IRSize;
    fn sizeof_pointer(&self) -> u32;
}
