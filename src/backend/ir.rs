#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IRFunction {
    pub name: String,
    pub return_size: IRSize,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRInstruction {
    Imm(IRSize, IRReg, i128),
    Ret(IRSize, IRReg),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRType {
    Imm,
    Ret,
}

type IRReg = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IRSize {
    I32,
}

impl IRInstruction {
    pub fn to_type(&self) -> IRType {
        match self {
            &Self::Imm(..) => IRType::Imm,
            &Self::Ret(..) => IRType::Ret,
        }
    }
    pub fn get_left(&self) -> Option<IRReg> {
        match self {
            &Self::Imm(..) => None,
            &Self::Ret(_, left) => Some(left),
        }
    }
    pub fn get_right(&self) -> Option<IRReg> {
        match self {
            &Self::Imm(..) => None,
            &Self::Ret(..) => None,
        }
    }
}

pub fn get_definition_indices(instructions: &Vec<IRInstruction>) -> Vec<u32> {
    use IRInstruction::*;
    instructions
        .iter()
        .filter_map(|ins| match ins {
            Imm(_, result, _) => Some(*result),
            _ => None,
        })
        .collect()
}
