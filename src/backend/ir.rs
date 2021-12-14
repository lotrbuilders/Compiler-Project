/// Stores a function and all the associated information
#[derive(Clone, Debug, PartialEq)]
pub struct IRFunction {
    pub name: String,
    pub return_size: IRSize,
    pub instructions: Vec<IRInstruction>,
    pub variables: Vec<IRSize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IRGlobal {
    pub name: String,
    pub size: IRSize,
    pub value: Option<i128>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IRPhi {
    pub targets: Vec<IRReg>,
    pub size: Vec<IRSize>,
    pub locations: Vec<IRLabel>,
    pub sources: Vec<Vec<IRReg>>,
}

impl IRPhi {
    pub fn empty(locations: Vec<IRLabel>) -> Box<IRPhi> {
        let len = locations.len();
        Box::new(IRPhi {
            targets: Vec::new(),
            size: Vec::new(),
            locations,
            sources: vec![Vec::new(); len],
        })
    }
    pub fn ternary(locations: (IRLabel, IRLabel), result: u32, vreg: (u32, u32)) -> Box<IRPhi> {
        let (l1, l2) = locations;
        let (v1, v2) = vreg;
        Box::new(IRPhi {
            targets: vec![result],
            size: Vec::new(),
            locations: vec![l1, l2],
            sources: vec![vec![v1], vec![v2]],
        })
    }
}

#[allow(dead_code)]
/// All instructions that are available in the Immediate representation
#[derive(Clone, Debug, PartialEq)]
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
    Or(IRSize, IRReg, IRReg, IRReg),
    And(IRSize, IRReg, IRReg, IRReg),

    Eq(IRSize, IRReg, IRReg, IRReg),
    Ne(IRSize, IRReg, IRReg, IRReg),
    Lt(IRSize, IRReg, IRReg, IRReg),
    Le(IRSize, IRReg, IRReg, IRReg),
    Gt(IRSize, IRReg, IRReg, IRReg),
    Ge(IRSize, IRReg, IRReg, IRReg),

    Jcc(IRSize, IRReg, IRLabel),
    Jnc(IRSize, IRReg, IRLabel),
    Jmp(IRLabel),
    Label(Option<Box<IRPhi>>, IRLabel),

    Phi(Box<IRPhi>),
    PhiSrc(IRLabel),

    Ret(IRSize, IRReg),
}

// This is a copy of IRInstruction without the inputs used to simplify generation
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
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
    Or,
    And,

    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    Jcc,
    Jnc,
    Jmp,
    Label,

    Phi,
    PhiSrc,

    Ret,
}

pub type IRReg = u32;
pub type IRLabel = u32;

// Stores the size of a particular operation
#[derive(Clone, Debug)]
pub enum IRSize {
    I32,
    S32,
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
            &Self::Or(..) => IRType::Or,
            &Self::And(..) => IRType::And,

            &Self::Eq(..) => IRType::Eq,
            &Self::Ne(..) => IRType::Ne,
            &Self::Lt(..) => IRType::Lt,
            &Self::Le(..) => IRType::Le,
            &Self::Gt(..) => IRType::Gt,
            &Self::Ge(..) => IRType::Ge,

            &Self::Jcc(..) => IRType::Jcc,
            &Self::Jnc(..) => IRType::Jnc,
            &Self::Jmp(..) => IRType::Jmp,
            &Self::Label(..) => IRType::Label,

            &Self::Phi(..) => IRType::Phi,
            &Self::PhiSrc(..) => IRType::PhiSrc,

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
            | &Self::Or(_, _, left, _)
            | &Self::And(_, _, left, _)
            | &Self::Eq(_, _, left, _)
            | &Self::Ne(_, _, left, _)
            | &Self::Lt(_, _, left, _)
            | &Self::Le(_, _, left, _)
            | &Self::Gt(_, _, left, _)
            | &Self::Ge(_, _, left, _)
            | &Self::Jcc(_, left, _)
            | &Self::Jnc(_, left, _) => Some(left),
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
            | &Self::Or(.., right)
            | &Self::And(.., right)
            | &Self::Eq(.., right)
            | &Self::Ne(.., right)
            | &Self::Lt(.., right)
            | &Self::Le(.., right)
            | &Self::Gt(.., right)
            | &Self::Ge(.., right) => Some(right),
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
            | &Self::Or(_, result, ..)
            | &Self::And(_, result, ..)
            | &Self::Eq(_, result, ..)
            | &Self::Ne(_, result, ..)
            | &Self::Lt(_, result, ..)
            | &Self::Le(_, result, ..)
            | &Self::Gt(_, result, ..)
            | &Self::Ge(_, result, ..) => Some(result),
            _ => None,
        }
    }

    pub fn get_size(&self) -> IRSize {
        match self {
            Self::Imm(size, ..)
            | Self::AddrL(size, ..)
            | Self::Load(size, ..)
            | Self::Store(size, ..)
            | Self::Add(size, ..)
            | Self::Sub(size, ..)
            | Self::Mul(size, ..)
            | Self::Div(size, ..)
            | Self::Xor(size, ..)
            | Self::Or(size, ..)
            | Self::And(size, ..)
            | Self::Eq(size, ..)
            | Self::Ne(size, ..)
            | Self::Lt(size, ..)
            | Self::Le(size, ..)
            | Self::Gt(size, ..)
            | Self::Ge(size, ..)
            | Self::Jnc(size, ..)
            | Self::Jcc(size, ..)
            | Self::Ret(size, ..) => size.clone(),

            Self::Jmp(_) | Self::Label(..) | Self::PhiSrc(..) | Self::Phi(..) => IRSize::P,
        }
    }
}

impl PartialEq for IRSize {
    fn eq(&self, other: &Self) -> bool {
        use IRSize::*;
        match (self, other) {
            (I32, I32) => true,
            (S32, S32) | (S32, I32) | (I32, S32) => true,
            (P, P) => true,
            _ => false,
        }
    }
}

// Get the indices at which virtual registers are defined
pub fn get_definition_indices(instructions: &Vec<IRInstruction>) -> Vec<u32> {
    let mut result = Vec::new();
    for i in 0..instructions.len() {
        match &instructions[i] {
            IRInstruction::Label(Some(phi), _) => {
                for _ in &phi.targets {
                    result.push(i as u32);
                }
            }
            _ => {
                if let Some(_) = instructions[i].get_result() {
                    result.push(i as u32);
                }
            }
        }
    }

    result
}
