use super::ir_phi::IRPhi;

/// Stores a function and all the associated information
#[derive(Clone, Debug, PartialEq)]
pub struct IRFunction {
    pub name: String,
    pub return_size: IRSize,
    pub instructions: Vec<IRInstruction>,
    pub arguments: IRArguments,
    pub variables: Vec<IRSize>,
    pub strings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IRGlobal {
    pub name: String,
    pub size: IRSize,
    pub value: Option<i128>,
    pub function: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IRArguments {
    pub sizes: Vec<IRSize>,
    pub arguments: Vec<Option<IRReg>>,
    pub count: usize,
}

#[allow(dead_code)]
/// All instructions that are available in the Immediate representation
#[derive(Clone, Debug, PartialEq)]
pub enum IRInstruction {
    Imm(IRSize, IRReg, i128),
    AddrL(IRSize, IRReg, usize),
    AddrG(IRSize, IRReg, String),
    Arg(IRSize, IRReg, Option<usize>),

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
    Call(IRSize, IRReg, String, Box<IRArguments>),
    Label(Option<Box<IRPhi>>, IRLabel),

    Cvp(IRSize, IRReg, IRSize, IRReg), // (to,from) to:IRSize=p
    Cvs(IRSize, IRReg, IRSize, IRReg), // (to,from) to:IRSize in {S8,S16,S32,S64}
    Cvu(IRSize, IRReg, IRSize, IRReg), // (to,from) //Unsupported

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
    AddrG,
    Arg,

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
    Call,
    Label,

    Cvp,
    Cvs,
    Cvu,

    Phi,
    PhiSrc,

    Ret,
}

pub type IRReg = u32;
pub type IRLabel = u32;

// Stores the size of a particular operation
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IRSize {
    S8,
    S16,
    S32,
    S64,
    P,
}

impl IRSize {
    pub fn to_bit_width(&self) -> usize {
        match self {
            IRSize::S8 => 8,
            IRSize::S16 => 16,
            IRSize::S32 => 32,
            IRSize::S64 => 64,
            IRSize::P => unreachable!(),
        }
    }
}

impl IRInstruction {
    // Transforms the IRInstruction to the simplified version
    pub fn to_type(&self) -> IRType {
        match self {
            &Self::Imm(..) => IRType::Imm,
            &Self::AddrL(..) => IRType::AddrL,
            &Self::AddrG(..) => IRType::AddrG,
            &Self::Arg(..) => IRType::Arg,

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
            &Self::Call(..) => IRType::Call,
            &Self::Label(..) => IRType::Label,

            &Self::Cvp(..) => IRType::Cvp,
            &Self::Cvs(..) => IRType::Cvs,
            &Self::Cvu(..) => IRType::Cvu,

            &Self::Phi(..) => IRType::Phi,
            &Self::PhiSrc(..) => IRType::PhiSrc,

            &Self::Ret(..) => IRType::Ret,
        }
    }

    // Returns the left or only operand vregister if it exists
    pub fn get_left(&self) -> Option<IRReg> {
        match self {
            &Self::Ret(_, left)
            | &Self::Arg(_, left, _)
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
            | &Self::Jnc(_, left, _)
            | &Self::Cvp(.., left)
            | &Self::Cvs(.., left)
            | &Self::Cvu(.., left) => Some(left),
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
            | &Self::AddrG(_, result, ..)
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
            | &Self::Ge(_, result, ..)
            | &Self::Call(_, result, ..)
            | &Self::Cvp(_, result, ..)
            | &Self::Cvs(_, result, ..)
            | &Self::Cvu(_, result, ..) => Some(result),
            _ => None,
        }
    }

    pub fn get_size(&self) -> IRSize {
        match self {
            Self::Imm(size, ..)
            | Self::AddrL(size, ..)
            | Self::AddrG(size, ..)
            | Self::Arg(size, ..)
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
            | Self::Ret(size, ..)
            | Self::Call(size, ..) => size.clone(),

            Self::Cvs(to, _, from, _) | Self::Cvu(to, _, from, _) | Self::Cvp(to, _, from, _) => {
                let _ = from;
                *to
            }

            Self::Jmp(_) | Self::Label(..) | Self::PhiSrc(..) | Self::Phi(..) => IRSize::P,
        }
    }
}
