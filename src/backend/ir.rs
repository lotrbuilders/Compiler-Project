/// Stores a function and all the associated information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IRFunction {
    pub name: String,
    pub return_size: IRSize,
    pub instructions: Vec<IRInstruction>,
    pub variables: Vec<IRSize>,
}

/// All instructions that are available in the Immediate representation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRInstruction {
    Imm(IRSize, IRReg, i128),
    AddrL(IRSize, IRReg, usize),

    Load(IRSize, IRReg, IRReg),  // Result address
    Store(IRSize, IRReg, IRReg), // From address

    Add(IRSize, IRReg, IRReg, IRReg),
    Sub(IRSize, IRReg, IRReg, IRReg),
    Mul(IRSize, IRReg, IRReg, IRReg),
    Div(IRSize, IRReg, IRReg, IRReg),

    Xor(IRSize, IRReg, IRReg, IRReg),

    Eq(IRSize, IRReg, IRReg, IRReg),

    Ret(IRSize, IRReg),
}

// This is a copy of IRInstruction without the inputs used to simplify generation
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRType {
    Imm,
    AddrL,

    Load,
    Store,

    Add,
    Sub,
    Mul,
    Div,

    Xor,

    Eq,

    Ret,
}

type IRReg = u32;

// Stores the size of a particular operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRSize {
    I32,
    P,
}

impl IRInstruction {
    // Transforms the IRInstruction to the simplified version
    pub fn to_type(&self) -> IRType {
        match self {
            &Self::Imm(..) => IRType::Imm,
            &Self::AddrL(..) => IRType::AddrL,

            &Self::Load(..) => IRType::Load,
            &Self::Store(..) => IRType::Store,

            &Self::Add(..) => IRType::Add,
            &Self::Sub(..) => IRType::Sub,
            &Self::Mul(..) => IRType::Mul,
            &Self::Div(..) => IRType::Div,

            &Self::Xor(..) => IRType::Xor,

            &Self::Eq(..) => IRType::Eq,

            &Self::Ret(..) => IRType::Ret,
        }
    }

    // Returns the left or only operand vregister if it exists
    pub fn get_left(&self) -> Option<IRReg> {
        match self {
            &Self::Ret(_, left)
            | &Self::Load(_, _, left)
            | &Self::Store(_, left, _)
            | &Self::Add(_, _, left, _)
            | &Self::Sub(_, _, left, _)
            | &Self::Mul(_, _, left, _)
            | &Self::Div(_, _, left, _)
            | &Self::Xor(_, _, left, _)
            | &Self::Eq(_, _, left, _) => Some(left),
            _ => None,
        }
    }

    // Returns the right vregister if it exists
    pub fn get_right(&self) -> Option<IRReg> {
        match self {
            &Self::Store(.., right)
            | &Self::Add(.., right)
            | &Self::Sub(.., right)
            | &Self::Mul(.., right)
            | &Self::Div(.., right)
            | &Self::Xor(.., right)
            | &Self::Eq(.., right) => Some(right),
            _ => None,
        }
    }

    // Returns the result vregister if it exists
    pub fn get_result(&self) -> Option<IRReg> {
        match self {
            &Self::Imm(_, result, ..)
            | &Self::AddrL(_, result, ..)
            | &Self::Load(_, result, ..)
            | &Self::Add(_, result, ..)
            | &Self::Sub(_, result, ..)
            | &Self::Mul(_, result, ..)
            | &Self::Div(_, result, ..)
            | &Self::Xor(_, result, ..)
            | &Self::Eq(_, result, ..) => Some(result),
            _ => None,
        }
    }

    // Get the value of immediate instructions in string form
}

// Get the indices at which virtual registers are defined
pub fn get_definition_indices(instructions: &Vec<IRInstruction>) -> Vec<u32> {
    (0..instructions.len())
        .filter_map(|i| instructions[i].get_result().map(|_| i as u32))
        .collect()
}
