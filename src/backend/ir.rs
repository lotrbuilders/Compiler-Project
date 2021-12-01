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

    Add(IRSize, IRReg, IRReg, IRReg),
    Sub(IRSize, IRReg, IRReg, IRReg),
    Mul(IRSize, IRReg, IRReg, IRReg),
    Div(IRSize, IRReg, IRReg, IRReg),

    Ret(IRSize, IRReg),
}

// This is a copy of IRInstruction without the inputs used to simplify generation
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRType {
    Imm,

    Add,
    Sub,
    Mul,
    Div,

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
            &Self::Add(..) => IRType::Add,
            &Self::Sub(..) => IRType::Sub,
            &Self::Mul(..) => IRType::Mul,
            &Self::Div(..) => IRType::Div,
        }
    }

    // Returns the left or only operand vregister if it exists
    pub fn get_left(&self) -> Option<IRReg> {
        match self {
            &Self::Ret(_, left)
            | &Self::Add(_, _, left, _)
            | &Self::Sub(_, _, left, _)
            | &Self::Mul(_, _, left, _)
            | &Self::Div(_, _, left, _) => Some(left),
            _ => None,
        }
    }

    // Returns the right vregister if it exists
    pub fn get_right(&self) -> Option<IRReg> {
        match self {
            &Self::Add(_, _, _, right)
            | &Self::Sub(_, _, _, right)
            | &Self::Mul(_, _, _, right)
            | &Self::Div(_, _, _, right) => Some(right),
            _ => None,
        }
    }

    // Returns the result vregister if it exists
    pub fn get_result(&self) -> Option<IRReg> {
        match self {
            &Self::Imm(_, result, _)
            | &Self::Add(_, result, _, _)
            | &Self::Sub(_, result, _, _)
            | &Self::Mul(_, result, _, _)
            | &Self::Div(_, result, _, _) => Some(result),
            _ => None,
        }
    }

    // Get the value of immediate instructions in string form
    pub fn get_value(&self) -> String {
        match self {
            &Self::Imm(_, _, value) => format!("{}", value),
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
