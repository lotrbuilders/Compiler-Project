mod amd64;
mod rburg_template;
mod register_allocation;

use std::collections::HashSet;

use crate::{options::Options, parser::TypeNode};

use crate::ir::*;

// Generates all functions for the specific backend specified

pub fn generate_code(
    backend: &mut dyn Backend,
    functions: Vec<IRFunction>,
    globals: Vec<IRGlobal>,
    function_names: HashSet<String>,
    options: &Options,
) -> Result<String, String> {
    let mut assembly = backend.generate_global_prologue();

    for function in &functions {
        assembly.push_str(&backend.generate(
            &function,
            &function_names,
            &options.register_allocator,
        ));
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

#[derive(Clone, Debug, PartialEq)]
pub struct TypeInfo {
    pub size: usize,
    pub align: usize,
    pub stack_align: usize,
    pub irsize: IRSize,
}

impl TypeInfo {
    pub fn new(size: usize, align: usize, stack_align: usize) -> TypeInfo {
        TypeInfo {
            irsize: IRSize::B(0),
            size: size,
            align: align,
            stack_align: stack_align,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeInfoTable {
    pub char: TypeInfo,
    pub short: TypeInfo,
    pub int: TypeInfo,
    pub long: TypeInfo,
    pub pointer: TypeInfo,

    pub size_t: TypeNode,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Left2Right,
    Right2Left,
}
pub trait Backend {
    // Gives the backend type for processing
    fn backend_type(&self) -> &'static str;

    fn to_string(&self) -> String {
        log::error!("To string is not implemented for this backend");
        String::new()
    }

    // Generates the assembly for a function
    fn generate(
        &mut self,
        function: &IRFunction,
        function_names: &HashSet<String>,
        register_allocator: &str,
    ) -> String {
        let _ = (function_names, register_allocator);
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

    fn get_type_info_table(&self) -> TypeInfoTable;
}

fn get_use_count(instructions: &Vec<IRInstruction>, definitions: &Vec<u32>) -> Vec<u32> {
    log::debug!(
        "Started get use count with {} definitions",
        definitions.len()
    );
    let mut use_count = vec![0u32; definitions.len()];
    for instruction in instructions {
        for vreg in instruction.get_used_vreg() {
            use_count[vreg as usize] += 1;
        }
    }
    log::debug!("Use count: {:?}", use_count);
    use_count
}

fn get_valid_until(function: &IRFunction) -> Vec<u32> {
    let length = function.instructions.len() as usize;
    let end = length as u32;
    let mut result = vec![end; length];
    let mut affected = Vec::new();

    for (index, instruction) in function.instructions.iter().enumerate() {
        if instruction.affected_by_side_effect() {
            let result = index;
            affected.push(result);
        } else if instruction.has_side_effect() {
            for vreg in affected.drain(0..) {
                result[vreg] = index as u32;
            }
        }
    }

    result
}
