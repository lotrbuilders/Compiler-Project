/// Stores a function and all the associated information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IRFunction {
    pub name: String,
    pub return_size: IRSize,
    pub instructions: Vec<IRInstruction>,
}

/// All instructions that are available in the Immediate representation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRInstruction {
    Imm(IRSize, IRReg, i128),
    Ret(IRSize, IRReg),
}

// This is a copy of IRInstruction without the inputs used to simplify generation
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRType {
    Imm,
    Ret,
}

type IRReg = u32;

// Stores the size of a particular operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRSize {
    I32,
}

impl IRInstruction {
    // Transforms the IRInstruction to the simplified version
    pub fn to_type(&self) -> IRType {
        match self {
            &Self::Imm(..) => IRType::Imm,
            &Self::Ret(..) => IRType::Ret,
        }
    }

    // Returns the left or only operand vregister if it exists
    pub fn get_left(&self) -> Option<IRReg> {
        match self {
            &Self::Imm(..) => None,
            &Self::Ret(_, left) => Some(left),
        }
    }

    // Returns the right vregister if it exists
    pub fn get_right(&self) -> Option<IRReg> {
        match self {
            &Self::Imm(..) => None,
            &Self::Ret(..) => None,
        }
    }

    // Returns the result vregister if it exists
    pub fn get_result(&self) -> Option<IRReg> {
        match self {
            &Self::Imm(_, result, _) => Some(result),
            _ => None,
        }
    }

    // Get the value of immediate instructions in string form
    pub fn get_value(&self) -> String {
        match self {
            &Self::Imm(_, _, value) => {
                format!("{}", value)
            }
            _ => {
                log::error!("get value called without value");
                format!("")
            }
        }
    }
}

// Get the indices at which virtual registers are defined
pub fn get_definition_indices(instructions: &Vec<IRInstruction>) -> Vec<u32> {
    (0..instructions.len())
        .filter_map(|i| instructions[i].get_result().map(|_| i as u32))
        .collect()
}
